#![allow(dead_code)]
#![allow(unused_variables)]

use core::panic;
use std::mem::discriminant;

use regex::Regex;

#[derive(Debug)]
pub struct IncompleteToken {
    pub string: String,
    pub range: Range,
}

impl IncompleteToken {
    pub fn start(&self) -> Position {
        self.range.start.clone()
    }
    pub fn end(&self) -> Position {
        self.range.end.clone()
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Vec<TerminalType>>, IncompleteToken> {
    let tokens = split_with_utf8_ranges(input);

    let mut doc = Vec::new();

    let re_modality = Regex::new(r"^(?:ha)(?:k[aiu]|m[eo]|t[eo]|s[eo]|h[eo]|pa)$")
        .map_err(|e| panic!("Failed to create regex: {}", e))?;
    let re_tense = Regex::new(r"^(?:hi)(?:t[aiu])$").unwrap();
    let re_aspect = Regex::new(r"^(?:hu)|(?:d[aeou])|(?:did[aiu])$").unwrap();
    let re_modality_negation = Regex::new(r"^(?:ha)(?:n[eo])$").unwrap();
    let re_tense_negation = Regex::new(r"^(?:hi)(?:n[eo])$").unwrap();
    let re_aspect_negation = Regex::new(r"^(?:hu)(?:n[eo])$").unwrap();
    let re_negation = Regex::new(r"^(?:n[eo])$").unwrap();
    let re_binary_predicate =
        Regex::new(r"^(?:[bdgmnr][aeiuo]|[pk]a)(?:[ptkshbdgmnr][aeiuo])+$").unwrap();
    let re_argument = Regex::new(r"^(?:m[aiue])|(?:b[aue])|(?:g[iue])$").unwrap();
    for token in &tokens {
        if token.0 == "ku" {
            doc.push(TerminalType::InterrogativeMarker(InterrogativeMarker {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if re_modality.is_match(token.0) {
            doc.push(TerminalType::Modality(Modality::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_tense.is_match(token.0) {
            doc.push(TerminalType::Tense(Tense::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_aspect.is_match(token.0) {
            doc.push(TerminalType::Aspect(Aspect::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_modality_negation.is_match(token.0) {
            doc.push(TerminalType::ModalityNegationBegin(
                ModalityNegationBegin::new(token.0, token.1, token.2, token.3, token.4),
            ));
        } else if re_tense_negation.is_match(token.0) {
            doc.push(TerminalType::TenseNegationBegin(TenseNegationBegin::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_aspect_negation.is_match(token.0) {
            doc.push(TerminalType::AspectNegationBegin(AspectNegationBegin::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_negation.is_match(token.0) {
            doc.push(TerminalType::Negation(Negation::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_binary_predicate.is_match(token.0) {
            doc.push(TerminalType::BinaryPredicate(BinaryPredicate::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if re_argument.is_match(token.0) {
            doc.push(TerminalType::Argument(Argument::new(
                token.0, token.1, token.2, token.3, token.4,
            )));
        } else if token.0 == "si" {
            doc.push(TerminalType::ArgumentPhraseSeparator(
                ArgumentPhraseSeparator {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "re" {
            doc.push(TerminalType::ArgumentPhraseTransposer(
                ArgumentPhraseTransposer {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "binary_operator" {
            doc.push(TerminalType::BinaryOperator(BinaryOperator {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if token.0 == "ro" {
            doc.push(TerminalType::ArgumentRetriever(ArgumentRetriever {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if token.0 == "quantifier" {
            doc.push(TerminalType::Quantifier(Quantifier {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if token.0 == "articlizer" {
            doc.push(TerminalType::Articlizer(Articlizer {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if token.0 == "se" {
            doc.push(TerminalType::MtaEventToArgumentPhraseConverter(
                MtaEventToArgumentPhraseConverter {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "so" {
            doc.push(TerminalType::MtaEventToArgumentPhraseConverterEnd(
                MtaEventToArgumentPhraseConverterEnd {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "da" {
            doc.push(TerminalType::PostMtaEventModifier(PostMtaEventModifier {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else if token.0 == "di" {
            doc.push(TerminalType::PostBinaryPredicatePhraseModifier(
                PostBinaryPredicatePhraseModifier {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "du" {
            doc.push(TerminalType::PostArgumentPhraseModifier(
                PostArgumentPhraseModifier {
                    range: Range {
                        start: Position {
                            line: token.1,
                            character: token.2,
                        },
                        end: Position {
                            line: token.3,
                            character: token.4,
                        },
                    },
                },
            ));
        } else if token.0 == "eos" {
            doc.push(TerminalType::Eos(Eos {
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            }));
        } else {
            return Err(IncompleteToken {
                string: token.0.to_string(),
                range: Range {
                    start: Position {
                        line: token.1,
                        character: token.2,
                    },
                    end: Position {
                        line: token.3,
                        character: token.4,
                    },
                },
            });
        }
    }

    let sentences = doc
        .split_inclusive(|x| {
            discriminant(&TerminalType::Eos(Eos::new_omitted_token())) == discriminant(&x) // eosで文分割
        })
        .map(|chunk| chunk.to_vec())
        .collect();

    Ok(sentences)
}

// 文の最後にEosトークンがない場合に追加する
pub fn add_eos_to_last_sentence(sentences: Vec<Vec<TerminalType>>) -> Vec<Vec<TerminalType>> {
    let mut sentences = sentences.clone();
    if let Some(last_sentence) = sentences.last_mut() {
        if let Some(last_token) = last_sentence.last() {
            if discriminant(&TerminalType::Eos(Eos::new_omitted_token()))
                != discriminant(last_token)
            {
                let start = if let Some(last_token) = last_sentence.last() {
                    last_token.end()
                } else {
                    Position {
                        line: 0,
                        character: 0,
                    }
                };
                let end = start.clone();
                last_sentence.push(TerminalType::Eos(Eos {
                    range: Range { start, end },
                }));
            }
        }
    }
    sentences
}

fn split_with_utf8_ranges(input: &str) -> Vec<(&str, usize, usize, usize, usize)> {
    let mut result = Vec::new();
    let mut in_word = false;
    let mut start_idx = 0;
    let mut start_row = 0;
    let mut start_col = 0;

    let mut row = 0;
    let mut col = 0;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if in_word {
                result.push((&input[start_idx..i], start_row, start_col, row, col));
                in_word = false;
            }

            if c == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        } else {
            if !in_word {
                start_idx = i;
                start_row = row;
                start_col = col;
                in_word = true;
            }
            col += 1;
        }
    }

    if in_word {
        result.push((&input[start_idx..], start_row, start_col, row, col));
    }

    result
}

// terminal types

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalType {
    InterrogativeMarker(InterrogativeMarker),
    Modality(Modality),
    Tense(Tense),
    Aspect(Aspect),
    ModalityNegationBegin(ModalityNegationBegin),
    TenseNegationBegin(TenseNegationBegin),
    AspectNegationBegin(AspectNegationBegin),
    Negation(Negation),
    BinaryPredicate(BinaryPredicate),
    Argument(Argument),
    ArgumentPhraseSeparator(ArgumentPhraseSeparator),
    ArgumentPhraseTransposer(ArgumentPhraseTransposer),
    BinaryOperator(BinaryOperator),
    ArgumentRetriever(ArgumentRetriever),
    Quantifier(Quantifier),
    Articlizer(Articlizer),
    MtaEventToArgumentPhraseConverter(MtaEventToArgumentPhraseConverter),
    MtaEventToArgumentPhraseConverterEnd(MtaEventToArgumentPhraseConverterEnd),
    PostMtaEventModifier(PostMtaEventModifier),
    PostBinaryPredicatePhraseModifier(PostBinaryPredicatePhraseModifier),
    PostArgumentPhraseModifier(PostArgumentPhraseModifier),
    Eos(Eos),
}

#[derive(Clone, Debug)]
pub enum DummyTerminalType {
    InterrogativeMarker,
    Modality,
    Tense,
    Aspect,
    ModalityNegationBegin,
    TenseNegationBegin,
    AspectNegationBegin,
    Negation,
    BinaryPredicate,
    Argument,
    ArgumentPhraseSeparator,
    ArgumentPhraseTransposer,
    BinaryOperator,
    ArgumentRetriever,
    Quantifier,
    Articlizer,
    MtaEventToArgumentPhraseConverter,
    MtaEventToArgumentPhraseConverterEnd,
    PostMtaEventModifier,
    PostBinaryPredicatePhraseModifier,
    PostArgumentPhraseModifier,
    Eos,
}

pub fn get_dummy_terminal_type(terminal: TerminalType) -> DummyTerminalType {
    match terminal {
        TerminalType::InterrogativeMarker(_) => DummyTerminalType::InterrogativeMarker,
        TerminalType::Modality(_) => DummyTerminalType::Modality,
        TerminalType::Tense(_) => DummyTerminalType::Tense,
        TerminalType::Aspect(_) => DummyTerminalType::Aspect,
        TerminalType::ModalityNegationBegin(_) => DummyTerminalType::ModalityNegationBegin,
        TerminalType::TenseNegationBegin(_) => DummyTerminalType::TenseNegationBegin,
        TerminalType::AspectNegationBegin(_) => DummyTerminalType::AspectNegationBegin,
        TerminalType::Negation(_) => DummyTerminalType::Negation,
        TerminalType::BinaryPredicate(_) => DummyTerminalType::BinaryPredicate,
        TerminalType::Argument(_) => DummyTerminalType::Argument,
        TerminalType::ArgumentPhraseSeparator(_) => DummyTerminalType::ArgumentPhraseSeparator,
        TerminalType::ArgumentPhraseTransposer(_) => DummyTerminalType::ArgumentPhraseTransposer,
        TerminalType::BinaryOperator(_) => DummyTerminalType::BinaryOperator,
        TerminalType::ArgumentRetriever(_) => DummyTerminalType::ArgumentRetriever,
        TerminalType::Quantifier(_) => DummyTerminalType::Quantifier,
        TerminalType::Articlizer(_) => DummyTerminalType::Articlizer,
        TerminalType::MtaEventToArgumentPhraseConverter(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverter
        }
        TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd
        }
        TerminalType::PostMtaEventModifier(_) => DummyTerminalType::PostMtaEventModifier,
        TerminalType::PostBinaryPredicatePhraseModifier(_) => {
            DummyTerminalType::PostBinaryPredicatePhraseModifier
        }
        TerminalType::PostArgumentPhraseModifier(_) => {
            DummyTerminalType::PostArgumentPhraseModifier
        }
        TerminalType::Eos(_) => DummyTerminalType::Eos,
    }
}

impl TerminalType {
    pub fn start(&self) -> Position {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.range.start.clone(),
            TerminalType::Modality(modality) => modality.range.start.clone(),
            TerminalType::Tense(tense) => tense.range.start.clone(),
            TerminalType::Aspect(aspect) => aspect.range.start.clone(),
            TerminalType::ModalityNegationBegin(negation) => negation.range.start.clone(),
            TerminalType::TenseNegationBegin(negation) => negation.range.start.clone(),
            TerminalType::AspectNegationBegin(negation) => negation.range.start.clone(),
            TerminalType::Negation(negation) => negation.range.start.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.start.clone(),
            TerminalType::Argument(argument) => argument.range.start.clone(),
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.start.clone(),
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.start.clone(),
            TerminalType::BinaryOperator(operator) => operator.range.start.clone(),
            TerminalType::ArgumentRetriever(retriever) => retriever.range.start.clone(),
            TerminalType::Quantifier(quantifier) => quantifier.range.start.clone(),
            TerminalType::Articlizer(articlizer) => articlizer.range.start.clone(),
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => {
                converter.range.start.clone()
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.start.clone()
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.start.clone(),
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => {
                modifier.range.start.clone()
            }
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.start.clone(),
            TerminalType::Eos(eos) => eos.range.start.clone(),
        }
    }
    pub fn end(&self) -> Position {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.range.end.clone(),
            TerminalType::Modality(modality) => modality.range.end.clone(),
            TerminalType::Tense(tense) => tense.range.end.clone(),
            TerminalType::Aspect(aspect) => aspect.range.end.clone(),
            TerminalType::ModalityNegationBegin(negation) => negation.range.end.clone(),
            TerminalType::TenseNegationBegin(negation) => negation.range.end.clone(),
            TerminalType::AspectNegationBegin(negation) => negation.range.end.clone(),
            TerminalType::Negation(negation) => negation.range.end.clone(),
            TerminalType::BinaryPredicate(predicate) => predicate.range.end.clone(),
            TerminalType::Argument(argument) => argument.range.end.clone(),
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.end.clone(),
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.end.clone(),
            TerminalType::BinaryOperator(operator) => operator.range.end.clone(),
            TerminalType::ArgumentRetriever(retriever) => retriever.range.end.clone(),
            TerminalType::Quantifier(quantifier) => quantifier.range.end.clone(),
            TerminalType::Articlizer(articlizer) => articlizer.range.end.clone(),
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => {
                converter.range.end.clone()
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.end.clone()
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.end.clone(),
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.range.end.clone(),
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.end.clone(),
            TerminalType::Eos(eos) => eos.range.end.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterrogativeMarker {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Modality {
    range: Range,
    kind: ModalityKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityKind {
    Haka, // 命令、しなさい
    Haki, // 勧誘、しませんか
    Haku, // 許可、してよい
    Hame, // 事実性高い
    Hamo, // 事実性普通
    Hasa, // 義務、すべき
    Hasi, // 許可、してよい
    Hake, // 能力、できる
    Hako, // 可能、ありうる
    Hahe, // 願望、したい
    Hata, // 意志、しよう
    Hapa, // 必然
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tense {
    range: Range,
    kind: TenseKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TenseKind {
    Hita, // 未来
    Hiti, // 現在
    Hitu, // 過去
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aspect {
    range: Range,
    kind: AspectKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AspectKind {
    Huda,   // 未然
    Hude,   // 開始
    Hudida, // 進行
    Hudidi, // 習慣
    Hudidu, // 反復
    Hudo,   // 終了
    Hudu,   // 事後
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalityNegationBegin {
    range: Range,
    kind: ModalityNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ModalityNegationKind {
    Hane, // 否定
    Hano, // 正反対
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenseNegationBegin {
    range: Range,
    kind: TenseNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TenseNegationKind {
    Hine, // 否定
    Hino, // 正反対
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AspectNegationBegin {
    range: Range,
    kind: AspectNegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AspectNegationKind {
    Hune, // 否定
    Huno, // 正反対
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Negation {
    range: Range,
    kind: NegationKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NegationKind {
    Ne, // 否定
    No, // 正反対
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryPredicate {
    range: Range,
    string: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Argument {
    range: Range,
    kind: ArgumentKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ArgumentKind {
    Ma, // 私たち(あなたを含む)
    Mi, // 私
    Mu, // 私たち(あなたを含まない)
    Me, // これ
    Ba, // あなた
    Bu, // あなたたち
    Be, // それ
    Gi, // 彼、彼女
    Gu, // 彼ら
    Ge, // あれ
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseSeparator {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseTransposer {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryOperator {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgumentRetriever {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quantifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Articlizer {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverter {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverterEnd {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostMtaEventModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostBinaryPredicatePhraseModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostArgumentPhraseModifier {
    range: Range,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eos {
    range: Range,
}

impl Modality {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "haka" => ModalityKind::Haka,
            "haki" => ModalityKind::Haki,
            "haku" => ModalityKind::Haku,
            "hame" => ModalityKind::Hame,
            "hamo" => ModalityKind::Hamo,
            "hasa" => ModalityKind::Hasa,
            "hasi" => ModalityKind::Hasi,
            "hake" => ModalityKind::Hake,
            "hako" => ModalityKind::Hako,
            "hahe" => ModalityKind::Hahe,
            "hata" => ModalityKind::Hata,
            "hapa" => ModalityKind::Hapa,
            _ => panic!("Invalid modality surface: {}", string),
        };
        Modality {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl Tense {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hita" => TenseKind::Hita,
            "hiti" => TenseKind::Hiti,
            "hitu" => TenseKind::Hitu,
            _ => panic!("Invalid tense surface: {}", string),
        };
        Tense {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl Aspect {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "huda" => AspectKind::Huda,
            "hude" => AspectKind::Hude,
            "hudida" => AspectKind::Hudida,
            "hudidi" => AspectKind::Hudidi,
            "hudidu" => AspectKind::Hudidu,
            "hudo" => AspectKind::Hudo,
            "hudu" => AspectKind::Hudu,
            _ => panic!("Invalid aspect surface: {}", string),
        };
        Aspect {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl ModalityNegationBegin {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hane" => ModalityNegationKind::Hane,
            "hano" => ModalityNegationKind::Hano,
            _ => panic!("Invalid modality negation surface: {}", string),
        };
        ModalityNegationBegin {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl TenseNegationBegin {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hine" => TenseNegationKind::Hine,
            "hino" => TenseNegationKind::Hino,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        TenseNegationBegin {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl AspectNegationBegin {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "hune" => AspectNegationKind::Hune,
            "huno" => AspectNegationKind::Huno,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        AspectNegationBegin {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl Negation {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ne" => NegationKind::Ne,
            "no" => NegationKind::No,
            _ => panic!("Invalid tense negation surface: {}", string),
        };
        Negation {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl BinaryPredicate {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        BinaryPredicate {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            string: string.to_string(),
        }
    }
}

impl Argument {
    pub fn new(
        string: &str,
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> Self {
        let kind = match string {
            "ma" => ArgumentKind::Ma,
            "mi" => ArgumentKind::Mi,
            "mu" => ArgumentKind::Mu,
            "me" => ArgumentKind::Me,
            "ba" => ArgumentKind::Ba,
            "bu" => ArgumentKind::Bu,
            "be" => ArgumentKind::Be,
            "gi" => ArgumentKind::Gi,
            "gu" => ArgumentKind::Gu,
            "ge" => ArgumentKind::Ge,
            _ => panic!("Invalid argument surface: {}", string),
        };
        Argument {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_character,
                },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            },
            kind,
        }
    }
}

impl Eos {
    pub fn new_omitted_token() -> Self {
        Eos {
            range: Range {
                start: Position {
                    line: usize::MAX,
                    character: usize::MAX,
                },
                end: Position {
                    line: usize::MAX,
                    character: usize::MAX,
                },
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    start: Position,
    end: Position,
}

// 並び順は、フィールドの順番に従う。
// つまり、lineが同じならcharacterの順番に従う。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    line: usize,
    character: usize,
}

impl Position {
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn character(&self) -> usize {
        self.character
    }
    pub fn diff_length(&self, other: Position) -> usize {
        if self.line == other.line {
            self.character.abs_diff(other.character)
        } else {
            0 // 異なる行の場合は0を返す（意味のある差分を計算できないため）
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_with_utf8_ranges() {
        let input = "binary_predicate argument argument_phrase_separator argument eos\nbinary_predicate argument argument_phrase_separator argument eos";
        let tokens = split_with_utf8_ranges(input);
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0], ("binary_predicate", 0, 0, 0, 16));
        assert_eq!(tokens[1], ("argument", 0, 17, 0, 25));
        assert_eq!(tokens[2], ("argument_phrase_separator", 0, 26, 0, 51));
        assert_eq!(tokens[3], ("argument", 0, 52, 0, 60));
        assert_eq!(tokens[4], ("eos", 0, 61, 0, 64));
        assert_eq!(tokens[5], ("binary_predicate", 1, 0, 1, 16));
        assert_eq!(tokens[6], ("argument", 1, 17, 1, 25));
        assert_eq!(tokens[7], ("argument_phrase_separator", 1, 26, 1, 51));
        assert_eq!(tokens[8], ("argument", 1, 52, 1, 60));
        assert_eq!(tokens[9], ("eos", 1, 61, 1, 64));
    }

    #[test]
    fn test_1文のトークン分割が成功する() {
        let input = "parobe ma si ba eos";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(
            sentences[0][0],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 0, 0, 6))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Argument(Argument::new("ma", 0, 7, 0, 9))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::ArgumentPhraseSeparator(ArgumentPhraseSeparator {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 10
                    },
                    end: Position {
                        line: 0,
                        character: 12
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::Argument(Argument::new("ba", 0, 13, 0, 15))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::Eos(Eos {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 16
                    },
                    end: Position {
                        line: 0,
                        character: 19
                    },
                }
            })
        );
    }

    #[test]
    fn test_eosなし1文のトークン分割が成功する() {
        let input = "hahe hita hudidi parobe me si gi";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 8);
        assert_eq!(
            sentences[0][0],
            TerminalType::Modality(Modality::new("hahe", 0, 0, 0, 4))
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Tense(Tense::new("hita", 0, 5, 0, 9))
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::Aspect(Aspect::new("hudidi", 0, 10, 0, 16))
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::BinaryPredicate(BinaryPredicate::new("parobe", 0, 17, 0, 23))
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::Argument(Argument::new("me", 0, 24, 0, 26))
        );
        assert_eq!(
            sentences[0][5],
            TerminalType::ArgumentPhraseSeparator(ArgumentPhraseSeparator {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 27
                    },
                    end: Position {
                        line: 0,
                        character: 29
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][6],
            TerminalType::Argument(Argument::new("gi", 0, 30, 0, 32))
        );
    }

    #[test]
    fn test_2文のトークン分割が成功する() {
        let input = "parobe me si ba eos ro parobe bu si gu";
        let sentences = tokenize(input).unwrap();
        let sentences = add_eos_to_last_sentence(sentences);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(sentences[1].len(), 6);
    }
}
