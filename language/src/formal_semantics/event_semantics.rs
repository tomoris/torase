//! 型チェック済みASTをネオ・デイヴィドソン流の一階述語論理式(`Formula`)へ変換する。

use std::collections::HashMap;

use crate::parser::{
    AEvent, ArgumentPhrase, ArgumentRetrievalBinaryPredicate, ArgumentRetrievalWithArgumentPhrase,
    Ast, BinaryPredicatePhrase, DeclarativeSentence, DeterminedArgument, DeterminerModifier, Event,
    InterrogativeSentence, MtaEvent, PostBinaryPredicatePhraseExpression,
    PostModifiedBinaryPredicatePhrase, PostModifiedMtaEvent, QuantifiedArgument,
    QuantifierModifier, RetrievedArgument, RoleArg, S, TaEvent, UniversalSentence,
};
use crate::tokenizer::{
    ArgumentRetrieverKind, CaseLabelKind, ConnectiveHeadKind, NegationKind, OperatorKind,
};

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
    /// P₁ di/hoba/hobi/hobu P₂ — both predicates apply to the same args.
    Coord {
        op: OperatorKind,
        left: Box<PredCtx>,
        right: Box<PredCtx>,
    },
    /// no P — derive a new, independent atom "P@anti" (not a `Not`-wrap of P).
    /// No logical relationship to P is asserted; unlike `Neg`, this leaves room
    /// for neither P nor P@anti to hold (a genuine contrary, not a contradictory).
    Antonym(Box<PredCtx>),
}

fn extract_pred(phrase: &PostModifiedBinaryPredicatePhrase) -> PredCtx {
    match phrase {
        PostModifiedBinaryPredicatePhrase::One(bp, post) => {
            let base = extract_bp(bp);
            match post {
                PostBinaryPredicatePhraseExpression::One(_, secondary) => PredCtx::Coord {
                    op: OperatorKind::And,
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
        BinaryPredicatePhrase::Two(neg, inner) => match neg.get_kind() {
            NegationKind::Ne => PredCtx::Neg(Box::new(extract_bp(inner))),
            NegationKind::No => PredCtx::Antonym(Box::new(extract_bp(inner))),
        },
        BinaryPredicatePhrase::Three(op, p1, p2) => PredCtx::Coord {
            op: op.get_kind().clone(),
            left: Box::new(extract_bp(p1)),
            right: Box::new(extract_bp(p2)),
        },
    }
}

/// Rename every `Simple` leaf `n` in `ctx` to `n@anti` (recursing through
/// `Neg`/`Coord`/nested `Antonym` so `no` still works under `hoba`/`ne`).
/// `@` never round-trips through the tokenizer — it only ever exists inside
/// `Formula`/`PredCtx`, so this can never collide with a real dictionary word.
fn antonym_rename(ctx: &PredCtx) -> PredCtx {
    match ctx {
        PredCtx::Simple(n) => PredCtx::Simple(format!("{n}@anti")),
        PredCtx::Neg(i) => PredCtx::Neg(Box::new(antonym_rename(i))),
        PredCtx::Coord { op, left, right } => PredCtx::Coord {
            op: op.clone(),
            left: Box::new(antonym_rename(left)),
            right: Box::new(antonym_rename(right)),
        },
        PredCtx::Antonym(i) => PredCtx::Antonym(Box::new(antonym_rename(i))),
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
    /// Schema variable (`za`/`zi`). Contributes `Term::Var(name)` like `ExistOnly`, but
    /// `scope()` is a no-op (unlike `ExistOnly`): the variable is left free here and is
    /// closed only by an enclosing `ze`(全称量化マーカー) via `UniversalSentence::to_fol`.
    FreeVar(String),
    /// Negated argument phrase.
    Neg(Box<ArgCtx>),
    /// Coordinated argument phrases (distributed in `build_event_inner`).
    Coord {
        op: OperatorKind,
        left: Box<ArgCtx>,
        right: Box<ArgCtx>,
    },
    /// Event nominalization (he … ho): ∃var.(inner ∧ body).
    Nominal { var: String, inner: Box<Formula> },
}

#[derive(Debug, Clone, PartialEq)]
enum QuantKind {
    Universal,   // ta  全ての
    Existential, // epsilon / to  少なくとも1つは存在する
    Many,        // te  多くの
    Half,        // ti  半分の
    Count,       // number
}

impl ArgCtx {
    /// The term this context contributes to the atomic formula.
    fn term(&self) -> Term {
        match self {
            ArgCtx::Quant { var, .. } | ArgCtx::ExistOnly(var) | ArgCtx::FreeVar(var) => {
                Term::Var(var.clone())
            }
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
            ArgCtx::FreeVar(_) => body,
            ArgCtx::Neg(inner) => {
                // Negate the restriction of the inner quantifier.
                let ctx = negate_restriction(*inner);
                ctx.scope(body)
            }
            ArgCtx::Coord {
                op: _,
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
        // De Morgan: ¬(A∧B)=¬A∨¬B, ¬(A∨B)=¬A∧¬B (flip the connective, negate both sides).
        ArgCtx::Coord {
            op: OperatorKind::And,
            left,
            right,
        } => ArgCtx::Coord {
            op: OperatorKind::Or,
            left: Box::new(negate_restriction(*left)),
            right: Box::new(negate_restriction(*right)),
        },
        ArgCtx::Coord {
            op: OperatorKind::Or,
            left,
            right,
        } => ArgCtx::Coord {
            op: OperatorKind::And,
            left: Box::new(negate_restriction(*left)),
            right: Box::new(negate_restriction(*right)),
        },
        // ¬(A⊕B) ≡ (¬A)⊕B: the connective stays Xor, only one side is negated.
        ArgCtx::Coord {
            op: OperatorKind::Xor,
            left,
            right,
        } => ArgCtx::Coord {
            op: OperatorKind::Xor,
            left: Box::new(negate_restriction(*left)),
            right,
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
            S::Three(u) => u.to_fol(vg),
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

impl UniversalSentence {
    /// ze(全称量化マーカー)。直後に宣言された量化子(ta→∀, to→∃)+変数のリストを宣言順にたどり、
    /// 逆順に畳み込んで本文を明示的に束縛する。NP量化詞の`ta`(すべての)が`Forall`に翻訳される
    /// のと同じ語を再利用する、素直なASTからの翻訳(本文を後から走査して推測することはしない)。
    /// `te`/`ti`はここでは無効な量化子だが、argument_type_checker側で別途エラーにするため、
    /// ここでは`to`扱い(Exists)にフォールバックしてパニックしない。
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            UniversalSentence::One(_, quantified_variable_list, pm) => {
                let body = pm.to_fol(vg);
                quantified_variable_list.variables().into_iter().rev().fold(
                    body,
                    |acc, (force, arg)| {
                        let name = arg.get_string();
                        match force.get_string().as_str() {
                            "ta" => Formula::Forall(name, Box::new(acc)),
                            _ => Formula::Exists(name, Box::new(acc)),
                        }
                    },
                )
            }
        }
    }
}

/// `za`/`zi`(定義文用のスキーマ変数)を指す名前かどうか。`free_variables`の結果から
/// スキーマ変数だけを絞り込むために使う(`lib.rs`参照)。
pub fn is_schema_variable_name(name: &str) -> bool {
    name == "za" || name == "zi"
}

/// `formula`に含まれる、`Forall`/`Exists`/`GenQ`で束縛されていない`Term::Var`の名前を、
/// 最初に出現した順で(重複無く)返す。`ze`マーカーの`to_fol`(上記)で使う。
///
/// 注意: `za`/`zi`以外にも、この関数は既存の(このスキーマ変数機能とは無関係な)
/// 述語生成コードに起因する未束縛変数(例: 項が省略された`ri`/`ra`によるイベント名詞化で
/// 生じるAgent/Patientのプレースホルダ変数)も返しうる。呼び出し側で`za`/`zi`かどうかを
/// `is_schema_variable_name`で絞り込むこと。
pub fn free_variables(formula: &Formula) -> Vec<String> {
    let mut bound = std::collections::HashSet::new();
    let mut free = Vec::new();
    collect_free_variables(formula, &mut bound, &mut free);
    free
}

fn collect_free_variables(
    formula: &Formula,
    bound: &mut std::collections::HashSet<String>,
    free: &mut Vec<String>,
) {
    match formula {
        Formula::Atom(_, args) => {
            for term in args {
                if let Term::Var(name) = term
                    && !bound.contains(name)
                    && !free.contains(name)
                {
                    free.push(name.clone());
                }
            }
        }
        Formula::Not(phi) => collect_free_variables(phi, bound, free),
        Formula::And(l, r) | Formula::Or(l, r) | Formula::Implies(l, r) => {
            collect_free_variables(l, bound, free);
            collect_free_variables(r, bound, free);
        }
        Formula::Forall(x, phi) | Formula::Exists(x, phi) | Formula::GenQ(_, x, phi) => {
            let newly_bound = bound.insert(x.clone());
            collect_free_variables(phi, bound, free);
            if newly_bound {
                bound.remove(x);
            }
        }
    }
}

/// `arg1`/`arg2`を`RoleLabelKind`(`Hesa`/`Hesi`)で引き当て、`(P, Q)`(`P`=`hesa`側, `Q`=`hesi`側)の
/// 順で返す。文法上は`hesa`/`hesi`以外のラベルは存在しないが、同じラベルの重複は構文的には
/// 通ってしまう(実際の検証は`argument_type_checker.rs`の`check_role_arg_pair`が担当)ため、
/// `lib.rs`の`diagnose()`が型検査の結果に関わらず`to_fol`を無条件に呼んでもパニックしないよう、
/// ラベルが揃わない場合はソース順にフォールバックする(`Event::to_fol`が`sa`/`si`欠如時に
/// `ArgCtx::ExistOnly`にフォールバックするのと同じ考え方)。
fn resolve_role_args<'a>(arg1: &'a RoleArg, arg2: &'a RoleArg) -> (&'a RoleArg, &'a RoleArg) {
    RoleArg::resolve_hesa_hesi(arg1, arg2).unwrap_or((arg1, arg2))
}

impl PostModifiedMtaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            PostModifiedMtaEvent::One(mta) => mta.to_fol(vg),
            PostModifiedMtaEvent::Two(head, arg1, arg2) => {
                let (p_arg, q_arg) = resolve_role_args(arg1, arg2);
                let p = p_arg.get_mta_event().to_fol(vg);
                let q = q_arg.get_mta_event().to_fol(vg);
                match head.get_kind() {
                    ConnectiveHeadKind::Hesuteto
                    | ConnectiveHeadKind::Henosuteto
                    | ConnectiveHeadKind::Hehoma
                    | ConnectiveHeadKind::Hehitatu
                    | ConnectiveHeadKind::Hehiti => Formula::And(Box::new(p), Box::new(q)),
                    ConnectiveHeadKind::Hehomi => Formula::Or(Box::new(p), Box::new(q)),
                    ConnectiveHeadKind::Hehomu => xor_formula(p, q),
                    ConnectiveHeadKind::Hehenu => {
                        Formula::And(Box::new(Formula::Not(Box::new(p))), Box::new(q))
                    }
                    ConnectiveHeadKind::Hehito | ConnectiveHeadKind::Henohito => {
                        Formula::Implies(Box::new(p), Box::new(q))
                    }
                    ConnectiveHeadKind::Hehezu => Formula::And(
                        Box::new(Formula::Implies(Box::new(p.clone()), Box::new(q.clone()))),
                        Box::new(Formula::Implies(Box::new(q), Box::new(p))),
                    ),
                }
            }
        }
    }
}

impl MtaEvent {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        match self {
            // ModalityBase is ignored.
            MtaEvent::One(_mod_expr, ta) => ta.to_fol(vg),
            // homa/homi/homu: proposition-level and/or/xor.
            MtaEvent::Two(op, m1, m2) => {
                let f1 = m1.to_fol(vg);
                let f2 = m2.to_fol(vg);
                match op.get_kind() {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
                }
            }
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

/// 宣言順が`Event::to_fol`でのスコープ順(外側→内側)になる。
const EXTRA_ROLE_TABLE: [(CaseLabelKind, &str); 12] = [
    (CaseLabelKind::Subu, "Comitative"),
    (CaseLabelKind::Sube, "AnimateSource"),
    (CaseLabelKind::Subo, "AnimateGoal"),
    (CaseLabelKind::Sute, "Cause"),
    (CaseLabelKind::Suto, "Purpose"),
    (CaseLabelKind::Sudu, "Instrument"),
    (CaseLabelKind::Suni, "Location"),
    (CaseLabelKind::Sune, "LocationSource"),
    (CaseLabelKind::Suno, "LocationGoal"),
    (CaseLabelKind::Sugi, "Time"),
    (CaseLabelKind::Suge, "TimeSource"),
    (CaseLabelKind::Sugo, "TimeGoal"),
];

impl Event {
    fn to_fol(&self, vg: &mut VarGen) -> Formula {
        let pred = extract_pred(self.get_pred());
        let case_args = self.case_args();
        // 格ラベルの種類ごとに引き当てる。同じ種類が複数回現れた場合は最初の1つを使う
        // (`argument_type_checker.rs`が重複をエラーにするため、実際には型検査を通った
        // 入力では起こらないが、`diagnose()`は型検査の結果に関わらず`to_fol`を呼ぶため
        // この関数単体でも壊れないようにしておく)。
        let mut by_kind: HashMap<&CaseLabelKind, &ArgumentPhrase> = HashMap::new();
        for (label, phrase) in case_args {
            by_kind.entry(label.get_kind()).or_insert(phrase);
        }
        let arg2 = if let Some(&phrase) = by_kind.get(&CaseLabelKind::Si) {
            phrase.to_fol(vg)
        } else {
            let y = vg.next();
            ArgCtx::ExistOnly(y)
        };
        let arg1 = if let Some(&phrase) = by_kind.get(&CaseLabelKind::Sa) {
            phrase.to_fol(vg)
        } else {
            let y = vg.next();
            ArgCtx::ExistOnly(y)
        };
        // 付加詞は省略時に存在閉包を作らず式に出さない。
        let extra_roles: Vec<(&'static str, ArgCtx)> = EXTRA_ROLE_TABLE
            .into_iter()
            .filter_map(|(kind, role_name)| {
                by_kind
                    .get(&kind)
                    .map(|&phrase| (role_name, phrase.to_fol(vg)))
            })
            .collect();
        build_event_inner(&pred, arg1, arg2, extra_roles, vg)
    }
}

/// Desugar `A ⊕ B` into `(A ∨ B) ∧ ¬(A ∧ B)` using only existing `Formula` combinators
/// (no native `Formula::Xor` variant, so `Display`/`contradiction_checker` need no changes).
fn xor_formula(f1: Formula, f2: Formula) -> Formula {
    Formula::And(
        Box::new(Formula::Or(Box::new(f1.clone()), Box::new(f2.clone()))),
        Box::new(Formula::Not(Box::new(Formula::And(
            Box::new(f1),
            Box::new(f2),
        )))),
    )
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

        PredCtx::Antonym(inner) => {
            // no P: derive a fresh, independent atom "P@anti" and compile it exactly
            // like an ordinary predicate (no `Not`, no asserted relationship to P).
            build_event_inner(&antonym_rename(inner), arg1, arg2, extra_roles, vg)
        }

        PredCtx::Coord { op, left, right } => {
            // P di Q / P hoba|hobi|hobu Q: both predicates apply to the same args.
            let f1 = build_event_inner(left, arg1.clone(), arg2.clone(), extra_roles.clone(), vg);
            let f2 = build_event_inner(right, arg1, arg2, extra_roles, vg);
            match op {
                OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                OperatorKind::Xor => xor_formula(f1, f2),
            }
        }

        PredCtx::Simple(name) => {
            // Distribute over arg1 coordination.
            if let ArgCtx::Coord { op, left, right } = arg1 {
                let p = PredCtx::Simple(name.clone());
                let f1 = build_event_inner(&p, *left, arg2.clone(), extra_roles.clone(), vg);
                let f2 = build_event_inner(&p, *right, arg2, extra_roles, vg);
                return match op {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
                };
            }
            // Distribute over arg2 coordination.
            if let ArgCtx::Coord { op, left, right } = arg2 {
                let p = PredCtx::Simple(name.clone());
                let f1 = build_event_inner(&p, arg1.clone(), *left, extra_roles.clone(), vg);
                let f2 = build_event_inner(&p, arg1, *right, extra_roles, vg);
                return match op {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
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
            // Bare pronoun → individual constant, except za/zi (schema variables) → free variable.
            ArgumentPhrase::One(arg) if arg.is_schema_variable() => {
                ArgCtx::FreeVar(arg.get_string())
            }
            ArgumentPhrase::One(arg) => ArgCtx::Const(arg.get_string()),

            // Negated argument phrase.
            ArgumentPhrase::Two(_neg, inner) => ArgCtx::Neg(Box::new(inner.to_fol(vg))),

            // Coordinated argument phrases (hoba/hobi/hobu arg1 arg2).
            ArgumentPhrase::Three(op, left, right) => ArgCtx::Coord {
                op: op.get_kind().clone(),
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

            // Proper noun: he<X> BODY hoho → individual constant, namespaced by the
            // opener so a quoted name can never collide with a pronoun constant or
            // with the same raw text quoted under a different he<X> class.
            ArgumentPhrase::Six(opener, body, _hoho) => {
                ArgCtx::Const(format!("{}:{}", opener.get_string(), body.get_string()))
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
        BinaryPredicatePhrase::Two(neg, inner) => match neg.get_kind() {
            NegationKind::Ne => Formula::Not(Box::new(build_bp_restriction(inner, var))),
            NegationKind::No => rename_formula_atoms(build_bp_restriction(inner, var)),
        },
        BinaryPredicatePhrase::Three(op, p1, p2) => {
            let f1 = build_bp_restriction(p1, var);
            let f2 = build_bp_restriction(p2, var);
            match op.get_kind() {
                OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                OperatorKind::Xor => xor_formula(f1, f2),
            }
        }
    }
}

/// Rename every atom name `n` to `n@anti` in a formula built by `build_bp_restriction`
/// (whose output is limited to `Atom`/`Not`/`And`/`Or` — see that function's own recursion).
fn rename_formula_atoms(f: Formula) -> Formula {
    match f {
        Formula::Atom(n, args) => Formula::Atom(format!("{n}@anti"), args),
        Formula::Not(inner) => Formula::Not(Box::new(rename_formula_atoms(*inner))),
        Formula::And(l, r) => Formula::And(
            Box::new(rename_formula_atoms(*l)),
            Box::new(rename_formula_atoms(*r)),
        ),
        Formula::Or(l, r) => Formula::Or(
            Box::new(rename_formula_atoms(*l)),
            Box::new(rename_formula_atoms(*r)),
        ),
        _ => unreachable!("build_bp_restriction only ever produces Atom/Not/And/Or"),
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
        PredCtx::Antonym(inner) => build_binary_restriction(&antonym_rename(inner), t1, t2, vg),
        PredCtx::Coord { op, left, right } => {
            let f1 = build_binary_restriction(left, t1.clone(), t2.clone(), vg);
            let f2 = build_binary_restriction(right, t1, t2, vg);
            match op {
                OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                OperatorKind::Xor => xor_formula(f1, f2),
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
    fn test_接続詞_hesuteto_結果から原因() {
        check(
            "hesuteto hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henosuteto_結果から反対の原因() {
        check(
            "henosuteto hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hesuteto_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "hesuteto hesi pahete sa ba si ri behoranage hesa pahete sa mi si ri behapare",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henosuteto_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "henosuteto hesi pahete sa ba si ri behoranage hesa pahete sa mi si ri behapare",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehoma_and() {
        check(
            "hehoma hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomi_or() {
        check(
            "hehomi hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomu_xor() {
        check(
            "hehomu hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "((∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))) ∧ ¬(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))))",
        );
    }

    #[test]
    fn test_接続詞_hehitatu_時間前() {
        check(
            "hehitatu hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehiti_同時() {
        check(
            "hehiti hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehitatu_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "hehitatu hesi pahete sa ba si ri behoranage hesa pahete sa mi si ri behapare",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehito_条件() {
        check(
            "hehito hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ⇒ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henohito_反実仮想条件() {
        check(
            "henohito hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ⇒ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehenu_対比() {
        check(
            "hehenu hesa pahete sa mi si ri behapare hesi pahete sa ba si ri behoranage",
            "(¬∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homa_and() {
        check(
            "homa pahete sa mi si ri behapare pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homi_or() {
        check(
            "homi pahete sa mi si ri behapare pahete sa ba si ri behoranage",
            "(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homu_xor() {
        check(
            "homu pahete sa mi si ri behapare pahete sa ba si ri behoranage",
            "((∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))) ∧ ¬(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃y.(behoranage(y) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, y))))))",
        );
    }

    #[test]
    fn test_演算子_述語_hobu_xor() {
        check(
            "hobu pahete pahete sa mi si ri behapare",
            "((∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∨ ∃x.(behapare(x) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))) ∧ ¬(∃x.(behapare(x) ∧ ∃e.(pahete(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃x.(behapare(x) ∧ ∃e1.(pahete(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))))",
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
    fn test_述語反意化_no_neとは異なる派生atomを作る() {
        // `no`はNotで全体を包まず、独立した新しい原子述語"pahete@anti"を作る。
        check(
            "no pahete sa mi si ri behapare",
            "∃x.(behapare(x) ∧ ∃e.(pahete@anti(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_述語反意化_noとneは異なる式になる() {
        let no_form = to_fol_string("no pahete sa mi si ri behapare").unwrap();
        let ne_form = to_fol_string("ne pahete sa mi si ri behapare").unwrap();
        assert_ne!(no_form, ne_form);
        assert!(!no_form.contains('¬'));
    }

    #[test]
    fn test_述語反意化_hoba配下にネストしても両方の葉に付く() {
        check(
            "no hoba pahete pahete sa mi si ri behapare",
            "(∃x.(behapare(x) ∧ ∃e.(pahete@anti(e) ∧ (Agent(e, mi) ∧ Patient(e, x)))) ∧ ∃x.(behapare(x) ∧ ∃e1.(pahete@anti(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))))",
        );
    }

    #[test]
    fn test_転置_りんごは食べられない() {
        check(
            "hapu ta pahete si ri behapare",
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
            "pahete sa mi si tupe ri behapare",
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
    fn test_固有名詞は開始マーカーで名前空間分けされた定項になる() {
        check(
            "parobe sa heba Kanye hoho si ba",
            "∃e.(parobe(e) ∧ (Agent(e, heba:Kanye) ∧ Patient(e, ba)))",
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
