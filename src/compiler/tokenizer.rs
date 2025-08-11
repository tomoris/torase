#![allow(dead_code)]
#![allow(unused_variables)]

pub fn tokenize(input: &str) -> Vec<TerminalType> {
    let tokens = split_with_utf8_ranges(input);

    let mut result = Vec::new();
    for token in &tokens {
        match token.0 {
            "?" => result.push(TerminalType::InterrogativeMarker(Some(
                InterrogativeMarker {
                    start: token.1,
                    end: token.2,
                },
            ))),
            "modal" => result.push(TerminalType::Modality(Some(Modality {
                start: token.1,
                end: token.2,
            }))),
            "tense" => result.push(TerminalType::Tense(Some(Tense {
                start: token.1,
                end: token.2,
            }))),
            "aspect" => result.push(TerminalType::Aspect(Some(Aspect {
                start: token.1,
                end: token.2,
            }))),
            "modality_negation_begin" => result.push(TerminalType::ModalityNegationBegin(Some(
                ModalityNegationBegin {
                    start: token.1,
                    end: token.2,
                },
            ))),
            "tense_negation_begin" => {
                result.push(TerminalType::TenseNegationBegin(Some(TenseNegationBegin {
                    start: token.1,
                    end: token.2,
                })))
            }
            "aspect_negation_begin" => result.push(TerminalType::AspectNegationBegin(Some(
                AspectNegationBegin {
                    start: token.1,
                    end: token.2,
                },
            ))),
            "negation" => result.push(TerminalType::Negation(Some(Negation {
                start: token.1,
                end: token.2,
            }))),
            "binary_predicate" => {
                result.push(TerminalType::BinaryPredicate(Some(BinaryPredicate {
                    start: token.1,
                    end: token.2,
                })))
            }
            "argument" => result.push(TerminalType::Argument(Some(Argument {
                start: token.1,
                end: token.2,
            }))),
            "argument_phrase_separator" => result.push(TerminalType::ArgumentPhraseSeparator(
                Some(ArgumentPhraseSeparator {
                    start: token.1,
                    end: token.2,
                }),
            )),
            "argument_phrase_transposer" => result.push(TerminalType::ArgumentPhraseTransposer(
                Some(ArgumentPhraseTransposer {
                    start: token.1,
                    end: token.2,
                }),
            )),
            "binary_operator" => result.push(TerminalType::BinaryOperator(Some(BinaryOperator {
                start: token.1,
                end: token.2,
            }))),
            "argument_retriever" => {
                result.push(TerminalType::ArgumentRetriever(Some(ArgumentRetriever {
                    start: token.1,
                    end: token.2,
                })))
            }
            "quantifier" => result.push(TerminalType::Quantifier(Some(Quantifier {
                start: token.1,
                end: token.2,
            }))),
            "articlizer" => result.push(TerminalType::Articlizer(Some(Articlizer {
                start: token.1,
                end: token.2,
            }))),
            "mta_event_to_argument_phrase_converter" => {
                result.push(TerminalType::MtaEventToArgumentPhraseConverter(Some(
                    MtaEventToArgumentPhraseConverter {
                        start: token.1,
                        end: token.2,
                    },
                )))
            }
            "mta_event_to_argument_phrase_converter_end" => {
                result.push(TerminalType::MtaEventToArgumentPhraseConverterEnd(Some(
                    MtaEventToArgumentPhraseConverterEnd {
                        start: token.1,
                        end: token.2,
                    },
                )))
            }
            "post_mta_event_modifier" => result.push(TerminalType::PostMtaEventModifier(Some(
                PostMtaEventModifier {
                    start: token.1,
                    end: token.2,
                },
            ))),
            "post_binary_predicate_phrase_modifier" => {
                result.push(TerminalType::PostBinaryPredicatePhraseModifier(Some(
                    PostBinaryPredicatePhraseModifier {
                        start: token.1,
                        end: token.2,
                    },
                )))
            }
            "post_argument_phrase_modifier" => result.push(
                TerminalType::PostArgumentPhraseModifier(Some(PostArgumentPhraseModifier {
                    start: token.1,
                    end: token.2,
                })),
            ),
            "eos" => result.push(TerminalType::Eos(Some(Eos {
                start: token.1,
                end: token.2,
            }))),
            _ => panic!("Unknown"),
        }
    }

    result
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
    InterrogativeMarker(Option<InterrogativeMarker>),
    Modality(Option<Modality>),
    Tense(Option<Tense>),
    Aspect(Option<Aspect>),
    ModalityNegationBegin(Option<ModalityNegationBegin>),
    TenseNegationBegin(Option<TenseNegationBegin>),
    AspectNegationBegin(Option<AspectNegationBegin>),
    Negation(Option<Negation>),
    BinaryPredicate(Option<BinaryPredicate>),
    Argument(Option<Argument>),
    ArgumentPhraseSeparator(Option<ArgumentPhraseSeparator>),
    ArgumentPhraseTransposer(Option<ArgumentPhraseTransposer>),
    BinaryOperator(Option<BinaryOperator>),
    ArgumentRetriever(Option<ArgumentRetriever>),
    Quantifier(Option<Quantifier>),
    Articlizer(Option<Articlizer>),
    MtaEventToArgumentPhraseConverter(Option<MtaEventToArgumentPhraseConverter>),
    MtaEventToArgumentPhraseConverterEnd(Option<MtaEventToArgumentPhraseConverterEnd>),
    PostMtaEventModifier(Option<PostMtaEventModifier>),
    PostBinaryPredicatePhraseModifier(Option<PostBinaryPredicatePhraseModifier>),
    PostArgumentPhraseModifier(Option<PostArgumentPhraseModifier>),
    Eos(Option<Eos>),
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
