#![allow(dead_code)]
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]

mod grammar;

use core::panic;
use std::mem::discriminant;

use crate::compiler::tokenizer::*;

pub fn parse(tokens: Vec<TerminalType>) -> Result<Ast, IncompleteAst> {
    let mut stack: Vec<DummySymbolType> = vec![
        DummySymbolType::DummyTerminal(DummyTerminalType::Eos),
        DummySymbolType::DummyNonTerminal(DummyNonTerminalType::S),
    ];
    let mut current_token_index = 0;

    let top = parse_node(&mut stack, &tokens, &mut current_token_index)?;
    if let SymbolType::NonTerminal(NonTerminalType::S(s)) = top {
        let eos = parse_node(&mut stack, &tokens, &mut current_token_index)?;
        if let SymbolType::Terminal(TerminalType::Eos(inner_eos)) = eos {
            if stack.is_empty() && current_token_index == tokens.len() {
                Ok(Ast { s })
            } else {
                Err(IncompleteAst {
                    error_token: TerminalType::Eos(inner_eos),
                    error_top: ErrorTop::SymbolType(SymbolType::NonTerminal(NonTerminalType::S(s))),
                    error_message: format!(
                        "Expected end of stream, but stack is not empty or current token index is not at the end: stack: {stack:?}, current_token_index: {current_token_index:}",
                    ),
                })
            }
        } else {
            Err(IncompleteAst {
                error_token: TerminalType::Eos(Eos::new_omitted_token()),
                error_top: ErrorTop::SymbolType(SymbolType::NonTerminal(NonTerminalType::S(s))),
                error_message: format!("Expected Eos as the last token, found: {eos:?}"),
            })
        }
    } else {
        Err(IncompleteAst {
            error_token: TerminalType::Eos(Eos::new_omitted_token()),
            error_top: ErrorTop::SymbolType(top.clone()),
            error_message: format!("Expected S as the root non-terminal, found: {top:?}"),
        })
    }
}

fn parse_node(
    stack: &mut Vec<DummySymbolType>,
    tokens: &Vec<TerminalType>,
    current_token_index: &mut usize,
) -> Result<SymbolType, IncompleteAst> {
    let current_token = tokens.get(*current_token_index).unwrap();
    let top_symbol = stack.pop().unwrap();

    match top_symbol {
        DummySymbolType::DummyTerminal(ref terminal) => match terminal {
            DummyTerminalType::Eos => {
                if discriminant(&TerminalType::Eos(Eos::new_omitted_token()))
                    == discriminant(current_token)
                {
                    *current_token_index += 1;
                    Ok(SymbolType::Terminal(current_token.clone()))
                } else {
                    Err(IncompleteAst {
                        error_token: current_token.clone(),
                        error_top: ErrorTop::DummySymbolType(top_symbol),
                        error_message: "Expected end of stream".to_string(),
                    })
                }
            }
            _ => {
                if discriminant(terminal)
                    == discriminant(&get_dummy_terminal_type(current_token.clone()))
                {
                    *current_token_index += 1;
                    Ok(SymbolType::Terminal(current_token.clone()))
                } else {
                    Err(IncompleteAst {
                        error_token: current_token.clone(),
                        error_top: ErrorTop::DummySymbolType(top_symbol.clone()),
                        error_message: format!(
                            "Expected token: {terminal:?}, found: {current_token:?}"
                        ),
                    })
                }
            }
        },
        DummySymbolType::DummyNonTerminal(dummy_non_terminal) => {
            let dummy_rhs = parse_table(dummy_non_terminal.clone(), get_dummy_terminal_type(current_token.clone())).map_err(|_| IncompleteAst {
                error_token: current_token.clone(),
                error_top: ErrorTop::DummySymbolType(DummySymbolType::DummyNonTerminal(dummy_non_terminal.clone())),
                error_message: format!(
                    "No production rule for non-terminal: {dummy_non_terminal:?} with token: {current_token:?}"
                ),
            })?;
            match dummy_rhs {
                DummyRhs::Sequence(ref symbols) => {
                    for symbol in symbols.iter().rev() {
                        stack.push(symbol.clone());
                    }
                    let mut child_nodes = Vec::new();
                    for _ in symbols.iter() {
                        match parse_node(stack, tokens, current_token_index) {
                            Ok(child_node) => {
                                child_nodes.push(child_node);
                            }
                            Err(e) => {
                                return Err(IncompleteAst {
                                    error_token: current_token.clone(),
                                    error_top: ErrorTop::DummySymbolType(
                                        DummySymbolType::DummyNonTerminal(dummy_non_terminal),
                                    ),
                                    error_message: format!("Failed to parse child node: {e:?}"),
                                });
                            }
                        }
                    }
                    Ok(SymbolType::NonTerminal(new_non_terminal(
                        &dummy_non_terminal,
                        Rhs::Sequence(child_nodes),
                    )))
                }
                DummyRhs::Epsilon => Ok(SymbolType::NonTerminal(new_non_terminal(
                    &dummy_non_terminal,
                    Rhs::Epsilon,
                ))),
            }
        }
    }
}

fn new_non_terminal(dummy_non_terminal: &DummyNonTerminalType, rhs: Rhs) -> NonTerminalType {
    match dummy_non_terminal {
        DummyNonTerminalType::S => NonTerminalType::S(S::new(rhs)),
        DummyNonTerminalType::DeclarativeSentence => {
            NonTerminalType::DeclarativeSentence(DeclarativeSentence::new(rhs))
        }
        DummyNonTerminalType::InterrogativeSentence => {
            NonTerminalType::InterrogativeSentence(InterrogativeSentence::new(rhs))
        }
        DummyNonTerminalType::PostModifiedMtaEvent => {
            NonTerminalType::PostModifiedMtaEvent(PostModifiedMtaEvent::new(rhs))
        }
        DummyNonTerminalType::PostMtaEventExpression => {
            NonTerminalType::PostMtaEventExpression(PostMtaEventExpression::new(rhs))
        }
        DummyNonTerminalType::ModalityExpression => {
            NonTerminalType::ModalityExpression(ModalityExpression::new(rhs))
        }
        DummyNonTerminalType::ModalityWithAgent => {
            NonTerminalType::ModalityWithAgent(ModalityWithAgent::new(rhs))
        }
        DummyNonTerminalType::ModifiedModality => {
            NonTerminalType::ModifiedModality(ModifiedModality::new(rhs))
        }
        DummyNonTerminalType::ModalityAgent => {
            NonTerminalType::ModalityAgent(ModalityAgent::new(rhs))
        }
        DummyNonTerminalType::TenseExpression => {
            NonTerminalType::TenseExpression(TenseExpression::new(rhs))
        }
        DummyNonTerminalType::ModifiedTense => {
            NonTerminalType::ModifiedTense(ModifiedTense::new(rhs))
        }
        DummyNonTerminalType::AspectExpression => {
            NonTerminalType::AspectExpression(AspectExpression::new(rhs))
        }
        DummyNonTerminalType::ModifiedAspect => {
            NonTerminalType::ModifiedAspect(ModifiedAspect::new(rhs))
        }
        DummyNonTerminalType::MtaEvent => NonTerminalType::MtaEvent(MtaEvent::new(rhs)),
        DummyNonTerminalType::TaEvent => NonTerminalType::TaEvent(TaEvent::new(rhs)),
        DummyNonTerminalType::AEvent => NonTerminalType::AEvent(AEvent::new(rhs)),
        DummyNonTerminalType::Event => NonTerminalType::Event(Event::new(rhs)),
        DummyNonTerminalType::PostModifiedBinaryPredicatePhrase => {
            NonTerminalType::PostModifiedBinaryPredicatePhrase(
                PostModifiedBinaryPredicatePhrase::new(rhs),
            )
        }
        DummyNonTerminalType::PostBinaryPredicatePhraseExpression => {
            NonTerminalType::PostBinaryPredicatePhraseExpression(
                PostBinaryPredicatePhraseExpression::new(rhs),
            )
        }
        DummyNonTerminalType::BinaryPredicatePhrase => {
            NonTerminalType::BinaryPredicatePhrase(BinaryPredicatePhrase::new(rhs))
        }
        DummyNonTerminalType::ArgumentPhrase => {
            NonTerminalType::ArgumentPhrase(ArgumentPhrase::new(rhs))
        }
        DummyNonTerminalType::ArgumentPhraseOrEpsilon => {
            NonTerminalType::ArgumentPhraseOrEpsilon(ArgumentPhraseOrEpsilon::new(rhs))
        }
        DummyNonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => {
            NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(
                DeterminedArgumentRetrievalBinaryPredicate::new(rhs),
            )
        }
        DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => {
            NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(
                ArticlizedArgumentRetrievalBinaryPredicate::new(rhs),
            )
        }
        DummyNonTerminalType::ArgumentRetrievalBinaryPredicate => {
            NonTerminalType::ArgumentRetrievalBinaryPredicate(
                ArgumentRetrievalBinaryPredicate::new(rhs),
            )
        }
        DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase => {
            NonTerminalType::ArgumentRetrievalWithArgumentPhrase(
                ArgumentRetrievalWithArgumentPhrase::new(rhs),
            )
        }
        DummyNonTerminalType::Quantification => {
            NonTerminalType::Quantification(Quantification::new(rhs))
        }
        DummyNonTerminalType::Article => NonTerminalType::Article(Article::new(rhs)),
    }
}

fn parse_table(nt: DummyNonTerminalType, t: DummyTerminalType) -> Result<DummyRhs, ()> {
    match nt {
        DummyNonTerminalType::S => S::parse_table(t),
        DummyNonTerminalType::DeclarativeSentence => DeclarativeSentence::parse_table(t),
        DummyNonTerminalType::InterrogativeSentence => InterrogativeSentence::parse_table(t),
        DummyNonTerminalType::PostModifiedMtaEvent => PostModifiedMtaEvent::parse_table(t),
        DummyNonTerminalType::PostMtaEventExpression => PostMtaEventExpression::parse_table(t),
        DummyNonTerminalType::ModalityExpression => ModalityExpression::parse_table(t),
        DummyNonTerminalType::ModalityWithAgent => ModalityWithAgent::parse_table(t),
        DummyNonTerminalType::ModifiedModality => ModifiedModality::parse_table(t),
        DummyNonTerminalType::ModalityAgent => ModalityAgent::parse_table(t),
        DummyNonTerminalType::TenseExpression => TenseExpression::parse_table(t),
        DummyNonTerminalType::ModifiedTense => ModifiedTense::parse_table(t),
        DummyNonTerminalType::AspectExpression => AspectExpression::parse_table(t),
        DummyNonTerminalType::ModifiedAspect => ModifiedAspect::parse_table(t),
        DummyNonTerminalType::MtaEvent => MtaEvent::parse_table(t),
        DummyNonTerminalType::TaEvent => TaEvent::parse_table(t),
        DummyNonTerminalType::AEvent => AEvent::parse_table(t),
        DummyNonTerminalType::Event => Event::parse_table(t),
        DummyNonTerminalType::PostModifiedBinaryPredicatePhrase => {
            PostModifiedBinaryPredicatePhrase::parse_table(t)
        }
        DummyNonTerminalType::PostBinaryPredicatePhraseExpression => {
            PostBinaryPredicatePhraseExpression::parse_table(t)
        }
        DummyNonTerminalType::BinaryPredicatePhrase => BinaryPredicatePhrase::parse_table(t),
        DummyNonTerminalType::ArgumentPhrase => ArgumentPhrase::parse_table(t),
        DummyNonTerminalType::ArgumentPhraseOrEpsilon => ArgumentPhraseOrEpsilon::parse_table(t),
        DummyNonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => {
            DeterminedArgumentRetrievalBinaryPredicate::parse_table(t)
        }
        DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => {
            ArticlizedArgumentRetrievalBinaryPredicate::parse_table(t)
        }
        DummyNonTerminalType::ArgumentRetrievalBinaryPredicate => {
            ArgumentRetrievalBinaryPredicate::parse_table(t)
        }
        DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase => {
            ArgumentRetrievalWithArgumentPhrase::parse_table(t)
        }
        DummyNonTerminalType::Quantification => Quantification::parse_table(t),
        DummyNonTerminalType::Article => Article::parse_table(t),
    }
}

#[derive(Clone, Debug)]
enum Rhs {
    Sequence(Vec<SymbolType>),
    Epsilon,
}

#[derive(Clone, Debug)]
enum DummyRhs {
    Sequence(Vec<DummySymbolType>),
    Epsilon,
}

#[derive(Debug)]
pub struct Ast {
    s: S,
}

#[derive(Debug)]
pub struct IncompleteAst {
    pub error_token: TerminalType,
    pub error_top: ErrorTop,
    pub error_message: String,
}

#[derive(Debug)]
pub enum ErrorTop {
    SymbolType(SymbolType),
    DummySymbolType(DummySymbolType),
}

#[derive(Clone, Debug)]
pub enum SymbolType {
    Terminal(TerminalType),
    NonTerminal(NonTerminalType),
}

#[derive(Clone, Debug)]
pub enum DummySymbolType {
    DummyTerminal(DummyTerminalType),
    DummyNonTerminal(DummyNonTerminalType),
}

#[derive(Clone, Debug)]
pub enum NonTerminalType {
    S(S),
    DeclarativeSentence(DeclarativeSentence),
    InterrogativeSentence(InterrogativeSentence),
    PostModifiedMtaEvent(PostModifiedMtaEvent),
    PostMtaEventExpression(PostMtaEventExpression),
    ModalityExpression(ModalityExpression),
    ModalityWithAgent(ModalityWithAgent),
    ModifiedModality(ModifiedModality),
    ModalityAgent(ModalityAgent),
    TenseExpression(TenseExpression),
    ModifiedTense(ModifiedTense),
    AspectExpression(AspectExpression),
    ModifiedAspect(ModifiedAspect),
    MtaEvent(MtaEvent),
    TaEvent(TaEvent),
    AEvent(AEvent),
    Event(Event),
    PostModifiedBinaryPredicatePhrase(PostModifiedBinaryPredicatePhrase),
    PostBinaryPredicatePhraseExpression(PostBinaryPredicatePhraseExpression),
    BinaryPredicatePhrase(BinaryPredicatePhrase),
    ArgumentPhrase(ArgumentPhrase),
    ArgumentPhraseOrEpsilon(ArgumentPhraseOrEpsilon),
    DeterminedArgumentRetrievalBinaryPredicate(DeterminedArgumentRetrievalBinaryPredicate),
    ArticlizedArgumentRetrievalBinaryPredicate(ArticlizedArgumentRetrievalBinaryPredicate),
    ArgumentRetrievalBinaryPredicate(ArgumentRetrievalBinaryPredicate),
    ArgumentRetrievalWithArgumentPhrase(ArgumentRetrievalWithArgumentPhrase),
    Quantification(Quantification),
    Article(Article),
}

#[derive(Clone, Debug)]
pub enum DummyNonTerminalType {
    S,
    DeclarativeSentence,
    InterrogativeSentence,
    PostModifiedMtaEvent,
    PostMtaEventExpression,
    ModalityExpression,
    ModalityWithAgent,
    ModifiedModality,
    ModalityAgent,
    TenseExpression,
    ModifiedTense,
    AspectExpression,
    ModifiedAspect,
    MtaEvent,
    TaEvent,
    AEvent,
    Event,
    PostModifiedBinaryPredicatePhrase,
    PostBinaryPredicatePhraseExpression,
    BinaryPredicatePhrase,
    ArgumentPhrase,
    ArgumentPhraseOrEpsilon,
    DeterminedArgumentRetrievalBinaryPredicate,
    ArticlizedArgumentRetrievalBinaryPredicate,
    ArgumentRetrievalBinaryPredicate,
    ArgumentRetrievalWithArgumentPhrase,
    Quantification,
    Article,
}

// non-terminal types

#[derive(Clone, Debug)]
pub enum S {
    One(DeclarativeSentence),
    Two(InterrogativeSentence),
}

#[derive(Clone, Debug)]
pub enum DeclarativeSentence {
    One(PostModifiedMtaEvent),
}

#[derive(Clone, Debug)]
pub enum InterrogativeSentence {
    One(InterrogativeMarker, PostModifiedMtaEvent),
}

#[derive(Clone, Debug)]
pub enum PostModifiedMtaEvent {
    One(MtaEvent, PostMtaEventExpression),
}

#[derive(Clone, Debug)]
pub enum PostMtaEventExpression {
    One(PostMtaEventModifier, MtaEvent),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModalityExpression {
    One(ModalityWithAgent),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModalityWithAgent {
    One(ModifiedModality, ModalityAgent),
}

#[derive(Clone, Debug)]
pub enum ModifiedModality {
    One(Modality),
    Two(ModalityNegation, Box<ModifiedModality>),
}

#[derive(Clone, Debug)]
pub enum ModalityAgent {
    One(PostArgumentPhraseModifier, ArgumentPhrase),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum TenseExpression {
    One(ModifiedTense),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModifiedTense {
    One(Tense),
    Two(TenseNegation, Box<ModifiedTense>),
}

#[derive(Clone, Debug)]
pub enum AspectExpression {
    One(ModifiedAspect),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModifiedAspect {
    One(Aspect),
    Two(AspectNegation, Box<ModifiedAspect>),
}

#[derive(Clone, Debug)]
pub enum MtaEvent {
    One(ModalityExpression, TaEvent),
}

#[derive(Clone, Debug)]
pub enum TaEvent {
    One(TenseExpression, AEvent),
}

#[derive(Clone, Debug)]
pub enum AEvent {
    One(AspectExpression, Event),
}

#[derive(Clone, Debug)]
pub enum Event {
    One(
        PostModifiedBinaryPredicatePhrase,
        ArgumentPhrase,
        ArgumentPhraseOrEpsilon,
    ),
}

#[derive(Clone, Debug)]
pub enum PostModifiedBinaryPredicatePhrase {
    One(BinaryPredicatePhrase, PostBinaryPredicatePhraseExpression),
}

#[derive(Clone, Debug)]
pub enum PostBinaryPredicatePhraseExpression {
    One(PostBinaryPredicatePhraseModifier, BinaryPredicatePhrase),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum BinaryPredicatePhrase {
    One(BinaryPredicate),
    Two(Negation, Box<BinaryPredicatePhrase>),
    Three(ArgumentPhraseTransposer, Box<BinaryPredicatePhrase>),
    Four(
        BinaryOperator,
        Box<BinaryPredicatePhrase>,
        Box<BinaryPredicatePhrase>,
    ),
}

#[derive(Clone, Debug)]
pub enum ArgumentPhrase {
    One(Argument),
    Two(Negation, Box<ArgumentPhrase>),
    Three(BinaryOperator, Box<ArgumentPhrase>, Box<ArgumentPhrase>),
    Four(
        ArgumentRetriever,
        DeterminedArgumentRetrievalBinaryPredicate,
    ),
    Five(
        MtaEventToArgumentPhraseConverter,
        Box<MtaEvent>,
        MtaEventToArgumentPhraseConverterEnd,
    ),
}

#[derive(Clone, Debug)]
pub enum ArgumentPhraseOrEpsilon {
    One(ArgumentPhraseSeparator, ArgumentPhrase),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum DeterminedArgumentRetrievalBinaryPredicate {
    One(Quantification, ArticlizedArgumentRetrievalBinaryPredicate),
}

#[derive(Clone, Debug)]
pub enum ArticlizedArgumentRetrievalBinaryPredicate {
    One(Article, ArgumentRetrievalBinaryPredicate),
}

#[derive(Clone, Debug)]
pub enum ArgumentRetrievalBinaryPredicate {
    One(
        PostModifiedBinaryPredicatePhrase,
        ArgumentRetrievalWithArgumentPhrase,
    ),
}

#[derive(Clone, Debug)]
pub enum ArgumentRetrievalWithArgumentPhrase {
    One(PostArgumentPhraseModifier, Box<ArgumentPhrase>),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum Quantification {
    One(Quantifier),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum Article {
    One(Articlizer),
    Epsilon,
}

impl S {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::InterrogativeMarker => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::InterrogativeSentence,
                )]))
            }
            DummyTerminalType::Modality => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::Tense => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::Aspect => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::ModalityNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::TenseNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::AspectNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::Negation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::BinaryPredicate => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::ArgumentPhraseTransposer => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::BinaryOperator => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(child1))] => {
                    Self::One(child1.clone())
                }
                [SymbolType::NonTerminal(NonTerminalType::InterrogativeSentence(child1))] => {
                    Self::Two(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl DeclarativeSentence {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::Tense => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::Aspect => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::ModalityNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::TenseNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::AspectNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::Negation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::BinaryPredicate => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::ArgumentPhraseTransposer => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::BinaryOperator => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl InterrogativeSentence {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::InterrogativeMarker => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::InterrogativeMarker),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostModifiedMtaEvent),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::InterrogativeMarker(child1)),
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl PostModifiedMtaEvent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::Tense => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::Aspect => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(child1)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl PostMtaEventExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::PostMtaEventModifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::PostMtaEventModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
            ])),
            DummyTerminalType::Eos => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::PostMtaEventModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl ModalityExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModalityWithAgent,
                )]))
            }
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ModalityNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModalityWithAgent,
                )]))
            }
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl ModalityWithAgent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModifiedModality),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityAgent),
            ])),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModifiedModality),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityAgent),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ModalityAgent(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ModifiedModality {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Modality,
                )]))
            }
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityNegation),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModifiedModality),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Modality(child1))] => Self::One(child1.clone()),
                [
                    SymbolType::Terminal(TerminalType::ModalityNegation(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.clone())),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ModalityAgent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostArgumentPhraseModifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::PostArgumentPhraseModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl TenseExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedTense,
                )]))
            }
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedTense,
                )]))
            }
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::ModifiedTense(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl ModifiedTense {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Tense,
                )]))
            }
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::TenseNegation),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModifiedTense),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Tense(child1))] => Self::One(child1.clone()),
                [
                    SymbolType::Terminal(TerminalType::TenseNegation(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.clone())),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl AspectExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Aspect => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedAspect,
                )]))
            }
            DummyTerminalType::AspectNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedAspect,
                )]))
            }
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl ModifiedAspect {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Aspect => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Aspect,
                )]))
            }
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::AspectNegation),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModifiedAspect),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Aspect(child1))] => Self::One(child1.clone()),
                [
                    SymbolType::Terminal(TerminalType::AspectNegation(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.clone())),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl MtaEvent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Modality => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::Tense => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::Aspect => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(child1)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl TaEvent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::Aspect => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TenseExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AEvent),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(child1)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl AEvent {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Aspect => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::AspectExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Event),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(child1)),
                    SymbolType::NonTerminal(NonTerminalType::Event(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl Event {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhraseOrEpsilon),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhraseOrEpsilon),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhraseOrEpsilon),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhraseOrEpsilon),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(
                        child1,
                    )),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(child3)),
                ] => Self::One(child1.clone(), child2.clone(), child3.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl PostModifiedBinaryPredicatePhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostBinaryPredicatePhraseExpression,
                ),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostBinaryPredicatePhraseExpression,
                ),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostBinaryPredicatePhraseExpression,
                ),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostBinaryPredicatePhraseExpression,
                ),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child1)),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(
                        child2,
                    )),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl PostBinaryPredicatePhraseExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Argument => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseSeparator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostMtaEventModifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostBinaryPredicatePhraseModifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(
                    DummyTerminalType::PostBinaryPredicatePhraseModifier,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
            ])),
            DummyTerminalType::PostArgumentPhraseModifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Eos => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::PostBinaryPredicatePhraseModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl BinaryPredicatePhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::Negation),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
            ])),
            DummyTerminalType::BinaryPredicate => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::BinaryPredicate,
                )]))
            }
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentPhraseTransposer),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::BinaryOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::Negation(child1)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.clone())),
                [SymbolType::Terminal(TerminalType::BinaryPredicate(child1))] => {
                    Self::One(child1.clone())
                }
                [
                    SymbolType::Terminal(TerminalType::ArgumentPhraseTransposer(child1)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)),
                ] => Self::Three(child1.clone(), Box::new(child2.clone())),
                [
                    SymbolType::Terminal(TerminalType::BinaryOperator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child3)),
                ] => Self::Four(
                    child1.clone(),
                    Box::new(child2.clone()),
                    Box::new(child3.clone()),
                ),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ArgumentPhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::Negation),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::Argument => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Argument,
                )]))
            }
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::BinaryOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentRetriever),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeterminedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(
                    DummyTerminalType::MtaEventToArgumentPhraseConverter,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyTerminal(
                    DummyTerminalType::MtaEventToArgumentPhraseConverterEnd,
                ),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::Negation(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.clone())),
                [SymbolType::Terminal(TerminalType::Argument(child1))] => Self::One(child1.clone()),
                [
                    SymbolType::Terminal(TerminalType::BinaryOperator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child3)),
                ] => Self::Three(
                    child1.clone(),
                    Box::new(child2.clone()),
                    Box::new(child3.clone()),
                ),
                [
                    SymbolType::Terminal(TerminalType::ArgumentRetriever(child1)),
                    SymbolType::NonTerminal(
                        NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(child2),
                    ),
                ] => Self::Four(child1.clone(), child2.clone()),
                [
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverter(child1)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(child2)),
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverterEnd(
                        child3,
                    )),
                ] => Self::Five(child1.clone(), Box::new(child2.clone()), child3.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ArgumentPhraseOrEpsilon {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::ArgumentPhraseSeparator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentPhraseSeparator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostMtaEventModifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Eos => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::ArgumentPhraseSeparator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl DeterminedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::Quantifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::Articlizer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Quantification),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate,
                ),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::Quantification(child1)),
                    SymbolType::NonTerminal(
                        NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(child2),
                    ),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ArticlizedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Article),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Article),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Article),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Article),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalBinaryPredicate,
                ),
            ])),
            DummyTerminalType::Articlizer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::Article),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalBinaryPredicate,
                ),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::Article(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(
                        child2,
                    )),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase,
                ),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase,
                ),
            ])),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase,
                ),
            ])),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase,
                ),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(
                        child1,
                    )),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(
                        child2,
                    )),
                ] => Self::One(child1.clone(), child2.clone()),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => {
                panic!("Unexpected epsilon");
            }
        }
    }
}

impl ArgumentRetrievalWithArgumentPhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Argument => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseSeparator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverterEnd => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostMtaEventModifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PostArgumentPhraseModifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::PostArgumentPhraseModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::Eos => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                ] => Self::One(child1.clone(), Box::new(child2.clone())),
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl Quantification {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Quantifier => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Quantifier,
                )]))
            }
            DummyTerminalType::Articlizer => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Quantifier(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl Article {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentPhraseTransposer => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Articlizer => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Articlizer,
                )]))
            }
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Articlizer(child1))] => {
                    Self::One(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_自動生成されたparse_tableと一致する() {
        use std::any::type_name;
        use std::collections::HashMap;

        let s = grammar::NonTerminal(type_name::<S>().split("::").last().unwrap().to_string());
        let declarative_sentence = grammar::NonTerminal(
            type_name::<DeclarativeSentence>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let interrogative_sentence = grammar::NonTerminal(
            type_name::<InterrogativeSentence>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_modified_mta_event = grammar::NonTerminal(
            type_name::<PostModifiedMtaEvent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_mta_event_expression = grammar::NonTerminal(
            type_name::<PostMtaEventExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality_expression = grammar::NonTerminal(
            type_name::<ModalityExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality_with_agent = grammar::NonTerminal(
            type_name::<ModalityWithAgent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modified_modality = grammar::NonTerminal(
            type_name::<ModifiedModality>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality_agent = grammar::NonTerminal(
            type_name::<ModalityAgent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let tense_expression = grammar::NonTerminal(
            type_name::<TenseExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modified_tense = grammar::NonTerminal(
            type_name::<ModifiedTense>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let aspect_expression = grammar::NonTerminal(
            type_name::<AspectExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modified_aspect = grammar::NonTerminal(
            type_name::<ModifiedAspect>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let mta_event = grammar::NonTerminal(
            type_name::<MtaEvent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let ta_event = grammar::NonTerminal(
            type_name::<TaEvent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let a_event = grammar::NonTerminal(
            type_name::<AEvent>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let event =
            grammar::NonTerminal(type_name::<Event>().split("::").last().unwrap().to_string());
        let post_modified_binary_predicate_phrase = grammar::NonTerminal(
            type_name::<PostModifiedBinaryPredicatePhrase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_binary_predicate_phrase_expression = grammar::NonTerminal(
            type_name::<PostBinaryPredicatePhraseExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let binary_predicate_phrase = grammar::NonTerminal(
            type_name::<BinaryPredicatePhrase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_phrase = grammar::NonTerminal(
            type_name::<ArgumentPhrase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_phrase_or_epsilon = grammar::NonTerminal(
            type_name::<ArgumentPhraseOrEpsilon>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let determined_argument_retrieval_binary_predicate = grammar::NonTerminal(
            type_name::<DeterminedArgumentRetrievalBinaryPredicate>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let articlized_argument_retrieval_binary_predicate = grammar::NonTerminal(
            type_name::<ArticlizedArgumentRetrievalBinaryPredicate>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_retrieval_binary_predicate = grammar::NonTerminal(
            type_name::<ArgumentRetrievalBinaryPredicate>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_retrieval_with_argument_phrase = grammar::NonTerminal(
            type_name::<ArgumentRetrievalWithArgumentPhrase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let quantification = grammar::NonTerminal(
            type_name::<Quantification>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let article = grammar::NonTerminal(
            type_name::<Article>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );

        let interrogative_marker = grammar::Terminal(
            type_name::<InterrogativeMarker>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality = grammar::Terminal(
            type_name::<Modality>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let tense = grammar::Terminal(type_name::<Tense>().split("::").last().unwrap().to_string());
        let aspect = grammar::Terminal(
            type_name::<Aspect>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality_negation_ = grammar::Terminal(
            type_name::<ModalityNegation>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let tense_negation_ = grammar::Terminal(
            type_name::<TenseNegation>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let aspect_negation_ = grammar::Terminal(
            type_name::<AspectNegation>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let negation = grammar::Terminal(
            type_name::<Negation>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let binary_predicate = grammar::Terminal(
            type_name::<BinaryPredicate>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument = grammar::Terminal(
            type_name::<Argument>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_phrase_separator = grammar::Terminal(
            type_name::<ArgumentPhraseSeparator>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_phrase_transposer = grammar::Terminal(
            type_name::<ArgumentPhraseTransposer>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let binary_operator = grammar::Terminal(
            type_name::<BinaryOperator>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_retriever = grammar::Terminal(
            type_name::<ArgumentRetriever>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let quantifier = grammar::Terminal(
            type_name::<Quantifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let articlizer = grammar::Terminal(
            type_name::<Articlizer>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let mta_event_to_argument_phrase_converter = grammar::Terminal(
            type_name::<MtaEventToArgumentPhraseConverter>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let mta_event_to_argument_phrase_converter_end = grammar::Terminal(
            type_name::<MtaEventToArgumentPhraseConverterEnd>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_mta_event_modifier = grammar::Terminal(
            type_name::<PostMtaEventModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_binary_predicate_phrase_modifier = grammar::Terminal(
            type_name::<PostBinaryPredicatePhraseModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let post_argument_phrase_modifier = grammar::Terminal(
            type_name::<PostArgumentPhraseModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let _eos = grammar::Terminal("Eos".split("::").last().unwrap().to_string());

        let mut rules = HashMap::new();
        rules.insert(
            s.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    declarative_sentence.clone(),
                )]),
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    interrogative_sentence.clone(),
                )]),
            ],
        );
        rules.insert(
            declarative_sentence.clone(),
            vec![grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                post_modified_mta_event.clone(),
            )])],
        );
        rules.insert(
            interrogative_sentence.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::Terminal(interrogative_marker.clone()),
                grammar::Symbol::NonTerminal(post_modified_mta_event.clone()),
            ])],
        );
        rules.insert(
            post_modified_mta_event.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(mta_event.clone()),
                grammar::Symbol::NonTerminal(post_mta_event_expression.clone()),
            ])],
        );
        rules.insert(
            post_mta_event_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_mta_event_modifier.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            modality_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    modality_with_agent.clone(),
                )]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            modality_with_agent.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(modified_modality.clone()),
                grammar::Symbol::NonTerminal(modality_agent.clone()),
            ])],
        );
        rules.insert(
            modified_modality.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(modality.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(modality_negation_.clone()),
                    grammar::Symbol::NonTerminal(modified_modality.clone()),
                ]),
            ],
        );
        rules.insert(
            modality_agent.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_argument_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            tense_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(modified_tense.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            modified_tense.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(tense.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(tense_negation_.clone()),
                    grammar::Symbol::NonTerminal(modified_tense.clone()),
                ]),
            ],
        );
        rules.insert(
            aspect_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(modified_aspect.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            modified_aspect.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(aspect.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(aspect_negation_.clone()),
                    grammar::Symbol::NonTerminal(modified_aspect.clone()),
                ]),
            ],
        );
        rules.insert(
            mta_event.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(modality_expression.clone()),
                grammar::Symbol::NonTerminal(ta_event.clone()),
            ])],
        );
        rules.insert(
            ta_event.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(tense_expression.clone()),
                grammar::Symbol::NonTerminal(a_event.clone()),
            ])],
        );
        rules.insert(
            a_event.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(aspect_expression.clone()),
                grammar::Symbol::NonTerminal(event.clone()),
            ])],
        );
        rules.insert(
            event.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(post_modified_binary_predicate_phrase.clone()),
                grammar::Symbol::NonTerminal(argument_phrase.clone()),
                grammar::Symbol::NonTerminal(argument_phrase_or_epsilon.clone()),
            ])],
        );
        rules.insert(
            post_modified_binary_predicate_phrase.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                grammar::Symbol::NonTerminal(post_binary_predicate_phrase_expression.clone()),
            ])],
        );
        rules.insert(
            post_binary_predicate_phrase_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_binary_predicate_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            binary_predicate_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(binary_predicate.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_phrase_transposer.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(binary_operator.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
            ],
        );
        rules.insert(
            argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(argument.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_retriever.clone()),
                    grammar::Symbol::NonTerminal(
                        determined_argument_retrieval_binary_predicate.clone(),
                    ),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(binary_operator.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(mta_event_to_argument_phrase_converter.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                    grammar::Symbol::Terminal(mta_event_to_argument_phrase_converter_end.clone()),
                ]),
            ],
        );
        rules.insert(
            argument_phrase_or_epsilon.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_phrase_separator.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            determined_argument_retrieval_binary_predicate.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(quantification.clone()),
                grammar::Symbol::NonTerminal(
                    articlized_argument_retrieval_binary_predicate.clone(),
                ),
            ])],
        );
        rules.insert(
            articlized_argument_retrieval_binary_predicate.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(article.clone()),
                grammar::Symbol::NonTerminal(argument_retrieval_binary_predicate.clone()),
            ])],
        );
        rules.insert(
            argument_retrieval_binary_predicate.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(post_modified_binary_predicate_phrase.clone()),
                grammar::Symbol::NonTerminal(argument_retrieval_with_argument_phrase.clone()),
            ])],
        );
        rules.insert(
            argument_retrieval_with_argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_argument_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            quantification.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(quantifier.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            article.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(articlizer.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );

        let g = grammar::LL1Grammar::new(rules);
        let non_terminals = g.get_non_terminals();
        let terminals = g.get_terminals();
        let parse_table = g.get_parse_table();

        for nt in non_terminals {
            for t in terminals {
                let lt = grammar::Lookahead::Terminal(t.clone());
                let tt = match t.0.as_str() {
                    "InterrogativeMarker" => DummyTerminalType::InterrogativeMarker,
                    "Modality" => DummyTerminalType::Modality,
                    "Tense" => DummyTerminalType::Tense,
                    "Aspect" => DummyTerminalType::Aspect,
                    "ModalityNegation" => DummyTerminalType::ModalityNegation,
                    "TenseNegation" => DummyTerminalType::TenseNegation,
                    "AspectNegation" => DummyTerminalType::AspectNegation,
                    "Negation" => DummyTerminalType::Negation,
                    "BinaryPredicate" => DummyTerminalType::BinaryPredicate,
                    "Argument" => DummyTerminalType::Argument,
                    "ArgumentPhraseSeparator" => DummyTerminalType::ArgumentPhraseSeparator,
                    "ArgumentPhraseTransposer" => DummyTerminalType::ArgumentPhraseTransposer,
                    "BinaryOperator" => DummyTerminalType::BinaryOperator,
                    "ArgumentRetriever" => DummyTerminalType::ArgumentRetriever,
                    "Quantifier" => DummyTerminalType::Quantifier,
                    "Articlizer" => DummyTerminalType::Articlizer,
                    "MtaEventToArgumentPhraseConverter" => {
                        DummyTerminalType::MtaEventToArgumentPhraseConverter
                    }
                    "MtaEventToArgumentPhraseConverterEnd" => {
                        DummyTerminalType::MtaEventToArgumentPhraseConverterEnd
                    }
                    "PostMtaEventModifier" => DummyTerminalType::PostMtaEventModifier,
                    "PostBinaryPredicatePhraseModifier" => {
                        DummyTerminalType::PostBinaryPredicatePhraseModifier
                    }
                    "PostArgumentPhraseModifier" => DummyTerminalType::PostArgumentPhraseModifier,
                    _ => self::panic!("Unknown terminal type: {}", t.0),
                };

                let auto_parse_result = parse_table.get(&(nt.clone(), lt.clone()));
                helper(nt, &lt, tt, &auto_parse_result);
            }
            let lt = grammar::Lookahead::Eos;
            let tt = DummyTerminalType::Eos;

            let auto_parse_result = parse_table.get(&(nt.clone(), lt.clone()));
            helper(nt, &grammar::Lookahead::Eos, tt, &auto_parse_result);
        }
    }

    fn helper(
        nt: &grammar::NonTerminal,
        lt: &grammar::Lookahead,
        tt: DummyTerminalType,
        auto_parse_result: &Option<&grammar::Rhs>,
    ) {
        let manual_parse_result = match nt.0.as_str() {
            "S" => S::parse_table(tt),
            "DeclarativeSentence" => DeclarativeSentence::parse_table(tt),
            "InterrogativeSentence" => InterrogativeSentence::parse_table(tt),
            "PostModifiedMtaEvent" => PostModifiedMtaEvent::parse_table(tt),
            "PostMtaEventExpression" => PostMtaEventExpression::parse_table(tt),
            "ModalityExpression" => ModalityExpression::parse_table(tt),
            "ModalityWithAgent" => ModalityWithAgent::parse_table(tt),
            "ModifiedModality" => ModifiedModality::parse_table(tt),
            "ModalityAgent" => ModalityAgent::parse_table(tt),
            "TenseExpression" => TenseExpression::parse_table(tt),
            "ModifiedTense" => ModifiedTense::parse_table(tt),
            "AspectExpression" => AspectExpression::parse_table(tt),
            "ModifiedAspect" => ModifiedAspect::parse_table(tt),
            "MtaEvent" => MtaEvent::parse_table(tt),
            "TaEvent" => TaEvent::parse_table(tt),
            "AEvent" => AEvent::parse_table(tt),
            "Event" => Event::parse_table(tt),
            "PostModifiedBinaryPredicatePhrase" => {
                PostModifiedBinaryPredicatePhrase::parse_table(tt)
            }
            "PostBinaryPredicatePhraseExpression" => {
                PostBinaryPredicatePhraseExpression::parse_table(tt)
            }
            "BinaryPredicatePhrase" => BinaryPredicatePhrase::parse_table(tt),
            "ArgumentPhrase" => ArgumentPhrase::parse_table(tt),
            "ArgumentPhraseOrEpsilon" => ArgumentPhraseOrEpsilon::parse_table(tt),
            "DeterminedArgumentRetrievalBinaryPredicate" => {
                DeterminedArgumentRetrievalBinaryPredicate::parse_table(tt)
            }
            "ArticlizedArgumentRetrievalBinaryPredicate" => {
                ArticlizedArgumentRetrievalBinaryPredicate::parse_table(tt)
            }
            "ArgumentRetrievalBinaryPredicate" => ArgumentRetrievalBinaryPredicate::parse_table(tt),
            "ArgumentRetrievalWithArgumentPhrase" => {
                ArgumentRetrievalWithArgumentPhrase::parse_table(tt)
            }
            "Quantification" => Quantification::parse_table(tt),
            "Article" => Article::parse_table(tt),
            _ => self::panic!("Unknown non-terminal type: {}", nt.0),
        };

        match (auto_parse_result, manual_parse_result) {
            (Some(auto_result), Ok(manual_result)) => match (auto_result, &manual_result) {
                (grammar::Rhs::Epsilon, DummyRhs::Epsilon) => {}
                (grammar::Rhs::Sequence(auto_seq), DummyRhs::Sequence(manual_seq)) => {
                    assert_eq!(
                        auto_seq.len(),
                        manual_seq.len(),
                        "Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto sequence length: {}, Manual sequence length: {}",
                        nt,
                        lt,
                        auto_seq.len(),
                        manual_seq.len()
                    );
                    for (auto_symbol, manual_symbol) in auto_seq.iter().zip(manual_seq.iter()) {
                        match (auto_symbol, manual_symbol) {
                            (
                                grammar::Symbol::NonTerminal(auto_nt),
                                DummySymbolType::DummyNonTerminal(manual_nt),
                            ) => {
                                let manual_nt_variant_name = match manual_nt {
                                                DummyNonTerminalType::S => "S",
                                                DummyNonTerminalType::DeclarativeSentence => "DeclarativeSentence",
                                                DummyNonTerminalType::InterrogativeSentence => "InterrogativeSentence",
                                                DummyNonTerminalType::PostModifiedMtaEvent => "PostModifiedMtaEvent",
                                                DummyNonTerminalType::PostMtaEventExpression => "PostMtaEventExpression",
                                                DummyNonTerminalType::ModalityExpression => "ModalityExpression",
                                                DummyNonTerminalType::ModalityWithAgent => "ModalityWithAgent",
                                                DummyNonTerminalType::ModifiedModality => "ModifiedModality",
                                                DummyNonTerminalType::ModalityAgent => "ModalityAgent",
                                                DummyNonTerminalType::TenseExpression => "TenseExpression",
                                                DummyNonTerminalType::ModifiedTense => "ModifiedTense",
                                                DummyNonTerminalType::AspectExpression => "AspectExpression",
                                                DummyNonTerminalType::ModifiedAspect => "ModifiedAspect",
                                                DummyNonTerminalType::MtaEvent => "MtaEvent",
                                                DummyNonTerminalType::TaEvent => "TaEvent",
                                                DummyNonTerminalType::AEvent => "AEvent",
                                                DummyNonTerminalType::Event => "Event",
                                                DummyNonTerminalType::PostModifiedBinaryPredicatePhrase => "PostModifiedBinaryPredicatePhrase",
                                                DummyNonTerminalType::PostBinaryPredicatePhraseExpression => "PostBinaryPredicatePhraseExpression",
                                                DummyNonTerminalType::BinaryPredicatePhrase => "BinaryPredicatePhrase",
                                                DummyNonTerminalType::ArgumentPhrase => "ArgumentPhrase",
                                                DummyNonTerminalType::ArgumentPhraseOrEpsilon => "ArgumentPhraseOrEpsilon",
                                                DummyNonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => "DeterminedArgumentRetrievalBinaryPredicate",
                                                DummyNonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => "ArticlizedArgumentRetrievalBinaryPredicate",
                                                DummyNonTerminalType::ArgumentRetrievalBinaryPredicate => "ArgumentRetrievalBinaryPredicate",
                                                DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase => "ArgumentRetrievalWithArgumentPhrase",
                                                DummyNonTerminalType::Quantification => "Quantification",
                                                DummyNonTerminalType::Article => "Article"
                                            };
                                assert_eq!(
                                    auto_nt.0, manual_nt_variant_name,
                                    "Parse table mismatch for non-terminal: {nt:?}, terminal: {lt:?}. Auto: {auto_nt:?}, Manual: {manual_nt:?}",
                                );
                            }
                            (
                                grammar::Symbol::Terminal(auto_t),
                                DummySymbolType::DummyTerminal(manual_t),
                            ) => {
                                let manual_nt_variant_name = match manual_t {
                                    DummyTerminalType::InterrogativeMarker => "InterrogativeMarker",
                                    DummyTerminalType::Modality => "Modality",
                                    DummyTerminalType::Tense => "Tense",
                                    DummyTerminalType::Aspect => "Aspect",
                                    DummyTerminalType::ModalityNegation => "ModalityNegation",
                                    DummyTerminalType::TenseNegation => "TenseNegation",
                                    DummyTerminalType::AspectNegation => "AspectNegation",
                                    DummyTerminalType::Negation => "Negation",
                                    DummyTerminalType::BinaryPredicate => "BinaryPredicate",
                                    DummyTerminalType::Argument => "Argument",
                                    DummyTerminalType::ArgumentPhraseSeparator => {
                                        "ArgumentPhraseSeparator"
                                    }
                                    DummyTerminalType::ArgumentPhraseTransposer => {
                                        "ArgumentPhraseTransposer"
                                    }
                                    DummyTerminalType::BinaryOperator => "BinaryOperator",
                                    DummyTerminalType::ArgumentRetriever => "ArgumentRetriever",
                                    DummyTerminalType::Quantifier => "Quantifier",
                                    DummyTerminalType::Articlizer => "Articlizer",
                                    DummyTerminalType::MtaEventToArgumentPhraseConverter => {
                                        "MtaEventToArgumentPhraseConverter"
                                    }
                                    DummyTerminalType::MtaEventToArgumentPhraseConverterEnd => {
                                        "MtaEventToArgumentPhraseConverterEnd"
                                    }
                                    DummyTerminalType::PostMtaEventModifier => {
                                        "PostMtaEventModifier"
                                    }
                                    DummyTerminalType::PostBinaryPredicatePhraseModifier => {
                                        "PostBinaryPredicatePhraseModifier"
                                    }
                                    DummyTerminalType::PostArgumentPhraseModifier => {
                                        "PostArgumentPhraseModifier"
                                    }
                                    _ => self::panic!("Unknown terminal type: maybe Eos"),
                                };
                                assert_eq!(
                                    auto_t.0, manual_nt_variant_name,
                                    "Parse table mismatch for terminal: {nt:?}, terminal: {lt:?}. Auto: {auto_t:?}, Manual: {manual_t:?}",
                                );
                            }
                            _ => {
                                self::panic!(
                                    "Unexpected symbol type in parse table for non-terminal: {:?}, terminal: {:?}",
                                    nt,
                                    lt
                                );
                            }
                        }
                    }
                }
                _ => {
                    self::panic!(
                        "Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}",
                        nt,
                        lt,
                        auto_result,
                        manual_result
                    );
                }
            },
            (None, Err(_)) => {}
            (Some(_auto_result), Err(_)) => {
                self::panic!(
                    "Auto parse result found but manual parse failed for non-terminal: {:?}, terminal: {:?}",
                    nt,
                    lt
                );
            }
            (None, Ok(_manual_result)) => {
                self::panic!(
                    "Manual parse result found but auto parse result is None for non-terminal: {:?}, terminal: {:?}",
                    nt,
                    lt
                );
            }
        }
    }

    #[test]
    fn test_parse() {
        let input = "parobe ma si ba eos";
        let sentences = tokenize(input).unwrap();
        let _ = parse(sentences[0].clone());
    }
}
