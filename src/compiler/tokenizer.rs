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
        self.range.start
    }
    pub fn end(&self) -> Position {
        self.range.end
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Vec<TerminalType>>, IncompleteToken> {
    let tokens = split_with_utf8_ranges(input);

    let mut doc = Vec::new();

    let re_modality = Regex::new(r"^(?:ha)(?:k[aiu]|m[eo]|t[eo]|s[aiu]|h[eo]|pa)$")
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
            doc.push(TerminalType::Modality(Modality {
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
        } else if re_tense.is_match(token.0) {
            doc.push(TerminalType::Tense(Tense {
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
        } else if re_aspect.is_match(token.0) {
            doc.push(TerminalType::Aspect(Aspect {
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
        } else if re_modality_negation.is_match(token.0) {
            doc.push(TerminalType::ModalityNegationBegin(ModalityNegationBegin {
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
        } else if re_tense_negation.is_match(token.0) {
            doc.push(TerminalType::TenseNegationBegin(TenseNegationBegin {
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
        } else if re_aspect_negation.is_match(token.0) {
            doc.push(TerminalType::AspectNegationBegin(AspectNegationBegin {
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
        } else if re_negation.is_match(token.0) {
            doc.push(TerminalType::Negation(Negation {
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
        } else if re_binary_predicate.is_match(token.0) {
            doc.push(TerminalType::BinaryPredicate(BinaryPredicate {
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
        } else if re_argument.is_match(token.0) {
            doc.push(TerminalType::Argument(Argument {
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
        .split_inclusive(|&x| {
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
                let end = start;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug)]
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

pub fn get_dummy_terminal_type(terminal: &TerminalType) -> DummyTerminalType {
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
            TerminalType::InterrogativeMarker(marker) => marker.range.start,
            TerminalType::Modality(modality) => modality.range.start,
            TerminalType::Tense(tense) => tense.range.start,
            TerminalType::Aspect(aspect) => aspect.range.start,
            TerminalType::ModalityNegationBegin(negation) => negation.range.start,
            TerminalType::TenseNegationBegin(negation) => negation.range.start,
            TerminalType::AspectNegationBegin(negation) => negation.range.start,
            TerminalType::Negation(negation) => negation.range.start,
            TerminalType::BinaryPredicate(predicate) => predicate.range.start,
            TerminalType::Argument(argument) => argument.range.start,
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.start,
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.start,
            TerminalType::BinaryOperator(operator) => operator.range.start,
            TerminalType::ArgumentRetriever(retriever) => retriever.range.start,
            TerminalType::Quantifier(quantifier) => quantifier.range.start,
            TerminalType::Articlizer(articlizer) => articlizer.range.start,
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => converter.range.start,
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.start
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.start,
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.range.start,
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.start,
            TerminalType::Eos(eos) => eos.range.start,
        }
    }
    pub fn end(&self) -> Position {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.range.end,
            TerminalType::Modality(modality) => modality.range.end,
            TerminalType::Tense(tense) => tense.range.end,
            TerminalType::Aspect(aspect) => aspect.range.end,
            TerminalType::ModalityNegationBegin(negation) => negation.range.end,
            TerminalType::TenseNegationBegin(negation) => negation.range.end,
            TerminalType::AspectNegationBegin(negation) => negation.range.end,
            TerminalType::Negation(negation) => negation.range.end,
            TerminalType::BinaryPredicate(predicate) => predicate.range.end,
            TerminalType::Argument(argument) => argument.range.end,
            TerminalType::ArgumentPhraseSeparator(separator) => separator.range.end,
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.range.end,
            TerminalType::BinaryOperator(operator) => operator.range.end,
            TerminalType::ArgumentRetriever(retriever) => retriever.range.end,
            TerminalType::Quantifier(quantifier) => quantifier.range.end,
            TerminalType::Articlizer(articlizer) => articlizer.range.end,
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => converter.range.end,
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.range.end
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.range.end,
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.range.end,
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.range.end,
            TerminalType::Eos(eos) => eos.range.end,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InterrogativeMarker {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modality {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tense {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aspect {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModalityNegationBegin {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TenseNegationBegin {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AspectNegationBegin {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Negation {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryPredicate {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Argument {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseSeparator {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentPhraseTransposer {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryOperator {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentRetriever {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quantifier {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Articlizer {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverter {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MtaEventToArgumentPhraseConverterEnd {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostMtaEventModifier {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostBinaryPredicatePhraseModifier {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PostArgumentPhraseModifier {
    range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eos {
    range: Range,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    start: Position,
    end: Position,
}

// 並び順は、フィールドの順番に従う。
// つまり、lineが同じならcharacterの順番に従う。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
            TerminalType::BinaryPredicate(BinaryPredicate {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0
                    },
                    end: Position {
                        line: 0,
                        character: 6
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Argument(Argument {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 7
                    },
                    end: Position {
                        line: 0,
                        character: 9
                    },
                }
            })
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
            TerminalType::Argument(Argument {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 13
                    },
                    end: Position {
                        line: 0,
                        character: 15
                    },
                }
            })
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
            TerminalType::Modality(Modality {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0
                    },
                    end: Position {
                        line: 0,
                        character: 4
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][1],
            TerminalType::Tense(Tense {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 5
                    },
                    end: Position {
                        line: 0,
                        character: 9
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][2],
            TerminalType::Aspect(Aspect {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 10
                    },
                    end: Position {
                        line: 0,
                        character: 16
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][3],
            TerminalType::BinaryPredicate(BinaryPredicate {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 17
                    },
                    end: Position {
                        line: 0,
                        character: 23
                    },
                }
            })
        );
        assert_eq!(
            sentences[0][4],
            TerminalType::Argument(Argument {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 24
                    },
                    end: Position {
                        line: 0,
                        character: 26
                    },
                }
            })
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
            TerminalType::Argument(Argument {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 30
                    },
                    end: Position {
                        line: 0,
                        character: 32
                    },
                }
            })
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
