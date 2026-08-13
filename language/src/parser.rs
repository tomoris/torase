#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]

mod grammar;

use core::panic;
use std::mem::discriminant;

use crate::tokenizer::*;

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
        DummyNonTerminalType::ModifiedModality => {
            NonTerminalType::ModifiedModality(ModifiedModality::new(rhs))
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
        DummyNonTerminalType::NonNegatedBinaryPredicatePhrase => {
            NonTerminalType::NonNegatedBinaryPredicatePhrase(NonNegatedBinaryPredicatePhrase::new(
                rhs,
            ))
        }
        DummyNonTerminalType::ArgumentPhrase => {
            NonTerminalType::ArgumentPhrase(ArgumentPhrase::new(rhs))
        }
        DummyNonTerminalType::NonNegatedArgumentPhrase => {
            NonTerminalType::NonNegatedArgumentPhrase(NonNegatedArgumentPhrase::new(rhs))
        }
        DummyNonTerminalType::OptionalCaseArgs => {
            NonTerminalType::OptionalCaseArgs(OptionalCaseArgs::new(rhs))
        }
        DummyNonTerminalType::QuantifiedArgument => {
            NonTerminalType::QuantifiedArgument(QuantifiedArgument::new(rhs))
        }
        DummyNonTerminalType::DeterminedArgument => {
            NonTerminalType::DeterminedArgument(DeterminedArgument::new(rhs))
        }
        DummyNonTerminalType::RetrievedArgument => {
            NonTerminalType::RetrievedArgument(RetrievedArgument::new(rhs))
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
        DummyNonTerminalType::QuantifierModifier => {
            NonTerminalType::QuantifierModifier(QuantifierModifier::new(rhs))
        }
        DummyNonTerminalType::NumericalExpression => {
            NonTerminalType::NumericalExpression(NumericalExpression::new(rhs))
        }
        DummyNonTerminalType::DeterminerModifier => {
            NonTerminalType::DeterminerModifier(DeterminerModifier::new(rhs))
        }
        DummyNonTerminalType::TypeOverrideModifier => {
            NonTerminalType::TypeOverrideModifier(TypeOverrideModifier::new(rhs))
        }
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
        DummyNonTerminalType::ModifiedModality => ModifiedModality::parse_table(t),
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
        DummyNonTerminalType::NonNegatedBinaryPredicatePhrase => {
            NonNegatedBinaryPredicatePhrase::parse_table(t)
        }
        DummyNonTerminalType::ArgumentPhrase => ArgumentPhrase::parse_table(t),
        DummyNonTerminalType::NonNegatedArgumentPhrase => NonNegatedArgumentPhrase::parse_table(t),
        DummyNonTerminalType::OptionalCaseArgs => OptionalCaseArgs::parse_table(t),
        DummyNonTerminalType::QuantifiedArgument => QuantifiedArgument::parse_table(t),
        DummyNonTerminalType::DeterminedArgument => DeterminedArgument::parse_table(t),
        DummyNonTerminalType::RetrievedArgument => RetrievedArgument::parse_table(t),
        DummyNonTerminalType::ArgumentRetrievalBinaryPredicate => {
            ArgumentRetrievalBinaryPredicate::parse_table(t)
        }
        DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase => {
            ArgumentRetrievalWithArgumentPhrase::parse_table(t)
        }
        DummyNonTerminalType::QuantifierModifier => QuantifierModifier::parse_table(t),
        DummyNonTerminalType::NumericalExpression => NumericalExpression::parse_table(t),
        DummyNonTerminalType::DeterminerModifier => DeterminerModifier::parse_table(t),
        DummyNonTerminalType::TypeOverrideModifier => TypeOverrideModifier::parse_table(t),
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

#[derive(Debug, Clone)]
pub struct Ast {
    s: S,
}

impl Ast {
    pub fn get_s(&self) -> &S {
        &self.s
    }
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
    ModifiedModality(ModifiedModality),
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
    NonNegatedBinaryPredicatePhrase(NonNegatedBinaryPredicatePhrase),
    ArgumentPhrase(ArgumentPhrase),
    NonNegatedArgumentPhrase(NonNegatedArgumentPhrase),
    OptionalCaseArgs(OptionalCaseArgs),
    QuantifiedArgument(QuantifiedArgument),
    DeterminedArgument(DeterminedArgument),
    RetrievedArgument(RetrievedArgument),
    ArgumentRetrievalBinaryPredicate(ArgumentRetrievalBinaryPredicate),
    ArgumentRetrievalWithArgumentPhrase(ArgumentRetrievalWithArgumentPhrase),
    QuantifierModifier(QuantifierModifier),
    NumericalExpression(NumericalExpression),
    DeterminerModifier(DeterminerModifier),
    TypeOverrideModifier(TypeOverrideModifier),
}

#[derive(Clone, Debug)]
pub enum DummyNonTerminalType {
    S,
    DeclarativeSentence,
    InterrogativeSentence,
    PostModifiedMtaEvent,
    PostMtaEventExpression,
    ModalityExpression,
    ModifiedModality,
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
    NonNegatedBinaryPredicatePhrase,
    ArgumentPhrase,
    NonNegatedArgumentPhrase,
    OptionalCaseArgs,
    QuantifiedArgument,
    DeterminedArgument,
    RetrievedArgument,
    ArgumentRetrievalBinaryPredicate,
    ArgumentRetrievalWithArgumentPhrase,
    QuantifierModifier,
    NumericalExpression,
    DeterminerModifier,
    TypeOverrideModifier,
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
    One(ModifiedModality),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModifiedModality {
    One(ModalityBase, ModalityForce),
    Two(ModalityNegation, ModalityBase, ModalityForce),
}

#[derive(Clone, Debug)]
pub enum TenseExpression {
    One(ModifiedTense),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModifiedTense {
    One(Tense),
    Two(TenseNegation, Tense),
}

#[derive(Clone, Debug)]
pub enum AspectExpression {
    One(ModifiedAspect),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum ModifiedAspect {
    One(Aspect),
    Two(AspectNegation, Aspect),
}

#[derive(Clone, Debug)]
pub enum MtaEvent {
    One(ModalityExpression, TaEvent),
    Two(MtaEventOperator, Box<MtaEvent>, Box<MtaEvent>),
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
    One(PostModifiedBinaryPredicatePhrase, OptionalCaseArgs),
}

#[derive(Clone, Debug)]
pub enum OptionalCaseArgs {
    One(CaseArg, Box<OptionalCaseArgs>),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum CaseArg {
    One(CaseLabel, TypeOverrideModifier, ArgumentPhrase),
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
    Three(
        PredicateOperator,
        Box<BinaryPredicatePhrase>,
        Box<BinaryPredicatePhrase>,
    ),
}

#[derive(Clone, Debug)]
pub enum ArgumentPhrase {
    One(Argument),
    Two(Negation, Box<ArgumentPhrase>),
    Three(ArgumentOperator, Box<ArgumentPhrase>, Box<ArgumentPhrase>),
    Four(QuantifiedArgument),
    Five(
        MtaEventToArgumentPhraseConverter,
        Box<MtaEvent>,
        MtaEventToArgumentPhraseConverterEnd,
    ),
}

/// BinaryPredicatePhrase の否定なしバリアント（多重否定防止用）
#[derive(Clone, Debug)]
pub struct NonNegatedBinaryPredicatePhrase(pub BinaryPredicatePhrase);

/// ArgumentPhrase の否定なしバリアント（多重否定防止用）
#[derive(Clone, Debug)]
pub struct NonNegatedArgumentPhrase(pub ArgumentPhrase);

#[derive(Clone, Debug)]
pub enum QuantifiedArgument {
    One(QuantifierModifier, DeterminedArgument),
}

#[derive(Clone, Debug)]
pub enum DeterminedArgument {
    One(DeterminerModifier, RetrievedArgument),
}

#[derive(Clone, Debug)]
pub enum RetrievedArgument {
    One(ArgumentRetriever, ArgumentRetrievalBinaryPredicate),
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
pub enum QuantifierModifier {
    One(Quantifier),
    Two(NumericalExpression),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum NumericalExpression {
    // TODO
    // 受理できる数値表現とできない数値表現を区別するために、数値表現を別の非終端記号として定義
    // 十　一 はok
    // 十　はok
    // 二 十 はok
    // 百　はok
    // 十　十　はNG
    // など
    One(Number),
}

#[derive(Clone, Debug)]
pub enum DeterminerModifier {
    One(Determiner),
    Epsilon,
}

#[derive(Clone, Debug)]
pub enum TypeOverrideModifier {
    One(TypeOverride),
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
            DummyTerminalType::ModalityBase => {
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
            DummyTerminalType::PredicateOperator => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::DeclarativeSentence,
                )]))
            }
            DummyTerminalType::MtaEventOperator => {
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
            DummyTerminalType::ModalityBase => {
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
            DummyTerminalType::PredicateOperator => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedMtaEvent,
                )]))
            }
            DummyTerminalType::MtaEventOperator => {
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
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::PostMtaEventExpression),
            ])),
            DummyTerminalType::MtaEventOperator => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::ModalityBase => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedModality,
                )]))
            }
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ModalityNegation => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::ModifiedModality,
                )]))
            }
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::NonTerminal(NonTerminalType::ModifiedModality(child1))] => {
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

impl ModifiedModality {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityBase),
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityForce),
            ])),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityNegation),
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityBase),
                DummySymbolType::DummyTerminal(DummyTerminalType::ModalityForce),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::Terminal(TerminalType::ModalityBase(child1)),
                    SymbolType::Terminal(TerminalType::ModalityForce(child2)),
                ] => Self::One(child1.clone(), child2.clone()),
                [
                    SymbolType::Terminal(TerminalType::ModalityNegation(child1)),
                    SymbolType::Terminal(TerminalType::ModalityBase(child2)),
                    SymbolType::Terminal(TerminalType::ModalityForce(child3)),
                ] => Self::Two(child1.clone(), child2.clone(), child3.clone()),
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
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
                DummySymbolType::DummyTerminal(DummyTerminalType::Tense),
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
                    SymbolType::Terminal(TerminalType::Tense(child2)),
                ] => Self::Two(child1.clone(), child2.clone()),
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
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
                DummySymbolType::DummyTerminal(DummyTerminalType::Aspect),
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
                    SymbolType::Terminal(TerminalType::Aspect(child2)),
                ] => Self::Two(child1.clone(), child2.clone()),
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
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ModalityExpression),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TaEvent),
            ])),
            DummyTerminalType::MtaEventOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::MtaEventOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::MtaEvent),
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
                [
                    SymbolType::Terminal(TerminalType::MtaEventOperator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(child2)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(child3)),
                ] => Self::Two(
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
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
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::OptionalCaseArgs),
            ])),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::OptionalCaseArgs),
            ])),
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::PostModifiedBinaryPredicatePhrase,
                ),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::OptionalCaseArgs),
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
                    SymbolType::NonTerminal(NonTerminalType::OptionalCaseArgs(child2)),
                ] => Event::One(child1.clone(), child2.clone()),
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

impl OptionalCaseArgs {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::CaseLabel => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::CaseLabel),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::TypeOverrideModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::OptionalCaseArgs),
            ])),
            // MtaEvent::Two (homa/homi/homu) lets an MtaEvent be immediately followed by
            // another MtaEvent, so OptionalCaseArgs must epsilon out on any token that can
            // start an MtaEvent too (same terminal set as MtaEvent::parse_table's FIRST set).
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventOperator => Ok(DummyRhs::Epsilon),
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
                    SymbolType::Terminal(TerminalType::CaseLabel(child1)),
                    SymbolType::NonTerminal(NonTerminalType::TypeOverrideModifier(child2)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child3)),
                    SymbolType::NonTerminal(NonTerminalType::OptionalCaseArgs(child4)),
                ] => Self::One(
                    CaseArg::One(child1.clone(), child2.clone(), child3.clone()),
                    Box::new(child4.clone()),
                ),
                _ => panic!("Unexpected symbols: {:?}", symbols),
            },
            Rhs::Epsilon => Self::Epsilon,
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Argument => Ok(DummyRhs::Epsilon),
            DummyTerminalType::CaseLabel => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Quantifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Number => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Determiner => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Epsilon),
            // MtaEvent::Two (homa/homi/homu) lets an MtaEvent be immediately followed by
            // another MtaEvent (see OptionalCaseArgs for the same rationale).
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventOperator => Ok(DummyRhs::Epsilon),
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
                DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::NonNegatedBinaryPredicatePhrase,
                ),
            ])),
            DummyTerminalType::BinaryPredicate => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::BinaryPredicate,
                )]))
            }
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::PredicateOperator),
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
                    SymbolType::NonTerminal(NonTerminalType::NonNegatedBinaryPredicatePhrase(
                        child2,
                    )),
                ] => Self::Two(child1.clone(), Box::new(child2.0.clone())),
                [SymbolType::Terminal(TerminalType::BinaryPredicate(child1))] => {
                    Self::One(child1.clone())
                }
                [
                    SymbolType::Terminal(TerminalType::PredicateOperator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child3)),
                ] => Self::Three(
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
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::NonNegatedArgumentPhrase),
            ])),
            DummyTerminalType::Argument => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Argument,
                )]))
            }
            DummyTerminalType::ArgumentOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::Quantifier => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::Number => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::Determiner => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::ArgumentRetriever => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
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
                    SymbolType::NonTerminal(NonTerminalType::NonNegatedArgumentPhrase(child2)),
                ] => Self::Two(child1.clone(), Box::new(child2.0.clone())),
                [SymbolType::Terminal(TerminalType::Argument(child1))] => Self::One(child1.clone()),
                [
                    SymbolType::Terminal(TerminalType::ArgumentOperator(child1)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child3)),
                ] => Self::Three(
                    child1.clone(),
                    Box::new(child2.clone()),
                    Box::new(child3.clone()),
                ),
                [SymbolType::NonTerminal(NonTerminalType::QuantifiedArgument(child1))] => {
                    Self::Four(child1.clone())
                }
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

impl NonNegatedBinaryPredicatePhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::BinaryPredicate => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::BinaryPredicate,
                )]))
            }
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::PredicateOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::BinaryPredicatePhrase),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        Self(BinaryPredicatePhrase::new(rhs))
    }
}

impl NonNegatedArgumentPhrase {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Argument => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Argument,
                )]))
            }
            DummyTerminalType::ArgumentOperator => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentOperator),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::ArgumentPhrase),
            ])),
            DummyTerminalType::Quantifier => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::Number => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::Determiner => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
            DummyTerminalType::ArgumentRetriever => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::QuantifiedArgument,
                )]))
            }
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
        Self(ArgumentPhrase::new(rhs))
    }
}

impl QuantifiedArgument {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Quantifier => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::QuantifierModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminedArgument),
            ])),
            DummyTerminalType::Number => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::QuantifierModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminedArgument),
            ])),
            DummyTerminalType::Determiner => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::QuantifierModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminedArgument),
            ])),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::QuantifierModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminedArgument),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::QuantifierModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::DeterminedArgument(child2)),
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

impl DeterminedArgument {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Determiner => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminerModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::RetrievedArgument),
            ])),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::DeterminerModifier),
                DummySymbolType::DummyNonTerminal(DummyNonTerminalType::RetrievedArgument),
            ])),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [
                    SymbolType::NonTerminal(NonTerminalType::DeterminerModifier(child1)),
                    SymbolType::NonTerminal(NonTerminalType::RetrievedArgument(child2)),
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

impl RetrievedArgument {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Sequence(vec![
                DummySymbolType::DummyTerminal(DummyTerminalType::ArgumentRetriever),
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
                    SymbolType::Terminal(TerminalType::ArgumentRetriever(child1)),
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
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Sequence(vec![
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
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Argument => Ok(DummyRhs::Epsilon),
            DummyTerminalType::CaseLabel => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Quantifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Number => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Determiner => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Epsilon),
            // MtaEvent::Two (homa/homi/homu) lets an MtaEvent be immediately followed by
            // another MtaEvent (see OptionalCaseArgs for the same rationale).
            DummyTerminalType::ModalityBase => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Tense => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Aspect => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ModalityNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::TenseNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::AspectNegation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::BinaryPredicate => Ok(DummyRhs::Epsilon),
            DummyTerminalType::PredicateOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventOperator => Ok(DummyRhs::Epsilon),
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

impl QuantifierModifier {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Quantifier => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Quantifier,
                )]))
            }
            DummyTerminalType::Number => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyNonTerminal(
                    DummyNonTerminalType::NumericalExpression,
                )]))
            }
            DummyTerminalType::Determiner => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Quantifier(child1))] => {
                    Self::One(child1.clone())
                }
                [SymbolType::NonTerminal(NonTerminalType::NumericalExpression(child1))] => {
                    Self::Two(child1.clone())
                }
                _ => {
                    panic!("Unexpected symbols: {:?}", symbols);
                }
            },
            Rhs::Epsilon => Self::Epsilon,
        }
    }
}

impl NumericalExpression {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Number => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Number,
                )]))
            }
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Number(child1))] => Self::One(child1.clone()),
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

impl DeterminerModifier {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::Determiner => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::Determiner,
                )]))
            }
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::Determiner(child1))] => {
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

impl TypeOverrideModifier {
    fn parse_table(current_token: DummyTerminalType) -> Result<DummyRhs, ()> {
        match current_token {
            DummyTerminalType::TypeOverride => {
                Ok(DummyRhs::Sequence(vec![DummySymbolType::DummyTerminal(
                    DummyTerminalType::TypeOverride,
                )]))
            }
            // Epsilon on every token in FIRST(ArgumentPhrase) (see ArgumentPhrase::parse_table)
            // so this modifier can be skipped when `se` is absent.
            DummyTerminalType::Negation => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Argument => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentOperator => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Quantifier => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Number => Ok(DummyRhs::Epsilon),
            DummyTerminalType::Determiner => Ok(DummyRhs::Epsilon),
            DummyTerminalType::ArgumentRetriever => Ok(DummyRhs::Epsilon),
            DummyTerminalType::MtaEventToArgumentPhraseConverter => Ok(DummyRhs::Epsilon),
            _ => Err(()),
        }
    }
    fn new(rhs: Rhs) -> Self {
        match rhs {
            Rhs::Sequence(ref symbols) => match symbols.as_slice() {
                [SymbolType::Terminal(TerminalType::TypeOverride(child1))] => {
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

// ---------------------------------------------------------------------------
// Range helpers — canonical implementations shared by all type checkers
// ---------------------------------------------------------------------------

impl BinaryPredicatePhrase {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            BinaryPredicatePhrase::One(binary_predicate) => binary_predicate.get_range(),
            BinaryPredicatePhrase::Two(negation, binary_predicate_phrase) => {
                let start = negation.get_range().start();
                let end = binary_predicate_phrase.get_range().end();
                Range::new(start, end)
            }
            BinaryPredicatePhrase::Three(predicate_operator, _, binary_predicate2) => {
                let start = predicate_operator.get_range().start();
                let end = binary_predicate2.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl PostBinaryPredicatePhraseExpression {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            PostBinaryPredicatePhraseExpression::One(
                post_binary_predicate_modifier,
                binary_predicate_phrase,
            ) => {
                let start = post_binary_predicate_modifier.get_range().start();
                let end = binary_predicate_phrase.get_range().end();
                Some(Range::new(start, end))
            }
            PostBinaryPredicatePhraseExpression::Epsilon => None,
        }
    }
}

impl PostModifiedBinaryPredicatePhrase {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                post_binary_predicate_phrase_expression,
            ) => {
                let start = binary_predicate_phrase.get_range().start();
                if let Some(end) = post_binary_predicate_phrase_expression.get_range() {
                    return Range::new(start, end.end());
                }
                let end = binary_predicate_phrase.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl QuantifierModifier {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            QuantifierModifier::One(quantifier) => Some(quantifier.get_range()),
            QuantifierModifier::Two(numerical_expression) => numerical_expression.get_range(),
            QuantifierModifier::Epsilon => None,
        }
    }
}

impl NumericalExpression {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            NumericalExpression::One(number) => Some(number.get_range()),
        }
    }
}

impl DeterminerModifier {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            DeterminerModifier::One(determiner) => Some(determiner.get_range()),
            DeterminerModifier::Epsilon => None,
        }
    }
}

impl QuantifiedArgument {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            QuantifiedArgument::One(quantifier_modifier, determined_argument) => {
                let end = determined_argument.get_range().end();
                if let Some(start) = quantifier_modifier.get_range() {
                    return Range::new(start.start(), end);
                }
                let start = determined_argument.get_range().start();
                Range::new(start, end)
            }
        }
    }
}

impl DeterminedArgument {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            DeterminedArgument::One(determiner_modifier, retrieved_argument) => {
                let end = retrieved_argument.get_range().end();
                if let Some(start) = determiner_modifier.get_range() {
                    return Range::new(start.start(), end);
                }
                let start = retrieved_argument.get_range().start();
                Range::new(start, end)
            }
        }
    }
}

impl RetrievedArgument {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                let start = argument_retriever.get_range().start();
                let end = argument_retrieval_binary_predicate.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl ArgumentRetrievalWithArgumentPhrase {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            ArgumentRetrievalWithArgumentPhrase::One(
                post_argument_phrase_modifier,
                argument_phrase,
            ) => {
                let start = post_argument_phrase_modifier.get_range().start();
                let end = argument_phrase.get_range().end();
                Some(Range::new(start, end))
            }
            ArgumentRetrievalWithArgumentPhrase::Epsilon => None,
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                let start = post_modified_binary_predicate_phrase.get_range().start();
                if let Some(end) = argument_retrieval_with_argument_phrase.get_range() {
                    return Range::new(start, end.end());
                }
                let end = post_modified_binary_predicate_phrase.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl ArgumentPhrase {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            ArgumentPhrase::One(argument) => argument.get_range(),
            ArgumentPhrase::Two(negation, argument) => {
                let start = negation.get_range().start();
                let end = argument.get_range().end();
                Range::new(start, end)
            }
            ArgumentPhrase::Three(argument_operator, _, argument2) => {
                let start = argument_operator.get_range().start();
                let end = argument2.get_range().end();
                Range::new(start, end)
            }
            ArgumentPhrase::Four(quantified_argument) => quantified_argument.get_range(),
            ArgumentPhrase::Five(
                mta_event_to_argument_phrase_converter,
                _,
                mta_event_to_argument_phrase_converter_end,
            ) => {
                let start = mta_event_to_argument_phrase_converter.get_range().start();
                let end = mta_event_to_argument_phrase_converter_end.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl ModifiedModality {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            ModifiedModality::One(modality, modality_force) => Range::new(
                modality.get_range().start(),
                modality_force.get_range().end(),
            ),
            ModifiedModality::Two(modality_negation, _, modality_force) => Range::new(
                modality_negation.get_range().start(),
                modality_force.get_range().end(),
            ),
        }
    }
}

impl ModalityExpression {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            ModalityExpression::One(modified_modality) => Some(modified_modality.get_range()),
            ModalityExpression::Epsilon => None,
        }
    }
}

impl ModifiedTense {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            ModifiedTense::One(tense) => tense.get_range(),
            ModifiedTense::Two(tense_negation, tense) => {
                Range::new(tense_negation.get_range().start(), tense.get_range().end())
            }
        }
    }
}

impl TenseExpression {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            TenseExpression::One(modified_tense) => Some(modified_tense.get_range()),
            TenseExpression::Epsilon => None,
        }
    }
}

impl ModifiedAspect {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            ModifiedAspect::One(aspect) => aspect.get_range(),
            ModifiedAspect::Two(aspect_negation, aspect) => Range::new(
                aspect_negation.get_range().start(),
                aspect.get_range().end(),
            ),
        }
    }
}

impl AspectExpression {
    pub(crate) fn get_range(&self) -> Option<Range> {
        match self {
            AspectExpression::One(modified_aspect) => Some(modified_aspect.get_range()),
            AspectExpression::Epsilon => None,
        }
    }
}

impl Event {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            Event::One(pred, args) => match args.last_range() {
                Some(last) => Range::new(pred.get_range().start(), last.end()),
                None => pred.get_range(),
            },
        }
    }

    pub fn get_pred(&self) -> &PostModifiedBinaryPredicatePhrase {
        match self {
            Event::One(pred, _) => pred,
        }
    }

    pub fn case_args(&self) -> Vec<(&CaseLabel, &ArgumentPhrase)> {
        match self {
            Event::One(_, args) => args.case_args(),
        }
    }

    /// `case_args`に加え、各項が`se`(TypeOverride)で型検査を除外指定されているかを返す。
    pub fn case_args_with_override(&self) -> Vec<(&CaseLabel, bool, &ArgumentPhrase)> {
        match self {
            Event::One(_, args) => args.case_args_with_override(),
        }
    }
}

impl OptionalCaseArgs {
    fn last_range(&self) -> Option<Range> {
        match self {
            OptionalCaseArgs::One(CaseArg::One(_label, _type_override, phrase), rest) => {
                Some(rest.last_range().unwrap_or_else(|| phrase.get_range()))
            }
            OptionalCaseArgs::Epsilon => None,
        }
    }

    fn case_args(&self) -> Vec<(&CaseLabel, &ArgumentPhrase)> {
        match self {
            OptionalCaseArgs::One(CaseArg::One(label, _type_override, phrase), rest) => {
                let mut result = vec![(label, phrase)];
                result.extend(rest.case_args());
                result
            }
            OptionalCaseArgs::Epsilon => vec![],
        }
    }

    fn case_args_with_override(&self) -> Vec<(&CaseLabel, bool, &ArgumentPhrase)> {
        match self {
            OptionalCaseArgs::One(CaseArg::One(label, type_override, phrase), rest) => {
                let is_overridden = matches!(type_override, TypeOverrideModifier::One(_));
                let mut result = vec![(label, is_overridden, phrase)];
                result.extend(rest.case_args_with_override());
                result
            }
            OptionalCaseArgs::Epsilon => vec![],
        }
    }
}

impl AEvent {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            AEvent::One(aspect_expression, event) => {
                let end = event.get_range().end();
                let start = aspect_expression
                    .get_range()
                    .map(|r| r.start())
                    .unwrap_or_else(|| event.get_range().start());
                Range::new(start, end)
            }
        }
    }
}

impl TaEvent {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            TaEvent::One(tense_expression, a_event) => {
                let end = a_event.get_range().end();
                let start = tense_expression
                    .get_range()
                    .map(|r| r.start())
                    .unwrap_or_else(|| a_event.get_range().start());
                Range::new(start, end)
            }
        }
    }
}

impl MtaEvent {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            MtaEvent::One(modality_expression, ta_event) => {
                let end = ta_event.get_range().end();
                let start = modality_expression
                    .get_range()
                    .map(|r| r.start())
                    .unwrap_or_else(|| ta_event.get_range().start());
                Range::new(start, end)
            }
            MtaEvent::Two(proposition_operator, _, mta_event2) => {
                let start = proposition_operator.get_range().start();
                let end = mta_event2.get_range().end();
                Range::new(start, end)
            }
        }
    }
}

impl PostModifiedMtaEvent {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            PostModifiedMtaEvent::One(mta_event, post_mta_event_expression) => {
                let start = mta_event.get_range().start();
                let end = match post_mta_event_expression {
                    PostMtaEventExpression::One(_, mta_event2) => mta_event2.get_range().end(),
                    PostMtaEventExpression::Epsilon => mta_event.get_range().end(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl DeclarativeSentence {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            DeclarativeSentence::One(post_modified_mta_event) => {
                post_modified_mta_event.get_range()
            }
        }
    }
}

impl InterrogativeSentence {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            InterrogativeSentence::One(interrogative_marker, post_modified_mta_event) => {
                Range::new(
                    interrogative_marker.get_range().start(),
                    post_modified_mta_event.get_range().end(),
                )
            }
        }
    }
}

impl S {
    pub(crate) fn get_range(&self) -> Range {
        match self {
            S::One(declarative_sentence) => declarative_sentence.get_range(),
            S::Two(interrogative_sentence) => interrogative_sentence.get_range(),
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

        // non-terminals
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
        let modified_modality = grammar::NonTerminal(
            type_name::<ModifiedModality>()
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
        let non_negated_binary_predicate_phrase = grammar::NonTerminal(
            type_name::<NonNegatedBinaryPredicatePhrase>()
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
        let non_negated_argument_phrase = grammar::NonTerminal(
            type_name::<NonNegatedArgumentPhrase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let optional_case_args = grammar::NonTerminal(
            type_name::<OptionalCaseArgs>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let quantified_argument = grammar::NonTerminal(
            type_name::<QuantifiedArgument>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let determined_argument = grammar::NonTerminal(
            type_name::<DeterminedArgument>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let retrieved_argument = grammar::NonTerminal(
            type_name::<RetrievedArgument>()
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
        let quantifier_modifier = grammar::NonTerminal(
            type_name::<QuantifierModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let numerical_expression = grammar::NonTerminal(
            type_name::<NumericalExpression>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let determiner_modifier = grammar::NonTerminal(
            type_name::<DeterminerModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let type_override_modifier = grammar::NonTerminal(
            type_name::<TypeOverrideModifier>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );

        // terminals
        let interrogative_marker = grammar::Terminal(
            type_name::<InterrogativeMarker>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality = grammar::Terminal(
            type_name::<ModalityBase>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let modality_force = grammar::Terminal(
            type_name::<ModalityForce>()
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
        let case_label = grammar::Terminal(
            type_name::<CaseLabel>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let type_override = grammar::Terminal(
            type_name::<TypeOverride>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let predicate_operator = grammar::Terminal(
            type_name::<PredicateOperator>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let argument_operator = grammar::Terminal(
            type_name::<ArgumentOperator>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let proposition_operator = grammar::Terminal(
            type_name::<MtaEventOperator>()
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
        let determiner = grammar::Terminal(
            type_name::<Determiner>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        );
        let number = grammar::Terminal(
            type_name::<Number>()
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
                    modified_modality.clone(),
                )]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            modified_modality.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(modality.clone()),
                    grammar::Symbol::Terminal(modality_force.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(modality_negation_.clone()),
                    grammar::Symbol::Terminal(modality.clone()),
                    grammar::Symbol::Terminal(modality_force.clone()),
                ]),
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
                    grammar::Symbol::Terminal(tense.clone()),
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
                    grammar::Symbol::Terminal(aspect.clone()),
                ]),
            ],
        );
        rules.insert(
            mta_event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(modality_expression.clone()),
                    grammar::Symbol::NonTerminal(ta_event.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(proposition_operator.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                ]),
            ],
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
                grammar::Symbol::NonTerminal(optional_case_args.clone()),
            ])],
        );
        rules.insert(
            optional_case_args.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(case_label.clone()),
                    grammar::Symbol::NonTerminal(type_override_modifier.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                    grammar::Symbol::NonTerminal(optional_case_args.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ],
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
                    grammar::Symbol::Terminal(predicate_operator.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(non_negated_binary_predicate_phrase.clone()),
                ]),
            ],
        );
        rules.insert(
            non_negated_binary_predicate_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(binary_predicate.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(predicate_operator.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
            ],
        );
        rules.insert(
            argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(argument.clone())]),
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    quantified_argument.clone(),
                )]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_operator.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(non_negated_argument_phrase.clone()),
                ]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(mta_event_to_argument_phrase_converter.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                    grammar::Symbol::Terminal(mta_event_to_argument_phrase_converter_end.clone()),
                ]),
            ],
        );
        rules.insert(
            non_negated_argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(argument.clone())]),
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    quantified_argument.clone(),
                )]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_operator.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
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
            quantified_argument.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(quantifier_modifier.clone()),
                grammar::Symbol::NonTerminal(determined_argument.clone()),
            ])],
        );
        rules.insert(
            determined_argument.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::NonTerminal(determiner_modifier.clone()),
                grammar::Symbol::NonTerminal(retrieved_argument.clone()),
            ])],
        );
        rules.insert(
            retrieved_argument.clone(),
            vec![grammar::Rhs::Sequence(vec![
                grammar::Symbol::Terminal(argument_retriever.clone()),
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
            quantifier_modifier.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(quantifier.clone())]),
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(
                    numerical_expression.clone(),
                )]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            numerical_expression.clone(),
            vec![grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(
                number.clone(),
            )])],
        );
        rules.insert(
            determiner_modifier.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(determiner.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );
        rules.insert(
            type_override_modifier.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(type_override.clone())]),
                grammar::Rhs::Epsilon,
            ],
        );

        let g = grammar::LL1Grammar::try_new(rules).unwrap();
        let non_terminals = g.get_non_terminals();
        let terminals = g.get_terminals();
        let parse_table = g.get_parse_table();

        for nt in non_terminals {
            for t in terminals {
                let lt = grammar::Lookahead::Terminal(t.clone());
                let tt = match t.0.as_str() {
                    "InterrogativeMarker" => DummyTerminalType::InterrogativeMarker,
                    "ModalityBase" => DummyTerminalType::ModalityBase,
                    "ModalityForce" => DummyTerminalType::ModalityForce,
                    "Tense" => DummyTerminalType::Tense,
                    "Aspect" => DummyTerminalType::Aspect,
                    "ModalityNegation" => DummyTerminalType::ModalityNegation,
                    "TenseNegation" => DummyTerminalType::TenseNegation,
                    "AspectNegation" => DummyTerminalType::AspectNegation,
                    "Negation" => DummyTerminalType::Negation,
                    "BinaryPredicate" => DummyTerminalType::BinaryPredicate,
                    "Argument" => DummyTerminalType::Argument,
                    "CaseLabel" => DummyTerminalType::CaseLabel,
                    "TypeOverride" => DummyTerminalType::TypeOverride,
                    "PredicateOperator" => DummyTerminalType::PredicateOperator,
                    "ArgumentOperator" => DummyTerminalType::ArgumentOperator,
                    "MtaEventOperator" => DummyTerminalType::MtaEventOperator,
                    "ArgumentRetriever" => DummyTerminalType::ArgumentRetriever,
                    "Quantifier" => DummyTerminalType::Quantifier,
                    "Determiner" => DummyTerminalType::Determiner,
                    "Number" => DummyTerminalType::Number,
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
            "ModifiedModality" => ModifiedModality::parse_table(tt),
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
            "NonNegatedBinaryPredicatePhrase" => NonNegatedBinaryPredicatePhrase::parse_table(tt),
            "ArgumentPhrase" => ArgumentPhrase::parse_table(tt),
            "NonNegatedArgumentPhrase" => NonNegatedArgumentPhrase::parse_table(tt),
            "OptionalCaseArgs" => OptionalCaseArgs::parse_table(tt),
            "QuantifiedArgument" => QuantifiedArgument::parse_table(tt),
            "DeterminedArgument" => DeterminedArgument::parse_table(tt),
            "RetrievedArgument" => RetrievedArgument::parse_table(tt),
            "ArgumentRetrievalBinaryPredicate" => ArgumentRetrievalBinaryPredicate::parse_table(tt),
            "ArgumentRetrievalWithArgumentPhrase" => {
                ArgumentRetrievalWithArgumentPhrase::parse_table(tt)
            }
            "QuantifierModifier" => QuantifierModifier::parse_table(tt),
            "NumericalExpression" => NumericalExpression::parse_table(tt),
            "DeterminerModifier" => DeterminerModifier::parse_table(tt),
            "TypeOverrideModifier" => TypeOverrideModifier::parse_table(tt),
            _ => self::panic!("Unknown non-terminal type: {}", nt.0),
        };

        match (auto_parse_result, manual_parse_result) {
            (Some(auto_result), Ok(manual_result)) => {
                match (auto_result, &manual_result) {
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
                                                DummyNonTerminalType::ModifiedModality => "ModifiedModality",
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
                                                DummyNonTerminalType::NonNegatedBinaryPredicatePhrase => "NonNegatedBinaryPredicatePhrase",
                                                DummyNonTerminalType::ArgumentPhrase => "ArgumentPhrase",
                                                DummyNonTerminalType::NonNegatedArgumentPhrase => "NonNegatedArgumentPhrase",
                                                DummyNonTerminalType::OptionalCaseArgs => "OptionalCaseArgs",
                                                DummyNonTerminalType::QuantifiedArgument => "QuantifiedArgument",
                                                DummyNonTerminalType::DeterminedArgument => "DeterminedArgument",
                                                DummyNonTerminalType::RetrievedArgument => "RetrievedArgument",
                                                DummyNonTerminalType::ArgumentRetrievalBinaryPredicate => "ArgumentRetrievalBinaryPredicate",
                                                DummyNonTerminalType::ArgumentRetrievalWithArgumentPhrase => "ArgumentRetrievalWithArgumentPhrase",
                                                DummyNonTerminalType::QuantifierModifier => "QuantifierModifier",
                                                DummyNonTerminalType::NumericalExpression => "NumericalExpression",
                                                DummyNonTerminalType::DeterminerModifier => "DeterminerModifier",
                                                DummyNonTerminalType::TypeOverrideModifier => "TypeOverrideModifier",
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
                                        DummyTerminalType::InterrogativeMarker => {
                                            "InterrogativeMarker"
                                        }
                                        DummyTerminalType::ModalityBase => "ModalityBase",
                                        DummyTerminalType::ModalityForce => "ModalityForce",
                                        DummyTerminalType::Tense => "Tense",
                                        DummyTerminalType::Aspect => "Aspect",
                                        DummyTerminalType::ModalityNegation => "ModalityNegation",
                                        DummyTerminalType::TenseNegation => "TenseNegation",
                                        DummyTerminalType::AspectNegation => "AspectNegation",
                                        DummyTerminalType::Negation => "Negation",
                                        DummyTerminalType::BinaryPredicate => "BinaryPredicate",
                                        DummyTerminalType::Argument => "Argument",
                                        DummyTerminalType::CaseLabel => "CaseLabel",
                                        DummyTerminalType::TypeOverride => "TypeOverride",
                                        DummyTerminalType::PredicateOperator => "PredicateOperator",
                                        DummyTerminalType::ArgumentOperator => "ArgumentOperator",
                                        DummyTerminalType::MtaEventOperator => "MtaEventOperator",
                                        DummyTerminalType::ArgumentRetriever => "ArgumentRetriever",
                                        DummyTerminalType::Quantifier => "Quantifier",
                                        DummyTerminalType::Number => "Number",
                                        DummyTerminalType::Determiner => "Determiner",
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
                }
            }
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
        let input = "parobe sa ma si ba eos";
        let sentences = tokenize(input).unwrap();
        let _ = parse(sentences[0].clone());
    }
}
