//! 辞書(`docs/dictionary.json`)を引く。形式は docs/language_design/borrowing.md を参照。

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Entry {
    /// 借用元の英単語。語源の無い語は`None`
    pub original_spelling: Option<String>,
    /// トークナイザでの品詞(`TerminalType`のバリアント名)
    pub terminal_type: String,
    /// 計画言語自身で書いた定義文
    pub definition: Option<String>,
    pub description: String,
    /// 既存語から合成して作る表現
    #[serde(default)]
    pub compositions: Vec<Composition>,
}

#[derive(Debug, Deserialize)]
pub struct Composition {
    /// 合成した表現の意味
    pub gloss: String,
    pub expression: String,
}

static DICTIONARY: LazyLock<HashMap<String, Entry>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../docs/dictionary.json"))
        .expect("docs/dictionary.jsonの形式が想定と違います")
});

/// 見出し語の項目を返す。数詞や固有名詞など、規則で作る語は辞書に載らないので`None`。
pub fn lookup(word: &str) -> Option<&'static Entry> {
    DICTIONARY.get(word)
}

/// 全項目を返す(順序は不定)。
pub fn entries() -> impl Iterator<Item = (&'static str, &'static Entry)> {
    DICTIONARY
        .iter()
        .map(|(word, entry)| (word.as_str(), entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{get_dummy_terminal_type, tokenize};

    #[test]
    fn test_辞書を読み込んで見出し語を引ける() {
        let entry = lookup("peheta").expect("pehetaが辞書にある");
        assert_eq!(entry.original_spelling.as_deref(), Some("eat"));
        assert_eq!(entry.terminal_type, "BinaryPredicate");
        assert!(lookup("kaho").is_some_and(|e| !e.compositions.is_empty()));
        assert!(lookup("xyz").is_none());
    }

    #[test]
    fn test_辞書の品詞はトークナイザの判定と一致する() {
        let mismatches: Vec<String> = entries()
            .filter_map(|(word, entry)| {
                let actual = match tokenize(word) {
                    Ok(sentences) => match sentences.concat().as_slice() {
                        [token] => format!("{:?}", get_dummy_terminal_type(token.clone())),
                        tokens => format!("{}トークン", tokens.len()),
                    },
                    Err(e) => e.message(),
                };
                (actual != entry.terminal_type)
                    .then(|| format!("{word}: 辞書={} 実際={actual}", entry.terminal_type))
            })
            .collect();
        assert!(mismatches.is_empty(), "{mismatches:#?}");
    }
}
