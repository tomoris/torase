mod grammar;

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn parse(tokens: Vec<Box<dyn Terminal>>) -> Result<Ast, IncompleteAst> {
    let mut stack: Vec<SymbolType> = vec![SymbolType::Terminal(TerminalType::Eos), SymbolType::NonTerminal(NonTerminalType::S)];
    let mut current_token_index = 0;

    while !stack.is_empty() {
        parse_node(&mut stack, &tokens, &mut current_token_index)?;
    }
    todo!()
}

fn parse_node(stack: &mut Vec<SymbolType>, tokens: &Vec<Box<dyn Terminal>>, current_token_index: &mut usize) -> Result<Box<dyn Symbol>, IncompleteAst> {
    let current_token = tokens.get(*current_token_index).unwrap();
    let top_symbol = stack.pop().unwrap();

    match top_symbol {
        SymbolType::Terminal(terminal) => {
            match terminal {
                TerminalType::Eos => {
                    if TerminalType::Eos == current_token.terminal_type() {
                        return Ok(current_token.clone_box());
                    } else {
                        return Err(IncompleteAst {});
                    }
                }
                _ => {
                    if terminal == current_token.terminal_type() {
                        return Ok(current_token.clone_box());
                    } else {
                        return Err(IncompleteAst {});
                    }
                }
            }
        }
        SymbolType::NonTerminal(non_terminal) => {
            let rhs = parse_table(non_terminal, current_token.terminal_type())?;
            match rhs {
                Rhs::Sequence(ref symbols) => {
                    for symbol in symbols.iter().rev() {
                        stack.push(*symbol);
                    }
                    let mut child_nodes = Vec::new();
                    for _ in symbols.iter() {
                        match parse_node(stack, tokens, current_token_index) {
                            Ok(child_node) => {
                                child_nodes.push(child_node);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }

                    }
                    return Ok(new_non_terminal(&non_terminal, &child_nodes, &rhs)?);
                }
                Rhs::Epsilon => {
                    return parse_node(stack, tokens, current_token_index);
                }
            }
        }
    }
}

fn new_non_terminal(non_terminal: &NonTerminalType, children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Box<dyn NonTerminal>, IncompleteAst> {
    match non_terminal {
        NonTerminalType::S => Ok(Box::new(S::try_new(children, rhs)?)),
        NonTerminalType::DeclarativeSentence => Ok(Box::new(DeclarativeSentence::try_new(children, rhs)?)),
        NonTerminalType::InterrogativeSentence => Ok(Box::new(InterrogativeSentence::try_new(children, rhs)?)),
        NonTerminalType::PostModifiedMtaEvent => Ok(Box::new(PostModifiedMtaEvent::try_new(children, rhs)?)),
        NonTerminalType::PostMtaEventExpression => Ok(Box::new(PostMtaEventExpression::try_new(children, rhs)?)),
        NonTerminalType::ModalityExpression => Ok(Box::new(ModalityExpression::try_new(children, rhs)?)),
        NonTerminalType::ModalityWithAgent => Ok(Box::new(ModalityWithAgent::try_new(children, rhs)?)),
        NonTerminalType::ModifiedModality => Ok(Box::new(ModifiedModality::try_new(children, rhs)?)),
        NonTerminalType::ModalityAgent => Ok(Box::new(ModalityAgent::try_new(children, rhs)?)),
        NonTerminalType::TenseExpression => Ok(Box::new(TenseExpression::try_new(children, rhs)?)),
        NonTerminalType::ModifiedTense => Ok(Box::new(ModifiedTense::try_new(children, rhs)?)),
        NonTerminalType::AspectExpression => Ok(Box::new(AspectExpression::try_new(children, rhs)?)),
        NonTerminalType::ModifiedAspect => Ok(Box::new(ModifiedAspect::try_new(children, rhs)?)),
        NonTerminalType::MtaEvent => Ok(Box::new(MtaEvent::try_new(children, rhs)?)),
        NonTerminalType::TaEvent => Ok(Box::new(TaEvent::try_new(children, rhs)?)),
        NonTerminalType::AEvent => Ok(Box::new(AEvent::try_new(children, rhs)?)),
        NonTerminalType::Event => Ok(Box::new(Event::try_new(children, rhs)?)),
        NonTerminalType::PostModifiedBinaryPredicatePhrase => Ok(Box::new(PostModifiedBinaryPredicatePhrase::try_new(children, rhs)?)),
        NonTerminalType::PostBinaryPredicatePhraseExpression => Ok(Box::new(PostBinaryPredicatePhraseExpression::try_new(children, rhs)?)),
        NonTerminalType::BinaryPredicatePhrase => Ok(Box::new(BinaryPredicatePhrase::try_new(children, rhs)?)),
        NonTerminalType::ArgumentPhrase => Ok(Box::new(ArgumentPhrase::try_new(children, rhs)?)),
        NonTerminalType::ArgumentPhraseOrEpsilon => Ok(Box::new(ArgumentPhraseOrEpsilon::try_new(children, rhs)?)),
        NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => Ok(Box::new(DeterminedArgumentRetrievalBinaryPredicate::try_new(children, rhs)?)),
        NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => Ok(Box::new(ArticlizedArgumentRetrievalBinaryPredicate::try_new(children, rhs)?)),
        NonTerminalType::ArgumentRetrievalBinaryPredicate => Ok(Box::new(ArgumentRetrievalBinaryPredicate::try_new(children, rhs)?)),
        NonTerminalType::ArgumentRetrievalWithArgumentPhrase => Ok(Box::new(ArgumentRetrievalWithArgumentPhrase::try_new(children, rhs)?)),
        NonTerminalType::Quantification => Ok(Box::new(Quantification::try_new(children, rhs)?)),
        NonTerminalType::Article => Ok(Box::new(Article::try_new(children, rhs)?)),
        _ => Err(IncompleteAst {}),
    }
}


fn parse_table(nt: NonTerminalType, t: TerminalType) -> Result<Rhs, IncompleteAst> {
    match nt {
        NonTerminalType::S => S::parse_table(t),
        NonTerminalType::DeclarativeSentence => DeclarativeSentence::parse_table(t),
        NonTerminalType::InterrogativeSentence => InterrogativeSentence::parse_table(t),
        NonTerminalType::PostModifiedMtaEvent => PostModifiedMtaEvent::parse_table(t),
        NonTerminalType::PostMtaEventExpression => PostMtaEventExpression::parse_table(t),
        NonTerminalType::ModalityExpression => ModalityExpression::parse_table(t),
        NonTerminalType::ModalityWithAgent => ModalityWithAgent::parse_table(t),
        NonTerminalType::ModifiedModality => ModifiedModality::parse_table(t),
        NonTerminalType::ModalityAgent => ModalityAgent::parse_table(t),
        NonTerminalType::TenseExpression => TenseExpression::parse_table(t),
        NonTerminalType::ModifiedTense => ModifiedTense::parse_table(t),
        NonTerminalType::AspectExpression => AspectExpression::parse_table(t),
        NonTerminalType::ModifiedAspect => ModifiedAspect::parse_table(t),
        NonTerminalType::MtaEvent => MtaEvent::parse_table(t),
        NonTerminalType::TaEvent => TaEvent::parse_table(t),
        NonTerminalType::AEvent => AEvent::parse_table(t),
        NonTerminalType::Event => Event::parse_table(t),
        NonTerminalType::PostModifiedBinaryPredicatePhrase => PostModifiedBinaryPredicatePhrase::parse_table(t),
        NonTerminalType::PostBinaryPredicatePhraseExpression => PostBinaryPredicatePhraseExpression::parse_table(t),
        NonTerminalType::BinaryPredicatePhrase => BinaryPredicatePhrase::parse_table(t),
        NonTerminalType::ArgumentPhrase => ArgumentPhrase::parse_table(t),
        NonTerminalType::ArgumentPhraseOrEpsilon => ArgumentPhraseOrEpsilon::parse_table(t),
        NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => DeterminedArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => ArticlizedArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArgumentRetrievalBinaryPredicate => ArgumentRetrievalBinaryPredicate::parse_table(t),
        NonTerminalType::ArgumentRetrievalWithArgumentPhrase => ArgumentRetrievalWithArgumentPhrase::parse_table(t),
        NonTerminalType::Quantification => Quantification::parse_table(t),
        NonTerminalType::Article => Article::parse_table(t),
    }
}

#[derive(Debug)]
pub enum Rhs {
    Sequence(Vec<SymbolType>),
    Epsilon,
}

#[derive(Debug)]
pub struct Ast {
    // Define the structure of the AST here
}

#[derive(Debug)]
pub struct IncompleteAst {
    // Define the structure of the ErrorAST here
}

trait Symbol {
    // Define the methods that all symbols (terminals and non-terminals) should implement
}

trait NonTerminal:  Symbol {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized;
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SymbolType {
    Terminal(TerminalType),
    NonTerminal(NonTerminalType),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NonTerminalType {
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum TerminalType {
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


trait Terminal: Symbol {
    fn terminal_type(&self) -> TerminalType;
    fn clone_box(&self) -> Box<dyn Terminal>;
}

impl Clone for Box<dyn Terminal> {
    fn clone(&self) -> Box<dyn Terminal> {
        self.clone_box()
    }
}

// non-terminal types

struct S;

struct DeclarativeSentence;

struct InterrogativeSentence;

struct PostModifiedMtaEvent;

struct PostMtaEventExpression;

struct ModalityExpression;

struct ModalityWithAgent;

struct ModifiedModality;

struct ModalityAgent;

struct TenseExpression;

struct ModifiedTense;

struct AspectExpression;

struct ModifiedAspect;

struct MtaEvent;

struct TaEvent;

struct AEvent;

struct Event;

struct PostModifiedBinaryPredicatePhrase;

struct PostBinaryPredicatePhraseExpression;

struct BinaryPredicatePhrase;

struct ArgumentPhrase;

struct ArgumentPhraseOrEpsilon;

struct DeterminedArgumentRetrievalBinaryPredicate;

struct ArticlizedArgumentRetrievalBinaryPredicate;

struct ArgumentRetrievalBinaryPredicate;

struct ArgumentRetrievalWithArgumentPhrase;

struct Quantification;

struct Article;

// terminal types

#[derive(Clone, Debug)]
struct InterrogativeMarker;

#[derive(Clone, Debug)]
struct Modality;

#[derive(Clone, Debug)]
struct Tense;

#[derive(Clone, Debug)]
struct Aspect;

#[derive(Clone, Debug)]
struct ModalityNegationBegin;

#[derive(Clone, Debug)]
struct TenseNegationBegin;

#[derive(Clone, Debug)]
struct AspectNegationBegin;

#[derive(Clone, Debug)]
struct Negation;

#[derive(Clone, Debug)]
struct BinaryPredicate;

#[derive(Clone, Debug)]
struct Argument;

#[derive(Clone, Debug)]
struct ArgumentPhraseSeparator;

#[derive(Clone, Debug)]
struct ArgumentPhraseTransposer;

#[derive(Clone, Debug)]
struct BinaryOperator;

#[derive(Clone, Debug)]
struct ArgumentRetriever;

#[derive(Clone, Debug)]
struct Quantifier;

#[derive(Clone, Debug)]
struct Articlizer;

#[derive(Clone, Debug)]
struct MtaEventToArgumentPhraseConverter;

#[derive(Clone, Debug)]
struct MtaEventToArgumentPhraseConverterEnd;

#[derive(Clone, Debug)]
struct PostMtaEventModifier;

#[derive(Clone, Debug)]
struct PostBinaryPredicatePhraseModifier;

#[derive(Clone, Debug)]
struct PostArgumentPhraseModifier;

#[derive(Clone, Debug)]
struct Eos;

impl Symbol for S {}
impl NonTerminal for S {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::InterrogativeMarker => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::InterrogativeSentence),
                ]))
            }
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::DeclarativeSentence),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        todo!()
    }
}

impl Symbol for DeclarativeSentence {}
impl NonTerminal for DeclarativeSentence {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }            
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        todo!()
    }
}

impl Symbol for InterrogativeSentence {}
impl NonTerminal for InterrogativeSentence {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::InterrogativeMarker => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::InterrogativeMarker),
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedMtaEvent),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        todo!()
    }
}

impl Symbol for PostModifiedMtaEvent {}
impl NonTerminal for PostModifiedMtaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::NonTerminal(NonTerminalType::PostMtaEventExpression),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new PostModifiedMtaEvent from children
        todo!()
    }
}

impl Symbol for PostMtaEventExpression {}
impl NonTerminal for PostMtaEventExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::PostMtaEventModifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostMtaEventModifier),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                ]))
            }
            TerminalType::Eos => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new PostMtaEventExpression from children
        todo!()
    }
}

impl Symbol for ModalityExpression {}
impl NonTerminal for ModalityExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent),
                ]))
            }
            TerminalType::Tense => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityWithAgent),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModalityExpression from children
        todo!()
    }
}

impl Symbol for ModalityWithAgent {}
impl NonTerminal for ModalityWithAgent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality),
                    SymbolType::NonTerminal(NonTerminalType::ModalityAgent),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality),
                    SymbolType::NonTerminal(NonTerminalType::ModalityAgent),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModalityWithAgent from children
        todo!()
    }
}

impl Symbol for ModifiedModality {}
impl NonTerminal for ModifiedModality {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Modality),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ModalityNegationBegin),
                    SymbolType::Terminal(TerminalType::Negation),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedModality),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModifiedModality from children
        todo!()
    }
}

impl Symbol for ModalityAgent {}
impl NonTerminal for ModalityAgent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostArgumentPhraseModifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModalityAgent from children
        todo!()
    }
}

impl Symbol for TenseExpression {}
impl NonTerminal for TenseExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new TenseExpression from children
        todo!()
    }
}

impl Symbol for ModifiedTense {}
impl NonTerminal for ModifiedTense {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Tense),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::TenseNegationBegin),
                    SymbolType::Terminal(TerminalType::Negation),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedTense),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModifiedTense from children
        todo!()
    }
}

impl Symbol for AspectExpression {}
impl NonTerminal for AspectExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new AspectExpression from children
        todo!()
    }
}

impl Symbol for ModifiedAspect {}
impl NonTerminal for ModifiedAspect {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Aspect),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::AspectNegationBegin),
                    SymbolType::Terminal(TerminalType::Negation),
                    SymbolType::NonTerminal(NonTerminalType::ModifiedAspect),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ModifiedAspect from children
        todo!()
    }
}

impl Symbol for MtaEvent {}
impl NonTerminal for MtaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Modality => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::ModalityNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::ModalityExpression),
                    SymbolType::NonTerminal(NonTerminalType::TaEvent),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new MtaEvent from children
        todo!()   
    }
}

impl Symbol for TaEvent {}
impl NonTerminal for TaEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::TenseExpression),
                    SymbolType::NonTerminal(NonTerminalType::AEvent),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new TaEvent from children
        todo!()
    }
}

impl Symbol for AEvent {}
impl NonTerminal for AEvent {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Aspect => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::AspectExpression),
                    SymbolType::NonTerminal(NonTerminalType::Event),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new AEvent from children
        todo!()
    }
}

impl Symbol for Event {}
impl NonTerminal for Event {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhraseOrEpsilon),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new Event from children
        todo!()
    }
}

impl Symbol for PostModifiedBinaryPredicatePhrase {}
impl NonTerminal for PostModifiedBinaryPredicatePhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::PostBinaryPredicatePhraseExpression),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new PostModifiedBinaryPredicatePhrase from children
        todo!()
    }
}

impl Symbol for PostBinaryPredicatePhraseExpression {}
impl NonTerminal for PostBinaryPredicatePhraseExpression {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Argument => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseSeparator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentRetriever => {
                Ok(Rhs::Epsilon)    
            }
            TerminalType::MtaEventToArgumentPhraseConverter => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostBinaryPredicatePhraseModifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostBinaryPredicatePhraseModifier),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                ]))
            }
            TerminalType::PostArgumentPhraseModifier => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Eos => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new PostBinaryPredicatePhraseExpression from children
        todo!()
    }
}

impl Symbol for BinaryPredicatePhrase {}
impl NonTerminal for BinaryPredicatePhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Negation),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryPredicate),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentPhraseTransposer),                    
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryOperator),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        match rhs {
            Rhs::Sequence(ref symbols) => {
                // Check if the sequence matches the expected structure for BinaryPredicatePhrase
                if symbols.len() == 2 && symbols[0] == SymbolType::Terminal(TerminalType::Negation) &&
                   symbols[1] == SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase) {
                    if children.len() == 2 {
                        return Ok(Negation {} BinaryPredicatePhrase {});
                    } else {
                        return Err(IncompleteAst {})
                    }
                    Ok(BinaryPredicatePhrase {});
                } else if sequence.len() == 3 && sequence[0] == SymbolType::Terminal(TerminalType::Negation) &&
                          sequence[1] == SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase) {
                    return Ok(BinaryPredicatePhrase {});
                } else if sequence.len() == 3 && sequence[0] == SymbolType::Terminal(TerminalType::ArgumentPhraseTransposer) &&
                          sequence[1] == SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase) {
                    return Ok(BinaryPredicatePhrase {});
                } else if sequence.len() == 4 && sequence[0] == SymbolType::Terminal(TerminalType::BinaryOperator) &&
                          sequence[1] == SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase) &&
                          sequence[2] == SymbolType::NonTerminal(NonTerminalType::BinaryPredicatePhrase) {
                    return Ok(BinaryPredicatePhrase {});
                }
            }
            _ => {IncompleteAst {}}
        }
        // for child in children {
        //     if let Some(binary_predicate) = child.as_any().downcast_ref::<BinaryPredicatePhrase>() {
        //         return Ok(binary_predicate.clone());
        //     }
        // }
        todo!()
    }
}

impl Symbol for ArgumentPhrase {}
impl NonTerminal for ArgumentPhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Negation),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                ]))
            }
            TerminalType::Argument => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Argument),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::BinaryOperator),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                ]))
            }
            TerminalType::ArgumentRetriever => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentRetriever),
                    SymbolType::NonTerminal(NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::MtaEventToArgumentPhraseConverter => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverter),
                    SymbolType::NonTerminal(NonTerminalType::MtaEvent),
                    SymbolType::Terminal(TerminalType::MtaEventToArgumentPhraseConverterEnd),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ArgumentPhrase from children
        todo!()
    }
}

impl Symbol for ArgumentPhraseOrEpsilon {}
impl NonTerminal for ArgumentPhraseOrEpsilon {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::ArgumentPhraseSeparator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::ArgumentPhraseSeparator),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                ]))
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Eos => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ArgumentPhraseOrEpsilon from children
        todo!()
    }
}

impl Symbol for DeterminedArgumentRetrievalBinaryPredicate {}
impl NonTerminal for DeterminedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::Quantifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::Articlizer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Quantification),
                    SymbolType::NonTerminal(NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new DeterminedArgumentRetrievalBinaryPredicate from children
        todo!()
    }   
}

impl Symbol for ArticlizedArgumentRetrievalBinaryPredicate {}
impl NonTerminal for ArticlizedArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate),
                ]))
            }
            TerminalType::Articlizer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::Article),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalBinaryPredicate),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ArticlizedArgumentRetrievalBinaryPredicate from children
        todo!()
    }
}

impl Symbol for ArgumentRetrievalBinaryPredicate {}
impl NonTerminal for ArgumentRetrievalBinaryPredicate {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase),
                ]))
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase),
                ]))
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase),
                ]))
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::NonTerminal(NonTerminalType::PostModifiedBinaryPredicatePhrase),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentRetrievalWithArgumentPhrase),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ArgumentRetrievalBinaryPredicate from children
        todo!()
    }
}

impl Symbol for ArgumentRetrievalWithArgumentPhrase {}
impl NonTerminal for ArgumentRetrievalWithArgumentPhrase {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Tense => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Aspect => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::TenseNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::AspectNegationBegin => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Argument => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseSeparator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentRetriever => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverter => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::MtaEventToArgumentPhraseConverterEnd => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostMtaEventModifier => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::PostArgumentPhraseModifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::PostArgumentPhraseModifier),
                    SymbolType::NonTerminal(NonTerminalType::ArgumentPhrase),
                ]))
            }
            TerminalType::Eos => {
                Ok(Rhs::Epsilon)
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new ArgumentRetrievalWithArgumentPhrase from children
        todo!()
    }
}

impl Symbol for Quantification {}
impl NonTerminal for Quantification {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Quantifier => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Quantifier),
                ]))
            }
            TerminalType::Articlizer => {
                Ok(Rhs::Epsilon)    
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new Quantification from children
        todo!()
    }
}

impl Symbol for Article {}
impl NonTerminal for Article {
    fn parse_table(current_token: TerminalType) -> Result<Rhs, IncompleteAst> where Self: Sized {
        match current_token {
            TerminalType::Negation => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryPredicate => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::ArgumentPhraseTransposer => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::BinaryOperator => {
                Ok(Rhs::Epsilon)
            }
            TerminalType::Articlizer => {
                Ok(Rhs::Sequence(vec![
                    SymbolType::Terminal(TerminalType::Articlizer),
                ]))
            }
            _ => {
                Err(IncompleteAst {})
            }
        }
    }
    fn try_new(children: &Vec<Box<dyn Symbol>>, rhs: &Rhs) -> Result<Self, IncompleteAst> where Self: Sized {
        // Implement the logic to create a new Article from children
        todo!()
    }
}

impl Symbol for BinaryPredicate {}
impl Terminal for BinaryPredicate {
    fn clone_box(&self) -> Box<dyn Terminal> {
        Box::new(self.clone())
    }
    fn terminal_type(&self) -> TerminalType {
        TerminalType::BinaryPredicate
    }
}

impl Symbol for Argument {}
impl Terminal for Argument {
    fn clone_box(&self) -> Box<dyn Terminal> {
        Box::new(self.clone())
    }
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Argument
    }
}

impl Symbol for ArgumentPhraseSeparator {}
impl Terminal for ArgumentPhraseSeparator {
    fn clone_box(&self) -> Box<dyn Terminal> {
        Box::new(self.clone())
    }
    fn terminal_type(&self) -> TerminalType {
        TerminalType::ArgumentPhraseSeparator
    }
}

impl Symbol for Eos {}
impl Terminal for Eos {
    fn clone_box(&self) -> Box<dyn Terminal> {
        Box::new(self.clone())
    }
    fn terminal_type(&self) -> TerminalType {
        TerminalType::Eos
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
                    "InterrogativeMarker" => TerminalType::InterrogativeMarker,
                    "Modality" => TerminalType::Modality,
                    "Tense" => TerminalType::Tense,
                    "Aspect" => TerminalType::Aspect,
                    "ModalityNegationBegin" => TerminalType::ModalityNegationBegin,
                    "TenseNegationBegin" => TerminalType::TenseNegationBegin,
                    "AspectNegationBegin" => TerminalType::AspectNegationBegin,
                    "Negation" => TerminalType::Negation,
                    "BinaryPredicate" => TerminalType::BinaryPredicate,
                    "Argument" => TerminalType::Argument,
                    "ArgumentPhraseSeparator" => TerminalType::ArgumentPhraseSeparator,
                    "ArgumentPhraseTransposer" => TerminalType::ArgumentPhraseTransposer,
                    "BinaryOperator" => TerminalType::BinaryOperator,
                    "ArgumentRetriever" => TerminalType::ArgumentRetriever,
                    "Quantifier" => TerminalType::Quantifier,
                    "Articlizer" => TerminalType::Articlizer,
                    "MtaEventToArgumentPhraseConverter" => TerminalType::MtaEventToArgumentPhraseConverter,
                    "MtaEventToArgumentPhraseConverterEnd" => TerminalType::MtaEventToArgumentPhraseConverterEnd,
                    "PostMtaEventModifier" => TerminalType::PostMtaEventModifier,
                    "PostBinaryPredicatePhraseModifier" => TerminalType::PostBinaryPredicatePhraseModifier,
                    "PostArgumentPhraseModifier" => TerminalType::PostArgumentPhraseModifier,
                    _ => panic!("Unknown terminal type: {}", t.0),
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
            let tt = TerminalType::Eos;

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
                    _ => panic!("Unknown non-terminal type: {}", nt.0),
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
                                                NonTerminalType::S => "S",
                                                NonTerminalType::DeclarativeSentence => "DeclarativeSentence",
                                                NonTerminalType::InterrogativeSentence => "InterrogativeSentence",
                                                NonTerminalType::PostModifiedMtaEvent => "PostModifiedMtaEvent",
                                                NonTerminalType::PostMtaEventExpression => "PostMtaEventExpression",
                                                NonTerminalType::ModalityExpression => "ModalityExpression",
                                                NonTerminalType::ModalityWithAgent => "ModalityWithAgent",
                                                NonTerminalType::ModifiedModality => "ModifiedModality",
                                                NonTerminalType::ModalityAgent => "ModalityAgent",
                                                NonTerminalType::TenseExpression => "TenseExpression",
                                                NonTerminalType::ModifiedTense => "ModifiedTense",
                                                NonTerminalType::AspectExpression => "AspectExpression",
                                                NonTerminalType::ModifiedAspect => "ModifiedAspect",
                                                NonTerminalType::MtaEvent => "MtaEvent",
                                                NonTerminalType::TaEvent => "TaEvent",
                                                NonTerminalType::AEvent => "AEvent",
                                                NonTerminalType::Event => "Event",
                                                NonTerminalType::PostModifiedBinaryPredicatePhrase => "PostModifiedBinaryPredicatePhrase",
                                                NonTerminalType::PostBinaryPredicatePhraseExpression => "PostBinaryPredicatePhraseExpression",
                                                NonTerminalType::BinaryPredicatePhrase => "BinaryPredicatePhrase",
                                                NonTerminalType::ArgumentPhrase => "ArgumentPhrase",
                                                NonTerminalType::ArgumentPhraseOrEpsilon => "ArgumentPhraseOrEpsilon",
                                                NonTerminalType::DeterminedArgumentRetrievalBinaryPredicate => "DeterminedArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArticlizedArgumentRetrievalBinaryPredicate => "ArticlizedArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArgumentRetrievalBinaryPredicate => "ArgumentRetrievalBinaryPredicate",
                                                NonTerminalType::ArgumentRetrievalWithArgumentPhrase => "ArgumentRetrievalWithArgumentPhrase",
                                                NonTerminalType::Quantification => "Quantification",
                                                NonTerminalType::Article => "Article"
                                            };
                                            assert_eq!(auto_nt.0, manual_nt_variant_name, "Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_nt, manual_nt);
                                        }
                                        (grammar::Symbol::Terminal(auto_t), SymbolType::Terminal(manual_t)) => {
                                            let manual_nt_variant_name = match manual_t {
                                                TerminalType::InterrogativeMarker => "InterrogativeMarker",
                                                TerminalType::Modality => "Modality",
                                                TerminalType::Tense => "Tense",
                                                TerminalType::Aspect => "Aspect",
                                                TerminalType::ModalityNegationBegin => "ModalityNegationBegin",
                                                TerminalType::TenseNegationBegin => "TenseNegationBegin",
                                                TerminalType::AspectNegationBegin => "AspectNegationBegin",
                                                TerminalType::Negation => "Negation",
                                                TerminalType::BinaryPredicate => "BinaryPredicate",
                                                TerminalType::Argument => "Argument",
                                                TerminalType::ArgumentPhraseSeparator => "ArgumentPhraseSeparator",
                                                TerminalType::ArgumentPhraseTransposer => "ArgumentPhraseTransposer",
                                                TerminalType::BinaryOperator => "BinaryOperator",
                                                TerminalType::ArgumentRetriever => "ArgumentRetriever",
                                                TerminalType::Quantifier => "Quantifier",
                                                TerminalType::Articlizer => "Articlizer",
                                                TerminalType::MtaEventToArgumentPhraseConverter => "MtaEventToArgumentPhraseConverter",
                                                TerminalType::MtaEventToArgumentPhraseConverterEnd => "MtaEventToArgumentPhraseConverterEnd",
                                                TerminalType::PostMtaEventModifier => "PostMtaEventModifier",
                                                TerminalType::PostBinaryPredicatePhraseModifier => "PostBinaryPredicatePhraseModifier",
                                                TerminalType::PostArgumentPhraseModifier => "PostArgumentPhraseModifier",
                                                _ => panic!("Unknown terminal type: maybe Eos"),
                                            };
                                            assert_eq!(auto_t.0, manual_nt_variant_name, "Parse table mismatch for terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_t, manual_t);
                                        }
                                        _ => {
                                            panic!("Unexpected symbol type in parse table for non-terminal: {:?}, terminal: {:?}", nt, lt);
                                        }
                                    }
                                }
                            }
                            _ => {
                                panic!("Parse table mismatch for non-terminal: {:?}, terminal: {:?}. Auto: {:?}, Manual: {:?}", nt, lt, auto_result, manual_result);
                            }
                        }
                    }
                    (None, Err(_)) => {}
                    (Some(_auto_result), Err(_)) => {
                        panic!("Auto parse result found but manual parse failed for non-terminal: {:?}, terminal: {:?}", nt, lt);
                    }
                    (None, Ok(_manual_result)) => {
                        panic!("Manual parse result found but auto parse result is None for non-terminal: {:?}, terminal: {:?}", nt, lt);
                    }
                }
    }

    #[test]
    fn test_parse() {
        let tokens: Vec<Box<dyn Terminal>> = vec![
            Box::new(BinaryPredicate {}),
            Box::new(Argument {}),
            Box::new(ArgumentPhraseSeparator {}),
            Box::new(Argument {}),
            Box::new(Eos {}),
        ];
        println!("Testing parse with tokens: {:?}", parse(tokens));
    }
}
