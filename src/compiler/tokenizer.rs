#![allow(dead_code)]
#![allow(unused_variables)]

use core::panic;
use std::mem::discriminant;

pub fn tokenize(input: &str) -> Vec<Vec<TerminalType>> {
    let tokens = split_with_utf8_ranges(input);

    let mut doc = Vec::new();
    for token in &tokens {
        match token.0 {
            "?" => doc.push(TerminalType::InterrogativeMarker(InterrogativeMarker {
                start: token.1,
                end: token.2,
            })),
            "modal" => doc.push(TerminalType::Modality(Modality {
                start: token.1,
                end: token.2,
            })),
            "tense" => doc.push(TerminalType::Tense(Tense {
                start: token.1,
                end: token.2,
            })),
            "aspect" => doc.push(TerminalType::Aspect(Aspect {
                start: token.1,
                end: token.2,
            })),
            "modality_negation_begin" => {
                doc.push(TerminalType::ModalityNegationBegin(ModalityNegationBegin {
                    start: token.1,
                    end: token.2,
                }))
            }
            "tense_negation_begin" => {
                doc.push(TerminalType::TenseNegationBegin(TenseNegationBegin {
                    start: token.1,
                    end: token.2,
                }))
            }
            "aspect_negation_begin" => {
                doc.push(TerminalType::AspectNegationBegin(AspectNegationBegin {
                    start: token.1,
                    end: token.2,
                }))
            }
            "negation" => doc.push(TerminalType::Negation(Negation {
                start: token.1,
                end: token.2,
            })),
            "binary_predicate" => doc.push(TerminalType::BinaryPredicate(BinaryPredicate {
                start: token.1,
                end: token.2,
            })),
            "argument" => doc.push(TerminalType::Argument(Argument {
                start: token.1,
                end: token.2,
            })),
            "argument_phrase_separator" => doc.push(TerminalType::ArgumentPhraseSeparator(
                ArgumentPhraseSeparator {
                    start: token.1,
                    end: token.2,
                },
            )),
            "argument_phrase_transposer" => doc.push(TerminalType::ArgumentPhraseTransposer(
                ArgumentPhraseTransposer {
                    start: token.1,
                    end: token.2,
                },
            )),
            "binary_operator" => doc.push(TerminalType::BinaryOperator(BinaryOperator {
                start: token.1,
                end: token.2,
            })),
            "argument_retriever" => doc.push(TerminalType::ArgumentRetriever(ArgumentRetriever {
                start: token.1,
                end: token.2,
            })),
            "quantifier" => doc.push(TerminalType::Quantifier(Quantifier {
                start: token.1,
                end: token.2,
            })),
            "articlizer" => doc.push(TerminalType::Articlizer(Articlizer {
                start: token.1,
                end: token.2,
            })),
            "mta_event_to_argument_phrase_converter" => {
                doc.push(TerminalType::MtaEventToArgumentPhraseConverter(
                    MtaEventToArgumentPhraseConverter {
                        start: token.1,
                        end: token.2,
                    },
                ))
            }
            "mta_event_to_argument_phrase_converter_end" => {
                doc.push(TerminalType::MtaEventToArgumentPhraseConverterEnd(
                    MtaEventToArgumentPhraseConverterEnd {
                        start: token.1,
                        end: token.2,
                    },
                ))
            }
            "post_mta_event_modifier" => {
                doc.push(TerminalType::PostMtaEventModifier(PostMtaEventModifier {
                    start: token.1,
                    end: token.2,
                }))
            }
            "post_binary_predicate_phrase_modifier" => {
                doc.push(TerminalType::PostBinaryPredicatePhraseModifier(
                    PostBinaryPredicatePhraseModifier {
                        start: token.1,
                        end: token.2,
                    },
                ))
            }
            "post_argument_phrase_modifier" => doc.push(TerminalType::PostArgumentPhraseModifier(
                PostArgumentPhraseModifier {
                    start: token.1,
                    end: token.2,
                },
            )),
            "eos" => doc.push(TerminalType::Eos(Eos {
                start: token.1,
                end: token.2,
            })),
            _ => panic!("Unknown"),
        }
    }

    doc.split_inclusive(|&x| {
        discriminant(&TerminalType::Eos(Eos::new_omitted_token())) == discriminant(&x) // eosで文分割
    })
    .map(|chunk| chunk.to_vec())
    .collect()
}

fn split_with_utf8_ranges(input: &str) -> Vec<(&str, usize, usize)> {
    let mut result = Vec::new();
    let mut in_word = false;
    let mut start = 0;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if in_word {
                result.push((&input[start..i], start, i)); // UTF-8 バイト範囲
                in_word = false;
            }
        } else if !in_word {
            start = i;
            in_word = true;
        }
    }

    if in_word {
        result.push((&input[start..], start, input.len()));
    }

    result
}

// terminal types

#[derive(Clone, Copy, Debug)]
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
    pub fn start(&self) -> usize {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.start,
            TerminalType::Modality(modality) => modality.start,
            TerminalType::Tense(tense) => tense.start,
            TerminalType::Aspect(aspect) => aspect.start,
            TerminalType::ModalityNegationBegin(negation) => negation.start,
            TerminalType::TenseNegationBegin(negation) => negation.start,
            TerminalType::AspectNegationBegin(negation) => negation.start,
            TerminalType::Negation(negation) => negation.start,
            TerminalType::BinaryPredicate(predicate) => predicate.start,
            TerminalType::Argument(argument) => argument.start,
            TerminalType::ArgumentPhraseSeparator(separator) => separator.start,
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.start,
            TerminalType::BinaryOperator(operator) => operator.start,
            TerminalType::ArgumentRetriever(retriever) => retriever.start,
            TerminalType::Quantifier(quantifier) => quantifier.start,
            TerminalType::Articlizer(articlizer) => articlizer.start,
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => converter.start,
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => {
                converter_end.start
            }
            TerminalType::PostMtaEventModifier(modifier) => modifier.start,
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.start,
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.start,
            TerminalType::Eos(eos) => eos.start,
        }
    }
    pub fn end(&self) -> usize {
        match self {
            TerminalType::InterrogativeMarker(marker) => marker.end,
            TerminalType::Modality(modality) => modality.end,
            TerminalType::Tense(tense) => tense.end,
            TerminalType::Aspect(aspect) => aspect.end,
            TerminalType::ModalityNegationBegin(negation) => negation.end,
            TerminalType::TenseNegationBegin(negation) => negation.end,
            TerminalType::AspectNegationBegin(negation) => negation.end,
            TerminalType::Negation(negation) => negation.end,
            TerminalType::BinaryPredicate(predicate) => predicate.end,
            TerminalType::Argument(argument) => argument.end,
            TerminalType::ArgumentPhraseSeparator(separator) => separator.end,
            TerminalType::ArgumentPhraseTransposer(transposer) => transposer.end,
            TerminalType::BinaryOperator(operator) => operator.end,
            TerminalType::ArgumentRetriever(retriever) => retriever.end,
            TerminalType::Quantifier(quantifier) => quantifier.end,
            TerminalType::Articlizer(articlizer) => articlizer.end,
            TerminalType::MtaEventToArgumentPhraseConverter(converter) => converter.end,
            TerminalType::MtaEventToArgumentPhraseConverterEnd(converter_end) => converter_end.end,
            TerminalType::PostMtaEventModifier(modifier) => modifier.end,
            TerminalType::PostBinaryPredicatePhraseModifier(modifier) => modifier.end,
            TerminalType::PostArgumentPhraseModifier(modifier) => modifier.end,
            TerminalType::Eos(eos) => eos.end,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InterrogativeMarker {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Modality {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Tense {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Aspect {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ModalityNegationBegin {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct TenseNegationBegin {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct AspectNegationBegin {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Negation {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct BinaryPredicate {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Argument {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ArgumentPhraseSeparator {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ArgumentPhraseTransposer {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct BinaryOperator {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ArgumentRetriever {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Quantifier {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Articlizer {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct MtaEventToArgumentPhraseConverter {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct MtaEventToArgumentPhraseConverterEnd {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct PostMtaEventModifier {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct PostBinaryPredicatePhraseModifier {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct PostArgumentPhraseModifier {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Eos {
    start: usize,
    end: usize,
}

impl Eos {
    pub fn new_omitted_token() -> Self {
        Eos {
            start: usize::MAX,
            end: usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1文のトークン分割が成功する() {
        let input = "binary_predicate argument argument_phrase_separator argument eos";
        let sentences = tokenize(input);
        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].len(), 5);
    }

    #[test]
    fn test_2文のトークン分割が成功する() {
        let input = "binary_predicate argument argument_phrase_separator argument eos binary_predicate argument argument_phrase_separator argument eos";
        let sentences = tokenize(input);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].len(), 5);
        assert_eq!(sentences[1].len(), 5);
    }
}
