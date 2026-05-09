use crate::parser::{
    AEvent, ArgumentPhrase, ArgumentRetrievalBinaryPredicate, ArgumentRetrievalWithArgumentPhrase,
    Ast, BinaryPredicatePhrase, DeclarativeSentence, DeterminedArgument, DeterminerModifier, Event,
    EventArguments, InterrogativeSentence, MtaEvent, OptionalAccusativeArg, OptionalNominativeArg,
    PostBinaryPredicatePhraseExpression, PostModifiedBinaryPredicatePhrase, PostModifiedMtaEvent,
    QuantifiedArgument, QuantifierModifier, RetrievedArgument, S, TaEvent,
};
use crate::tokenizer::ArgumentRetrieverKind;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A first-order term: either a bound variable or an individual constant.
#[derive(Debug, Clone)]
pub enum Term {
    Var(String),
    Const(String),
}

/// A first-order formula extended with generalized quantifiers for `te`/`ti`/`number`.
#[derive(Debug, Clone)]
pub enum Formula {
    /// P(t₁, …, tₙ)
    Atom(String, Vec<Term>),
    /// ¬φ
    Not(Box<Formula>),
    /// φ ∧ ψ
    And(Box<Formula>, Box<Formula>),
    /// φ ∨ ψ
    Or(Box<Formula>, Box<Formula>),
    /// φ ⇒ ψ
    Implies(Box<Formula>, Box<Formula>),
    /// ∀x. φ
    Forall(String, Box<Formula>),
    /// ∃x. φ
    Exists(String, Box<Formula>),
    /// Generalized quantifier (te 多くの / ti 半分の / number 数量)
    GenQ(GenQKind, String, Box<Formula>),
}

/// Kinds of generalized quantifiers that cannot be expressed with ∀/∃.
#[derive(Debug, Clone)]
pub enum GenQKind {
    Many,  // te  多くの
    Half,  // ti  半分の
    Count, // number  数量
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(v) | Term::Const(v) => write!(f, "{v}"),
        }
    }
}

impl std::fmt::Display for GenQKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenQKind::Many => write!(f, "∃_many"),
            GenQKind::Half => write!(f, "∃_half"),
            GenQKind::Count => write!(f, "∃_#"),
        }
    }
}

impl std::fmt::Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Atom(p, args) => {
                let s = args
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                if s.is_empty() {
                    write!(f, "{p}")
                } else {
                    write!(f, "{p}({s})")
                }
            }
            Formula::Not(phi) => write!(f, "¬{phi}"),
            Formula::And(l, r) => write!(f, "({l} ∧ {r})"),
            Formula::Or(l, r) => write!(f, "({l} ∨ {r})"),
            Formula::Implies(l, r) => write!(f, "({l} ⇒ {r})"),
            Formula::Forall(x, phi) => write!(f, "∀{x}.{phi}"),
            Formula::Exists(x, phi) => write!(f, "∃{x}.{phi}"),
            Formula::GenQ(k, x, phi) => write!(f, "{k} {x}.{phi}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Variable generator
// ---------------------------------------------------------------------------

struct VarGen(usize);

impl VarGen {
    fn new() -> Self {
        VarGen(0)
    }

    fn next(&mut self) -> String {
        let n = self.0;
        self.0 += 1;
        match n {
            0 => "x".to_string(),
            1 => "y".to_string(),
            2 => "z".to_string(),
            _ => format!("x{}", n - 2),
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate context
// ---------------------------------------------------------------------------

/// Internal representation of a predicate phrase.
#[derive(Debug, Clone)]
enum PredCtx {
    /// Lexical predicate name.
    Simple(String),
    /// ¬P — negate the whole proposition produced by P.
    Neg(Box<PredCtx>),
    /// P₁ di/binary_operator P₂ — both predicates apply to the same args.
    Coord {
        and_op: bool,
        left: Box<PredCtx>,
        right: Box<PredCtx>,
    },
}

fn extract_pred(phrase: &PostModifiedBinaryPredicatePhrase) -> PredCtx {
    match phrase {
        PostModifiedBinaryPredicatePhrase::One(bp, post) => {
            let base = extract_bp(bp);
            match post {
                PostBinaryPredicatePhraseExpression::One(_, secondary) => PredCtx::Coord {
                    and_op: true,
                    left: Box::new(base),
                    right: Box::new(extract_bp(secondary)),
                },
                PostBinaryPredicatePhraseExpression::Epsilon => base,
            }
        }
    }
}

fn extract_bp(phrase: &BinaryPredicatePhrase) -> PredCtx {
    match phrase {
        BinaryPredicatePhrase::One(pred) => PredCtx::Simple(pred.get_string()),
        BinaryPredicatePhrase::Two(_neg, inner) => PredCtx::Neg(Box::new(extract_bp(inner))),
        BinaryPredicatePhrase::Three(_op, p1, p2) => PredCtx::Coord {
            and_op: true,
            left: Box::new(extract_bp(p1)),
            right: Box::new(extract_bp(p2)),
        },
    }
}

// ---------------------------------------------------------------------------
// Argument context
// ---------------------------------------------------------------------------

/// Internal translation result for an argument phrase.
/// Carries both the term and how to scope a body formula.
#[derive(Debug, Clone)]
enum ArgCtx {
    /// Quantified NP: bind `var` with `kind` and `restriction`.
    Quant {
        var: String,
        kind: QuantKind,
        restriction: Formula,
    },
    /// Individual constant (pronoun).
    Const(String),
    /// Fresh existential variable with no restriction (from epsilon arg2).
    /// `scope()` wraps with ∃var.body.
    ExistOnly(String),
    /// Negated argument phrase.
    Neg(Box<ArgCtx>),
    /// Coordinated argument phrases (distributed in `build_event_inner`).
    Coord {
        and_op: bool,
        left: Box<ArgCtx>,
        right: Box<ArgCtx>,
    },
    /// Event nominalization (se … so): ∃var.(inner ∧ body).
    Nominal { var: String, inner: Box<Formula> },
}

#[derive(Debug, Clone, PartialEq)]
enum QuantKind {
    Universal,   // ta  全ての
    Existential, // epsilon / to  少しの・存在する
    Many,        // te  多くの
    Half,        // ti  半分の
    Count,       // number
}

impl ArgCtx {
    /// The term this context contributes to the atomic formula.
    fn term(&self) -> Term {
        match self {
            ArgCtx::Quant { var, .. } | ArgCtx::ExistOnly(var) => Term::Var(var.clone()),
            ArgCtx::Const(s) => Term::Const(s.clone()),
            ArgCtx::Neg(inner) => inner.term(),
            ArgCtx::Coord { left, .. } => left.term(), // distribution is handled externally
            ArgCtx::Nominal { var, .. } => Term::Var(var.clone()),
        }
    }

    /// Apply quantifier scope to `body`.
    fn scope(self, body: Formula) -> Formula {
        match self {
            ArgCtx::Quant {
                var,
                kind,
                restriction,
            } => match kind {
                QuantKind::Universal => Formula::Forall(
                    var,
                    Box::new(Formula::Implies(Box::new(restriction), Box::new(body))),
                ),
                QuantKind::Existential => Formula::Exists(
                    var,
                    Box::new(Formula::And(Box::new(restriction), Box::new(body))),
                ),
                QuantKind::Many => Formula::GenQ(
                    GenQKind::Many,
                    var,
                    Box::new(Formula::And(Box::new(restriction), Box::new(body))),
                ),
                QuantKind::Half => Formula::GenQ(
                    GenQKind::Half,
                    var,
                    Box::new(Formula::And(Box::new(restriction), Box::new(body))),
                ),
                QuantKind::Count => Formula::GenQ(
                    GenQKind::Count,
                    var,
                    Box::new(Formula::And(Box::new(restriction), Box::new(body))),
                ),
            },
            ArgCtx::Const(_) => body,
            ArgCtx::ExistOnly(var) => Formula::Exists(var, Box::new(body)),
            ArgCtx::Neg(inner) => {
                // Negate the restriction of the inner quantifier.
                let ctx = negate_restriction(*inner);
                ctx.scope(body)
            }
            ArgCtx::Coord {
                and_op: _,
                left,
                right: _,
            } => {
                // Direct scope on Coord should not happen; distribution is done in
                // build_event_inner.  Fall back to scoping only with the left branch.
                left.scope(body)
            }
            ArgCtx::Nominal { var, inner } => {
                Formula::Exists(var, Box::new(Formula::And(inner, Box::new(body))))
            }
        }
    }
}

/// Push negation into the restriction of an ArgCtx (for `ne arg` NPs).
fn negate_restriction(ctx: ArgCtx) -> ArgCtx {
    match ctx {
        ArgCtx::Quant {
            var,
            kind,
            restriction,
        } => ArgCtx::Quant {
            var,
            kind,
            restriction: Formula::Not(Box::new(restriction)),
        },
        ArgCtx::Neg(inner) => *inner, // ¬¬ = identity
        ArgCtx::Coord {
            and_op,
            left,
            right,
        } => ArgCtx::Coord {
            and_op: !and_op, // De Morgan: ¬(A∧B)=¬A∨¬B, ¬(A∨B)=¬A∧¬B
            left: Box::new(negate_restriction(*left)),
            right: Box::new(negate_restriction(*right)),
        },
        other => other, // Const / ExistOnly / Nominal: cannot negate meaningfully
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Translate an `Ast` to a first-order logic `Formula`.
pub fn to_fol(ast: &Ast) -> Formula {
    let mut vg = VarGen::new();
    ast.get_s().to_fol(&mut vg)
}

// ---------------------------------------------------------------------------
// AST traversal — pass-through layers (modality / tense / aspect ignored)
// ---------------------------------------------------------------------------

impl S {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            S::One(d) => d.to_fol(vg),
            S::Two(i) => i.to_fol(vg),
        }
    }
}

impl DeclarativeSentence {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            DeclarativeSentence::One(pm) => pm.to_fol(vg),
        }
    }
}

impl InterrogativeSentence {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            InterrogativeSentence::One(_, pm) => pm.to_fol(vg),
        }
    }
}

impl PostModifiedMtaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // PostMtaEventExpression (da …) is ignored in the FOL translation.
            PostModifiedMtaEvent::One(mta, _post) => mta.to_fol(vg),
        }
    }
}

impl MtaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // Modality is ignored.
            MtaEvent::One(_mod_expr, ta) => ta.to_fol(vg),
        }
    }
}

impl TaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // Tense is ignored.
            TaEvent::One(_tense_expr, a) => a.to_fol(vg),
        }
    }
}

impl AEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // Aspect is ignored.
            AEvent::One(_asp_expr, event) => event.to_fol(vg),
        }
    }
}

// ---------------------------------------------------------------------------
// Event translation
// ---------------------------------------------------------------------------

impl Event {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            Event::One(pred_phrase, event_args) => {
                let pred = extract_pred(pred_phrase);
                match event_args {
                    EventArguments::NominativeFirst(_nom, arg1_phrase, opt_acc, _other) => {
                        let arg1 = arg1_phrase.to_fol(vg);
                        match opt_acc {
                            OptionalAccusativeArg::One(_, arg2_phrase) => {
                                let arg2 = arg2_phrase.to_fol(vg);
                                build_event_inner(&pred, arg1, arg2)
                            }
                            OptionalAccusativeArg::Epsilon => {
                                let y = vg.next();
                                build_event_inner(&pred, arg1, ArgCtx::ExistOnly(y))
                            }
                        }
                    }
                    EventArguments::AccusativeFirst(_acc, arg2_phrase, opt_nom, _other) => {
                        let arg2 = arg2_phrase.to_fol(vg);
                        match opt_nom {
                            OptionalNominativeArg::One(_, arg1_phrase) => {
                                let arg1 = arg1_phrase.to_fol(vg);
                                build_event_inner(&pred, arg1, arg2)
                            }
                            OptionalNominativeArg::Epsilon => {
                                let y = vg.next();
                                build_event_inner(&pred, ArgCtx::ExistOnly(y), arg2)
                            }
                        }
                    }
                    EventArguments::OtherCaseFirst(_prep, _label, _arg, _rest) => {
                        let x = vg.next();
                        let y = vg.next();
                        build_event_inner(&pred, ArgCtx::ExistOnly(x), ArgCtx::ExistOnly(y))
                    }
                    EventArguments::Epsilon => {
                        let x = vg.next();
                        let y = vg.next();
                        build_event_inner(&pred, ArgCtx::ExistOnly(x), ArgCtx::ExistOnly(y))
                    }
                }
            }
        }
    }
}

/// Build a scoped formula for `pred(arg1, arg2)`, handling all coordination and
/// scoping rules recursively.
///
/// Scope order: arg2 is inner (narrow scope), arg1 is outer (wide scope).
fn build_event_inner(pred: &PredCtx, arg1: ArgCtx, arg2: ArgCtx) -> Formula {
    match pred {
        PredCtx::Neg(inner) => {
            // ¬ scopes over everything (Montague-style wide-scope negation).
            Formula::Not(Box::new(build_event_inner(inner, arg1, arg2)))
        }

        PredCtx::Coord {
            and_op,
            left,
            right,
        } => {
            // P di Q / binary_operator P Q: both predicates apply to the same args.
            let f1 = build_event_inner(left, arg1.clone(), arg2.clone());
            let f2 = build_event_inner(right, arg1, arg2);
            if *and_op {
                Formula::And(Box::new(f1), Box::new(f2))
            } else {
                Formula::Or(Box::new(f1), Box::new(f2))
            }
        }

        PredCtx::Simple(name) => {
            // Distribute over arg1 coordination.
            if let ArgCtx::Coord {
                and_op,
                left,
                right,
            } = arg1
            {
                let p = PredCtx::Simple(name.clone());
                let f1 = build_event_inner(&p, *left, arg2.clone());
                let f2 = build_event_inner(&p, *right, arg2);
                return if and_op {
                    Formula::And(Box::new(f1), Box::new(f2))
                } else {
                    Formula::Or(Box::new(f1), Box::new(f2))
                };
            }
            // Distribute over arg2 coordination.
            if let ArgCtx::Coord {
                and_op,
                left,
                right,
            } = arg2
            {
                let p = PredCtx::Simple(name.clone());
                let f1 = build_event_inner(&p, arg1.clone(), *left);
                let f2 = build_event_inner(&p, arg1, *right);
                return if and_op {
                    Formula::And(Box::new(f1), Box::new(f2))
                } else {
                    Formula::Or(Box::new(f1), Box::new(f2))
                };
            }

            // Base case: build atomic and apply quantifier scopes.
            let t1 = arg1.term();
            let t2 = arg2.term();
            let atomic = Formula::Atom(name.clone(), vec![t1, t2]);
            // arg2 scopes inside (narrow), arg1 scopes outside (wide).
            let with_q2 = arg2.scope(atomic);
            arg1.scope(with_q2)
        }
    }
}

// ---------------------------------------------------------------------------
// Argument phrase translation
// ---------------------------------------------------------------------------

impl ArgumentPhrase {
    fn to_fol(&self, vg: &mut VarGen) -> ArgCtx {
        match self {
            // Bare pronoun → individual constant.
            ArgumentPhrase::One(arg) => ArgCtx::Const(arg.get_string()),

            // Negated argument phrase.
            ArgumentPhrase::Two(_neg, inner) => ArgCtx::Neg(Box::new(inner.to_fol(vg))),

            // Coordinated argument phrases (binary_operator arg1 arg2).
            ArgumentPhrase::Three(_op, left, right) => ArgCtx::Coord {
                and_op: true, // binary_operator は論理積として扱う
                left: Box::new(left.to_fol(vg)),
                right: Box::new(right.to_fol(vg)),
            },

            // Quantified argument phrase.
            ArgumentPhrase::Four(quant_arg) => quant_arg.to_fol(vg),

            // Event nominalization: se MtaEvent so.
            ArgumentPhrase::Five(_se, mta_event, _so) => {
                let var = vg.next();
                let inner = mta_event.to_fol(vg);
                ArgCtx::Nominal {
                    var,
                    inner: Box::new(inner),
                }
            }
        }
    }
}

impl QuantifiedArgument {
    fn to_fol(&self, vg: &mut VarGen) -> ArgCtx {
        match self {
            QuantifiedArgument::One(quant_mod, det_arg) => {
                let kind = extract_quant_kind(quant_mod);
                let var = vg.next();
                let restriction = det_arg.to_fol(&var, vg);
                ArgCtx::Quant {
                    var,
                    kind,
                    restriction,
                }
            }
        }
    }
}

fn extract_quant_kind(qm: &QuantifierModifier) -> QuantKind {
    match qm {
        QuantifierModifier::One(q) => match q.get_string().as_str() {
            "ta" => QuantKind::Universal,
            "te" => QuantKind::Many,
            "ti" => QuantKind::Half,
            "to" => QuantKind::Existential,
            _ => QuantKind::Existential,
        },
        QuantifierModifier::Two(_number) => QuantKind::Count,
        QuantifierModifier::Epsilon => QuantKind::Existential,
    }
}

impl DeterminedArgument {
    fn to_fol(&self, var: &str, vg: &mut VarGen) -> Formula {
        match self {
            DeterminedArgument::One(det_mod, ret_arg) => {
                let base = ret_arg.to_fol(var, vg);
                match det_mod {
                    // Determiner mo/bo/go adds a property predicate: mo(x), bo(x), go(x).
                    DeterminerModifier::One(det) => Formula::And(
                        Box::new(Formula::Atom(
                            det.get_string(),
                            vec![Term::Var(var.to_string())],
                        )),
                        Box::new(base),
                    ),
                    DeterminerModifier::Epsilon => base,
                }
            }
        }
    }
}

impl RetrievedArgument {
    fn to_fol(&self, var: &str, vg: &mut VarGen) -> Formula {
        match self {
            RetrievedArgument::One(retriever, pred) => pred.to_fol(retriever.get_kind(), var, vg),
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    fn to_fol(&self, kind: &ArgumentRetrieverKind, var: &str, vg: &mut VarGen) -> Formula {
        match self {
            ArgumentRetrievalBinaryPredicate::One(pred_phrase, with_arg) => {
                match with_arg {
                    ArgumentRetrievalWithArgumentPhrase::Epsilon => {
                        build_restriction(pred_phrase, var)
                    }
                    ArgumentRetrievalWithArgumentPhrase::One(_du, extra_arg) => {
                        let pred_ctx = extract_pred(pred_phrase);
                        let extra_ctx = extra_arg.to_fol(vg);
                        let extra_term = extra_ctx.term();
                        match kind {
                            ArgumentRetrieverKind::Nominative => {
                                // ra: var は主格（第1引数）、extra は対格（第2引数）
                                build_binary_restriction(
                                    &pred_ctx,
                                    Term::Var(var.to_string()),
                                    extra_term,
                                )
                            }
                            ArgumentRetrieverKind::Accusative => {
                                // ri: extra は主格（第1引数）、var は対格（第2引数）
                                build_binary_restriction(
                                    &pred_ctx,
                                    extra_term,
                                    Term::Var(var.to_string()),
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Build a unary restriction formula for an NP predicate, respecting negation.
fn build_restriction(phrase: &PostModifiedBinaryPredicatePhrase, var: &str) -> Formula {
    match phrase {
        PostModifiedBinaryPredicatePhrase::One(bp, _post) => build_bp_restriction(bp, var),
    }
}

fn build_bp_restriction(phrase: &BinaryPredicatePhrase, var: &str) -> Formula {
    match phrase {
        BinaryPredicatePhrase::One(pred) => {
            Formula::Atom(pred.get_string(), vec![Term::Var(var.to_string())])
        }
        BinaryPredicatePhrase::Two(_neg, inner) => {
            Formula::Not(Box::new(build_bp_restriction(inner, var)))
        }
        BinaryPredicatePhrase::Three(_op, p1, p2) => Formula::And(
            Box::new(build_bp_restriction(p1, var)),
            Box::new(build_bp_restriction(p2, var)),
        ),
    }
}

fn build_binary_restriction(pred: &PredCtx, t1: Term, t2: Term) -> Formula {
    match pred {
        PredCtx::Simple(name) => Formula::Atom(name.clone(), vec![t1, t2]),
        PredCtx::Neg(inner) => Formula::Not(Box::new(build_binary_restriction(inner, t1, t2))),
        PredCtx::Coord {
            and_op,
            left,
            right,
        } => {
            let f1 = build_binary_restriction(left, t1.clone(), t2.clone());
            let f2 = build_binary_restriction(right, t1, t2);
            if *and_op {
                Formula::And(Box::new(f1), Box::new(f2))
            } else {
                Formula::Or(Box::new(f1), Box::new(f2))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Diagnostic, diagnose};

    /// Parse `text`, run all checks, and return the FOL formula string for the first sentence.
    fn to_fol_string(text: &str) -> Result<String, Vec<Diagnostic>> {
        let asts = diagnose(text.to_string())?;
        Ok(to_fol(&asts[0]).to_string())
    }

    fn check(text: &str, expected: &str) {
        let fol = to_fol_string(text).unwrap();
        assert_eq!(fol, expected, "input: {text}");
    }

    #[test]
    fn test_存在量化_私はりんごを食べる() {
        check(
            "pahete sa mi si ri behapare",
            "∃x.(behapare(x) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_全称量化_私は全てのりんごを食べる() {
        check(
            "pahete sa mi si ta ri behapare",
            "∀x.(behapare(x) ⇒ pahete(mi, x))",
        );
    }

    #[test]
    fn test_述語否定_私はりんごを食べない() {
        check(
            "ne pahete sa mi si ri behapare",
            "¬∃x.(behapare(x) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_転置_りんごは食べられない() {
        check(
            "hapo pahete si ri behapare",
            "∃y.∃x.(behapare(x) ∧ pahete(y, x))",
        );
    }

    #[test]
    fn test_転置と否定_みかんは私に食べられていない() {
        check(
            "ne pahete sa mi si ri behoranage",
            "¬∃x.(behoranage(x) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_指示詞_このりんご() {
        check(
            "pahete sa mi si mo ri behapare",
            "∃x.((mo(x) ∧ behapare(x)) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_数量化_数のりんご() {
        check(
            "pahete sa mi si number ri behapare",
            "∃_# x.(behapare(x) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_多量化_多くのりんご() {
        check(
            "pahete sa mi si te ri behapare",
            "∃_many x.(behapare(x) ∧ pahete(mi, x))",
        );
    }

    #[test]
    fn test_関係節_私はあなたをパートナーとする人を愛している() {
        check(
            "parobe sa mi si ri paparatanera du ba",
            "∃x.(paparatanera(ba, x) ∧ parobe(mi, x))",
        );
    }

    #[test]
    #[ignore]
    fn test_関係節_私はあなたのパートナーを愛している() {
        check(
            "parobe sa mi si ro re paparatanera du ba",
            "∃x.(paparatanera(ba, x) ∧ parobe(mi, x))",
        );
    }

    #[test]
    #[ignore]
    fn test_事象名詞化_あなたは_私はりんごを食べる_と言った() {
        check(
            "kasahi ba si se pahete mi si ri behapare so",
            "kasahi(ba, ∃x.(behapare(x) ∧ pahete(mi, x))",
        );
    }
}
