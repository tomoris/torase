use crate::parser::*;
use crate::tokenizer::{ArgumentRetrieverKind, Range};

// ---------------------------------------------------------------------------
// Semantic types (Montague-style)
// ---------------------------------------------------------------------------

/// Formal semantic type.
///
/// - `E`          : entity domain (individuals)
/// - `T`          : truth-value domain (propositions)
/// - `Arrow(a, b)`: functional type  a → b
///
/// Common derived types:
/// - generalized quantifier   : `(e → t) → t`          (NP / argument type)
/// - binary predicate         : `((e → t) → t) → (((e → t) → t) → t`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    E,
    T,
    Arrow(Box<Type>, Box<Type>),
}

impl Type {
    pub fn arrow(a: Type, b: Type) -> Type {
        Type::Arrow(Box::new(a), Box::new(b))
    }

    /// `e → t`
    pub fn e_to_t() -> Type {
        Type::arrow(Type::E, Type::T)
    }

    /// `(e → t) → t`  — generalized quantifier (NP / argument type)
    pub fn gq() -> Type {
        Type::arrow(Type::e_to_t(), Type::T)
    }

    /// `((e → t) → t) → (((e → t) → t) → t)`  — binary predicate over generalized quantifiers
    pub fn pred2() -> Type {
        Type::arrow(Type::gq(), Type::arrow(Type::gq(), Type::T))
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::E => write!(f, "e"),
            Type::T => write!(f, "t"),
            Type::Arrow(a, b) => match a.as_ref() {
                Type::Arrow(_, _) => write!(f, "({a}) -> {b}"),
                _ => write!(f, "{a} -> {b}"),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

pub struct TypeMismatch {
    pub range: Range,
    pub actual: Type,
    pub expected: Type,
    pub message: String,
}

impl TypeMismatch {
    fn new(range: Range, actual: Type, expected: Type) -> Self {
        let message = format!("expected type `{expected}`, got `{actual}`");
        TypeMismatch {
            range,
            actual,
            expected,
            message,
        }
    }
}

type TCResult = Result<Type, TypeMismatch>;

// ---------------------------------------------------------------------------
// Lexical types — terminal tokens carry their own semantic type
// ---------------------------------------------------------------------------

/// A terminal token that has a fixed semantic type in the formal semantics.
trait LexicalType {
    fn lexical_type(&self) -> Type;
}

impl LexicalType for crate::tokenizer::Argument {
    /// Individual argument (pronoun / name) is lifted to a generalized quantifier.
    /// e.g.  mi  ↦  λP. P(mi)  :  (e → t) → t
    fn lexical_type(&self) -> Type {
        Type::gq()
    }
}

impl LexicalType for crate::tokenizer::BinaryPredicate {
    /// A binary predicate takes two generalized-quantifier arguments.
    /// e.g.  pahete  ↦  λQ₁ Q₂. pahete(Q₁)(Q₂)  :  ((e → t) → t) → (((e → t) → t) → t)
    fn lexical_type(&self) -> Type {
        Type::pred2()
    }
}

impl LexicalType for crate::tokenizer::ArgumentRetriever {
    /// Iota / choice operator: picks a GQ satisfying a property.
    /// ro  ↦  λP. ιQ. P(Q)  :  (GQ → t) → GQ
    ///     where GQ = (e → t) → t
    fn lexical_type(&self) -> Type {
        let gq_to_t = Type::arrow(Type::gq(), Type::T);
        Type::arrow(gq_to_t, Type::gq())
    }
}

impl LexicalType for crate::tokenizer::MtaEventToArgumentPhraseConverter {
    /// Event nominalization opener: converts a proposition into a GQ.
    /// se…so  ↦  λt. ιQ. Q = ↑t  :  t → GQ
    fn lexical_type(&self) -> Type {
        Type::arrow(Type::T, Type::gq())
    }
}

// ---------------------------------------------------------------------------
// Helper: function application type check
// ---------------------------------------------------------------------------

/// Check that `func_type` is `a → b` and `arg_type` is `a`; return `b`.
fn apply(func_type: Type, arg_type: Type, error_range: Range) -> TCResult {
    match func_type {
        Type::Arrow(param, ret) if *param == arg_type => Ok(*ret),
        Type::Arrow(param, _) => Err(TypeMismatch::new(error_range, arg_type, *param)),
        other => Err(TypeMismatch::new(error_range, other, Type::pred2())),
    }
}

// ---------------------------------------------------------------------------
// Type inference
// ---------------------------------------------------------------------------

pub fn type_check(ast: Ast) -> Result<(), TypeMismatch> {
    let s = ast.get_s();
    let ty = s.infer_type()?;
    if ty != Type::T {
        return Err(TypeMismatch::new(s.get_range(), ty, Type::T));
    }
    Ok(())
}

impl S {
    /// S : t
    fn infer_type(&self) -> TCResult {
        match self {
            S::One(d) => d.infer_type(),
            S::Two(i) => i.infer_type(),
        }
    }
}

impl DeclarativeSentence {
    /// DeclarativeSentence : t
    fn infer_type(&self) -> TCResult {
        match self {
            DeclarativeSentence::One(pm) => pm.infer_type(),
        }
    }
}

impl InterrogativeSentence {
    /// InterrogativeSentence : t  (propositional content of the question)
    fn infer_type(&self) -> TCResult {
        match self {
            InterrogativeSentence::One(_marker, pm) => pm.infer_type(),
        }
    }
}

impl PostModifiedMtaEvent {
    /// PostModifiedMtaEvent : t
    fn infer_type(&self) -> TCResult {
        match self {
            PostModifiedMtaEvent::One(mta, post_expr) => {
                let t = mta.infer_type()?;
                if let PostMtaEventExpression::One(_modifier, mta2) = post_expr {
                    mta2.infer_type()?;
                }
                Ok(t)
            }
        }
    }
}

impl MtaEvent {
    /// MtaEvent : t  (modality operators are type-preserving)
    fn infer_type(&self) -> TCResult {
        match self {
            MtaEvent::One(_modality, ta_event) => ta_event.infer_type(),
        }
    }
}

impl TaEvent {
    /// TaEvent : t  (tense operators are type-preserving)
    fn infer_type(&self) -> TCResult {
        match self {
            TaEvent::One(_tense, a_event) => a_event.infer_type(),
        }
    }
}

impl AEvent {
    /// AEvent : t  (aspect operators are type-preserving)
    fn infer_type(&self) -> TCResult {
        match self {
            AEvent::One(_aspect, event) => event.infer_type(),
        }
    }
}

impl Event {
    /// Event : t
    ///
    /// Typing rule:
    ///   pred : e → e → t,  arg1 : e,  arg2 : e
    ///   ──────────────────────────────────────
    ///   pred(arg1)(arg2) : t
    ///
    /// If arg2 is omitted, existential closure applies:  ∃x. pred(arg1)(x) : t
    fn infer_type(&self) -> TCResult {
        match self {
            Event::One(pred_phrase, event_args) => {
                let pred_type = pred_phrase.infer_type()?;
                match event_args {
                    EventArguments::NominativeFirst(_nom, arg1, opt_acc, _other) => {
                        let arg1_type = arg1.infer_type()?;
                        let after_arg1 = apply(pred_type, arg1_type, arg1.get_range())?;
                        match opt_acc {
                            OptionalAccusativeArg::One(_, arg2) => {
                                let arg2_type = arg2.infer_type()?;
                                apply(after_arg1, arg2_type, arg2.get_range())
                            }
                            OptionalAccusativeArg::Epsilon => {
                                let gq_to_t = Type::arrow(Type::gq(), Type::T);
                                if after_arg1 != gq_to_t {
                                    return Err(TypeMismatch::new(
                                        arg1.get_range(),
                                        after_arg1,
                                        gq_to_t,
                                    ));
                                }
                                Ok(Type::T)
                            }
                        }
                    }
                    EventArguments::AccusativeFirst(_acc, arg2, opt_nom, _other) => {
                        // pred : GQ → GQ → t; arg2 fills the second slot
                        let inner_type = match pred_type.clone() {
                            Type::Arrow(_, b) => *b,
                            _ => {
                                return Err(TypeMismatch::new(
                                    arg2.get_range(),
                                    pred_type,
                                    Type::pred2(),
                                ));
                            }
                        };
                        let arg2_type = arg2.infer_type()?;
                        apply(inner_type, arg2_type, arg2.get_range())?;
                        match opt_nom {
                            OptionalNominativeArg::One(_, arg1) => {
                                let arg1_type = arg1.infer_type()?;
                                apply(pred_type, arg1_type, arg1.get_range())?;
                            }
                            OptionalNominativeArg::Epsilon => {}
                        }
                        Ok(Type::T)
                    }
                    EventArguments::OtherCaseFirst(_prep, _label, _arg, _rest) => Ok(Type::T),
                    EventArguments::Epsilon => {
                        let expected = Type::pred2();
                        if pred_type != expected {
                            return Err(TypeMismatch::new(
                                pred_phrase.get_range(),
                                pred_type,
                                expected,
                            ));
                        }
                        Ok(Type::T)
                    }
                }
            }
        }
    }
}

impl PostModifiedBinaryPredicatePhrase {
    /// PostModifiedBinaryPredicatePhrase : ((e → t) → t) → (((e → t) → t) → t)
    fn infer_type(&self) -> TCResult {
        match self {
            PostModifiedBinaryPredicatePhrase::One(phrase, post_expr) => {
                let t = phrase.infer_type()?;
                if let PostBinaryPredicatePhraseExpression::One(_modifier, phrase2) = post_expr {
                    let t2 = phrase2.infer_type()?;
                    if t != t2 {
                        return Err(TypeMismatch::new(phrase2.get_range(), t2, t));
                    }
                }
                Ok(t)
            }
        }
    }
}

impl BinaryPredicatePhrase {
    /// BinaryPredicatePhrase : ((e → t) → t) → (((e → t) → t) → t)
    ///
    /// All constructors preserve the binary predicate type:
    /// - negation   ¬P          : same type as P
    /// - coordination P₁ ∧/∨ P₂ : both must have matching types
    fn infer_type(&self) -> TCResult {
        match self {
            BinaryPredicatePhrase::One(pred) => Ok(pred.lexical_type()),
            BinaryPredicatePhrase::Two(_neg, phrase) => phrase.infer_type(),
            BinaryPredicatePhrase::Three(_op, p1, p2) => {
                let t1 = p1.infer_type()?;
                let t2 = p2.infer_type()?;
                if t1 != t2 {
                    return Err(TypeMismatch::new(p2.get_range(), t2, t1));
                }
                Ok(t1)
            }
        }
    }
}

impl ArgumentPhrase {
    /// ArgumentPhrase : (e → t) → t
    ///
    /// All argument phrases are generalized quantifiers (NP-type):
    /// - bare argument             : lifts individual to GQ      (e → t) → t
    /// - negated argument   ¬arg   : complement GQ               (e → t) → t
    /// - coordinated args          : both must be (e → t) → t        (e → t) → t
    /// - quantified argument       : quantifier gives GQ         (e → t) → t
    /// - event nominalization se…so: t →(nominalize)→            (e → t) → t
    fn infer_type(&self) -> TCResult {
        match self {
            // Bare argument: type comes from the token's lexical entry
            ArgumentPhrase::One(arg) => Ok(arg.lexical_type()),

            // Negated argument: complement, type is preserved
            ArgumentPhrase::Two(_neg, inner) => inner.infer_type(),

            // Boolean coordination of argument phrases: both must have the same type
            ArgumentPhrase::Three(_op, a1, a2) => {
                let t1 = a1.infer_type()?;
                let t2 = a2.infer_type()?;
                if t1 != t2 {
                    return Err(TypeMismatch::new(a2.get_range(), t2, t1));
                }
                Ok(t1)
            }

            // Quantified argument: the iota / choice operator gives an entity
            ArgumentPhrase::Four(quant_arg) => quant_arg.infer_type(),

            // Event nominalization se…so:  se : t → GQ
            ArgumentPhrase::Five(conv, mta_event, _end) => {
                let conv_type = conv.lexical_type();
                let event_type = mta_event.infer_type()?;
                apply(conv_type, event_type, conv.get_range())
            }
        }
    }
}

impl QuantifiedArgument {
    /// QuantifiedArgument : (e → t) → t
    /// The quantifier + determiner + retrieval structure denotes a generalized quantifier
    /// (via the iota / choice operator applied to the restriction predicate).
    fn infer_type(&self) -> TCResult {
        match self {
            QuantifiedArgument::One(_quant_mod, det_arg) => det_arg.infer_type(),
        }
    }
}

impl DeterminedArgument {
    /// DeterminedArgument : (e → t) → t
    fn infer_type(&self) -> TCResult {
        match self {
            DeterminedArgument::One(_det_mod, ret_arg) => ret_arg.infer_type(),
        }
    }
}

impl RetrievedArgument {
    /// RetrievedArgument : (e → t) → t
    ///
    /// `ra/ri P` = ιQ. P(Q)  where P : GQ → t  →  GQ = (e → t) → t
    fn infer_type(&self) -> TCResult {
        match self {
            RetrievedArgument::One(retriever, pred) => {
                let retriever_type = retriever.lexical_type();
                let pred_type = pred.infer_type(retriever.get_kind())?;
                apply(retriever_type, pred_type, retriever.get_range())
            }
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    /// ArgumentRetrievalBinaryPredicate : GQ → t  (= ((e → t) → t) → t)
    ///
    /// ra (nominative retriever): λR. P(R)(Q)  — R is applied first (nominative slot)
    /// ri (accusative retriever): λR. P(Q)(R)  — R is applied second (accusative slot)
    fn infer_type(&self, kind: &ArgumentRetrieverKind) -> TCResult {
        match self {
            ArgumentRetrievalBinaryPredicate::One(pred_phrase, with_arg) => {
                let pred_type = pred_phrase.infer_type()?;

                match with_arg {
                    ArgumentRetrievalWithArgumentPhrase::One(_modifier, extra_arg) => {
                        let extra_arg_type = extra_arg.infer_type()?;
                        match kind {
                            ArgumentRetrieverKind::Nominative => {
                                // λR. P(R)(extra_arg): apply R first, then extra_arg
                                let after_r =
                                    apply(pred_type, Type::gq(), pred_phrase.get_range())?;
                                apply(after_r, extra_arg_type, extra_arg.get_range())?;
                            }
                            ArgumentRetrieverKind::Accusative => {
                                // λR. P(extra_arg)(R): apply extra_arg first, then R
                                let after_extra =
                                    apply(pred_type, extra_arg_type, extra_arg.get_range())?;
                                apply(after_extra, Type::gq(), pred_phrase.get_range())?;
                            }
                        }
                        Ok(Type::arrow(Type::gq(), Type::T))
                    }
                    ArgumentRetrievalWithArgumentPhrase::Epsilon => {
                        apply(pred_type, Type::gq(), pred_phrase.get_range())?;
                        Ok(Type::arrow(Type::gq(), Type::T))
                    }
                }
            }
        }
    }
}
