//! 数詞(`tu...`)の符号化/復号化。子音×母音の音節を1桁(0〜15)または桁位置(指数)に
//! 対応させる独自の位取り記数法(既定は10進、`ru`を付けると16進)。
//! アラビア数字表記(算用数字)も代替入力として受け付ける。

use super::regex_engine::Regex;
use super::{Range, make_range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Number {
    range: Range,
    value: u64,
    is_hex: bool,
}

impl Number {
    pub(crate) fn get_range(&self) -> Range {
        self.range.clone()
    }

    // 数字テーブル: 子音p/k/m/s x 母音a/e/i/o で1桁(0〜15)を表す。
    const DIGIT_CONSONANTS: [char; 4] = ['p', 'k', 'm', 's'];
    // 桁位置テーブル: 子音d/g/z/n x 母音a/e/i/o で指数(0〜15)を表す。
    const PLACE_CONSONANTS: [char; 4] = ['d', 'g', 'z', 'n'];
    const VOWELS: [char; 4] = ['a', 'e', 'i', 'o'];

    pub(super) fn is_match(string: &str) -> bool {
        if Self::is_arabic_numeral_match(string) {
            return true;
        }
        const RE_NUMBER: Regex = Regex::compile(
            r"^tu(?:ru)?(?:(?:[pkms][aeio][dgzn][aeio])+(?:[pkms][aeio])?|[pkms][aeio])$",
        );
        if !RE_NUMBER.is_match(string) {
            return false;
        }
        let rest = &string[2..];
        let explicit_hex = rest.starts_with("ru");
        if explicit_hex {
            return true;
        }
        // ruが無ければ10進数として扱うので、桁の値はすべて0〜9でなければならない
        // (10〜15を含む桁を言いたい場合は必ずruを付ける)。
        Self::parse_terms(rest).iter().all(|&(digit, _)| digit <= 9)
    }
    // アラビア数字表記(算用数字)。数詞の代替入力表記として1桁目〜任意桁の数字列、
    // または"1_000"のように3桁ごとにアンダースコアで区切った数字列を受け付ける
    // (常に10進数として解釈する)。
    fn is_arabic_numeral_match(string: &str) -> bool {
        const RE_ARABIC: Regex = Regex::compile(r"^(?:[0-9]+|[0-9]{1,3}(?:_[0-9]{3})*)$");
        if !RE_ARABIC.is_match(string) {
            return false;
        }
        let digits_only: String = string.chars().filter(|&c| c != '_').collect();
        digits_only.parse::<u64>().is_ok()
    }
    fn parse_terms(rest: &str) -> Vec<(u64, u32)> {
        let chars: Vec<char> = rest.chars().collect();
        let mut terms: Vec<(u64, u32)> = Vec::new();
        let mut i = 0;
        while i < chars.len() {
            let digit = Self::digit_from_syllable(chars[i], chars[i + 1]);
            i += 2;
            if i + 1 < chars.len() && Self::PLACE_CONSONANTS.contains(&chars[i]) {
                let exponent = Self::place_from_syllable(chars[i], chars[i + 1]);
                i += 2;
                terms.push((digit, exponent));
            } else {
                terms.push((digit, 0));
            }
        }
        terms
    }
    fn digit_from_syllable(consonant: char, vowel: char) -> u64 {
        let row = Self::DIGIT_CONSONANTS
            .iter()
            .position(|&c| c == consonant)
            .unwrap();
        let col = Self::VOWELS.iter().position(|&v| v == vowel).unwrap();
        (row * 4 + col) as u64
    }
    fn place_from_syllable(consonant: char, vowel: char) -> u32 {
        let row = Self::PLACE_CONSONANTS
            .iter()
            .position(|&c| c == consonant)
            .unwrap();
        let col = Self::VOWELS.iter().position(|&v| v == vowel).unwrap();
        (row * 4 + col) as u32
    }
    fn digit_to_syllable(digit: u64) -> String {
        let row = (digit / 4) as usize;
        let col = (digit % 4) as usize;
        format!("{}{}", Self::DIGIT_CONSONANTS[row], Self::VOWELS[col])
    }
    fn place_to_syllable(exponent: u32) -> String {
        let row = (exponent / 4) as usize;
        let col = (exponent % 4) as usize;
        format!("{}{}", Self::PLACE_CONSONANTS[row], Self::VOWELS[col])
    }
    pub(super) fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        if !Self::is_match(string) {
            panic!("Invalid number surface: {}", string);
        }
        let (value, is_hex) = if Self::is_arabic_numeral_match(string) {
            let digits_only: String = string.chars().filter(|&c| c != '_').collect();
            (digits_only.parse::<u64>().unwrap(), false)
        } else {
            let mut rest = &string[2..]; // "tu"を読み飛ばす
            let is_hex = rest.starts_with("ru");
            if is_hex {
                rest = &rest[2..];
            }
            let terms = Self::parse_terms(rest);
            let base: u64 = if is_hex { 16 } else { 10 };
            let value = terms
                .iter()
                .map(|(digit, exponent)| digit * base.pow(*exponent))
                .sum();
            (value, is_hex)
        };
        Number {
            range: make_range(start_line, start_character, end_line, end_character),
            value,
            is_hex,
        }
    }
    pub fn get_value(&self) -> u64 {
        self.value
    }
    pub fn is_hex(&self) -> bool {
        self.is_hex
    }
    pub fn get_string(&self) -> String {
        let base: u64 = if self.is_hex { 16 } else { 10 };
        let mut digits: Vec<u64> = Vec::new();
        if self.value == 0 {
            digits.push(0);
        } else {
            let mut v = self.value;
            while v > 0 {
                digits.push(v % base);
                v /= base;
            }
            digits.reverse();
        }
        let digit_count = digits.len();
        let mut result = String::from("tu");
        if self.is_hex {
            result.push_str("ru");
        }
        for (index, &digit) in digits.iter().enumerate() {
            let exponent = (digit_count - 1 - index) as u32;
            if exponent == 0 {
                if digit != 0 || digit_count == 1 {
                    result.push_str(&Self::digit_to_syllable(digit));
                }
            } else if digit != 0 {
                result.push_str(&Self::digit_to_syllable(digit));
                result.push_str(&Self::place_to_syllable(exponent));
            }
        }
        result
    }
}
