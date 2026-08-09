use crate::parser::{
    AEvent, ArgumentPhrase, ArgumentRetrievalBinaryPredicate, ArgumentRetrievalWithArgumentPhrase,
    Ast, BinaryPredicatePhrase, DeclarativeSentence, DeterminedArgument, DeterminerModifier, Event,
    InterrogativeSentence, MtaEvent, PostBinaryPredicatePhraseExpression,
    PostModifiedBinaryPredicatePhrase, PostModifiedMtaEvent, PostMtaEventExpression,
    QuantifiedArgument, QuantifierModifier, RetrievedArgument, S, TaEvent,
};
use crate::tokenizer::{ArgumentRetrieverKind, CaseLabelKind, PostMtaEventModifierKind};

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

struct VarGen {
    obj: usize,
    evt: usize,
}

impl VarGen {
    fn new() -> Self {
        VarGen { obj: 0, evt: 0 }
    }

    fn next(&mut self) -> String {
        let n = self.obj;
        self.obj += 1;
        match n {
            0 => "x".to_string(),
            1 => "y".to_string(),
            2 => "z".to_string(),
            _ => format!("x{}", n - 2),
        }
    }

    fn next_event(&mut self) -> String {
        let n = self.evt;
        self.evt += 1;
        if n == 0 {
            "e".to_string()
        } else {
            format!("e{n}")
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
    /// Event nominalization (he … ho): ∃var.(inner ∧ body).
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
            PostModifiedMtaEvent::One(mta, post) => match post {
                PostMtaEventExpression::Epsilon => mta.to_fol(vg),
                PostMtaEventExpression::One(modifier, mta2) => match modifier.get_kind() {
                    // da (命題修飾句の前置詞) is still unimplemented; ignored in the FOL translation.
                    PostMtaEventModifierKind::Da => mta.to_fol(vg),
                    // hesute/henesute/hesuto/henesuto (因果) and hehita/hehiti/hehitu (時間前後・
                    // 同時)、hehema (例示)、hehegi (言い換え)、hehenu (対比) assert that both
                    // propositions hold; the discourse relation itself is not truth-conditional
                    // and is not represented in FOL (same treatment as modality/tense/aspect).
                    PostMtaEventModifierKind::Hesute
                    | PostMtaEventModifierKind::Henesute
                    | PostMtaEventModifierKind::Hesuto
                    | PostMtaEventModifierKind::Henesuto
                    | PostMtaEventModifierKind::Hehoma
                    | PostMtaEventModifierKind::Hehita
                    | PostMtaEventModifierKind::Hehiti
                    | PostMtaEventModifierKind::Hehitu
                    | PostMtaEventModifierKind::Hehema
                    | PostMtaEventModifierKind::Hehegi
                    | PostMtaEventModifierKind::Hehenu => {
                        Formula::And(Box::new(mta.to_fol(vg)), Box::new(mta2.to_fol(vg)))
                    }
                    // hehomi (or)
                    PostMtaEventModifierKind::Hehomi => {
                        Formula::Or(Box::new(mta.to_fol(vg)), Box::new(mta2.to_fol(vg)))
                    }
                    // hehomu (xor): (P1 ∨ P2) ∧ ¬(P1 ∧ P2)
                    PostMtaEventModifierKind::Hehomu => {
                        let f1 = mta.to_fol(vg);
                        let f2 = mta2.to_fol(vg);
                        Formula::And(
                            Box::new(Formula::Or(Box::new(f1.clone()), Box::new(f2.clone()))),
                            Box::new(Formula::Not(Box::new(Formula::And(
                                Box::new(f1),
                                Box::new(f2),
                            )))),
                        )
                    }
                    // hezu (条件): P1ならP2する
                    PostMtaEventModifierKind::Hezu => {
                        Formula::Implies(Box::new(mta.to_fol(vg)), Box::new(mta2.to_fol(vg)))
                    }
                },
            },
        }
    }
}

impl MtaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // ModalityBase is ignored.
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
        let pred = extract_pred(self.get_pred());
        let case_args = self.case_args();
        let nom = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sa));
        let acc = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Si));
        let comitative = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Subu));
        let animate_source = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sube));
        let animate_goal = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Subo));
        let cause = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sute));
        let purpose = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Suto));
        let instrument = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sudu));
        let location = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Suni));
        let location_source = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sune));
        let location_goal = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Suno));
        let time = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sugi));
        let time_source = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Suge));
        let time_goal = case_args
            .iter()
            .find(|(l, _)| matches!(l.get_kind(), CaseLabelKind::Sugo));
        let arg2 = if let Some((_, phrase)) = acc {
            phrase.to_fol(vg)
        } else {
            let y = vg.next();
            ArgCtx::ExistOnly(y)
        };
        let arg1 = if let Some((_, phrase)) = nom {
            phrase.to_fol(vg)
        } else {
            let y = vg.next();
            ArgCtx::ExistOnly(y)
        };
        // 随伴格・奪格・与格・原因格・目的格・具格・空間系・時間系は付加詞のため、
        // 省略時は存在閉包を作らず式に出さない。
        let extra_roles: Vec<(&'static str, ArgCtx)> = [
            comitative.map(|(_, phrase)| ("Comitative", phrase.to_fol(vg))),
            animate_source.map(|(_, phrase)| ("AnimateSource", phrase.to_fol(vg))),
            animate_goal.map(|(_, phrase)| ("AnimateGoal", phrase.to_fol(vg))),
            cause.map(|(_, phrase)| ("Cause", phrase.to_fol(vg))),
            purpose.map(|(_, phrase)| ("Purpose", phrase.to_fol(vg))),
            instrument.map(|(_, phrase)| ("Instrument", phrase.to_fol(vg))),
            location.map(|(_, phrase)| ("Location", phrase.to_fol(vg))),
            location_source.map(|(_, phrase)| ("LocationSource", phrase.to_fol(vg))),
            location_goal.map(|(_, phrase)| ("LocationGoal", phrase.to_fol(vg))),
            time.map(|(_, phrase)| ("Time", phrase.to_fol(vg))),
            time_source.map(|(_, phrase)| ("TimeSource", phrase.to_fol(vg))),
            time_goal.map(|(_, phrase)| ("TimeGoal", phrase.to_fol(vg))),
        ]
        .into_iter()
        .flatten()
        .collect();
        build_event_inner(&pred, arg1, arg2, extra_roles, vg)
    }
}

/// Build a scoped formula for `pred(arg1, arg2)` in neo-Davidsonian event semantics:
/// `∃e.(pred(e) ∧ Agent(e, arg1) ∧ Patient(e, arg2) ∧ <extra_roles>)`
///
/// Scope order (outer → inner): arg1, extra_roles (declared order), arg2.
fn build_event_inner(
    pred: &PredCtx,
    arg1: ArgCtx,
    arg2: ArgCtx,
    extra_roles: Vec<(&'static str, ArgCtx)>,
    vg: &mut VarGen,
) -> Formula {
    match pred {
        PredCtx::Neg(inner) => {
            // ¬ scopes over everything (Montague-style wide-scope negation).
            Formula::Not(Box::new(build_event_inner(
                inner,
                arg1,
                arg2,
                extra_roles,
                vg,
            )))
        }

        PredCtx::Coord {
            and_op,
            left,
            right,
        } => {
            // P di Q / binary_operator P Q: both predicates apply to the same args.
            let f1 = build_event_inner(left, arg1.clone(), arg2.clone(), extra_roles.clone(), vg);
            let f2 = build_event_inner(right, arg1, arg2, extra_roles, vg);
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
                let f1 = build_event_inner(&p, *left, arg2.clone(), extra_roles.clone(), vg);
                let f2 = build_event_inner(&p, *right, arg2, extra_roles, vg);
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
                let f1 = build_event_inner(&p, arg1.clone(), *left, extra_roles.clone(), vg);
                let f2 = build_event_inner(&p, arg1, *right, extra_roles, vg);
                return if and_op {
                    Formula::And(Box::new(f1), Box::new(f2))
                } else {
                    Formula::Or(Box::new(f1), Box::new(f2))
                };
            }

            // Base case: ∃e.(pred(e) ∧ Agent(e, t1) ∧ Patient(e, t2) ∧ <extra_roles>)
            let t1 = arg1.term();
            let t2 = arg2.term();
            let e = vg.next_event();
            let ev = Term::Var(e.clone());
            let pred_atom = Formula::Atom(name.clone(), vec![ev.clone()]);
            let agent_atom = Formula::Atom("Agent".to_string(), vec![ev.clone(), t1]);
            let patient_atom = Formula::Atom("Patient".to_string(), vec![ev.clone(), t2]);
            let mut event_body = Formula::And(
                Box::new(pred_atom),
                Box::new(Formula::And(Box::new(agent_atom), Box::new(patient_atom))),
            );
            for (role_name, ctx) in &extra_roles {
                let role_atom = Formula::Atom(role_name.to_string(), vec![ev.clone(), ctx.term()]);
                event_body = Formula::And(Box::new(event_body), Box::new(role_atom));
            }
            let event_formula = Formula::Exists(e, Box::new(event_body));
            // arg2 scopes innermost, extra_roles in between, arg1 scopes outermost.
            let mut f = arg2.scope(event_formula);
            for (_, ctx) in extra_roles.into_iter().rev() {
                f = ctx.scope(f);
            }
            arg1.scope(f)
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

            // Event nominalization: he MtaEvent ho.
            ArgumentPhrase::Five(_he, mta_event, _ho) => {
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
                                    vg,
                                )
                            }
                            ArgumentRetrieverKind::Accusative => {
                                // ri: extra は主格（第1引数）、var は対格（第2引数）
                                build_binary_restriction(
                                    &pred_ctx,
                                    extra_term,
                                    Term::Var(var.to_string()),
                                    vg,
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

fn build_binary_restriction(pred: &PredCtx, t1: Term, t2: Term, vg: &mut VarGen) -> Formula {
    match pred {
        PredCtx::Simple(name) => {
            let e = vg.next_event();
            let ev = Term::Var(e.clone());
            let pred_atom = Formula::Atom(name.clone(), vec![ev.clone()]);
            let agent_atom = Formula::Atom("Agent".to_string(), vec![ev.clone(), t1]);
            let patient_atom = Formula::Atom("Patient".to_string(), vec![ev, t2]);
            Formula::Exists(
                e,
                Box::new(Formula::And(
                    Box::new(pred_atom),
                    Box::new(Formula::And(Box::new(agent_atom), Box::new(patient_atom))),
                )),
            )
        }
        PredCtx::Neg(inner) => Formula::Not(Box::new(build_binary_restriction(inner, t1, t2, vg))),
        PredCtx::Coord {
            and_op,
            left,
            right,
        } => {
            let f1 = build_binary_restriction(left, t1.clone(), t2.clone(), vg);
            let f2 = build_binary_restriction(right, t1, t2, vg);
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
    fn test_随伴格_私はあなたとりんごを食べる() {
        check(
            "pahete sa mi si ri behapare subu ba",
            "∃x.(behapare(x) ∧ ∃e.((pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Comitative(e, ba)))",
        );
    }

    #[test]
    fn test_奪格与格_私はあなたにりんごをあげる() {
        check(
            "pamobe sa mi si ri behapare sube mi subo ba",
            "∃x.(behapare(x) ∧ ∃e.(((pamobe(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ AnimateSource(e, mi)) ∧ AnimateGoal(e, ba)))",
        );
    }

    #[test]
    fn test_原因格目的格_私はりんごを理由があって目的のために食べる() {
        check(
            "pahete sa mi si ri behapare sute ri kahoraka suto ri kahoraka",
            "∃y.(kahoraka(y) ∧ ∃z.(kahoraka(z) ∧ ∃x.(behapare(x) ∧ ∃e.(((pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Cause(e, y)) ∧ Purpose(e, z)))))",
        );
    }

    #[test]
    fn test_具格_私は道具を使ってりんごを食べる() {
        check(
            "pahete sa mi si ri behapare sudu ri didora",
            "∃y.(didora(y) ∧ ∃x.(behapare(x) ∧ ∃e.((pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Instrument(e, y))))",
        );
    }

    #[test]
    fn test_空間系_私はある空間からある空間へある空間でりんごを食べる() {
        check(
            "pahete sa mi si ri behapare suni me sune be suno ge",
            "∃x.(behapare(x) ∧ ∃e.((((pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Location(e, me)) ∧ LocationSource(e, be)) ∧ LocationGoal(e, ge)))",
        );
    }

    #[test]
    fn test_時間系_私はある時からある時までりんごを食べる() {
        check(
            "pahete sa mi si ri behapare sugi me suge be sugo ge",
            "∃x.(behapare(x) ∧ ∃e.((((pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Time(e, me)) ∧ TimeSource(e, be)) ∧ TimeGoal(e, ge)))",
        );
    }

    #[test]
    fn test_存在量化_私はりんごを食べる() {
        check(
            "pahete sa mi si ri behapare",
            "∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_接続詞_hesute_結果から原因() {
        check(
            "pahete sa mi si ri behapare hesute pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henesute_結果から反対の原因() {
        check(
            "pahete sa mi si ri behapare henesute pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hesuto_原因から結果() {
        check(
            "pahete sa mi si ri behapare hesuto pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henesuto_反対の原因から結果() {
        check(
            "pahete sa mi si ri behapare henesuto pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehoma_and() {
        check(
            "pahete sa mi si ri behapare hehoma pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomi_or() {
        check(
            "pahete sa mi si ri behapare hehomi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomu_xor() {
        check(
            "pahete sa mi si ri behapare hehomu pahete sa ba si ri behoranage",
            "((∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))) ∧ ¬(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))))",
        );
    }

    #[test]
    fn test_接続詞_hehita_時間前() {
        check(
            "pahete sa mi si ri behapare hehita pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehiti_同時() {
        check(
            "pahete sa mi si ri behapare hehiti pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehitu_時間後() {
        check(
            "pahete sa mi si ri behapare hehitu pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hezu_条件() {
        check(
            "pahete sa mi si ri behapare hezu pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ⇒ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehema_例示() {
        check(
            "pahete sa mi si ri behapare hehema pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehegi_言い換え() {
        check(
            "pahete sa mi si ri behapare hehegi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehenu_対比() {
        check(
            "pahete sa mi si ri behapare hehenu pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_全称量化_私は全てのりんごを食べる() {
        check(
            "pahete sa mi si ta ri behapare",
            "∀x.(behapare(x) ⇒ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_述語否定_私はりんごを食べない() {
        check(
            "ne pahete sa mi si ri behapare",
            "¬∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_転置_りんごは食べられない() {
        check(
            "hapu □ pahete si ri behapare",
            "∃y.∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, y) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_転置と否定_みかんは私に食べられていない() {
        check(
            "ne pahete sa mi si ri behoranage",
            "¬∃x.(behoranage(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_指示詞_このりんご() {
        check(
            "pahete sa mi si mo ri behapare",
            "∃x.((mo(x) ∧ behapare(x)) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_数量化_数のりんご() {
        check(
            "pahete sa mi si number ri behapare",
            "∃_# x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_多量化_多くのりんご() {
        check(
            "pahete sa mi si te ri behapare",
            "∃_many x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_関係節_私はあなたをパートナーとする人を愛している() {
        check(
            "parobe sa mi si ri paparatanera du ba",
            "∃x.(∃e.(paparatanera(e) ∧ (Agent(e, ba) ∧ Patient(e, x))) ∧ ∃e1.(parobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    #[ignore]
    fn test_関係節_私はあなたのパートナーを愛している() {
        check(
            "parobe sa mi si ro re paparatanera du ba",
            "∃x.(∃e.(paparatanera(e) ∧ (Agent(e, ba) ∧ Patient(e, x))) ∧ ∃e1.(parobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    #[ignore]
    fn test_事象名詞化_あなたは_私はりんごを食べる_と言った() {
        check(
            "kasahi ba si he pahete mi si ri behapare ho",
            "∃x.(∃y.(behapare(y) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, y)))) ∧ ∃e1.(kasahi(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, x))))",
        );
    }
}
