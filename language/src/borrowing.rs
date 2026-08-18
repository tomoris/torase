//! 英語からの述語借用アルゴリズム。
//!
//! 手順・音素対応表・再音節化規則・語源記録フォーマットは
//! `docs/language_design/borrowing.md` を正とする。このモジュールはその手順のうち、
//! 機械的に決定できる部分(発音の音素変換・CV音節への再構成・辞書との衝突チェック)を実装する。
//!
//! 借用の手順1(既存述語の組み合わせで表現できないかの検討)と手順2(意味に基づく先頭音節の決定)は
//! 人手の判断であるため、このモジュールでは扱わない。呼び出し側があらかじめ決定した
//! [`PredicatePrefix`] を渡すこと。
//!
//! 英語の発音は、[CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict)などで
//!広く使われているARPAbet風の音素記号をベースにした [`EnglishConsonant`] / [`EnglishVowel`] で表す。

use std::collections::HashSet;

/// 述語の先頭音節。名詞述語のWordNet分類、または動詞述語の項の型を表す文法標識であり、
/// 借用元の英語の発音とは無関係に、借用する概念の意味だけから選ぶ(`morphology.md`参照)。
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

/// 借用元の英語の子音。`borrowing.md`の子音対応表に対応する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnglishConsonant {
    P,
    B,
    T,
    D,
    K,
    G,
    M,
    N,
    R,
    S,
    Z,
    Hh,
    F,
    V,
    Th,
    Dh,
    Sh,
    Zh,
    Ch,
    Jh,
    L,
    W,
    Y,
    Ng,
}

impl EnglishConsonant {
    fn substitute(self) -> char {
        match self {
            Self::P => 'p',
            Self::B => 'b',
            Self::T => 't',
            Self::D => 'd',
            Self::K => 'k',
            Self::G => 'g',
            Self::M => 'm',
            Self::N => 'n',
            Self::R => 'r',
            Self::S => 's',
            Self::Z => 'z',
            Self::Hh => 'h',
            Self::F => 'h',
            Self::V => 'b',
            Self::Th => 's',
            Self::Dh => 'd',
            Self::Sh => 's',
            Self::Zh => 'z',
            Self::Ch => 't',
            Self::Jh => 'd',
            Self::L => 'r',
            Self::W => 'h',
            Self::Y => 'h',
            Self::Ng => 'n',
        }
    }

    fn ipa_label(self) -> &'static str {
        match self {
            Self::P => "p",
            Self::B => "b",
            Self::T => "t",
            Self::D => "d",
            Self::K => "k",
            Self::G => "g",
            Self::M => "m",
            Self::N => "n",
            Self::R => "r",
            Self::S => "s",
            Self::Z => "z",
            Self::Hh => "h",
            Self::F => "f",
            Self::V => "v",
            Self::Th => "θ",
            Self::Dh => "ð",
            Self::Sh => "ʃ",
            Self::Zh => "ʒ",
            Self::Ch => "tʃ",
            Self::Jh => "dʒ",
            Self::L => "l",
            Self::W => "w",
            Self::Y => "j",
            Self::Ng => "ŋ",
        }
    }
}

/// 借用元の英語の母音。`borrowing.md`の母音対応表に対応する。
///
/// `Er`は"-er"(bird, her等)のようなrに色付けされた母音を表し、母音`a`と子音`r`の2音に展開される
/// (`ɑːr`/`ɔːr`/`ɪr`/`ʊr`のような組み合わせは、`Aa`/`Ao`/`Ih`/`Uh`に続けて`R`を並べることで表現できるため
/// 個別のバリアントを設けていない)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnglishVowel {
    Iy,
    Ih,
    Ey,
    Eh,
    Ae,
    Ah,
    Ao,
    Aa,
    Ow,
    Uw,
    Uh,
    Ay,
    Aw,
    Oy,
    Er,
}

impl EnglishVowel {
    fn substitute(self) -> char {
        match self {
            Self::Iy | Self::Ih => 'i',
            Self::Ey | Self::Eh => 'e',
            // シュワ(ə)はストレスのない`Ah`として表す(CMU辞書でも別音素を設けない慣例に合わせる)。
            Self::Ae | Self::Ah => 'a',
            Self::Ao | Self::Aa | Self::Ow => 'o',
            Self::Uw | Self::Uh => 'u',
            Self::Ay | Self::Aw => 'a',
            Self::Oy => 'o',
            Self::Er => 'a',
        }
    }

    fn ipa_label(self) -> &'static str {
        match self {
            Self::Iy => "iː",
            Self::Ih => "ɪ",
            Self::Ey => "eɪ",
            Self::Eh => "e",
            Self::Ae => "æ",
            Self::Ah => "ʌ",
            Self::Ao => "ɔː",
            Self::Aa => "ɑː",
            Self::Ow => "oʊ",
            Self::Uw => "uː",
            Self::Uh => "ʊ",
            Self::Ay => "aɪ",
            Self::Aw => "aʊ",
            Self::Oy => "ɔɪ",
            Self::Er => "ɜːr",
        }
    }
}

/// 変換元となる英語の発音の1単位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnglishPhoneme {
    Consonant(EnglishConsonant),
    Vowel(EnglishVowel),
}

impl EnglishPhoneme {
    fn ipa_label(self) -> &'static str {
        match self {
            Self::Consonant(c) => c.ipa_label(),
            Self::Vowel(v) => v.ipa_label(),
        }
    }

    /// この音素が採用音素へ変換された結果(等音節化前)。
    fn substituted(self) -> String {
        match self {
            Self::Consonant(c) => c.substitute().to_string(),
            // Erのみ母音+子音rの2文字になる。
            Self::Vowel(EnglishVowel::Er) => "ar".to_string(),
            Self::Vowel(v) => v.substitute().to_string(),
        }
    }
}

/// 音素変換・再音節化の途中で使う内部表現。
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
}

/// 手順3: 発音ベースの音素変換。英語の発音記号列を採用音素に写像する。
fn substitute_phonemes(phonemes: &[EnglishPhoneme]) -> Vec<Segment> {
    let mut segments = Vec::new();
    for phoneme in phonemes {
        match phoneme {
            EnglishPhoneme::Consonant(c) => segments.push(Segment::Consonant(c.substitute())),
            EnglishPhoneme::Vowel(EnglishVowel::Er) => {
                // rを伴う母音: 母音aと子音rの2音に展開する。
                segments.push(Segment::Vowel('a'));
                segments.push(Segment::Consonant('r'));
            }
            EnglishPhoneme::Vowel(v) => segments.push(Segment::Vowel(v.substitute())),
        }
    }
    segments
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
                unreachable!("substitute_phonemesの出力にEpentheticVowelは含まれない")
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

/// 借用元の発音のIPA表記(スラッシュなし)。`dictionary.json`の`original_pronunciation`にそのまま使える形式。
fn ipa_string(phonemes: &[EnglishPhoneme]) -> String {
    phonemes.iter().map(|p| p.ipa_label()).collect()
}

/// 音素変換・再音節化の過程を人が読める形にまとめた説明文(語源の確認・議論用の補助であり、
/// `dictionary.json`のフィールドをそのまま置き換えるものではない。`dictionary.json`への記録方法は
/// `docs/dictionary_format.md`を参照)。
fn etymology_note(
    source_word: &str,
    phonemes: &[EnglishPhoneme],
    resyllabified: &[Segment],
    epenthetic_vowel: char,
) -> String {
    let ipa = ipa_string(phonemes);
    let root = render(resyllabified, epenthetic_vowel);
    let mut substitutions: Vec<String> = phonemes
        .iter()
        .map(|p| format!("{}→{}", p.ipa_label(), p.substituted()))
        .collect();
    let epenthesis_count = resyllabified
        .iter()
        .filter(|s| matches!(s, Segment::EpentheticVowel))
        .count();
    if epenthesis_count > 0 {
        substitutions.push(format!("+挿入母音{epenthetic_vowel}×{epenthesis_count}"));
    }
    format!(
        "借用: en \"{source_word}\" /{ipa}/ → {root} ({})",
        substitutions.join(", ")
    )
}

/// 借用によって生成された述語。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedWord {
    /// 先頭音節を含む述語の完全な形。`dictionary.json`のキーにそのまま使える。
    pub word: String,
    /// 先頭音節を除いた語根の部分。
    pub root: String,
    /// 借用元の発音のIPA表記(スラッシュなし)。`dictionary.json`の`original_pronunciation`にそのまま使える。
    pub pronunciation: String,
    /// 音素変換・再音節化の過程を人が読める形にまとめた説明文(記録用の補助)。
    pub etymology_note: String,
}

/// 手順3〜7: 音素変換・CV音節への再構成・先頭音節の結合・辞書との衝突チェック・語源記録の組み立てを行う。
///
/// 手順1(既存述語の組み合わせで表現できないかの検討)と手順2(先頭音節の意味的な決定)は人手の判断のため、
/// `prefix`として呼び出し側があらかじめ決定した値を渡すこと。
///
/// 衝突チェック(手順6)では、再音節化で挿入した母音を`a`→`e`→`i`→`o`→`u`の順に一括で差し替えて
/// 再試行し、`existing_words`に含まれない最初の結果を採用する。5通りすべてが衝突する場合は`None`を返す
/// (挿入母音を含まない借用語で衝突した場合も含む。この場合は差し替える余地がないため常に`None`になる)。
pub fn borrow_predicate(
    prefix: PredicatePrefix,
    source_word: &str,
    phonemes: &[EnglishPhoneme],
    existing_words: &HashSet<String>,
) -> Option<BorrowedWord> {
    let segments = substitute_phonemes(phonemes);
    let resyllabified = resyllabify(&segments);

    const FALLBACK_VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
    for &vowel in &FALLBACK_VOWELS {
        let root = render(&resyllabified, vowel);
        let word = format!("{}{root}", prefix.as_str());
        if !existing_words.contains(&word) {
            return Some(BorrowedWord {
                word,
                root,
                pronunciation: ipa_string(phonemes),
                etymology_note: etymology_note(source_word, phonemes, &resyllabified, vowel),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_existing_words() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn test_動詞述語_runは生物x1のみの自動詞としてpiranaになる() {
        // run /rʌn/: r, ʌ(→a), n。語末子音nを閉じるためaを追加してrana。
        // pi(生物x1のみ)と結合してpirana。borrowing.mdの用例と同じ結果になる。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::R),
            EnglishPhoneme::Vowel(EnglishVowel::Ah),
            EnglishPhoneme::Consonant(EnglishConsonant::N),
        ];
        let result =
            borrow_predicate(PredicatePrefix::Pi, "run", &phonemes, &no_existing_words()).unwrap();
        assert_eq!(result.word, "pirana");
        assert_eq!(result.root, "rana");
        // dictionary.jsonのoriginal_pronunciationにそのまま使える形式(スラッシュなし)。
        assert_eq!(result.pronunciation, "rʌn");
    }

    #[test]
    fn test_名詞述語_coffeeは物質としてnikohiになる() {
        // coffee /ˈkɒfi/: k, ɒ(→o), f(→h), i。クラスタも語末子音もないためそのままkohi。
        // ni(substance)と結合してnikohi。borrowing.mdの用例と同じ結果になる。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::K),
            EnglishPhoneme::Vowel(EnglishVowel::Ao),
            EnglishPhoneme::Consonant(EnglishConsonant::F),
            EnglishPhoneme::Vowel(EnglishVowel::Iy),
        ];
        let result = borrow_predicate(
            PredicatePrefix::Ni,
            "coffee",
            &phonemes,
            &no_existing_words(),
        )
        .unwrap();
        assert_eq!(result.word, "nikohi");
        assert_eq!(result.root, "kohi");
    }

    #[test]
    fn test_動詞述語_holdは既存の借用語kahoradaと一致する() {
        // hold /hoʊld/: h, oʊ(→o), l(→r), d。l,dの子音連続をaで割り、語末dの後にaを追加してhorada。
        // ka(生物x1が抽象概念x2に対して行う)と結合してkahorada。
        // これは既存のdictionary.txtの「x1(人)がx2という会議を開く」の借用語と一致する
        // (アルゴリズムが場当たり的だった過去の造語と偶然同じ結果を再現できる例)。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::Hh),
            EnglishPhoneme::Vowel(EnglishVowel::Ow),
            EnglishPhoneme::Consonant(EnglishConsonant::L),
            EnglishPhoneme::Consonant(EnglishConsonant::D),
        ];
        let result =
            borrow_predicate(PredicatePrefix::Ka, "hold", &phonemes, &no_existing_words()).unwrap();
        assert_eq!(result.word, "kahorada");
        assert_eq!(result.root, "horada");
    }

    #[test]
    fn test_辞書と衝突した場合は挿入母音をaからeに差し替える() {
        // "pirana"が既に辞書にある想定で再度runを借用すると、
        // クラスタ回避・語末閉音節化の挿入母音がaからeに差し替わりpiraneになる
        // (runの発音由来の母音aはそのまま、挿入母音だけが差し替わる)。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::R),
            EnglishPhoneme::Vowel(EnglishVowel::Ah),
            EnglishPhoneme::Consonant(EnglishConsonant::N),
        ];
        let mut existing_words = no_existing_words();
        existing_words.insert("pirana".to_string());

        let result =
            borrow_predicate(PredicatePrefix::Pi, "run", &phonemes, &existing_words).unwrap();
        assert_eq!(result.word, "pirane");
    }

    #[test]
    fn test_衝突が5通りの挿入母音すべてで解決しない場合はnoneを返す() {
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::R),
            EnglishPhoneme::Vowel(EnglishVowel::Ah),
            EnglishPhoneme::Consonant(EnglishConsonant::N),
        ];
        let mut existing_words = no_existing_words();
        for word in ["pirana", "pirane", "pirani", "pirano", "piranu"] {
            existing_words.insert(word.to_string());
        }

        let result = borrow_predicate(PredicatePrefix::Pi, "run", &phonemes, &existing_words);
        assert_eq!(result, None);
    }

    #[test]
    fn test_挿入母音を含まない語が衝突すると差し替える余地がなくnoneを返す() {
        // coffee由来のnikohiには挿入母音が無いため、衝突しても差し替えようがない。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::K),
            EnglishPhoneme::Vowel(EnglishVowel::Ao),
            EnglishPhoneme::Consonant(EnglishConsonant::F),
            EnglishPhoneme::Vowel(EnglishVowel::Iy),
        ];
        let mut existing_words = no_existing_words();
        existing_words.insert("nikohi".to_string());

        let result = borrow_predicate(PredicatePrefix::Ni, "coffee", &phonemes, &existing_words);
        assert_eq!(result, None);
    }

    #[test]
    fn test_先頭が母音の語には子音hが補われる() {
        // "app"のような先頭母音の語(例: æ, p)を想定: 語頭にhを補いhapaになる。
        let phonemes = [
            EnglishPhoneme::Vowel(EnglishVowel::Ae),
            EnglishPhoneme::Consonant(EnglishConsonant::P),
        ];
        let result =
            borrow_predicate(PredicatePrefix::Du, "app", &phonemes, &no_existing_words()).unwrap();
        // 語頭母音æ(→a)の前にhが補われ、語末子音pの後にもaが追加される。
        assert_eq!(result.root, "hapa");
        assert_eq!(result.word, "duhapa");
    }

    #[test]
    fn test_子音クラスタを含む語も述語のトークン規則を満たす() {
        // "category"の簡略化した音素列を想定し、子音クラスタ(g, r)を含む語を確認する。
        let phonemes = [
            EnglishPhoneme::Consonant(EnglishConsonant::K),
            EnglishPhoneme::Vowel(EnglishVowel::Ae),
            EnglishPhoneme::Consonant(EnglishConsonant::T),
            EnglishPhoneme::Vowel(EnglishVowel::Ah),
            EnglishPhoneme::Consonant(EnglishConsonant::G),
            EnglishPhoneme::Consonant(EnglishConsonant::R),
            EnglishPhoneme::Vowel(EnglishVowel::Iy),
        ];
        let result = borrow_predicate(
            PredicatePrefix::Ga,
            "category",
            &phonemes,
            &no_existing_words(),
        )
        .unwrap();

        const RE: crate::regex_engine::Regex = crate::regex_engine::Regex::compile(
            r"^(?:[bdgmnrpkz][aeiuo])(?:[ptkshbdgmnrz][aeiuo])+$",
        );
        assert!(
            RE.is_match(&result.word),
            "生成された語 {} は述語のトークン規則を満たさない",
            result.word
        );
    }
}
