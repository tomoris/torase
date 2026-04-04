use crate::parser::*;
use crate::tokenizer::Range;

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

impl LexicalType for crate::tokenizer::ArgumentPhraseTransposer {
    /// Argument transposition: swaps the two GQ argument slots.
    /// re  ↦  λP Q₁ Q₂. P(Q₂)(Q₁)  :  pred2 → pred2
    fn lexical_type(&self) -> Type {
        Type::arrow(Type::pred2(), Type::pred2())
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
// Range helpers via a separate trait to avoid naming conflicts with
// argument_type_checker.rs which defines private `get_range()` methods
// on the same types.
//
// Note: Modality / Tense / Aspect / ModalityNegation / TenseNegation /
//       AspectNegation tokens do NOT expose get_range(), so MTA wrappers
//       (MtaEvent, TaEvent, AEvent) delegate to the inner Event range.
// ---------------------------------------------------------------------------

trait SemRange {
    fn sem_range(&self) -> Range;
}

impl SemRange for BinaryPredicatePhrase {
    fn sem_range(&self) -> Range {
        match self {
            BinaryPredicatePhrase::One(p) => p.get_range(),
            BinaryPredicatePhrase::Two(neg, phrase) => {
                Range::new(neg.get_range().start(), phrase.sem_range().end())
            }
            BinaryPredicatePhrase::Three(trans, phrase) => {
                Range::new(trans.get_range().start(), phrase.sem_range().end())
            }
            BinaryPredicatePhrase::Four(op, _, p2) => {
                Range::new(op.get_range().start(), p2.sem_range().end())
            }
        }
    }
}

impl SemRange for PostModifiedBinaryPredicatePhrase {
    fn sem_range(&self) -> Range {
        match self {
            PostModifiedBinaryPredicatePhrase::One(phrase, post) => {
                let start = phrase.sem_range().start();
                let end = match post {
                    PostBinaryPredicatePhraseExpression::One(_, p2) => p2.sem_range().end(),
                    PostBinaryPredicatePhraseExpression::Epsilon => phrase.sem_range().end(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl SemRange for ArgumentPhrase {
    fn sem_range(&self) -> Range {
        match self {
            ArgumentPhrase::One(arg) => arg.get_range(),
            ArgumentPhrase::Two(neg, inner) => {
                Range::new(neg.get_range().start(), inner.sem_range().end())
            }
            ArgumentPhrase::Three(op, _, a2) => {
                Range::new(op.get_range().start(), a2.sem_range().end())
            }
            ArgumentPhrase::Four(quant) => quant.sem_range(),
            ArgumentPhrase::Five(conv, _, end_marker) => {
                Range::new(conv.get_range().start(), end_marker.get_range().end())
            }
        }
    }
}

impl SemRange for QuantifiedArgument {
    fn sem_range(&self) -> Range {
        match self {
            QuantifiedArgument::One(qm, det) => {
                let end = det.sem_range().end();
                let start = match qm {
                    QuantifierModifier::One(q) => q.get_range().start(),
                    QuantifierModifier::Two(ne) => match ne {
                        NumericalExpression::One(n) => n.get_range().start(),
                    },
                    QuantifierModifier::Epsilon => det.sem_range().start(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl SemRange for DeterminedArgument {
    fn sem_range(&self) -> Range {
        match self {
            DeterminedArgument::One(dm, ret) => {
                let end = ret.sem_range().end();
                let start = match dm {
                    DeterminerModifier::One(d) => d.get_range().start(),
                    DeterminerModifier::Epsilon => ret.sem_range().start(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl SemRange for RetrievedArgument {
    fn sem_range(&self) -> Range {
        match self {
            RetrievedArgument::One(ret, pred) => {
                Range::new(ret.get_range().start(), pred.sem_range().end())
            }
        }
    }
}

impl SemRange for ArgumentRetrievalBinaryPredicate {
    fn sem_range(&self) -> Range {
        match self {
            ArgumentRetrievalBinaryPredicate::One(pred, with_arg) => {
                let start = pred.sem_range().start();
                let end = match with_arg {
                    ArgumentRetrievalWithArgumentPhrase::One(_, arg) => arg.sem_range().end(),
                    ArgumentRetrievalWithArgumentPhrase::Epsilon => pred.sem_range().end(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl SemRange for Event {
    fn sem_range(&self) -> Range {
        match self {
            Event::One(pred, arg1, arg2) => {
                let start = pred.sem_range().start();
                let end = match arg2 {
                    ArgumentPhraseOrEpsilon::One(_, a2) => a2.sem_range().end(),
                    ArgumentPhraseOrEpsilon::Epsilon => arg1.sem_range().end(),
                };
                Range::new(start, end)
            }
        }
    }
}

// Modality / Tense / Aspect tokens lack get_range(), so MTA wrappers
// delegate to the inner Event.
impl SemRange for AEvent {
    fn sem_range(&self) -> Range {
        match self {
            AEvent::One(_, event) => event.sem_range(),
        }
    }
}

impl SemRange for TaEvent {
    fn sem_range(&self) -> Range {
        match self {
            TaEvent::One(_, a_event) => a_event.sem_range(),
        }
    }
}

impl SemRange for MtaEvent {
    fn sem_range(&self) -> Range {
        match self {
            MtaEvent::One(_, ta_event) => ta_event.sem_range(),
        }
    }
}

impl SemRange for PostModifiedMtaEvent {
    fn sem_range(&self) -> Range {
        match self {
            PostModifiedMtaEvent::One(mta, post_expr) => {
                let start = mta.sem_range().start();
                let end = match post_expr {
                    PostMtaEventExpression::One(_, mta2) => mta2.sem_range().end(),
                    PostMtaEventExpression::Epsilon => mta.sem_range().end(),
                };
                Range::new(start, end)
            }
        }
    }
}

impl SemRange for DeclarativeSentence {
    fn sem_range(&self) -> Range {
        match self {
            DeclarativeSentence::One(pm) => pm.sem_range(),
        }
    }
}

impl SemRange for InterrogativeSentence {
    fn sem_range(&self) -> Range {
        match self {
            // InterrogativeMarker does not expose get_range(); use the event range
            InterrogativeSentence::One(_, pm) => pm.sem_range(),
        }
    }
}

impl SemRange for S {
    fn sem_range(&self) -> Range {
        match self {
            S::One(d) => d.sem_range(),
            S::Two(i) => i.sem_range(),
        }
    }
}

// ---------------------------------------------------------------------------
// Type inference
// ---------------------------------------------------------------------------

pub fn type_check(ast: Ast) -> Result<(), TypeMismatch> {
    let s = ast.get_s();
    let ty = s.infer_type()?;
    if ty != Type::T {
        return Err(TypeMismatch::new(s.sem_range(), ty, Type::T));
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
            Event::One(pred_phrase, arg1, arg2_or_eps) => {
                // pred_phrase : e → e → t
                let pred_type = pred_phrase.infer_type()?;

                // Apply predicate to first argument: (e → e → t)(e) = e → t
                let arg1_type = arg1.infer_type()?;
                let after_arg1 = apply(pred_type, arg1_type, arg1.sem_range())?;

                match arg2_or_eps {
                    ArgumentPhraseOrEpsilon::One(_sep, arg2) => {
                        // Apply (e → t)(e) = t
                        let arg2_type = arg2.infer_type()?;
                        apply(after_arg1, arg2_type, arg2.sem_range())
                    }
                    ArgumentPhraseOrEpsilon::Epsilon => {
                        // Existential closure: ∃Q. after_arg1(Q) : t
                        // after_arg1 must be (e → t) → t → t  (i.e. GQ → t)
                        let gq_to_t = Type::arrow(Type::gq(), Type::T);
                        if after_arg1 != gq_to_t {
                            return Err(TypeMismatch::new(arg1.sem_range(), after_arg1, gq_to_t));
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
                        return Err(TypeMismatch::new(phrase2.sem_range(), t2, t));
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
    /// - transposition re P     : same type as P  (argument order swapped semantically)
    /// - coordination P₁ ∧/∨ P₂ : both must have matching types
    fn infer_type(&self) -> TCResult {
        match self {
            // Lexical binary predicate: type comes from the token's lexical entry
            BinaryPredicatePhrase::One(pred) => Ok(pred.lexical_type()),

            // Negation is type-preserving: ¬P has the same type as P
            BinaryPredicatePhrase::Two(_neg, phrase) => phrase.infer_type(),

            // Argument transposition: re : pred2 → pred2
            BinaryPredicatePhrase::Three(trans, phrase) => {
                let trans_type = trans.lexical_type();
                let phrase_type = phrase.infer_type()?;
                apply(trans_type, phrase_type, trans.get_range())
            }

            // Boolean coordination: both operands must have the same type
            BinaryPredicatePhrase::Four(_op, p1, p2) => {
                let t1 = p1.infer_type()?;
                let t2 = p2.infer_type()?;
                if t1 != t2 {
                    return Err(TypeMismatch::new(p2.sem_range(), t2, t1));
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
                    return Err(TypeMismatch::new(a2.sem_range(), t2, t1));
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
    /// `ro P` = ιQ. P(Q)  where P : GQ → t  →  GQ = (e → t) → t
    fn infer_type(&self) -> TCResult {
        match self {
            RetrievedArgument::One(retriever, pred) => {
                // ro : (GQ → t) → GQ
                let ro_type = retriever.lexical_type();
                let pred_type = pred.infer_type()?;
                apply(ro_type, pred_type, retriever.get_range())
            }
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    /// ArgumentRetrievalBinaryPredicate : GQ → t  (= ((e → t) → t) → t)
    ///
    /// Given a binary predicate P : GQ → GQ → t and an optional extra argument Q : GQ,
    /// this term represents  λR. P(R)(Q)  or  λR. ∃Q. P(R)(Q),  both of type GQ → t.
    /// The iota operator in `RetrievedArgument` then picks the GQ.
    fn infer_type(&self) -> TCResult {
        match self {
            ArgumentRetrievalBinaryPredicate::One(pred_phrase, with_arg) => {
                // P : GQ → GQ → t
                let pred_type = pred_phrase.infer_type()?;

                match with_arg {
                    ArgumentRetrievalWithArgumentPhrase::One(_modifier, extra_arg) => {
                        // Semantic structure: λR. P(R)(extra_arg)
                        // Apply P to the retrieved variable slot (GQ):  P(R) : GQ → t
                        let after_r = apply(pred_type, Type::gq(), pred_phrase.sem_range())?;
                        // Apply P(R) to extra_arg: P(R)(extra_arg) : t
                        let extra_arg_type = extra_arg.infer_type()?;
                        apply(after_r, extra_arg_type, extra_arg.sem_range())?;
                        // λR. P(R)(extra_arg) : GQ → t
                        Ok(Type::arrow(Type::gq(), Type::T))
                    }
                    ArgumentRetrievalWithArgumentPhrase::Epsilon => {
                        // Semantic structure: λR. ∃Q. P(R)(Q)
                        // Verify the first argument slot accepts GQ
                        apply(pred_type, Type::gq(), pred_phrase.sem_range())?;
                        // λR. ∃Q. P(R)(Q) : GQ → t
                        Ok(Type::arrow(Type::gq(), Type::T))
                    }
                }
            }
        }
    }
}
