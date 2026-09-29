//! `regex`クレートの代替となる、自前の正規表現エンジン。
//!
//! [`Regex::new`]/[`Regex::compile`]はヒープ確保を行わない`const fn`であり、
//! パターンを`const`にバインドして使えば、不正な正規表現はコンパイルエラーとして検出される
//! (到達しない関数内であっても、Rustは`const`アイテムの初期化式を必ずコンパイル時に評価するため)。
//!
//! サポートする構文はこのプロジェクトで実際に使う範囲に限定される: アンカー `^ $`、
//! 文字クラス(範囲・列挙、否定 `[^...]`)、非キャプチャグループ `(?:...)`、
//! 量指定子 `* + ? {n} {n,m}`、選択 `|`、バックスラッシュエスケープ。
//! キャプチャ抽出・後方参照・先読み/後読み・ショートハンドクラス(`\d`等)は未サポート。
//!
//! 文字要素は`u8`ではなく`char`単位で扱うため、`□`/`◇`のようなマルチバイトのUnicode文字も
//! リテラル・文字クラスの要素としてそのまま使える。

/// コンパイル後の命令列の最大長。既存の最も複雑なパターンでも50命令未満のため十分な余裕を持たせている。
const MAX_INSTRUCTIONS: usize = 128;
/// 文字クラス1つに詰め込める要素(単独文字またはレンジ)の最大数。
const MAX_CLASS_ITEMS: usize = 16;
/// パッチリスト・分岐先が「未確定」であることを表す番兵値。
const NIL: usize = usize::MAX;

#[derive(Debug, Clone, Copy)]
enum Instr {
    /// 1文字消費し、`ranges[..ranges_len]`のいずれかの範囲(両端含む)に含まれるか判定する。
    /// `negated`が真なら判定を反転する。単独リテラル文字も`(c, c)`という1要素の範囲として表す。
    Class {
        negated: bool,
        ranges: [(char, char); MAX_CLASS_ITEMS],
        ranges_len: usize,
    },
    /// 2方向に分岐する(選択・量指定子のループ/オプションに使う)。
    Split(usize, usize),
    Jmp(usize),
    /// 文字列の先頭(位置0)でのみ通過できるゼロ幅アサーション。
    StartAnchor,
    /// 文字列の末尾でのみ通過できるゼロ幅アサーション。
    EndAnchor,
    Match,
}

/// パターン文字列が不正な正規表現構文だった場合のエラー。`position`はパターン文字列中のバイト位置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexError {
    /// `(`に対応する`)`が無い、または対応の無い`)`がある。
    UnbalancedGroup { position: usize },
    /// `[`に対応する`]`が無い。
    UnbalancedCharClass { position: usize },
    /// `[]`のように要素が1つも無い文字クラス。
    EmptyCharClass { position: usize },
    /// `[z-a]`のように開始文字が終了文字より後になっているレンジ。
    InvalidCharClassRange { position: usize },
    /// `*abc`や`a**`のように、量指定子の対象となる直前の要素が無い。
    DanglingQuantifier { position: usize },
    /// `{n,m}`の書式が不正(数字が無い・`}`が無い・`n > m`など)。
    InvalidRepetitionRange { position: usize },
    /// パターン末尾が孤立した`\`で終わっている。
    TrailingBackslash,
    /// `(?:`以外の`(?...)`構文など、サポートしていない構文。
    UnsupportedSyntax { position: usize },
    /// コンパイル後の命令数、または文字クラスの要素数が固定バッファの上限を超えた。
    PatternTooComplex,
}

/// コンパイル済みの正規表現。
#[derive(Debug, Clone, Copy)]
pub struct Regex {
    instructions: [Instr; MAX_INSTRUCTIONS],
    len: usize,
    /// マッチング開始時に実行を始める命令番号。
    start: usize,
}

/// 部分的に構築中のバイトコード断片。`start`から実行を始め、`out`は
/// まだ行き先が確定していない`Jmp`命令たちの連結リスト(先頭のインデックス、無ければ`None`)。
/// リストは各`Jmp`命令の(仮の)ターゲット値に「次のノードのインデックス、末尾なら`NIL`」を
/// 一時的に格納することで、追加の記憶領域無しに表現している。
#[derive(Clone, Copy)]
struct Frag {
    start: usize,
    out: Option<usize>,
}

const fn utf8_seq_len(first_byte: u8) -> usize {
    if first_byte & 0b1000_0000 == 0 {
        1
    } else if first_byte & 0b1110_0000 == 0b1100_0000 {
        2
    } else if first_byte & 0b1111_0000 == 0b1110_0000 {
        3
    } else {
        4
    }
}

/// `bytes[i]`から始まる1文字分のUTF-8バイト列を`char`にデコードする。
/// `bytes`は有効な`&str`由来であることが前提(`str::chars()`のconst fn版を自前で書いている)。
const fn decode_char_at(bytes: &[u8], i: usize) -> (char, usize) {
    let b0 = bytes[i];
    let len = utf8_seq_len(b0);
    let cp: u32 = match len {
        1 => b0 as u32,
        2 => (((b0 & 0x1F) as u32) << 6) | ((bytes[i + 1] & 0x3F) as u32),
        3 => {
            (((b0 & 0x0F) as u32) << 12)
                | (((bytes[i + 1] & 0x3F) as u32) << 6)
                | ((bytes[i + 2] & 0x3F) as u32)
        }
        _ => {
            (((b0 & 0x07) as u32) << 18)
                | (((bytes[i + 1] & 0x3F) as u32) << 12)
                | (((bytes[i + 2] & 0x3F) as u32) << 6)
                | ((bytes[i + 3] & 0x3F) as u32)
        }
    };
    let c = match char::from_u32(cp) {
        Some(c) => c,
        None => panic!("invalid utf-8 in regex pattern"),
    };
    (c, len)
}

const fn single_char_class(c: char) -> Instr {
    let mut ranges = [('\0', '\0'); MAX_CLASS_ITEMS];
    ranges[0] = (c, c);
    Instr::Class {
        negated: false,
        ranges,
        ranges_len: 1,
    }
}

enum Quant {
    Star,
    Plus,
    Question,
    Range { min: usize, max: Option<usize> },
}

struct Compiler<'p> {
    bytes: &'p [u8],
    pos: usize,
    instructions: [Instr; MAX_INSTRUCTIONS],
    len: usize,
}

impl<'p> Compiler<'p> {
    const fn new(bytes: &'p [u8]) -> Self {
        Compiler {
            bytes,
            pos: 0,
            instructions: [Instr::Match; MAX_INSTRUCTIONS],
            len: 0,
        }
    }

    const fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    const fn peek_byte(&self) -> Option<u8> {
        if self.at_end() {
            None
        } else {
            Some(self.bytes[self.pos])
        }
    }

    const fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        let p = self.pos + offset;
        if p >= self.bytes.len() {
            None
        } else {
            Some(self.bytes[p])
        }
    }

    const fn peek_char(&self) -> Option<(char, usize)> {
        if self.at_end() {
            None
        } else {
            Some(decode_char_at(self.bytes, self.pos))
        }
    }

    const fn emit(&mut self, instr: Instr) -> Result<usize, RegexError> {
        if self.len >= MAX_INSTRUCTIONS {
            return Err(RegexError::PatternTooComplex);
        }
        let idx = self.len;
        self.instructions[idx] = instr;
        self.len += 1;
        Ok(idx)
    }

    const fn emit_dangling_jmp(&mut self) -> Result<Frag, RegexError> {
        let idx = match self.emit(Instr::Jmp(NIL)) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        Ok(Frag {
            start: idx,
            out: Some(idx),
        })
    }

    /// 1文字を消費する命令(リテラル・文字クラス・アンカー)を、その直後に置く
    /// 「行き先未確定のJmp」とセットで発行する。以後この命令の「次」は常に`pc + 1`になる。
    const fn emit_frag(&mut self, instr: Instr) -> Result<Frag, RegexError> {
        let idx = match self.emit(instr) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        let jmp_idx = match self.emit(Instr::Jmp(NIL)) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        Ok(Frag {
            start: idx,
            out: Some(jmp_idx),
        })
    }

    /// パッチリスト`list`内の全ての`Jmp`の行き先を`to`に確定させる。
    const fn patch(&mut self, list: Option<usize>, to: usize) {
        let mut cur = list;
        while let Some(idx) = cur {
            let next = match self.instructions[idx] {
                Instr::Jmp(n) => {
                    if n == NIL {
                        None
                    } else {
                        Some(n)
                    }
                }
                _ => None,
            };
            self.instructions[idx] = Instr::Jmp(to);
            cur = next;
        }
    }

    /// 2つのパッチリストを連結する。
    const fn append_patch(&mut self, a: Option<usize>, b: Option<usize>) -> Option<usize> {
        let a_head = match a {
            None => return b,
            Some(h) => h,
        };
        let b_head = match b {
            None => return Some(a_head),
            Some(h) => h,
        };
        let mut cur = a_head;
        loop {
            let next = match self.instructions[cur] {
                Instr::Jmp(n) => {
                    if n == NIL {
                        None
                    } else {
                        Some(n)
                    }
                }
                _ => None,
            };
            match next {
                Some(n) => cur = n,
                None => break,
            }
        }
        self.instructions[cur] = Instr::Jmp(b_head);
        Some(a_head)
    }

    /// `a`の後に`b`が続く連結断片を作る。
    const fn concat(&mut self, a: Frag, b: Frag) -> Frag {
        self.patch(a.out, b.start);
        Frag {
            start: a.start,
            out: b.out,
        }
    }

    /// `a`または`b`にマッチする選択断片を作る。
    const fn alt(&mut self, a: Frag, b: Frag) -> Result<Frag, RegexError> {
        let split_idx = match self.emit(Instr::Split(a.start, b.start)) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        let out = self.append_patch(a.out, b.out);
        Ok(Frag {
            start: split_idx,
            out,
        })
    }

    /// `e*`(0回以上)の断片を作る。
    const fn star(&mut self, e: Frag) -> Result<Frag, RegexError> {
        let split_idx = match self.emit(Instr::Split(e.start, NIL)) {
            Ok(i) => i,
            Err(err) => return Err(err),
        };
        self.patch(e.out, split_idx);
        let exit = match self.emit_dangling_jmp() {
            Ok(f) => f,
            Err(err) => return Err(err),
        };
        self.instructions[split_idx] = Instr::Split(e.start, exit.start);
        Ok(Frag {
            start: split_idx,
            out: exit.out,
        })
    }

    /// `e?`(0回または1回)の断片を作る。
    const fn quest(&mut self, e: Frag) -> Result<Frag, RegexError> {
        let split_idx = match self.emit(Instr::Split(e.start, NIL)) {
            Ok(i) => i,
            Err(err) => return Err(err),
        };
        let exit = match self.emit_dangling_jmp() {
            Ok(f) => f,
            Err(err) => return Err(err),
        };
        self.instructions[split_idx] = Instr::Split(e.start, exit.start);
        let out = self.append_patch(e.out, exit.out);
        Ok(Frag {
            start: split_idx,
            out,
        })
    }

    const fn parse_digits(&mut self) -> Option<usize> {
        let start = self.pos;
        let mut n: usize = 0;
        loop {
            match self.peek_byte() {
                Some(b) if b.is_ascii_digit() => {
                    n = n * 10 + (b - b'0') as usize;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        if self.pos == start { None } else { Some(n) }
    }

    const fn try_parse_quantifier(&mut self) -> Result<Option<Quant>, RegexError> {
        match self.peek_byte() {
            Some(b'*') => {
                self.pos += 1;
                Ok(Some(Quant::Star))
            }
            Some(b'+') => {
                self.pos += 1;
                Ok(Some(Quant::Plus))
            }
            Some(b'?') => {
                self.pos += 1;
                Ok(Some(Quant::Question))
            }
            Some(b'{') => {
                let start = self.pos;
                self.pos += 1;
                let min = match self.parse_digits() {
                    Some(n) => n,
                    None => return Err(RegexError::InvalidRepetitionRange { position: start }),
                };
                let max = match self.peek_byte() {
                    Some(b',') => {
                        self.pos += 1;
                        self.parse_digits()
                    }
                    _ => Some(min),
                };
                match self.peek_byte() {
                    Some(b'}') => {
                        self.pos += 1;
                    }
                    _ => return Err(RegexError::InvalidRepetitionRange { position: start }),
                }
                if let Some(m) = max
                    && m < min
                {
                    return Err(RegexError::InvalidRepetitionRange { position: start });
                }
                Ok(Some(Quant::Range { min, max }))
            }
            _ => Ok(None),
        }
    }

    const fn parse_atom(&mut self) -> Result<Frag, RegexError> {
        let start = self.pos;
        match self.peek_byte() {
            Some(b'(') => self.parse_group(),
            Some(b'[') => self.parse_char_class(),
            Some(b'^') => {
                self.pos += 1;
                self.emit_frag(Instr::StartAnchor)
            }
            Some(b'$') => {
                self.pos += 1;
                self.emit_frag(Instr::EndAnchor)
            }
            Some(b'\\') => {
                self.pos += 1;
                match self.peek_char() {
                    None => Err(RegexError::TrailingBackslash),
                    Some((c, clen)) => {
                        self.pos += clen;
                        self.emit_frag(single_char_class(c))
                    }
                }
            }
            Some(b'*') | Some(b'+') | Some(b'?') | Some(b'{') => {
                Err(RegexError::DanglingQuantifier { position: start })
            }
            Some(_) => match self.peek_char() {
                None => Err(RegexError::UnbalancedGroup { position: start }),
                Some((c, clen)) => {
                    self.pos += clen;
                    self.emit_frag(single_char_class(c))
                }
            },
            None => Err(RegexError::UnbalancedGroup { position: start }),
        }
    }

    const fn parse_group(&mut self) -> Result<Frag, RegexError> {
        let group_start = self.pos;
        self.pos += 1; // '('
        if let Some(b'?') = self.peek_byte() {
            if let Some(b':') = self.peek_byte_at(1) {
                self.pos += 2; // "?:"
            } else {
                return Err(RegexError::UnsupportedSyntax { position: self.pos });
            }
        }
        let inner = match self.parse_alternation() {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        match self.peek_byte() {
            Some(b')') => {
                self.pos += 1;
                Ok(inner)
            }
            _ => Err(RegexError::UnbalancedGroup {
                position: group_start,
            }),
        }
    }

    const fn parse_char_class(&mut self) -> Result<Frag, RegexError> {
        let class_start = self.pos;
        self.pos += 1; // '['
        let negated = if let Some(b'^') = self.peek_byte() {
            self.pos += 1;
            true
        } else {
            false
        };
        let mut ranges = [('\0', '\0'); MAX_CLASS_ITEMS];
        let mut ranges_len = 0usize;
        loop {
            match self.peek_byte() {
                None => {
                    return Err(RegexError::UnbalancedCharClass {
                        position: class_start,
                    });
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => {}
            }
            let item_pos = self.pos;
            let c1 = match self.read_class_char(class_start) {
                Ok(c) => c,
                Err(e) => return Err(e),
            };
            let has_dash = matches!(self.peek_byte(), Some(b'-'));
            let dash_is_range = has_dash && !matches!(self.peek_byte_at(1), None | Some(b']'));
            let (lo, hi) = if dash_is_range {
                self.pos += 1; // '-'
                let c2 = match self.read_class_char(class_start) {
                    Ok(c) => c,
                    Err(e) => return Err(e),
                };
                if c2 < c1 {
                    return Err(RegexError::InvalidCharClassRange { position: item_pos });
                }
                (c1, c2)
            } else {
                (c1, c1)
            };
            if ranges_len >= MAX_CLASS_ITEMS {
                return Err(RegexError::PatternTooComplex);
            }
            ranges[ranges_len] = (lo, hi);
            ranges_len += 1;
        }
        if ranges_len == 0 {
            return Err(RegexError::EmptyCharClass {
                position: class_start,
            });
        }
        self.emit_frag(Instr::Class {
            negated,
            ranges,
            ranges_len,
        })
    }

    /// 文字クラス内の1文字を読む(`\`によるエスケープを許す)。
    const fn read_class_char(&mut self, class_start: usize) -> Result<char, RegexError> {
        if let Some(b'\\') = self.peek_byte() {
            self.pos += 1;
            match self.peek_char() {
                None => Err(RegexError::TrailingBackslash),
                Some((c, clen)) => {
                    self.pos += clen;
                    Ok(c)
                }
            }
        } else {
            match self.peek_char() {
                None => Err(RegexError::UnbalancedCharClass {
                    position: class_start,
                }),
                Some((c, clen)) => {
                    self.pos += clen;
                    Ok(c)
                }
            }
        }
    }

    const fn parse_repeat(&mut self) -> Result<Frag, RegexError> {
        let atom_start = self.pos;
        let first = match self.parse_atom() {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        let quant = match self.try_parse_quantifier() {
            Ok(q) => q,
            Err(e) => return Err(e),
        };
        match quant {
            None => Ok(first),
            Some(Quant::Star) => self.star(first),
            Some(Quant::Plus) => {
                self.pos = atom_start;
                let copy = match self.parse_atom() {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                };
                // 量指定子トークン('+')を読み飛ばす
                let _ = self.try_parse_quantifier();
                let starred = match self.star(copy) {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                };
                Ok(self.concat(first, starred))
            }
            Some(Quant::Question) => self.quest(first),
            Some(Quant::Range { min, max }) => {
                // `first`(量指定子を知る前に1回試しにコンパイルしたもの)は使わず、
                // min/maxが分かった上で必要な回数だけ`atom_start`から再パースする。
                let mut result: Option<Frag> = None;
                let mut i = 0;
                while i < min {
                    self.pos = atom_start;
                    let copy = match self.parse_atom() {
                        Ok(f) => f,
                        Err(e) => return Err(e),
                    };
                    result = Some(match result {
                        Some(r) => self.concat(r, copy),
                        None => copy,
                    });
                    i += 1;
                }
                match max {
                    Some(m) => {
                        let mut j = min;
                        while j < m {
                            self.pos = atom_start;
                            let copy = match self.parse_atom() {
                                Ok(f) => f,
                                Err(e) => return Err(e),
                            };
                            let opt = match self.quest(copy) {
                                Ok(f) => f,
                                Err(e) => return Err(e),
                            };
                            result = Some(match result {
                                Some(r) => self.concat(r, opt),
                                None => opt,
                            });
                            j += 1;
                        }
                    }
                    None => {
                        self.pos = atom_start;
                        let copy = match self.parse_atom() {
                            Ok(f) => f,
                            Err(e) => return Err(e),
                        };
                        let starred = match self.star(copy) {
                            Ok(f) => f,
                            Err(e) => return Err(e),
                        };
                        result = Some(match result {
                            Some(r) => self.concat(r, starred),
                            None => starred,
                        });
                    }
                }
                // 量指定子トークン(の再読み込み)を最終位置まで進める
                self.pos = atom_start;
                let _ = self.parse_atom();
                let _ = self.try_parse_quantifier();
                match result {
                    Some(f) => Ok(f),
                    None => self.emit_dangling_jmp(),
                }
            }
        }
    }

    const fn parse_concat(&mut self) -> Result<Frag, RegexError> {
        let mut result: Option<Frag> = None;
        loop {
            match self.peek_byte() {
                None | Some(b'|') | Some(b')') => break,
                _ => {}
            }
            let atom_frag = match self.parse_repeat() {
                Ok(f) => f,
                Err(e) => return Err(e),
            };
            result = Some(match result {
                None => atom_frag,
                Some(prev) => self.concat(prev, atom_frag),
            });
        }
        match result {
            Some(f) => Ok(f),
            None => self.emit_dangling_jmp(),
        }
    }

    const fn parse_alternation(&mut self) -> Result<Frag, RegexError> {
        let mut combined = match self.parse_concat() {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        while let Some(b'|') = self.peek_byte() {
            self.pos += 1;
            let next_branch = match self.parse_concat() {
                Ok(f) => f,
                Err(e) => return Err(e),
            };
            combined = match self.alt(combined, next_branch) {
                Ok(f) => f,
                Err(e) => return Err(e),
            };
        }
        Ok(combined)
    }
}

impl Regex {
    /// パターン文字列をコンパイルする。ヒープ確保を行わないため`const fn`であり、
    /// `const`にバインドして呼び出せばパターンの構文チェックはコンパイル時に行われる。
    pub const fn new(pattern: &str) -> Result<Self, RegexError> {
        let bytes = pattern.as_bytes();
        let mut compiler = Compiler::new(bytes);
        let frag = match compiler.parse_alternation() {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        if compiler.pos != bytes.len() {
            // 対応の無い ')' などがパターン末尾に残っている
            return Err(RegexError::UnbalancedGroup {
                position: compiler.pos,
            });
        }
        let match_idx = match compiler.emit(Instr::Match) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        compiler.patch(frag.out, match_idx);
        Ok(Regex {
            instructions: compiler.instructions,
            len: compiler.len,
            start: frag.start,
        })
    }

    /// [`Regex::new`]のconst版`unwrap`。呼び出し側はこちらを`const`にバインドすることで、
    /// 不正なパターンを実行時パニックではなくコンパイルエラーとして検出できる。
    pub const fn compile(pattern: &str) -> Self {
        match Self::new(pattern) {
            Ok(re) => re,
            Err(_) => panic!("invalid regex pattern"),
        }
    }

    /// `text`の中にこの正規表現にマッチする部分があるかどうかを判定する
    /// (`regex`クレートの`is_match`と同じセマンティクス。アンカーが無ければ部分一致で探索する)。
    pub fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        for start in 0..=chars.len() {
            if self.run_from(&chars, start) {
                return true;
            }
        }
        false
    }

    fn class_matches(instr: &Instr, c: char) -> bool {
        if let Instr::Class {
            negated,
            ranges,
            ranges_len,
        } = instr
        {
            let mut found = false;
            for &(lo, hi) in &ranges[..*ranges_len] {
                if c >= lo && c <= hi {
                    found = true;
                    break;
                }
            }
            found != *negated
        } else {
            false
        }
    }

    /// `pc`からのε遷移(Split/Jmp/アンカー)を辿り、1文字消費する命令(`Class`)か
    /// `Match`に到達したスレッドを`list`に追加する(Pike VMのスレッドリスト構築)。
    fn add_thread(
        &self,
        list: &mut Vec<usize>,
        visited: &mut [bool],
        pc: usize,
        pos: usize,
        len: usize,
    ) {
        if visited[pc] {
            return;
        }
        visited[pc] = true;
        match self.instructions[pc] {
            Instr::Jmp(target) => self.add_thread(list, visited, target, pos, len),
            Instr::Split(a, b) => {
                self.add_thread(list, visited, a, pos, len);
                self.add_thread(list, visited, b, pos, len);
            }
            Instr::StartAnchor => {
                if pos == 0 {
                    self.add_thread(list, visited, pc + 1, pos, len);
                }
            }
            Instr::EndAnchor => {
                if pos == len {
                    self.add_thread(list, visited, pc + 1, pos, len);
                }
            }
            Instr::Class { .. } | Instr::Match => list.push(pc),
        }
    }

    fn run_from(&self, chars: &[char], start: usize) -> bool {
        let n = chars.len();
        let mut clist: Vec<usize> = Vec::new();
        let mut visited = vec![false; self.len];
        self.add_thread(&mut clist, &mut visited, self.start, start, n);

        let mut pos = start;
        loop {
            if clist
                .iter()
                .any(|&pc| matches!(self.instructions[pc], Instr::Match))
            {
                return true;
            }
            if pos >= n || clist.is_empty() {
                return false;
            }
            let c = chars[pos];
            let mut nlist: Vec<usize> = Vec::new();
            let mut nvisited = vec![false; self.len];
            for &pc in &clist {
                if matches!(self.instructions[pc], Instr::Class { .. })
                    && Self::class_matches(&self.instructions[pc], c)
                {
                    self.add_thread(&mut nlist, &mut nvisited, pc + 1, pos + 1, n);
                }
            }
            clist = nlist;
            pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_predicate_pattern() {
        let re = Regex::new(r"^(?:[bdgmnrpkz][aeiuo])(?:[ptkshbdgmnrz][aeiuo])+$").unwrap();
        assert!(re.is_match("parobe"));
        assert!(re.is_match("bage"));
        assert!(!re.is_match("ba"));
        assert!(!re.is_match(""));
        assert!(!re.is_match("xarobe"));
    }

    #[test]
    fn test_number_pattern() {
        let re = Regex::new(
            r"^tu(?:ru)?(?:(?:[pkms][aeio][dgzn][aeio])+(?:[pkms][aeio])?|[pkms][aeio])$",
        )
        .unwrap();
        assert!(re.is_match("tuso"));
        assert!(re.is_match("tusada"));
        assert!(re.is_match("turusademedase"));
        assert!(!re.is_match("tu"));
        assert!(!re.is_match("turu"));
        assert!(!re.is_match("tuda"));
        assert!(!re.is_match("tup"));
        assert!(!re.is_match("tuba"));
        assert!(!re.is_match("tururu"));
        assert!(!re.is_match("pa"));
    }

    #[test]
    fn test_arabic_numeral_pattern() {
        let re = Regex::new(r"^(?:[0-9]+|[0-9]{1,4}(?:_[0-9]{4})*)$").unwrap();
        assert!(re.is_match("12345"));
        assert!(re.is_match("0"));
        assert!(re.is_match("1_0000"));
        assert!(re.is_match("1_2345"));
        assert!(!re.is_match("1_000"));
        assert!(!re.is_match("1_00000"));
        assert!(!re.is_match("_10000"));
        assert!(!re.is_match("10000_"));
        assert!(!re.is_match("1__0000"));
        assert!(!re.is_match("12a45"));
    }

    #[test]
    fn test_simple_literal_prefix_pattern() {
        let re = Regex::new(r"^(?:b)").unwrap();
        assert!(re.is_match("b"));
        assert!(re.is_match("ba"));
        assert!(!re.is_match("a"));
    }

    #[test]
    fn test_char_class_range() {
        let re = Regex::new(r"^[0-9]$").unwrap();
        for c in "0123456789".chars() {
            assert!(re.is_match(&c.to_string()));
        }
        assert!(!re.is_match("a"));
        assert!(!re.is_match(""));
    }

    #[test]
    fn test_quantifiers() {
        assert!(Regex::new(r"^a+$").unwrap().is_match("aaa"));
        assert!(!Regex::new(r"^a+$").unwrap().is_match(""));
        assert!(Regex::new(r"^a*$").unwrap().is_match(""));
        assert!(Regex::new(r"^a?$").unwrap().is_match(""));
        assert!(Regex::new(r"^a?$").unwrap().is_match("a"));
        assert!(!Regex::new(r"^a?$").unwrap().is_match("aa"));
        assert!(Regex::new(r"^a{1,3}$").unwrap().is_match("a"));
        assert!(Regex::new(r"^a{1,3}$").unwrap().is_match("aaa"));
        assert!(!Regex::new(r"^a{1,3}$").unwrap().is_match("aaaa"));
        assert!(!Regex::new(r"^a{1,3}$").unwrap().is_match(""));
    }

    #[test]
    fn test_alternation() {
        let re = Regex::new(r"^(?:ab|cd)$").unwrap();
        assert!(re.is_match("ab"));
        assert!(re.is_match("cd"));
        assert!(!re.is_match("ac"));
        assert!(!re.is_match("abcd"));
    }

    #[test]
    fn test_unicode_literal_and_class() {
        let re1 = Regex::new(r"^(?:□|◇)$").unwrap();
        assert!(re1.is_match("□"));
        assert!(re1.is_match("◇"));
        assert!(!re1.is_match("x"));

        let re2 = Regex::new(r"^[□◇]$").unwrap();
        assert!(re2.is_match("□"));
        assert!(re2.is_match("◇"));
        assert!(!re2.is_match("a"));
    }

    #[test]
    fn test_invalid_patterns_are_rejected() {
        assert!(matches!(
            Regex::new("(?:abc"),
            Err(RegexError::UnbalancedGroup { .. })
        ));
        assert!(matches!(
            Regex::new("[abc"),
            Err(RegexError::UnbalancedCharClass { .. })
        ));
        assert!(matches!(
            Regex::new("[]"),
            Err(RegexError::EmptyCharClass { .. })
        ));
        assert!(matches!(
            Regex::new("*abc"),
            Err(RegexError::DanglingQuantifier { .. })
        ));
        assert!(matches!(
            Regex::new("a**"),
            Err(RegexError::DanglingQuantifier { .. })
        ));
        assert!(matches!(
            Regex::new("a{3,1}"),
            Err(RegexError::InvalidRepetitionRange { .. })
        ));
        assert!(matches!(
            Regex::new("abc\\"),
            Err(RegexError::TrailingBackslash)
        ));
        assert!(matches!(
            Regex::new("[z-a]"),
            Err(RegexError::InvalidCharClassRange { .. })
        ));
        assert!(matches!(
            Regex::new("a)"),
            Err(RegexError::UnbalancedGroup { .. })
        ));
    }

    #[test]
    fn test_const_compile_valid_pattern() {
        const RE: Regex = Regex::compile(r"^(?:ab|cd)+$");
        assert!(RE.is_match("abcd"));
        assert!(RE.is_match("ab"));
        assert!(!RE.is_match("abx"));
    }
}
