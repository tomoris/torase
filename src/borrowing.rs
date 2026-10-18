//! 英語からの述語借用アルゴリズム。
//!
//! 手順・綴りの対応表・再音節化規則・語源記録フォーマットは
//! `docs/language_design/borrowing.md` を正とする。このモジュールはその手順のうち、
//! 機械的に決定できる部分(綴りの置き換え・CV音節への再構成・辞書との衝突チェック)を実装する。
//!
//! 借用の手順1(既存述語の組み合わせで表現できないかの検討)と手順2(意味に基づく先頭音節の決定)は
//! 人手の判断であるため、このモジュールでは扱わない。呼び出し側があらかじめ決定した
//! [`PredicatePrefix`] を渡すこと。

use std::collections::HashSet;

/// 述語の先頭音節。名詞述語のWordNet分類、または動詞述語の項の型を表す文法標識であり、
/// 借用元の英語の綴りとは無関係に、借用する概念の意味だけから選ぶ(`morphology.md`参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredicatePrefix {
    Ba,
    Be,
    Bi,
    Bu,
    Da,
    Di,
    Du,
    Na,
    Ne,
    Ni,
    No,
    Nu,
    Ga,
    Ge,
    Gi,
    Gu,
    Ma,
    Mi,
    Mu,
    Ra,
    Re,
    Ri,
    Ro,
    Ru,
    Za,
    Zi,
    Zu,
    Ze,
    Pa,
    Pe,
    Pi,
    Po,
    Pu,
    Ka,
    Ki,
    Ku,
}

impl PredicatePrefix {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ba => "ba",
            Self::Be => "be",
            Self::Bi => "bi",
            Self::Bu => "bu",
            Self::Da => "da",
            Self::Di => "di",
            Self::Du => "du",
            Self::Na => "na",
            Self::Ne => "ne",
            Self::Ni => "ni",
            Self::No => "no",
            Self::Nu => "nu",
            Self::Ga => "ga",
            Self::Ge => "ge",
            Self::Gi => "gi",
            Self::Gu => "gu",
            Self::Ma => "ma",
            Self::Mi => "mi",
            Self::Mu => "mu",
            Self::Ra => "ra",
            Self::Re => "re",
            Self::Ri => "ri",
            Self::Ro => "ro",
            Self::Ru => "ru",
            Self::Za => "za",
            Self::Zi => "zi",
            Self::Zu => "zu",
            Self::Ze => "ze",
            Self::Pa => "pa",
            Self::Pe => "pe",
            Self::Pi => "pi",
            Self::Po => "po",
            Self::Pu => "pu",
            Self::Ka => "ka",
            Self::Ki => "ki",
            Self::Ku => "ku",
        }
    }
}

/// 綴りの置き換え・再音節化の途中で使う内部表現。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment {
    Consonant(char),
    Vowel(char),
    /// 再音節化規則(子音クラスタの回避・語末の閉音節化)で挿入された母音。
    /// 実際の文字は、辞書との衝突チェック(手順6)で`a`→`e`→`i`→`o`→`u`の順に確定する。
    EpentheticVowel,
}

impl Segment {
    fn is_consonant(self) -> bool {
        matches!(self, Segment::Consonant(_))
    }

    fn is_vowel(self) -> bool {
        matches!(self, Segment::Vowel(_))
    }
}

/// 借用に失敗した理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowError {
    /// 綴りが空、または英小文字(a〜z)以外の文字を含む。
    InvalidSpelling,
    /// 挿入母音を5通りすべて試しても既存の語と衝突した。
    Collision,
}

fn is_vowel_letter(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// 手順3: 綴りの置き換え。英語の綴りを採用音素に写像し、重なった子音と母音の連続をまとめる。
fn spelling_to_segments(spelling: &str) -> Result<Vec<Segment>, BorrowError> {
    if spelling.is_empty() || !spelling.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(BorrowError::InvalidSpelling);
    }
    let letters: Vec<char> = spelling.chars().collect();
    let at = |i: usize| letters.get(i).copied();
    let starts_with = |i: usize, s: &str| {
        s.chars()
            .enumerate()
            .all(|(k, c)| letters.get(i + k) == Some(&c))
    };

    let mut raw: Vec<Segment> = Vec::new();
    let consonants = |raw: &mut Vec<Segment>, cs: &str| {
        raw.extend(cs.chars().map(Segment::Consonant));
    };
    let mut i = 0;
    while i < letters.len() {
        let c = letters[i];
        // 規則1: 語頭の黙字。
        if i == 0
            && let Some(mapped) = ["kn", "wr", "gn", "ps"]
                .iter()
                .find(|s| starts_with(0, s))
                .map(|s| &s[1..])
        {
            consonants(&mut raw, mapped);
            i += 2;
            continue;
        }
        // 規則2: 語中のti/siの直後にo/aが続く(-tion, -sion, -tial)ならs。
        if i > 0
            && matches!(c, 't' | 's')
            && at(i + 1) == Some('i')
            && matches!(at(i + 2), Some('o' | 'a'))
        {
            consonants(&mut raw, "s");
            i += 2;
            continue;
        }
        // 規則3: 2〜3文字で1音を表す綴り(長い方を優先)。
        if let Some((len, mapped)) = [
            ("sch", "sk"),
            ("tch", "t"),
            ("ch", "t"),
            ("sh", "s"),
            ("th", "s"),
            ("ph", "h"),
            ("ck", "k"),
            ("wh", "h"),
            ("qu", "kh"),
        ]
        .iter()
        .find(|(s, _)| starts_with(i, s))
        .map(|(s, mapped)| (s.len(), *mapped))
        {
            consonants(&mut raw, mapped);
            i += len;
            continue;
        }
        // 規則4: ghは語頭ならg、それ以外は発音しない。
        if starts_with(i, "gh") {
            if i == 0 {
                consonants(&mut raw, "g");
            }
            i += 2;
            continue;
        }
        // 規則5: 1文字の置き換え。
        match c {
            'c' if matches!(at(i + 1), Some('e' | 'i' | 'y')) => consonants(&mut raw, "s"),
            'c' | 'q' => consonants(&mut raw, "k"),
            'f' => consonants(&mut raw, "h"),
            'v' => consonants(&mut raw, "b"),
            'l' => consonants(&mut raw, "r"),
            'j' => consonants(&mut raw, "d"),
            'x' => consonants(&mut raw, "ks"),
            'w' if i > 0 && is_vowel_letter(letters[i - 1]) => {}
            'w' => consonants(&mut raw, "h"),
            'y' if i == 0 && at(1).is_some_and(is_vowel_letter) => consonants(&mut raw, "h"),
            'y' => raw.push(Segment::Vowel('i')),
            c if is_vowel_letter(c) => raw.push(Segment::Vowel(c)),
            c => raw.push(Segment::Consonant(c)),
        }
        i += 1;
    }

    // 規則6: 重なった子音を1つにまとめる。規則7: 母音の連続は最初の1つだけ残す。
    let mut segments: Vec<Segment> = Vec::new();
    for segment in raw {
        let merged = match (segments.last(), segment) {
            (Some(Segment::Consonant(prev)), Segment::Consonant(c)) => *prev == c,
            (Some(prev), segment) => prev.is_vowel() && segment.is_vowel(),
            (None, _) => false,
        };
        if !merged {
            segments.push(segment);
        }
    }
    Ok(segments)
}

/// 手順4: CV音節への再構成。子音クラスタ・語頭(語中)の無初頭母音・語末子音を解消する。
fn resyllabify(segments: &[Segment]) -> Vec<Segment> {
    let mut result: Vec<Segment> = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        match segment {
            Segment::Consonant(_) => {
                result.push(*segment);
                // 規則1: 子音が2つ以上連続する場合、最初の子音の直後に母音を挿入する。
                if matches!(segments.get(i + 1), Some(next) if next.is_consonant()) {
                    result.push(Segment::EpentheticVowel);
                }
            }
            Segment::Vowel(_) => {
                // 規則2: 直前に子音のない母音には子音hを補う。
                let has_onset = matches!(result.last(), Some(s) if s.is_consonant());
                if !has_onset {
                    result.push(Segment::Consonant('h'));
                }
                result.push(*segment);
            }
            Segment::EpentheticVowel => {
                unreachable!("spelling_to_segmentsの出力にEpentheticVowelは含まれない")
            }
        }
    }

    // 規則3: 語末が子音の場合、母音を追加して閉じる。
    if matches!(result.last(), Some(s) if s.is_consonant()) {
        result.push(Segment::EpentheticVowel);
    }

    result
}

/// `EpentheticVowel`を`vowel`で確定して文字列化する。
fn render(segments: &[Segment], epenthetic_vowel: char) -> String {
    segments
        .iter()
        .map(|s| match s {
            Segment::Consonant(c) => *c,
            Segment::Vowel(c) => *c,
            Segment::EpentheticVowel => epenthetic_vowel,
        })
        .collect()
}

/// 綴りの置き換え・再音節化の過程を人が読める形にまとめた説明文(語源の確認・議論用の補助であり、
/// `dictionary.json`のフィールドをそのまま置き換えるものではない。`dictionary.json`への記録方法は
/// `docs/language_design/borrowing.md`の「語源記録のフォーマット」を参照)。
fn etymology_note(
    spelling: &str,
    segments: &[Segment],
    resyllabified: &[Segment],
    epenthetic_vowel: char,
) -> String {
    let substituted = render(segments, epenthetic_vowel);
    let root = render(resyllabified, epenthetic_vowel);
    let epenthesis_count = resyllabified
        .iter()
        .filter(|s| matches!(s, Segment::EpentheticVowel))
        .count();
    let epenthesis = if epenthesis_count > 0 {
        format!(", +挿入母音{epenthetic_vowel}×{epenthesis_count}")
    } else {
        String::new()
    };
    format!("借用: en \"{spelling}\" → {root} (綴りの置き換え: {substituted}{epenthesis})")
}

/// 借用によって生成された述語。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedWord {
    /// 先頭音節を含む述語の完全な形。`dictionary.json`のキーにそのまま使える。
    pub word: String,
    /// 先頭音節を除いた語根の部分。
    pub root: String,
    /// 綴りの置き換え・再音節化の過程を人が読める形にまとめた説明文(記録用の補助)。
    pub etymology_note: String,
}

/// 手順3〜6: 綴りの置き換え・CV音節への再構成・先頭音節の結合・辞書との衝突チェックを行う。
///
/// 手順1(既存述語の組み合わせで表現できないかの検討)と手順2(先頭音節の意味的な決定)は人手の判断のため、
/// `prefix`として呼び出し側があらかじめ決定した値を渡すこと。`spelling`は英語の綴り(英小文字のみ)。
///
/// 衝突チェック(手順6)では、再音節化で挿入した母音を`a`→`e`→`i`→`o`→`u`の順に一括で差し替えて
/// 再試行し、`existing_words`に含まれない最初の結果を採用する。5通りすべてが衝突する場合は
/// `BorrowError::Collision`を返す(挿入母音を含まない借用語で衝突した場合も、差し替える余地がないため同じ)。
pub fn borrow_predicate(
    prefix: PredicatePrefix,
    spelling: &str,
    existing_words: &HashSet<String>,
) -> Result<BorrowedWord, BorrowError> {
    let segments = spelling_to_segments(spelling)?;
    let resyllabified = resyllabify(&segments);

    const FALLBACK_VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
    for &vowel in &FALLBACK_VOWELS {
        let root = render(&resyllabified, vowel);
        let word = format!("{}{root}", prefix.as_str());
        if !existing_words.contains(&word) {
            return Ok(BorrowedWord {
                word,
                root,
                etymology_note: etymology_note(spelling, &segments, &resyllabified, vowel),
            });
        }
    }
    Err(BorrowError::Collision)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_existing_words() -> HashSet<String> {
        HashSet::new()
    }

    fn root(spelling: &str) -> String {
        borrow_predicate(PredicatePrefix::Du, spelling, &no_existing_words())
            .unwrap()
            .root
    }

    #[test]
    fn test_全ての先頭音節が述語として字句解析され項の型が決まる() {
        use crate::tokenizer::{TerminalType, tokenize};
        use PredicatePrefix::*;
        let prefixes = [
            Ba, Be, Bi, Bu, Da, Di, Du, Na, Ne, Ni, No, Nu, Ga, Ge, Gi, Gu, Ma, Mi, Mu, Ra, Re, Ri,
            Ro, Ru, Za, Zi, Zu, Ze, Pa, Pe, Pi, Po, Pu, Ka, Ki, Ku,
        ];
        for prefix in prefixes {
            let word = format!("{}ba", prefix.as_str());
            let sentences = tokenize(&word).unwrap();
            let TerminalType::BinaryPredicate(predicate) = &sentences[0][0] else {
                panic!("{word} should be a binary predicate: {:?}", sentences[0][0]);
            };
            predicate.get_argument_types();
        }
    }

    #[test]
    fn test_動詞述語_runは生物x1のみの自動詞としてpirunaになる() {
        // run: r, u, n。語末子音nを閉じるためaを追加してruna。borrowing.mdの用例と同じ結果になる。
        let result = borrow_predicate(PredicatePrefix::Pi, "run", &no_existing_words()).unwrap();
        assert_eq!(result.word, "piruna");
        assert_eq!(result.root, "runa");
    }

    #[test]
    fn test_名詞述語_coffeeは物質としてnikoheになる() {
        // coffee: c(→k), o, ff(→h), ee(→e)。borrowing.mdの用例と同じ結果になる。
        let result = borrow_predicate(PredicatePrefix::Ni, "coffee", &no_existing_words()).unwrap();
        assert_eq!(result.word, "nikohe");
        assert_eq!(result.root, "kohe");
    }

    #[test]
    fn test_動詞述語_holdは既存の借用語kahoradaと一致する() {
        // hold: h, o, l(→r), d。r,dの子音連続をaで割り、語末dの後にaを追加してhorada。
        let result = borrow_predicate(PredicatePrefix::Ka, "hold", &no_existing_words()).unwrap();
        assert_eq!(result.word, "kahorada");
    }

    #[test]
    fn test_語頭の母音にはhを補い語末のeは母音として残す() {
        let result = borrow_predicate(PredicatePrefix::No, "apple", &no_existing_words()).unwrap();
        assert_eq!(result.word, "nohapare");
    }

    #[test]
    fn test_語頭の黙字を落とす() {
        assert_eq!(root("knight"), "nita");
        assert_eq!(root("write"), "rite");
    }

    #[test]
    fn test_2から3文字で1音を表す綴りを置き換える() {
        assert_eq!(root("school"), "sakora");
        assert_eq!(root("church"), "turata");
        assert_eq!(root("thing"), "sinaga");
        assert_eq!(root("phone"), "hone");
        assert_eq!(root("queen"), "kahena");
    }

    #[test]
    fn test_ghは語頭ならgでそれ以外は発音しない() {
        assert_eq!(root("ghost"), "gosata");
        assert_eq!(root("night"), "nita");
    }

    #[test]
    fn test_cはeiyの前ならsでそれ以外はk() {
        assert_eq!(root("city"), "siti");
        assert_eq!(root("cat"), "kata");
    }

    #[test]
    fn test_母音の後のwは落としそれ以外はh() {
        assert_eq!(root("snow"), "sano");
        assert_eq!(root("water"), "hatera");
    }

    #[test]
    fn test_yは語頭で母音の前ならhでそれ以外は母音i() {
        assert_eq!(root("yes"), "hesa");
        assert_eq!(root("day"), "da");
    }

    #[test]
    fn test_語中のtiとsiの後にoかaが続けばs() {
        assert_eq!(root("nation"), "nasona");
        assert_eq!(root("question"), "kahesona");
    }

    #[test]
    fn test_重なった子音と母音の連続をまとめる() {
        assert_eq!(root("hello"), "hero");
        assert_eq!(root("eat"), "heta");
        assert_eq!(root("box"), "bokasa");
    }

    #[test]
    fn test_英小文字以外を含む綴りはエラーになる() {
        for spelling in ["", "Apple", "ice-cream", "café"] {
            assert_eq!(
                borrow_predicate(PredicatePrefix::Du, spelling, &no_existing_words()),
                Err(BorrowError::InvalidSpelling),
                "{spelling}"
            );
        }
    }

    #[test]
    fn test_辞書と衝突した場合は挿入母音をaからeに差し替える() {
        // "piruna"が既に辞書にある想定で再度runを借用すると、挿入母音だけがaからeに差し替わる。
        let mut existing_words = no_existing_words();
        existing_words.insert("piruna".to_string());

        let result = borrow_predicate(PredicatePrefix::Pi, "run", &existing_words).unwrap();
        assert_eq!(result.word, "pirune");
    }

    #[test]
    fn test_衝突が5通りの挿入母音すべてで解決しない場合はエラーになる() {
        let mut existing_words = no_existing_words();
        for word in ["piruna", "pirune", "piruni", "piruno", "pirunu"] {
            existing_words.insert(word.to_string());
        }

        let result = borrow_predicate(PredicatePrefix::Pi, "run", &existing_words);
        assert_eq!(result, Err(BorrowError::Collision));
    }

    #[test]
    fn test_挿入母音を含まない語が衝突すると差し替える余地がなくエラーになる() {
        // coffee由来のnikoheには挿入母音が無いため、衝突しても差し替えようがない。
        let mut existing_words = no_existing_words();
        existing_words.insert("nikohe".to_string());

        let result = borrow_predicate(PredicatePrefix::Ni, "coffee", &existing_words);
        assert_eq!(result, Err(BorrowError::Collision));
    }

    #[test]
    fn test_子音クラスタを含む語も述語のトークン規則を満たす() {
        let result =
            borrow_predicate(PredicatePrefix::Ga, "strength", &no_existing_words()).unwrap();

        const RE: crate::tokenizer::regex_engine::Regex =
            crate::tokenizer::regex_engine::Regex::compile(
                r"^(?:[bdgmnrpkz][aeiuo])(?:[ptkshbdgmnrz][aeiuo])+$",
            );
        assert!(
            RE.is_match(&result.word),
            "生成された語 {} は述語のトークン規則を満たさない",
            result.word
        );
    }
}
