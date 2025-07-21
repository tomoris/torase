#![allow(dead_code)]
#![allow(unused_variables)]

pub fn tokenize(input: &str) -> Vec<TerminalType> {
    let tokens: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

    let mut result = Vec::new();
    for token in &tokens {
        match token.as_str() {
            "?" => result.push(TerminalType::InterrogativeMarker(Some(
                InterrogativeMarker {},
            ))),
            "modal" => result.push(TerminalType::Modality(Some(Modality {}))),
            "tense" => result.push(TerminalType::Tense(Some(Tense {}))),
            "aspect" => result.push(TerminalType::Aspect(Some(Aspect {}))),
            "modality_negation_begin" => result.push(TerminalType::ModalityNegationBegin(Some(
                ModalityNegationBegin {},
            ))),
            "tense_negation_begin" => result.push(TerminalType::TenseNegationBegin(Some(
                TenseNegationBegin {},
            ))),
            "aspect_negation_begin" => result.push(TerminalType::AspectNegationBegin(Some(
                AspectNegationBegin {},
            ))),
            "negation" => result.push(TerminalType::Negation(Some(Negation {}))),
            "binary_predicate" => {
                result.push(TerminalType::BinaryPredicate(Some(BinaryPredicate {})))
            }
            "argument" => result.push(TerminalType::Argument(Some(Argument {}))),
            "argument_phrase_separator" => result.push(TerminalType::ArgumentPhraseSeparator(
                Some(ArgumentPhraseSeparator {}),
            )),
            "argument_phrase_transposer" => result.push(TerminalType::ArgumentPhraseTransposer(
                Some(ArgumentPhraseTransposer {}),
            )),
            "binary_operator" => result.push(TerminalType::BinaryOperator(Some(BinaryOperator {}))),
            "argument_retriever" => {
                result.push(TerminalType::ArgumentRetriever(Some(ArgumentRetriever {})))
            }
            "quantifier" => result.push(TerminalType::Quantifier(Some(Quantifier {}))),
            "articlizer" => result.push(TerminalType::Articlizer(Some(Articlizer {}))),
            "mta_event_to_argument_phrase_converter" => {
                result.push(TerminalType::MtaEventToArgumentPhraseConverter(Some(
                    MtaEventToArgumentPhraseConverter {},
                )))
            }
            "mta_event_to_argument_phrase_converter_end" => {
                result.push(TerminalType::MtaEventToArgumentPhraseConverterEnd(Some(
                    MtaEventToArgumentPhraseConverterEnd {},
                )))
            }
            "post_mta_event_modifier" => result.push(TerminalType::PostMtaEventModifier(Some(
                PostMtaEventModifier {},
            ))),
            "post_binary_predicate_phrase_modifier" => {
                result.push(TerminalType::PostBinaryPredicatePhraseModifier(Some(
                    PostBinaryPredicatePhraseModifier {},
                )))
            }
            "post_argument_phrase_modifier" => result.push(
                TerminalType::PostArgumentPhraseModifier(Some(PostArgumentPhraseModifier {})),
            ),
            "eos" => result.push(TerminalType::Eos(Some(Eos {}))),
            _ => panic!("Unknown"),
        }
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
pub struct InterrogativeMarker;

#[derive(Clone, Copy, Debug)]
pub struct Modality;

#[derive(Clone, Copy, Debug)]
pub struct Tense;

#[derive(Clone, Copy, Debug)]
pub struct Aspect;

#[derive(Clone, Copy, Debug)]
pub struct ModalityNegationBegin;

#[derive(Clone, Copy, Debug)]
pub struct TenseNegationBegin;

#[derive(Clone, Copy, Debug)]
pub struct AspectNegationBegin;

#[derive(Clone, Copy, Debug)]
pub struct Negation;

#[derive(Clone, Copy, Debug)]
pub struct BinaryPredicate;

#[derive(Clone, Copy, Debug)]
pub struct Argument;

#[derive(Clone, Copy, Debug)]
pub struct ArgumentPhraseSeparator;

#[derive(Clone, Copy, Debug)]
pub struct ArgumentPhraseTransposer;

#[derive(Clone, Copy, Debug)]
pub struct BinaryOperator;

#[derive(Clone, Copy, Debug)]
pub struct ArgumentRetriever;

#[derive(Clone, Copy, Debug)]
pub struct Quantifier;

#[derive(Clone, Copy, Debug)]
pub struct Articlizer;

#[derive(Clone, Copy, Debug)]
pub struct MtaEventToArgumentPhraseConverter;

#[derive(Clone, Copy, Debug)]
pub struct MtaEventToArgumentPhraseConverterEnd;

#[derive(Clone, Copy, Debug)]
pub struct PostMtaEventModifier;

#[derive(Clone, Copy, Debug)]
pub struct PostBinaryPredicatePhraseModifier;

#[derive(Clone, Copy, Debug)]
pub struct PostArgumentPhraseModifier;

#[derive(Clone, Copy, Debug)]
pub struct Eos;
