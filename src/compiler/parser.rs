mod grammar;

use core::panic;
use std::{f32::consts::E, mem::discriminant};

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn parse(tokens: Vec<TerminalType>) -> Result<Ast, IncompleteAst> {
    let mut stack: Vec<SymbolType> = vec![SymbolType::Terminal(TerminalType::Eos(None)), SymbolType::NonTerminal(NonTerminalType::S(None))];
    let mut current_token_index = 0;

    let s = parse_node(&mut stack, &tokens, &mut current_token_index)?;
    if let SymbolType::NonTerminal(NonTerminalType::S(s)) = s {
        if let Some(s) = s {
            let eos = parse_node(&mut stack, &tokens, &mut current_token_index)?;
            if let SymbolType::Terminal(TerminalType::Eos(_)) = eos {
                if stack.is_empty() && current_token_index == tokens.len() {
                    return Ok(Ast { s });
                } else {
                    panic!("Stack is not empty or current token index is not at the end of tokens after parsing S");
                }
            } else {
                panic!("Expected end of stream after parsing S, found: {:?}", eos);
            }
        } else {
            panic!("Parsed S is None");
        }
    } else {
        panic!("Expected S as the root non-terminal, found: {:?}", s);
    }
}

fn parse_node(stack: &mut Vec<SymbolType>, tokens: &Vec<TerminalType>, current_token_index: &mut usize) -> Result<SymbolType, IncompleteAst> {
    let current_token = tokens.get(*current_token_index).unwrap();
    let top_symbol = stack.pop().unwrap();

    match top_symbol {
        SymbolType::Terminal(terminal) => {
            match terminal {
                TerminalType::Eos(_) => {
                    if discriminant(&TerminalType::Eos(None)) == discriminant(current_token) {
                        *current_token_index += 1;
                        return Ok(SymbolType::Terminal(current_token.clone()));
                    } else {
                        panic!("Expected end of stream, found: {:?}", current_token);
                        return Err(IncompleteAst {});
                    }
                }
                _ => {
                    if discriminant(&terminal) == discriminant(current_token) {
                        *current_token_index += 1;
                        return Ok(SymbolType::Terminal(current_token.clone()));
                    } else {
                        panic!("Expected token: {:?}, found: {:?}", terminal, current_token);
                        return Err(IncompleteAst {});
                    }
                }
            }
        }
        SymbolType::NonTerminal(non_terminal) => {
            let rhs = parse_table(non_terminal.clone(), current_token)?;
            match rhs {
                Rhs::Sequence(ref symbols) => {
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
                                panic!("Failed to parse child node: {:?}", e);
                                return Err(e);
                            }
                        }

                    }
                    return Ok(SymbolType::NonTerminal(new_non_terminal(&non_terminal, Rhs::Sequence(child_nodes))?));
                }
                Rhs::Epsilon => {
                    return Ok(SymbolType::NonTerminal(new_non_terminal(&non_terminal, rhs)?));
                }
            }
        }
    }
}

fn new_non_terminal(non_terminal: &NonTerminalType, rhs: Rhs) -> Result<NonTerminalType, IncompleteAst> {
    match non_terminal {
        NonTerminalType::S(_) => Ok(NonTerminalType::S(Some(S::try_new(rhs)?))),
        NonTerminalType::DeclarativeSentence(_) => Ok(NonTerminalType::DeclarativeSentence(Some(DeclarativeSentence::try_new(rhs)?))),
        NonTerminalType::InterrogativeSentence(_) => Ok(NonTerminalType::InterrogativeSentence(Some(InterrogativeSentence::try_new(rhs)?))),
        NonTerminalType::PostModifiedMtaEvent(_) => Ok(NonTerminalType::PostModifiedMtaEvent(Some(PostModifiedMtaEvent::try_new(rhs)?))),
        NonTerminalType::PostMtaEventExpression(_) => Ok(NonTerminalType::PostMtaEventExpression(Some(PostMtaEventExpression::try_new(rhs)?))),
        NonTerminalType::ModalityExpression(_) => Ok(NonTerminalType::ModalityExpression(Some(ModalityExpression::try_new(rhs)?))),
        NonTerminalType::ModalityWithAgent(_) => Ok(NonTerminalType::ModalityWithAgent(Some(ModalityWithAgent::try_new(rhs)?))),
        NonTerminalType::ModifiedModality(_) => Ok(NonTerminalType::ModifiedModality(Some(ModifiedModality::try_new(rhs)?))),
        NonTerminalType::ModalityAgent(_) => Ok(NonTerminalType::ModalityAgent(Some(ModalityAgent::try_new(rhs)?))),
        NonTerminalType::TenseExpression(_) => Ok(NonTerminalType::TenseExpression(Some(TenseExpression::try_new(rhs)?))),
        NonTerminalType::ModifiedTense(_) => Ok(NonTerminalType::ModifiedTense(Some(ModifiedTense::try_new(rhs)?))),
        NonTerminalType::AspectExpression(_) => Ok(NonTerminalType::AspectExpression(Some(AspectExpression::try_new(rhs)?))),
        NonTerminalType::ModifiedAspect(_) => Ok(NonTerminalType::ModifiedAspect(Some(ModifiedAspect::try_new(rhs)?))),
        NonTerminalType::MtaEvent(_) => Ok(NonTerminalType::MtaEvent(Some(MtaEvent::try_new(rhs)?))),
        NonTerminalType::TaEvent(_) => Ok(NonTerminalType::TaEvent(Some(TaEvent::try_new(rhs)?))),
        NonTerminalType::AEvent(_) => Ok(NonTerminalType::AEvent(Some(AEvent::try_new(rhs)?))),
        NonTerminalType::Event(_) => Ok(NonTerminalType::Event(Some(Event::try_new(rhs)?))),
        NonTerminalType::PostModifiedBinaryPredicatePhrase(_) => Ok(NonTerminalType::PostModifiedBinaryPredicatePhrase(Some(PostModifiedBinaryPredicatePhrase::try_new(rhs)?))),
        NonTerminalType::PostBinaryPredicatePhraseExpression(_) => Ok(NonTerminalType::PostBinaryPredicatePhraseExpression(Some(PostBinaryPredicatePhraseExpression::try_new(rhs)?))),
        NonTerminalType::BinaryPredicatePhrase(_) => Ok(NonTerminalType::BinaryPredicatePhrase(Some(BinaryPredicatePhrase::try_new(rhs)?))),
        NonTerminalType::ArgumentPhrase(_) => Ok(NonTerminalType::ArgumentPhrase(Some(ArgumentPhrase::try_new(rhs)?))),
        NonTerminalType::ArgumentPhraseOrEpsilon(_) => Ok(NonTerminalType::ArgumentPhraseOrEpsilon(Some(ArgumentPhraseOrEpsilon::try_new(rhs)?))),
        NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(_) => Ok(NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(Some(DeterminedArgumentRetrievalBinaryPredicate::try_new(rhs)?))),
        NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(_) => Ok(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(Some(ArticlizedArgumentRetrievalBinaryPredicate::try_new(rhs)?))),
        NonTerminalType::ArgumentRetrievalBinaryPredicate(_) => Ok(NonTerminalType::ArgumentRetrievalBinaryPredicate(Some(ArgumentRetrievalBinaryPredicate::try_new(rhs)?))),
        NonTerminalType::ArgumentRetrievalWithArgumentPhrase(_) => Ok(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(Some(ArgumentRetrievalWithArgumentPhrase::try_new(rhs)?))),
        NonTerminalType::Quantification(_) => Ok(NonTerminalType::Quantification(Some(Quantification::try_new(rhs)?))),
        NonTerminalType::Article(_) => Ok(NonTerminalType::Article(Some(Article::try_new(rhs)?))),
        _ => Err(IncompleteAst {}),
    }
}


fn parse_table(nt: NonTerminalType, &t: &TerminalType) -> Result<Rhs, IncompleteAst> {
    match nt {
        NonTerminalType::S(_) => S::parse_table(t),
        NonTerminalType::DeclarativeSentence(_) => DeclarativeSentence::parse_table(t),
        NonTerminalType::InterrogativeSentence(_) => InterrogativeSentence::parse_table(t),
        NonTerminalType::PostModifiedMtaEvent(_) => PostModifiedMtaEvent::parse_table(t),
        NonTerminalType::PostMtaEventExpression(_) => PostMtaEventExpression::parse_table(t),
        NonTerminalType::ModalityExpression(_) => ModalityExpression::parse_table(t),
        NonTerminalType::ModalityWithAgent(_) => ModalityWithAgent::parse_table(t),
        NonTerminalType::ModifiedModality(_) => ModifiedModality::parse_table(t),
        NonTerminalType::ModalityAgent(_) => ModalityAgent::parse_table(t),
        NonTerminalType::TenseExpression(_) => TenseExpression::parse_table(t),
        NonTerminalType::ModifiedTense(_) => ModifiedTense::parse_table(t),
        NonTerminalType::AspectExpression(_) => AspectExpression::parse_table(t),
        NonTerminalType::ModifiedAspect(_) => ModifiedAspect::parse_table(t),
        NonTerminalType::MtaEvent(_) => MtaEvent::parse_table(t),
        NonTerminalType::TaEvent(_) => TaEvent::parse_table(t),
        NonTerminalType::AEvent(_) => AEvent::parse_table(t),
        NonTerminalType::Event(_) => Event::parse_table(t),
        NonTerminalType::PostModifiedBinaryPredicatePhrase(_) => PostModifiedBinaryPredicatePhrase::parse_table(t),
        NonTerminalType::PostBinaryPredicatePhraseExpression(_) => PostBinaryPredicatePhraseExpression::parse_table(t),
        NonTerminalType::BinaryPredicatePhrase(_) => BinaryPredicatePhrase::parse_table(t),
        NonTerminalType::ArgumentPhrase(_) => ArgumentPhrase::parse_table(t),
        NonTerminalType::ArgumentPhraseOrEpsilon(_) => ArgumentPhraseOrEpsilon::parse_table(t),
        NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(_) => DeterminedArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(_) => ArticlizedArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArgumentRetrievalBinaryPredicate(_) => ArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArgumentRetrievalWithArgumentPhrase(_) => ArgumentRetrievalWithArgumentPhrase::parse_table(t),
        NonTerminalType::Quantification(_) => Quantification::parse_table(t),
        NonTerminalType::Article(_) => Article::parse_table(t),
    }
}

#[derive(Clone, Debug)]
pub enum Rhs {
    Sequence(Vec<SymbolType>),
    Epsilon,
}

#[derive(Debug)]
pub struct Ast {
    s: S,
}

#[derive(Debug)]
pub struct IncompleteAst {
    // Define the structure of the ErrorAST here
}

trait NonTerminal {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized;
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized;
}

#[derive(Clone, Debug)]
enum SymbolType {
    Terminal(TerminalType),
    NonTerminal(NonTerminalType),
}

#[derive(Clone, Debug)]
enum NonTerminalType {
    S(Option<S>),
    DeclarativeSentence(Option<DeclarativeSentence>),
    InterrogativeSentence(Option<InterrogativeSentence>),
    PostModifiedMtaEvent(Option<PostModifiedMtaEvent>),
    PostMtaEventExpression(Option<PostMtaEventExpression>),
    ModalityExpression(Option<ModalityExpression>),
    ModalityWithAgent(Option<ModalityWithAgent>),
    ModifiedModality(Option<ModifiedModality>),
    ModalityAgent(Option<ModalityAgent>),
    TenseExpression(Option<TenseExpression>),
    ModifiedTense(Option<ModifiedTense>),
    AspectExpression(Option<AspectExpression>),
    ModifiedAspect(Option<ModifiedAspect>),
    MtaEvent(Option<MtaEvent>),
    TaEvent(Option<TaEvent>),
    AEvent(Option<AEvent>),
    Event(Option<Event>),
    PostModifiedBinaryPredicatePhrase(Option<PostModifiedBinaryPredicatePhrase>),
    PostBinaryPredicatePhraseExpression(Option<PostBinaryPredicatePhraseExpression>),
    BinaryPredicatePhrase(Option<BinaryPredicatePhrase>),
    ArgumentPhrase(Option<ArgumentPhrase>),
    ArgumentPhraseOrEpsilon(Option<ArgumentPhraseOrEpsilon>),
    DeterminedArgumentRetrievalBinaryPredicate(Option<DeterminedArgumentRetrievalBinaryPredicate>),
    ArticlizedArgumentRetrievalBinaryPredicate(Option<ArticlizedArgumentRetrievalBinaryPredicate>),
    ArgumentRetrievalBinaryPredicate(Option<ArgumentRetrievalBinaryPredicate>),
    ArgumentRetrievalWithArgumentPhrase(Option<ArgumentRetrievalWithArgumentPhrase>),
    Quantification(Option<Quantification>),
    Article(Option<Article>),
}

#[derive(Clone, Copy, Debug)]
enum TerminalType {
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

// non-terminal types

#[derive(Clone, Debug)]
struct S {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct DeclarativeSentence {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct InterrogativeSentence {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct PostModifiedMtaEvent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct PostMtaEventExpression {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModalityExpression {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModalityWithAgent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModifiedModality {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModalityAgent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct TenseExpression {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModifiedTense {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct AspectExpression {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ModifiedAspect {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct MtaEvent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct TaEvent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct AEvent {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct Event {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct PostModifiedBinaryPredicatePhrase {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct PostBinaryPredicatePhraseExpression {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct BinaryPredicatePhrase {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ArgumentPhrase {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ArgumentPhraseOrEpsilon {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct DeterminedArgumentRetrievalBinaryPredicate {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ArticlizedArgumentRetrievalBinaryPredicate {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ArgumentRetrievalBinaryPredicate {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct ArgumentRetrievalWithArgumentPhrase {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct Quantification {
    rhs: Rhs,
}

#[derive(Clone, Debug)]
struct Article {
    rhs: Rhs,
}

// terminal types

#[derive(Clone, Copy, Debug)]
struct InterrogativeMarker;

#[derive(Clone, Copy, Debug)]
struct Modality;

#[derive(Clone, Copy, Debug)]
struct Tense;

#[derive(Clone, Copy, Debug)]
struct Aspect;

#[derive(Clone, Copy, Debug)]
struct ModalityNegationBegin;

#[derive(Clone, Copy, Debug)]
struct TenseNegationBegin;

#[derive(Clone, Copy, Debug)]
struct AspectNegationBegin;

#[derive(Clone, Copy, Debug)]
struct Negation;

#[derive(Clone, Copy, Debug)]
struct BinaryPredicate;

#[derive(Clone, Copy, Debug)]
struct Argument;

#[derive(Clone, Copy, Debug)]
struct ArgumentPhraseSeparator;

#[derive(Clone, Copy, Debug)]
struct ArgumentPhraseTransposer;

#[derive(Clone, Copy, Debug)]
struct BinaryOperator;

#[derive(Clone, Copy, Debug)]
struct ArgumentRetriever;

#[derive(Clone, Copy, Debug)]
struct Quantifier;

#[derive(Clone, Copy, Debug)]
struct Articlizer;

#[derive(Clone, Copy, Debug)]
struct MtaEventToArgumentPhraseConverter;

#[derive(Clone, Copy, Debug)]
struct MtaEventToArgumentPhraseConverterEnd;

#[derive(Clone, Copy, Debug)]
struct PostMtaEventModifier;

#[derive(Clone, Copy, Debug)]
struct PostBinaryPredicatePhraseModifier;

#[derive(Clone, Copy, Debug)]
struct PostArgumentPhraseModifier;

#[derive(Clone, Copy, Debug)]
struct Eos;

impl NonTerminal for S {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::InterrogativeMarker(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::InterrogativeSentence(None)),
                ]))
            }
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::InterrogativeSentence(child1))] => {
                        if child1.is_none() {
                            panic!("InterrogativeSentence is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence(child1))] => {
                        if child1.is_none() {
                            panic!("DeclarativeSentence is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in S");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for DeclarativeSentence {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }            
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(child1))] => {
                        if child1.is_none() {
                            panic!("PostModifiedMtaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in DeclarativeSentence");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for InterrogativeSentence {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::InterrogativeMarker(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::InterrogativeMarker(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::InterrogativeMarker(child1)), SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent(child2))] => {
                        if child1.is_none() {
                            panic!("InterrogativeMarker is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("PostModifiedMtaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in InterrogativeSentence");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for PostModifiedMtaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::MtaEvent(child1)), SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression(child2))] => {
                        if child1.is_none() {
                            panic!("MtaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("PostMtaEventExpression is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in PostModifiedMtaEvent");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for PostMtaEventExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::PostMtaEventModifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostMtaEventModifier(None)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                ]))
            }
            TerminalType::Eos(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::PostMtaEventModifier(child1)), SymbolType::NonTerminal(NonTerminalType::MtaEvent(child2))] => {
                        if child1.is_none() {
                            panic!("PostMtaEventModifier is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("MtaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for ModalityExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent(None)),
                ]))
            }
            TerminalType::Tense(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent(child1))] => {
                        if child1.is_none() {
                            panic!("ModalityWithAgent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for ModalityWithAgent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality(None)),
                    SymbolType::NonTerminal(NonTerminalType::ModalityAgent(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality(None)),
                    SymbolType::NonTerminal(NonTerminalType::ModalityAgent(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::ModifiedModality(child1)), SymbolType::NonTerminal(NonTerminalType::ModalityAgent(child2))] => {
                        if child1.is_none() {
                            panic!("ModifiedModality is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ModalityAgent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ModalityWithAgent");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for ModifiedModality {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Modality(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ModalityNegationBegin(None)),
                    SymbolType::Terminal(TerminalType::Negation(None)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Modality(child1))] => {
                        if child1.is_none() {
                            panic!("Modality is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::ModalityNegationBegin(child1)), SymbolType::Terminal(TerminalType::Negation(child2)), SymbolType::NonTerminal(NonTerminalType::ModifiedModality(child3))] => {
                        if child1.is_none() {
                            panic!("ModalityNegationBegin is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("Negation is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("ModifiedModality is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ModifiedModality");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for ModalityAgent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostArgumentPhraseModifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2))] => {
                        if child1.is_none() {
                            panic!("PostArgumentPhraseModifier is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for TenseExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::ModifiedTense(child1))] => {
                        if child1.is_none() {
                            panic!("ModifiedTense is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for ModifiedTense {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Tense(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::TenseNegationBegin(None)),
                    SymbolType::Terminal(TerminalType::Negation(None)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Tense(child1))] => {
                        if child1.is_none() {
                            panic!("Tense is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::TenseNegationBegin(child1)), SymbolType::Terminal(TerminalType::Negation(child2)), SymbolType::NonTerminal(NonTerminalType::ModifiedTense(child3))] => {
                        if child1.is_none() {
                            panic!("TenseNegationBegin is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("Negation is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("ModifiedTense is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ModifiedModality");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for AspectExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(child1))] => {
                        if child1.is_none() {
                            panic!("ModifiedAspect is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for ModifiedAspect {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Aspect(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::AspectNegationBegin(None)),
                    SymbolType::Terminal(TerminalType::Negation(None)),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Aspect(child1))] => {
                        if child1.is_none() {
                            panic!("Aspect is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::AspectNegationBegin(child1)), SymbolType::Terminal(TerminalType::Negation(child2)), SymbolType::NonTerminal(NonTerminalType::ModifiedAspect(child3))] => {
                        if child1.is_none() {
                            panic!("AspectNegationBegin is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("Negation is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("ModifiedAspect is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ModifiedModality");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for MtaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::ModalityNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::ModalityExpression(child1)), SymbolType::NonTerminal(NonTerminalType::TaEvent(child2))] => {
                        if child1.is_none() {
                            panic!("ModalityExpression is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("TaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in MtaEvent");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for TaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::AEvent(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::TenseExpression(child1)), SymbolType::NonTerminal(NonTerminalType::AEvent(child2))] => {
                        if child1.is_none() {
                            panic!("TenseExpression is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("AEvent is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in TaEvent");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for AEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression(None)),
                    SymbolType::NonTerminal(NonTerminalType::Event(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::AspectExpression(child1)), SymbolType::NonTerminal(NonTerminalType::Event(child2))] => {
                        if child1.is_none() {
                            panic!("AspectExpression is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("Event is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in AEvent");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for Event {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon(child3))] => {
                        if child1.is_none() {
                            panic!("PostModifiedBinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("ArgumentPhraseOrEpsilon is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Event cannot be epsilon");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for PostModifiedBinaryPredicatePhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child1)), SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression(child2))] => {
                        if child1.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("PostBinaryPredicatePhraseExpression is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in PostModifiedBinaryPredicatePhrase: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("PostModifiedBinaryPredicatePhrase cannot be epsilon");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for PostBinaryPredicatePhraseExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Argument(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseSeparator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentRetriever(_) => {
                Ok(Rhs::Epsilon)    
            }
            TerminalType::MtaEventToArgumentPhraseConverter(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostBinaryPredicatePhraseModifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostBinaryPredicatePhraseModifier(None)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                ]))
            }
            TerminalType::PostArgumentPhraseModifier(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Eos(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::PostBinaryPredicatePhraseModifier(child1)), SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2))] => {
                        if child1.is_none() {
                            panic!("PostBinaryPredicatePhraseModifier is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in PostBinaryPredicatePhraseExpression");
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for BinaryPredicatePhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Negation(None)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryPredicate(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentPhraseTransposer(None)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryOperator(None)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Negation(child1)), SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2))] => {
                        if child1.is_none() {
                            panic!("Negation is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::BinaryPredicate(child1))] => {
                        if child1.is_none() {
                            panic!("BinaryPredicate is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::ArgumentPhraseTransposer(child1)), SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2))] => {
                        if child1.is_none() {
                            panic!("ArgumentPhraseTransposer is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::BinaryOperator(child1)), SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child2)), SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase(child3))] => {
                        if child1.is_none() {
                            panic!("BinaryOperator is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("BinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in BinaryPredicatePhrase");
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("BinaryPredicatePhrase cannot be epsilon");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for ArgumentPhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Negation(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                ]))
            }
            TerminalType::Argument(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Argument(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryOperator(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                ]))
            }
            TerminalType::ArgumentRetriever(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentRetriever(None)),
                    SymbolType::NonTerminal(NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::MtaEventToArgumentPhraseConverter(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverter(None)),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent(None)),
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverterEnd(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Negation(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2))] => {
                        if child1.is_none() {
                            panic!("Negation is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::Argument(child1))] => {
                        if child1.is_none() {
                            panic!("Argument is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::BinaryOperator(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child3))] => {
                        if child1.is_none() {
                            panic!("BinaryOperator is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::ArgumentRetriever(child1)), SymbolType::NonTerminal(NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(child2))] => {
                        if child1.is_none() {
                            panic!("ArgumentRetriever is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("DeterminedArgumentRetrievalBinaryPredicate is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    [SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverter(child1)), SymbolType::NonTerminal(NonTerminalType::MtaEvent(child2)), SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverterEnd(child3))] => {
                        if child1.is_none() {
                            panic!("MtaEventToArgumentPhraseConverter is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("MtaEvent is None");
                            return Err(IncompleteAst {});
                        }
                        if child3.is_none() {
                            panic!("MtaEventToArgumentPhraseConverterEnd is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in ArgumentPhrase");
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("ArgumentPhrase cannot be epsilon");
                return Err(IncompleteAst {})
            }
        }
    }
}

impl NonTerminal for ArgumentPhraseOrEpsilon {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::ArgumentPhraseSeparator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentPhraseSeparator(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                ]))
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Eos(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::ArgumentPhraseSeparator(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2))] => {
                        if child1.is_none() {
                            panic!("ArgumentPhraseSeparator is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for DeterminedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::Quantifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::Articlizer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::Quantification(child1)), SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(child2))] => {
                        if child1.is_none() {
                            panic!("Quantification is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArticlizedArgumentRetrievalBinaryPredicate is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ModifiedModality");
                return Err(IncompleteAst {});
            }
        }
    }   
}

impl NonTerminal for ArticlizedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            TerminalType::Articlizer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::Article(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate(child2))] => {
                        if child1.is_none() {
                            panic!("Article is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentRetrievalBinaryPredicate is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ArticlizedArgumentRetrievalBinaryPredicate");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for ArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(None)),
                ]))
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(None)),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(None)),
                ]))
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase(child2))] => {
                        if child1.is_none() {
                            panic!("PostModifiedBinaryPredicatePhrase is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentRetrievalWithArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                panic!("Unexpected epsilon in ArgumentRetrievalBinaryPredicate");
                return Err(IncompleteAst {});
            }
        }
    }
}

impl NonTerminal for ArgumentRetrievalWithArgumentPhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Argument(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseSeparator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentRetriever(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverter(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostArgumentPhraseModifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(None)),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(None)),
                ]))
            }
            TerminalType::Eos(_) => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier(child1)), SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase(child2))] => {
                        if child1.is_none() {
                            panic!("PostArgumentPhraseModifier is None");
                            return Err(IncompleteAst {});
                        }
                        if child2.is_none() {
                            panic!("ArgumentPhrase is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for Quantification {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Quantifier(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Quantifier(None)),
                ]))
            }
            TerminalType::Articlizer(_) => {
                Ok(Rhs::Epsilon)    
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Quantifier(child1))] => {
                        if child1.is_none() {
                            panic!("Quantifier is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

impl NonTerminal for Article {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator(_) => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Articlizer(_) => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Articlizer(None)),
                ]))
            }
            _ => {
                panic!("Unexpected token: {:?}", current_token);
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(rhs: Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                match symbols.as_slice() {
                    [SymbolType::Terminal(TerminalType::Articlizer(child1))] => {
                        if child1.is_none() {
                            panic!("Articlizer is None");
                            return Err(IncompleteAst {});
                        }
                        return Ok(Self {rhs})
                    }
                    _ => {
                        panic!("Unexpected symbols in S: {:?}", symbols);
                        return Err(IncompleteAst {})
                    }
                }
            }
            Rhs::Epsilon => {
                return Ok(Self {rhs})
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_自動生成されたparse_tableと一致する() {
        use std::any::type_name;
        use std::collections::HashMap;

        let s = grammar::NonTerminal(type_name::<S>().split("::").last().unwrap().to_string());
        let declarative_sentence = grammar::NonTerminal(type_name::<DeclarativeSentence>().split("::").last().unwrap().to_string());
        let interrogative_sentence = grammar::NonTerminal(type_name::<InterrogativeSentence>().split("::").last().unwrap().to_string());
        let post_modified_mta_event = grammar::NonTerminal(type_name::<PostModifiedMtaEvent>().split("::").last().unwrap().to_string());
        let post_mta_event_expression = grammar::NonTerminal(type_name::<PostMtaEventExpression>().split("::").last().unwrap().to_string());
        let modality_expression = grammar::NonTerminal(type_name::<ModalityExpression>().split("::").last().unwrap().to_string());
        let modality_with_agent = grammar::NonTerminal(type_name::<ModalityWithAgent>().split("::").last().unwrap().to_string());
        let modified_modality = grammar::NonTerminal(type_name::<ModifiedModality>().split("::").last().unwrap().to_string());
        let modality_agent = grammar::NonTerminal(type_name::<ModalityAgent>().split("::").last().unwrap().to_string());
        let tense_expression = grammar::NonTerminal(type_name::<TenseExpression>().split("::").last().unwrap().to_string());
        let modified_tense = grammar::NonTerminal(type_name::<ModifiedTense>().split("::").last().unwrap().to_string());
        let aspect_expression = grammar::NonTerminal(type_name::<AspectExpression>().split("::").last().unwrap().to_string());
        let modified_aspect = grammar::NonTerminal(type_name::<ModifiedAspect>().split("::").last().unwrap().to_string());
        let mta_event = grammar::NonTerminal(type_name::<MtaEvent>().split("::").last().unwrap().to_string());
        let ta_event = grammar::NonTerminal(type_name::<TaEvent>().split("::").last().unwrap().to_string());
        let a_event = grammar::NonTerminal(type_name::<AEvent>().split("::").last().unwrap().to_string());
        let event = grammar::NonTerminal(type_name::<Event>().split("::").last().unwrap().to_string());
        let post_modified_binary_predicate_phrase = grammar::NonTerminal(type_name::<PostModifiedBinaryPredicatePhrase>().split("::").last().unwrap().to_string());
        let post_binary_predicate_phrase_expression = grammar::NonTerminal(type_name::<PostBinaryPredicatePhraseExpression>().split("::").last().unwrap().to_string());
        let binary_predicate_phrase = grammar::NonTerminal(type_name::<BinaryPredicatePhrase>().split("::").last().unwrap().to_string());
        let argument_phrase = grammar::NonTerminal(type_name::<ArgumentPhrase>().split("::").last().unwrap().to_string());
        let argument_phrase_or_epsilon = grammar::NonTerminal(type_name::<ArgumentPhraseOrEpsilon>().split("::").last().unwrap().to_string());
        let determined_argument_retrieval_binary_predicate = grammar::NonTerminal(type_name::<DeterminedArgumentRetrievalBinaryPredicate>().split("::").last().unwrap().to_string());
        let articlized_argument_retrieval_binary_predicate = grammar::NonTerminal(type_name::<ArticlizedArgumentRetrievalBinaryPredicate>().split("::").last().unwrap().to_string());
        let argument_retrieval_binary_predicate = grammar::NonTerminal(type_name::<ArgumentRetrievalBinaryPredicate>().split("::").last().unwrap().to_string());
        let argument_retrieval_with_argument_phrase = grammar::NonTerminal(type_name::<ArgumentRetrievalWithArgumentPhrase>().split("::").last().unwrap().to_string());
        let quantification = grammar::NonTerminal(type_name::<Quantification>().split("::").last().unwrap().to_string());
        let article = grammar::NonTerminal(type_name::<Article>().split("::").last().unwrap().to_string());

        let interrogative_marker = grammar::Terminal(type_name::<InterrogativeMarker>().split("::").last().unwrap().to_string());
        let modality = grammar::Terminal(type_name::<Modality>().split("::").last().unwrap().to_string());
        let tense = grammar::Terminal(type_name::<Tense>().split("::").last().unwrap().to_string());
        let aspect = grammar::Terminal(type_name::<Aspect>().split("::").last().unwrap().to_string());
        let modality_negation_begin = grammar::Terminal(type_name::<ModalityNegationBegin>().split("::").last().unwrap().to_string());
        let tense_negation_begin = grammar::Terminal(type_name::<TenseNegationBegin>().split("::").last().unwrap().to_string());
        let aspect_negation_begin = grammar::Terminal(type_name::<AspectNegationBegin>().split("::").last().unwrap().to_string());
        let negation = grammar::Terminal(type_name::<Negation>().split("::").last().unwrap().to_string());
        let binary_predicate = grammar::Terminal(type_name::<BinaryPredicate>().split("::").last().unwrap().to_string());
        let argument = grammar::Terminal(type_name::<Argument>().split("::").last().unwrap().to_string());
        let argument_phrase_separator = grammar::Terminal(type_name::<ArgumentPhraseSeparator>().split("::").last().unwrap().to_string());
        let argument_phrase_transposer = grammar::Terminal(type_name::<ArgumentPhraseTransposer>().split("::").last().unwrap().to_string());
        let binary_operator = grammar::Terminal(type_name::<BinaryOperator>().split("::").last().unwrap().to_string());
        let argument_retriever = grammar::Terminal(type_name::<ArgumentRetriever>().split("::").last().unwrap().to_string());
        let quantifier = grammar::Terminal(type_name::<Quantifier>().split("::").last().unwrap().to_string());
        let articlizer = grammar::Terminal(type_name::<Articlizer>().split("::").last().unwrap().to_string());
        let mta_event_to_argument_phrase_converter = grammar::Terminal(type_name::<MtaEventToArgumentPhraseConverter>().split("::").last().unwrap().to_string());
        let mta_event_to_argument_phrase_converter_end = grammar::Terminal(type_name::<MtaEventToArgumentPhraseConverterEnd>().split("::").last().unwrap().to_string());
        let post_mta_event_modifier = grammar::Terminal(type_name::<PostMtaEventModifier>().split("::").last().unwrap().to_string());
        let post_binary_predicate_phrase_modifier = grammar::Terminal(type_name::<PostBinaryPredicatePhraseModifier>().split("::").last().unwrap().to_string());
        let post_argument_phrase_modifier = grammar::Terminal(type_name::<PostArgumentPhraseModifier>().split("::").last().unwrap().to_string());
        let eos = grammar::Terminal("Eos".split("::").last().unwrap().to_string());

        let mut rules = HashMap::new();
        rules.insert(
            s.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(declarative_sentence.clone())]),
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(interrogative_sentence.clone())]),
            ]
        );
        rules.insert(
            declarative_sentence.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(post_modified_mta_event.clone())]),
            ]
        );
        rules.insert(
            interrogative_sentence.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(interrogative_marker.clone()),
                    grammar::Symbol::NonTerminal(post_modified_mta_event.clone()),
                ]),
            ]
        );
        rules.insert(
            post_modified_mta_event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                    grammar::Symbol::NonTerminal(post_mta_event_expression.clone()),
                ]),
            ]
        );
        rules.insert(
            post_mta_event_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_mta_event_modifier.clone()),
                    grammar::Symbol::NonTerminal(mta_event.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            modality_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(modality_with_agent.clone()), 
                ]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            modality_with_agent.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(modified_modality.clone()),
                    grammar::Symbol::NonTerminal(modality_agent.clone()),
                ]),
            ]
        );
        rules.insert(
            modified_modality.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(modality.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(modality_negation_begin.clone()),
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(modified_modality.clone()),
                ]),
            ]
        );
        rules.insert(
            modality_agent.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_argument_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            tense_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(modified_tense.clone())]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            modified_tense.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(tense.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(tense_negation_begin.clone()),
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(modified_tense.clone()),
                ]),
            ]
        );
        rules.insert(
            aspect_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::NonTerminal(modified_aspect.clone())]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            modified_aspect.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(aspect.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(aspect_negation_begin.clone()),
                    grammar::Symbol::Terminal(negation.clone()),
                    grammar::Symbol::NonTerminal(modified_aspect.clone()),
                ]),
            ]
        );
        rules.insert(
            mta_event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(modality_expression.clone()),
                    grammar::Symbol::NonTerminal(ta_event.clone()),
                ]),
            ]
        );
        rules.insert(
            ta_event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(tense_expression.clone()),
                    grammar::Symbol::NonTerminal(a_event.clone()),
                ]),
            ]
        );
        rules.insert(
            a_event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(aspect_expression.clone()),
                    grammar::Symbol::NonTerminal(event.clone()),
                ]),
            ]
        );
        rules.insert(
            event.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(post_modified_binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase_or_epsilon.clone()),
                ]),
            ]
        );
        rules.insert(
            post_modified_binary_predicate_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(post_binary_predicate_phrase_expression.clone()),
                ]),
            ]
        );
        rules.insert(
            post_binary_predicate_phrase_expression.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_binary_predicate_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(binary_predicate_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ]
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
            ]
        );
        rules.insert(
            argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(argument.clone())]),
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_retriever.clone()),
                    grammar::Symbol::NonTerminal(determined_argument_retrieval_binary_predicate.clone()),
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
            ]
        );
        rules.insert(
            argument_phrase_or_epsilon.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(argument_phrase_separator.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            determined_argument_retrieval_binary_predicate.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(quantification.clone()),
                    grammar::Symbol::NonTerminal(articlized_argument_retrieval_binary_predicate.clone()),
                ]),
            ]
        );
        rules.insert(
            articlized_argument_retrieval_binary_predicate.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(article.clone()),
                    grammar::Symbol::NonTerminal(argument_retrieval_binary_predicate.clone()),
                ]),
            ]
        );
        rules.insert(
            argument_retrieval_binary_predicate.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::NonTerminal(post_modified_binary_predicate_phrase.clone()),
                    grammar::Symbol::NonTerminal(argument_retrieval_with_argument_phrase.clone()),
                ]),
            ]
        );
        rules.insert(
            argument_retrieval_with_argument_phrase.clone(),
            vec![
                grammar::Rhs::Sequence(vec![
                    grammar::Symbol::Terminal(post_argument_phrase_modifier.clone()),
                    grammar::Symbol::NonTerminal(argument_phrase.clone()),
                ]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            quantification.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(quantifier.clone())]),
                grammar::Rhs::Epsilon,
            ]
        );
        rules.insert(
            article.clone(),
            vec![
                grammar::Rhs::Sequence(vec![grammar::Symbol::Terminal(articlizer.clone())]),
                grammar::Rhs::Epsilon,
            ]
        );

        let g = grammar::LL1Grammar::new(rules);
        let non_terminals = g.get_non_terminals();
        let terminals = g.get_terminals();
        let parse_table = g.get_parse_table();

        for nt in non_terminals {
            for t in terminals {
                let lt = grammar::Lookahead::Terminal(t.clone());
                let tt = match t.0.as_str() {
                    "InterrogativeMarker" => TerminalType::InterrogativeMarker(None),
                    "Modality" => TerminalType::Modality(None),
                    "Tense" => TerminalType::Tense(None),
                    "Aspect" => TerminalType::Aspect(None),
                    "ModalityNegationBegin" => TerminalType::ModalityNegationBegin(None),
                    "TenseNegationBegin" => TerminalType::TenseNegationBegin(None),
                    "AspectNegationBegin" => TerminalType::AspectNegationBegin(None),
                    "Negation" => TerminalType::Negation(None),
                    "BinaryPredicate" => TerminalType::BinaryPredicate(None),
                    "Argument" => TerminalType::Argument(None),
                    "ArgumentPhraseSeparator" => TerminalType::ArgumentPhraseSeparator(None),
                    "ArgumentPhraseTransposer" => TerminalType::ArgumentPhraseTransposer(None),
                    "BinaryOperator" => TerminalType::BinaryOperator(None),
                    "ArgumentRetriever" => TerminalType::ArgumentRetriever(None),
                    "Quantifier" => TerminalType::Quantifier(None),
                    "Articlizer" => TerminalType::Articlizer(None),
                    "MtaEventToArgumentPhraseConverter" => TerminalType::MtaEventToArgumentPhraseConverter(None),
                    "MtaEventToArgumentPhraseConverterEnd" => TerminalType::MtaEventToArgumentPhraseConverterEnd(None),
                    "PostMtaEventModifier" => TerminalType::PostMtaEventModifier(None),
                    "PostBinaryPredicatePhraseModifier" => TerminalType::PostBinaryPredicatePhraseModifier(None),
                    "PostArgumentPhraseModifier" => TerminalType::PostArgumentPhraseModifier(None),
                    _ => self::panic!("Unknown terminal type: {}", t.0),
                };

                let auto_parse_result = parse_table.get(&(nt.clone(), lt.clone()));
                helper(
                    &nt,
                    &lt,
                    tt,
                    &auto_parse_result,
                );
            }
            let lt = grammar::Lookahead::Eos;
            let tt = TerminalType::Eos(None);

            let auto_parse_result = parse_table.get(&(nt.clone(), lt.clone()));
            helper(&nt, &grammar::Lookahead::Eos, tt, &auto_parse_result);
        }
    }

    fn helper(nt: &grammar::NonTerminal, lt: &grammar::Lookahead, tt: TerminalType, auto_parse_result: &Option<&grammar::Rhs>) {
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
                    "PostModifiedBinaryPredicatePhrase" => PostModifiedBinaryPredicatePhrase::parse_table(tt),
                    "PostBinaryPredicatePhraseExpression" => PostBinaryPredicatePhraseExpression::parse_table(tt),
                    "BinaryPredicatePhrase" => BinaryPredicatePhrase::parse_table(tt),
                    "ArgumentPhrase" => ArgumentPhrase::parse_table(tt),
                    "ArgumentPhraseOrEpsilon" => ArgumentPhraseOrEpsilon::parse_table(tt),
                    "DeterminedArgumentRetrievalBinaryPredicate" => DeterminedArgumentRetrievalBinaryPredicate::parse_table(tt),
                    "ArticlizedArgumentRetrievalBinaryPredicate" => ArticlizedArgumentRetrievalBinaryPredicate::parse_table(tt),
                    "ArgumentRetrievalBinaryPredicate" => ArgumentRetrievalBinaryPredicate::parse_table(tt),
                    "ArgumentRetrievalWithArgumentPhrase" => ArgumentRetrievalWithArgumentPhrase::parse_table(tt),
                    "Quantification" => Quantification::parse_table(tt),
                    "Article" => Article::parse_table(tt),
                    _ => self::panic!("Unknown non-terminal type: {}", nt.0),
                };

                match (auto_parse_result, manual_parse_result) {
                    (Some(auto_result), Ok(manual_result)) => {
                        match (auto_result, &manual_result) {
                            (grammar::Rhs::Epsilon, Rhs::Epsilon) => {}
                            (grammar::Rhs::Sequence(auto_seq), Rhs::Sequence(manual_seq)) => {
                                assert_eq!(auto_seq.len(), manual_seq.len(), "Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto sequence length: {}, Manual sequence length: {}", nt, lt, auto_seq.len(), manual_seq.len());
                                for (auto_symbol, manual_symbol) in auto_seq.iter().zip(manual_seq.iter()) {
                                    match (auto_symbol, manual_symbol) {
                                        (grammar::Symbol::NonTerminal(auto_nt), SymbolType::NonTerminal(manual_nt)) => {
                                            let manual_nt_variant_name = match manual_nt {
                                                NonTerminalType::S(_) => "S",
                                                NonTerminalType::DeclarativeSentence(_) => "DeclarativeSentence",
                                                NonTerminalType::InterrogativeSentence(_) => "InterrogativeSentence",
                                                NonTerminalType::PostModifiedMtaEvent(_) => "PostModifiedMtaEvent",
                                                NonTerminalType::PostMtaEventExpression(_) => "PostMtaEventExpression",
                                                NonTerminalType::ModalityExpression(_) => "ModalityExpression",
                                                NonTerminalType::ModalityWithAgent(_) => "ModalityWithAgent",
                                                NonTerminalType::ModifiedModality(_) => "ModifiedModality",
                                                NonTerminalType::ModalityAgent(_) => "ModalityAgent",
                                                NonTerminalType::TenseExpression(_) => "TenseExpression",
                                                NonTerminalType::ModifiedTense(_) => "ModifiedTense",
                                                NonTerminalType::AspectExpression(_) => "AspectExpression",
                                                NonTerminalType::ModifiedAspect(_) => "ModifiedAspect",
                                                NonTerminalType::MtaEvent(_) => "MtaEvent",
                                                NonTerminalType::TaEvent(_) => "TaEvent",
                                                NonTerminalType::AEvent(_) => "AEvent",
                                                NonTerminalType::Event(_) => "Event",
                                                NonTerminalType::PostModifiedBinaryPredicatePhrase(_) => "PostModifiedBinaryPredicatePhrase",
                                                NonTerminalType::PostBinaryPredicatePhraseExpression(_) => "PostBinaryPredicatePhraseExpression",
                                                NonTerminalType::BinaryPredicatePhrase(_) => "BinaryPredicatePhrase",
                                                NonTerminalType::ArgumentPhrase(_) => "ArgumentPhrase",
                                                NonTerminalType::ArgumentPhraseOrEpsilon(_) => "ArgumentPhraseOrEpsilon",
                                                NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate(_) => "DeterminedArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate(_) => "ArticlizedArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArgumentRetrievalBinaryPredicate(_) => "ArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArgumentRetrievalWithArgumentPhrase(_) => "ArgumentRetrievalWithArgumentPhrase",
                                                NonTerminalType::Quantification(_) => "Quantification",
                                                NonTerminalType::Article(_) => "Article"
                                            };
                                            assert_eq!(auto_nt.0, manual_nt_variant_name, "Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_nt, manual_nt);
                                        }
                                        (grammar::Symbol::Terminal(auto_t), SymbolType::Terminal(manual_t)) => {
                                            let manual_nt_variant_name = match manual_t {
                                                TerminalType::InterrogativeMarker(_) => "InterrogativeMarker",
                                                TerminalType::Modality(_) => "Modality",
                                                TerminalType::Tense(_) => "Tense",
                                                TerminalType::Aspect(_) => "Aspect",
                                                TerminalType::ModalityNegationBegin(_) => "ModalityNegationBegin",
                                                TerminalType::TenseNegationBegin(_) => "TenseNegationBegin",
                                                TerminalType::AspectNegationBegin(_) => "AspectNegationBegin",
                                                TerminalType::Negation(_) => "Negation",
                                                TerminalType::BinaryPredicate(_) => "BinaryPredicate",
                                                TerminalType::Argument(_) => "Argument",
                                                TerminalType::ArgumentPhraseSeparator(_) => "ArgumentPhraseSeparator",
                                                TerminalType::ArgumentPhraseTransposer(_) => "ArgumentPhraseTransposer",
                                                TerminalType::BinaryOperator(_) => "BinaryOperator",
                                                TerminalType::ArgumentRetriever(_) => "ArgumentRetriever",
                                                TerminalType::Quantifier(_) => "Quantifier",
                                                TerminalType::Articlizer(_) => "Articlizer",
                                                TerminalType::MtaEventToArgumentPhraseConverter(_) => "MtaEventToArgumentPhraseConverter",
                                                TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => "MtaEventToArgumentPhraseConverterEnd",
                                                TerminalType::PostMtaEventModifier(_) => "PostMtaEventModifier",
                                                TerminalType::PostBinaryPredicatePhraseModifier(_) => "PostBinaryPredicatePhraseModifier",
                                                TerminalType::PostArgumentPhraseModifier(_) => "PostArgumentPhraseModifier",
                                                _ => self::panic!("Unknown terminal type: maybe Eos"),
                                            };
                                            assert_eq!(auto_t.0, manual_nt_variant_name, "Parse table mismatch for terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_t, manual_t);
                                        }
                                        _ => {
                                            self::panic!("Unexpected symbol type in parse table for non-terminal: {:?}, terminal: {:?}", nt, lt);
                                        }
                                    }
                                }
                            }
                            _ => {
                                self::panic!("Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_result, manual_result);
                            }
                        }
                    }
                    (None, Err(_)) => {}
                    (Some(_auto_result), Err(_)) => {
                        self::panic!("Auto parse result found but manual parse failed for non-terminal: {:?}, terminal: {:?}", nt, lt);
                    }
                    (None, Ok(_manual_result)) => {
                        self::panic!("Manual parse result found but auto parse result is None for non-terminal: {:?}, terminal: {:?}", nt, lt);
                    }
                }
    }

    #[test]
    fn test_parse() {
        let tokens: Vec<TerminalType> = vec![
            TerminalType::BinaryPredicate(Some(BinaryPredicate {})),
            TerminalType::Argument(Some(Argument {})),
            TerminalType::ArgumentPhraseSeparator(Some(ArgumentPhraseSeparator {})),
            TerminalType::Argument(Some(Argument {})),
            TerminalType::Eos(Some(Eos {})),
        ];
        println!("Testing parse with tokens: {:?}", parse(tokens));
    }
}
