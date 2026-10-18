//! 型チェック済みASTをネオ・デイヴィドソン流の一階述語論理式(`Formula`)へ変換する。

use std::collections::HashMap;

use crate::parser::{
    AEvent, ArgumentPhrase, ArgumentRetrievalBinaryPredicate, ArgumentRetrievalWithArgumentPhrase,
    AspectExpression, Ast, BinaryPredicatePhrase, DeclarativeSentence, DeterminedArgument,
    DeterminerModifier, Event, InterrogativeSentence, ModalityExpression, ModifiedAspect,
    ModifiedModality, ModifiedTense, MtaEvent, NumericalExpression,
    PostBinaryPredicatePhraseExpression, PostModifiedBinaryPredicatePhrase, PostModifiedMtaEvent,
    QuantifiedArgument, QuantifierModifier, RetrievedArgument, RoleArg, S, TaEvent,
    TenseExpression, UniversalSentence,
};
use crate::tokenizer::{
    AspectKind, CaseLabelKind, ConnectiveHeadKind, ModalityBaseKind, NegationKind, OperatorKind,
    QuantifierKind, TenseKind,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A first-order term: a bound variable, an individual constant, or a function application
/// (used for time terms such as `τ(e)` and `max(tR)`).
#[derive(Debug, Clone)]
pub enum Term {
    Var(String),
    Const(String),
    Func(String, Vec<Term>),
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
    /// Generalized quantifier (te 多くの / ti 半分の / number 数量 / 習慣相)
    GenQ(GenQKind, String, Box<Formula>),
    /// □_b φ / ◇_b φ
    Modal(ModalForce, ModalBase, Box<Formula>),
    /// ?x. φ — 疑問詞`ko`が尋ねる項xを束縛する(疑問文の文頭でのみ現れる)
    Question(String, Box<Formula>),
}

/// Kinds of generalized quantifiers that cannot be expressed with ∀/∃.
#[derive(Debug, Clone)]
pub enum GenQKind {
    Many,     // te  多くの
    Half,     // ti  半分の
    Count,    // number  数量
    Habitual, // hudidi  習慣相(参照時の中の多くの機会)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalForce {
    Necessity,   // ta  必然 □
    Possibility, // to  可能 ◇
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModalBase {
    Epistemic,      // hapa  認識
    Deontic,        // hapi  義務
    Circumstantial, // hapu  能力・状況
}

impl ModalBase {
    pub fn label(&self) -> &'static str {
        match self {
            ModalBase::Epistemic => "epi",
            ModalBase::Deontic => "deon",
            ModalBase::Circumstantial => "cir",
        }
    }
}

/// 中置表示する時間関係の述語名。
const LT: &str = "<";
const LE: &str = "≤";
const OVERLAP: &str = "*";
const SUBINTERVAL: &str = "⊆";
const EQ: &str = "=";

fn is_infix(pred: &str) -> bool {
    matches!(pred, LT | LE | OVERLAP | SUBINTERVAL | EQ)
}

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(v) | Term::Const(v) => write!(f, "{v}"),
            Term::Func(name, args) => {
                let s = args
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{name}({s})")
            }
        }
    }
}

impl std::fmt::Display for GenQKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenQKind::Many => write!(f, "∃_many"),
            GenQKind::Half => write!(f, "∃_half"),
            GenQKind::Count => write!(f, "∃_#"),
            GenQKind::Habitual => write!(f, "Hab"),
        }
    }
}

impl std::fmt::Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Atom(p, args) if is_infix(p) && args.len() == 2 => {
                write!(f, "{} {p} {}", args[0], args[1])
            }
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
            Formula::Not(phi) => match phi.as_ref() {
                Formula::Atom(p, _) if is_infix(p) => write!(f, "¬({phi})"),
                _ => write!(f, "¬{phi}"),
            },
            Formula::And(l, r) => write!(f, "({l} ∧ {r})"),
            Formula::Or(l, r) => write!(f, "({l} ∨ {r})"),
            Formula::Implies(l, r) => write!(f, "({l} ⇒ {r})"),
            Formula::Forall(x, phi) => write!(f, "∀{x}.{phi}"),
            Formula::Exists(x, phi) => write!(f, "∃{x}.{phi}"),
            Formula::GenQ(k, x, phi) => write!(f, "{k} {x}.{phi}"),
            Formula::Question(x, phi) => write!(f, "?{x}.{phi}"),
            Formula::Modal(force, base, phi) => {
                let op = match force {
                    ModalForce::Necessity => "□",
                    ModalForce::Possibility => "◇",
                };
                write!(f, "{op}_{} {phi}", base.label())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Variable generator
// ---------------------------------------------------------------------------

struct VarGen {
    obj: usize,
    evt: usize,
    ref_time: usize,
    occasion: usize,
    /// 疑問詞`ko`に割り当てた変数。`InterrogativeSentence::to_fol`が文頭で`?x`として束縛する。
    question_vars: Vec<String>,
}

impl VarGen {
    fn new() -> Self {
        VarGen {
            obj: 0,
            evt: 0,
            ref_time: 0,
            occasion: 0,
            question_vars: Vec::new(),
        }
    }

    fn next_ref_time(&mut self) -> String {
        let n = self.ref_time;
        self.ref_time += 1;
        if n == 0 {
            "tR".to_string()
        } else {
            format!("tR{n}")
        }
    }

    fn next_occasion(&mut self) -> String {
        let n = self.occasion;
        self.occasion += 1;
        if n == 0 {
            "s".to_string()
        } else {
            format!("s{n}")
        }
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
}

#[derive(Debug, Clone, PartialEq)]
enum QuantKind {
    Universal,   // ta  全ての
    Existential, // epsilon / to  少なくとも1つは存在する
    Many,        // te  多くの
    Half,        // ti  半分の
    Count(u64),  // number
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
                QuantKind::Count(0) => Formula::Not(Box::new(Formula::Exists(
                    var,
                    Box::new(Formula::And(Box::new(restriction), Box::new(body))),
                ))),
                QuantKind::Count(_) => Formula::GenQ(
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
        other => other, // Const / ExistOnly / FreeVar: cannot negate meaningfully
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
// AST traversal
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
            InterrogativeSentence::One(_, pm) => {
                let body = pm.to_fol(vg);
                std::mem::take(&mut vg.question_vars)
                    .into_iter()
                    .rev()
                    .fold(body, |acc, var| Formula::Question(var, Box::new(acc)))
            }
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
                collect_free_term_variables(term, bound, free);
            }
        }
        Formula::Not(phi) | Formula::Modal(_, _, phi) => collect_free_variables(phi, bound, free),
        Formula::And(l, r) | Formula::Or(l, r) | Formula::Implies(l, r) => {
            collect_free_variables(l, bound, free);
            collect_free_variables(r, bound, free);
        }
        Formula::Forall(x, phi)
        | Formula::Exists(x, phi)
        | Formula::GenQ(_, x, phi)
        | Formula::Question(x, phi) => {
            let newly_bound = bound.insert(x.clone());
            collect_free_variables(phi, bound, free);
            if newly_bound {
                bound.remove(x);
            }
        }
    }
}

fn collect_free_term_variables(
    term: &Term,
    bound: &std::collections::HashSet<String>,
    free: &mut Vec<String>,
) {
    match term {
        Term::Var(name) => {
            if !bound.contains(name) && !free.contains(name) {
                free.push(name.clone());
            }
        }
        Term::Const(_) => {}
        Term::Func(_, args) => {
            for arg in args {
                collect_free_term_variables(arg, bound, free);
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
            PostModifiedMtaEvent::One(mta) => mta.to_fol(Gap::None, vg),
            PostModifiedMtaEvent::Two(head, arg1, arg2) => {
                let (p_arg, q_arg) = resolve_role_args(arg1, arg2);
                let p = p_arg.get_mta_event().to_fol(Gap::None, vg);
                let q = q_arg.get_mta_event().to_fol(Gap::None, vg);
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

/// 節からの項抽出の空所。変数は外側で量化される。
#[derive(Clone, Copy)]
enum Gap<'a> {
    None,
    /// `ra`/`ri`/`ru K`: その格の項に変数が入る。
    Case(&'a CaseLabelKind, &'a str),
    /// `re`: 節内のイベント変数そのものが変数になる。
    Event(&'a str),
}

impl MtaEvent {
    /// `gap`は節からの項抽出(`ri he … ho`など)のとき、節内の全Eventで共有される空所。
    fn to_fol(&self, gap: Gap<'_>, vg: &mut VarGen) -> Formula {
        match self {
            MtaEvent::One(mod_expr, ta) => {
                let body = ta.to_fol(gap, vg);
                match mod_expr {
                    ModalityExpression::Epsilon => body,
                    ModalityExpression::One(ModifiedModality::One(base, force)) => {
                        modal(base.get_kind(), force.get_kind(), body)
                    }
                    ModalityExpression::One(ModifiedModality::Two(_, base, force)) => {
                        Formula::Not(Box::new(modal(base.get_kind(), force.get_kind(), body)))
                    }
                }
            }
            // homa/homi/homu: proposition-level and/or/xor.
            MtaEvent::Two(op, m1, m2) => {
                let f1 = m1.to_fol(gap, vg);
                let f2 = m2.to_fol(gap, vg);
                match op.get_kind() {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
                }
            }
        }
    }
}

/// 量化力`ta`は必然(□)、それ以外は可能(◇)。`te`/`ti`は型検査でエラーになるので、
/// ここではパニックせず`to`扱いにフォールバックする。
fn modal(base: &ModalityBaseKind, force: &QuantifierKind, body: Formula) -> Formula {
    let base = match base {
        ModalityBaseKind::Hapa => ModalBase::Epistemic,
        ModalityBaseKind::Hapi => ModalBase::Deontic,
        ModalityBaseKind::Hapu => ModalBase::Circumstantial,
    };
    let force = match force {
        QuantifierKind::Ta => ModalForce::Necessity,
        _ => ModalForce::Possibility,
    };
    Formula::Modal(force, base, Box::new(body))
}

/// `(テンス, 否定されているか)`
type TenseSpec<'a> = Option<(&'a TenseKind, bool)>;

fn tense_spec(expr: &TenseExpression) -> TenseSpec<'_> {
    match expr {
        TenseExpression::One(ModifiedTense::One(t)) => Some((t.get_kind(), false)),
        TenseExpression::One(ModifiedTense::Two(_, t)) => Some((t.get_kind(), true)),
        TenseExpression::Epsilon => None,
    }
}

fn aspect_spec(expr: &AspectExpression) -> Option<(&AspectKind, bool)> {
    match expr {
        AspectExpression::One(ModifiedAspect::One(a)) => Some((a.get_kind(), false)),
        AspectExpression::One(ModifiedAspect::Two(_, a)) => Some((a.get_kind(), true)),
        AspectExpression::Epsilon => None,
    }
}

impl TaEvent {
    fn to_fol(&self, gap: Gap<'_>, vg: &mut VarGen) -> Formula {
        match self {
            TaEvent::One(tense_expr, a) => a.to_fol(tense_spec(tense_expr), gap, vg),
        }
    }
}

impl AEvent {
    /// アスペクトが無ければテンスはイベント時`τ(e)`と発話時`t0`を直接関係づける。
    /// アスペクトがあれば参照時`tR`を導入し、テンスは`tR`と`t0`を、アスペクトは`tR`とイベントを
    /// 関係づける。`hine`はテンスの関係だけを否定し、`hune`は参照時におけるアスペクト命題全体を否定する。
    fn to_fol(&self, tense: TenseSpec<'_>, gap: Gap<'_>, vg: &mut VarGen) -> Formula {
        match self {
            AEvent::One(asp_expr, event) => match aspect_spec(asp_expr) {
                None => {
                    let frame = match tense {
                        Some((kind, negated)) => EventFrame::Single(TimeCond::Tense {
                            kind: kind.clone(),
                            negated,
                        }),
                        None => EventFrame::Plain,
                    };
                    event.to_fol(&frame, gap, vg)
                }
                Some((aspect, aspect_negated)) => {
                    let r = vg.next_ref_time();
                    let mut body = aspect_formula(aspect, &r, event, gap, vg);
                    if aspect_negated {
                        body = Formula::Not(Box::new(body));
                    }
                    if let Some((kind, negated)) = tense {
                        let cond = negate_if(ref_tense_cond(kind, &r), negated);
                        body = Formula::And(Box::new(cond), Box::new(body));
                    }
                    Formula::Exists(r, Box::new(body))
                }
            },
        }
    }
}

fn aspect_formula(
    aspect: &AspectKind,
    r: &str,
    event: &Event,
    gap: Gap<'_>,
    vg: &mut VarGen,
) -> Formula {
    let overlap = |point| {
        EventFrame::Single(TimeCond::Overlap {
            r: r.to_string(),
            point,
        })
    };
    match aspect {
        AspectKind::Huda => Formula::Not(Box::new(event.to_fol(
            &EventFrame::Single(TimeCond::BegunBy(r.to_string())),
            gap,
            vg,
        ))),
        AspectKind::Hude => event.to_fol(&overlap(EventPoint::Begin), gap, vg),
        AspectKind::Hudida => event.to_fol(&overlap(EventPoint::Whole), gap, vg),
        AspectKind::Hudidi => {
            let s = vg.next_occasion();
            let frame = EventFrame::Single(TimeCond::Overlap {
                r: s.clone(),
                point: EventPoint::Whole,
            });
            let occasion = time_rel(SUBINTERVAL, Term::Var(s.clone()), Term::Var(r.to_string()));
            let body = event.to_fol(&frame, gap, vg);
            Formula::GenQ(
                GenQKind::Habitual,
                s,
                Box::new(Formula::And(Box::new(occasion), Box::new(body))),
            )
        }
        AspectKind::Hudidu => event.to_fol(
            &EventFrame::Iterative(TimeCond::Overlap {
                r: r.to_string(),
                point: EventPoint::Whole,
            }),
            gap,
            vg,
        ),
        AspectKind::Hudo => event.to_fol(&overlap(EventPoint::End), gap, vg),
        AspectKind::Hudu => event.to_fol(
            &EventFrame::Single(TimeCond::Perfect(r.to_string())),
            gap,
            vg,
        ),
    }
}

// ---------------------------------------------------------------------------
// Time
// ---------------------------------------------------------------------------

/// 発話時。
const SPEECH_TIME: &str = "t0";

fn t0() -> Term {
    Term::Const(SPEECH_TIME.to_string())
}

fn time_fn(name: &str, arg: Term) -> Term {
    Term::Func(name.to_string(), vec![arg])
}

fn time_rel(pred: &str, a: Term, b: Term) -> Formula {
    Formula::Atom(pred.to_string(), vec![a, b])
}

fn negate_if(f: Formula, negated: bool) -> Formula {
    if negated {
        Formula::Not(Box::new(f))
    } else {
        f
    }
}

/// アスペクトがあるときの、参照時`tR`と発話時の関係。
fn ref_tense_cond(kind: &TenseKind, r: &str) -> Formula {
    let r = Term::Var(r.to_string());
    match kind {
        TenseKind::Hita => time_rel(LT, time_fn("max", r), t0()),
        TenseKind::Hiti => time_rel(OVERLAP, r, t0()),
        TenseKind::Hitu => time_rel(LT, t0(), time_fn("min", r)),
    }
}

#[derive(Debug, Clone)]
enum EventPoint {
    Begin,
    Whole,
    End,
}

impl EventPoint {
    fn of(&self, e: &Term) -> Term {
        let name = match self {
            EventPoint::Begin => "τ_begin",
            EventPoint::Whole => "τ",
            EventPoint::End => "τ_end",
        };
        time_fn(name, e.clone())
    }
}

/// イベント変数に課す時間の条件。
#[derive(Debug, Clone)]
enum TimeCond {
    /// アスペクト無しのテンス: `τ(e)`と`t0`の関係(`negated`なら関係の否定)
    Tense { kind: TenseKind, negated: bool },
    /// `tR * τ_point(e)`
    Overlap { r: String, point: EventPoint },
    /// 未然の否定対象: `τ_begin(e) ≤ max(tR)`
    BegunBy(String),
    /// 事後: `τ_end(e) ≤ min(tR) ∧ ResultState(e, tR)`
    Perfect(String),
}

impl TimeCond {
    fn apply(&self, e: &Term) -> Formula {
        match self {
            TimeCond::Tense { kind, negated } => {
                let te = EventPoint::Whole.of(e);
                let rel = match kind {
                    TenseKind::Hita => time_rel(LT, te, t0()),
                    TenseKind::Hiti => time_rel(OVERLAP, te, t0()),
                    TenseKind::Hitu => time_rel(LT, t0(), te),
                };
                negate_if(rel, *negated)
            }
            TimeCond::Overlap { r, point } => time_rel(OVERLAP, Term::Var(r.clone()), point.of(e)),
            TimeCond::BegunBy(r) => time_rel(
                LE,
                EventPoint::Begin.of(e),
                time_fn("max", Term::Var(r.clone())),
            ),
            TimeCond::Perfect(r) => Formula::And(
                Box::new(time_rel(
                    LE,
                    EventPoint::End.of(e),
                    time_fn("min", Term::Var(r.clone())),
                )),
                Box::new(Formula::Atom(
                    "ResultState".to_string(),
                    vec![e.clone(), Term::Var(r.clone())],
                )),
            ),
        }
    }
}

/// `build_event_inner`の基底ケースでイベント変数をどう導入するか。
#[derive(Debug, Clone)]
enum EventFrame {
    /// `∃e.body(e)`
    Plain,
    /// `∃e.(body(e) ∧ cond(e))`
    Single(TimeCond),
    /// 反復: 項を共有する2つの異なるイベント
    /// `∃e.∃e'.(((body(e) ∧ body(e')) ∧ ¬(e = e')) ∧ (cond(e) ∧ cond(e')))`
    Iterative(TimeCond),
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

fn role_name(kind: &CaseLabelKind) -> &'static str {
    match kind {
        CaseLabelKind::Sa => "Agent",
        CaseLabelKind::Si => "Patient",
        _ => EXTRA_ROLE_TABLE
            .iter()
            .find(|(k, _)| k == kind)
            .map(|(_, name)| *name)
            .unwrap(),
    }
}

impl Event {
    fn to_fol(&self, frame: &EventFrame, gap: Gap<'_>, vg: &mut VarGen) -> Formula {
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
        // 空所の格は節内に書かれていても(型エラー)空所を優先する。
        let arg_of = |kind: &CaseLabelKind, vg: &mut VarGen| match gap {
            Gap::Case(gap_kind, var) if gap_kind == kind => Some(ArgCtx::FreeVar(var.to_string())),
            _ => by_kind.get(kind).map(|&phrase| phrase.to_fol(vg)),
        };
        let arg2 = arg_of(&CaseLabelKind::Si, vg).unwrap_or_else(|| ArgCtx::ExistOnly(vg.next()));
        let arg1 = arg_of(&CaseLabelKind::Sa, vg).unwrap_or_else(|| ArgCtx::ExistOnly(vg.next()));
        // 付加詞は省略時に存在閉包を作らず式に出さない。
        let extra_roles: Vec<(&'static str, ArgCtx)> = EXTRA_ROLE_TABLE
            .into_iter()
            .filter_map(|(kind, role_name)| arg_of(&kind, vg).map(|ctx| (role_name, ctx)))
            .collect();
        let event_var = match gap {
            Gap::Event(var) => Some(var),
            _ => None,
        };
        build_event_inner(&pred, arg1, arg2, extra_roles, frame, event_var, vg)
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
///
/// `event_var`が指定されていれば、それをイベント変数として使い、存在閉包しない(`re`の抽出)。
fn build_event_inner(
    pred: &PredCtx,
    arg1: ArgCtx,
    arg2: ArgCtx,
    extra_roles: Vec<(&'static str, ArgCtx)>,
    frame: &EventFrame,
    event_var: Option<&str>,
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
                frame,
                event_var,
                vg,
            )))
        }

        PredCtx::Antonym(inner) => {
            // no P: derive a fresh, independent atom "P@anti" and compile it exactly
            // like an ordinary predicate (no `Not`, no asserted relationship to P).
            build_event_inner(
                &antonym_rename(inner),
                arg1,
                arg2,
                extra_roles,
                frame,
                event_var,
                vg,
            )
        }

        PredCtx::Coord { op, left, right } => {
            // P di Q / P hoba|hobi|hobu Q: both predicates apply to the same args.
            let f1 = build_event_inner(
                left,
                arg1.clone(),
                arg2.clone(),
                extra_roles.clone(),
                frame,
                event_var,
                vg,
            );
            let f2 = build_event_inner(right, arg1, arg2, extra_roles, frame, event_var, vg);
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
                let f1 = build_event_inner(
                    &p,
                    *left,
                    arg2.clone(),
                    extra_roles.clone(),
                    frame,
                    event_var,
                    vg,
                );
                let f2 = build_event_inner(&p, *right, arg2, extra_roles, frame, event_var, vg);
                return match op {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
                };
            }
            // Distribute over arg2 coordination.
            if let ArgCtx::Coord { op, left, right } = arg2 {
                let p = PredCtx::Simple(name.clone());
                let f1 = build_event_inner(
                    &p,
                    arg1.clone(),
                    *left,
                    extra_roles.clone(),
                    frame,
                    event_var,
                    vg,
                );
                let f2 = build_event_inner(&p, arg1, *right, extra_roles, frame, event_var, vg);
                return match op {
                    OperatorKind::And => Formula::And(Box::new(f1), Box::new(f2)),
                    OperatorKind::Or => Formula::Or(Box::new(f1), Box::new(f2)),
                    OperatorKind::Xor => xor_formula(f1, f2),
                };
            }

            // Base case: ∃e.(pred(e) ∧ Agent(e, t1) ∧ Patient(e, t2) ∧ <extra_roles>)
            let t1 = arg1.term();
            let t2 = arg2.term();
            let event_body = |ev: &Term| {
                let pred_atom = Formula::Atom(name.clone(), vec![ev.clone()]);
                let agent_atom = Formula::Atom("Agent".to_string(), vec![ev.clone(), t1.clone()]);
                let patient_atom =
                    Formula::Atom("Patient".to_string(), vec![ev.clone(), t2.clone()]);
                let mut body = Formula::And(
                    Box::new(pred_atom),
                    Box::new(Formula::And(Box::new(agent_atom), Box::new(patient_atom))),
                );
                for (role_name, ctx) in &extra_roles {
                    let role_atom =
                        Formula::Atom(role_name.to_string(), vec![ev.clone(), ctx.term()]);
                    body = Formula::And(Box::new(body), Box::new(role_atom));
                }
                body
            };
            let e = event_var.map_or_else(|| vg.next_event(), str::to_string);
            let ev = Term::Var(e.clone());
            let event_formula = match frame {
                EventFrame::Plain => Formula::Exists(e, Box::new(event_body(&ev))),
                EventFrame::Single(cond) => Formula::Exists(
                    e,
                    Box::new(Formula::And(
                        Box::new(event_body(&ev)),
                        Box::new(cond.apply(&ev)),
                    )),
                ),
                EventFrame::Iterative(cond) => {
                    let e2 = vg.next_event();
                    let ev2 = Term::Var(e2.clone());
                    let bodies =
                        Formula::And(Box::new(event_body(&ev)), Box::new(event_body(&ev2)));
                    let distinct = Formula::Not(Box::new(time_rel(EQ, ev.clone(), ev2.clone())));
                    let conds = Formula::And(Box::new(cond.apply(&ev)), Box::new(cond.apply(&ev2)));
                    Formula::Exists(
                        e,
                        Box::new(Formula::Exists(
                            e2,
                            Box::new(Formula::And(
                                Box::new(Formula::And(Box::new(bodies), Box::new(distinct))),
                                Box::new(conds),
                            )),
                        )),
                    )
                }
            };
            let event_formula = match (event_var, event_formula) {
                (Some(_), Formula::Exists(_, body)) => *body,
                (_, f) => f,
            };
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
            // ga(誰か) → fresh existential variable, not an individual constant.
            ArgumentPhrase::One(arg) if arg.is_indefinite() => ArgCtx::ExistOnly(vg.next()),
            // ko(疑問詞) → fresh variable, closed by `?x` in `InterrogativeSentence::to_fol`.
            ArgumentPhrase::One(arg) if arg.is_interrogative() => {
                let var = vg.next();
                vg.question_vars.push(var.clone());
                ArgCtx::FreeVar(var)
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

            // Proper noun: he<X> BODY hoho → individual constant, namespaced by the
            // opener so a quoted name can never collide with a pronoun constant or
            // with the same raw text quoted under a different he<X> class.
            ArgumentPhrase::Five(opener, body, _hoho) => {
                ArgCtx::Const(format!("{}:{}", opener.get_string(), body.get_string()))
            }
            ArgumentPhrase::Six(proper_noun) => ArgCtx::Const(proper_noun.get_string()),
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
        QuantifierModifier::Two(NumericalExpression::One(number)) => {
            QuantKind::Count(number.get_value())
        }
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
        // `None`は`re`(事象そのものの抽出)。
        let (gap_kind, body) = match self {
            RetrievedArgument::One(retriever, body) => (retriever.gap_case(), body),
            RetrievedArgument::Two(_ru, label, body) => (Some(label.get_kind().clone()), body),
        };
        let x = Term::Var(var.to_string());
        match body {
            // 節からの抽出: 節内のEventの空所の格(`re`ならイベント変数)に`var`を入れる。
            ArgumentRetrievalBinaryPredicate::Two(_he, mta_event, _ho) => {
                let gap = match &gap_kind {
                    Some(kind) => Gap::Case(kind, var),
                    None => Gap::Event(var),
                };
                mta_event.to_fol(gap, vg)
            }
            ArgumentRetrievalBinaryPredicate::One(pred, with_arg) => {
                let base = build_role_restriction(
                    &extract_pred(pred),
                    gap_kind.as_ref().map(role_name),
                    x.clone(),
                    vg,
                );
                match with_arg {
                    ArgumentRetrievalWithArgumentPhrase::Epsilon => base,
                    // du: 関係を指定しない「〜の」。
                    ArgumentRetrievalWithArgumentPhrase::One(_du, extra_arg) => {
                        let extra_ctx = extra_arg.to_fol(vg);
                        let of_atom = Formula::Atom("Of".to_string(), vec![x, extra_ctx.term()]);
                        Formula::And(Box::new(base), Box::new(extra_ctx.scope(of_atom)))
                    }
                }
            }
        }
    }
}

/// `ra`/`ri`/`ru K`で抽出した項`x`を制限する式 `∃e.(P(e) ∧ Role(e, x))`。
/// `role`が`None`(`re`)なら`x`自体がイベントで、式は`P(x)`になる。
fn build_role_restriction(pred: &PredCtx, role: Option<&str>, x: Term, vg: &mut VarGen) -> Formula {
    match pred {
        PredCtx::Simple(name) => {
            let Some(role) = role else {
                return Formula::Atom(name.clone(), vec![x]);
            };
            let e = vg.next_event();
            let ev = Term::Var(e.clone());
            Formula::Exists(
                e,
                Box::new(Formula::And(
                    Box::new(Formula::Atom(name.clone(), vec![ev.clone()])),
                    Box::new(Formula::Atom(role.to_string(), vec![ev, x])),
                )),
            )
        }
        PredCtx::Neg(inner) => Formula::Not(Box::new(build_role_restriction(inner, role, x, vg))),
        PredCtx::Antonym(inner) => build_role_restriction(&antonym_rename(inner), role, x, vg),
        PredCtx::Coord { op, left, right } => {
            let f1 = build_role_restriction(left, role, x.clone(), vg);
            let f2 = build_role_restriction(right, role, x, vg);
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
        let asts = diagnose(text.to_string())?.asts;
        Ok(to_fol(&asts[0]).to_string())
    }

    fn check(text: &str, expected: &str) {
        let fol = to_fol_string(text).unwrap();
        assert_eq!(fol, expected, "input: {text}");
    }

    #[test]
    fn test_随伴格_私はあなたとりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare subu ba",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ Comitative(e1, ba)))",
        );
    }

    #[test]
    fn test_奪格与格_私はあなたにりんごをあげる() {
        check(
            "pemobe sa mi si ri nohapare sube mi subo ba",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(((pemobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ AnimateSource(e1, mi)) ∧ AnimateGoal(e1, ba)))",
        );
    }

    #[test]
    fn test_原因格目的格_私はりんごを理由があって目的のために食べる() {
        check(
            "peheta sa mi si ri nohapare sute ri kahoraka suto ri kahoraka",
            "∃y.(∃e1.(kahoraka(e1) ∧ Patient(e1, y)) ∧ ∃z.(∃e2.(kahoraka(e2) ∧ Patient(e2, z)) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e3.(((peheta(e3) ∧ (Agent(e3, mi) ∧ Patient(e3, x))) ∧ Cause(e3, y)) ∧ Purpose(e3, z)))))",
        );
    }

    #[test]
    fn test_具格_私は道具を使ってりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare sudu ri didora",
            "∃y.(∃e1.(didora(e1) ∧ Patient(e1, y)) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.((peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))) ∧ Instrument(e2, y))))",
        );
    }

    #[test]
    fn test_空間系_私はある空間からある空間へある空間でりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare suni me sune be suno ge",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ Location(e1, me)) ∧ LocationSource(e1, be)) ∧ LocationGoal(e1, ge)))",
        );
    }

    #[test]
    fn test_時間系_私はある時からある時までりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare sugi me suge be sugo ge",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ Time(e1, me)) ∧ TimeSource(e1, be)) ∧ TimeGoal(e1, ge)))",
        );
    }

    #[test]
    fn test_存在量化_私はりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_接続詞_hesuteto_結果から原因() {
        check(
            "hesuteto hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henosuteto_結果から反対の原因() {
        check(
            "henosuteto hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hesuteto_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "hesuteto hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henosuteto_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "henosuteto hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehoma_and() {
        check(
            "hehoma hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomi_or() {
        check(
            "hehomi hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∨ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehomu_xor() {
        check(
            "hehomu hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "((∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∨ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y))))) ∧ ¬(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y))))))",
        );
    }

    #[test]
    fn test_接続詞_hehitatu_時間前() {
        check(
            "hehitatu hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehiti_同時() {
        check(
            "hehiti hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehitatu_役割の順序を入れ替えても同じ論理式になる() {
        check(
            "hehitatu hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehito_条件() {
        check(
            "hehito hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ⇒ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_henohito_反実仮想条件() {
        check(
            "henohito hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ⇒ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_接続詞_hehenu_対比() {
        check(
            "hehenu hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage",
            "(¬∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homa_and() {
        check(
            "homa peheta sa mi si ri nohapare peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homi_or() {
        check(
            "homi peheta sa mi si ri nohapare peheta sa ba si ri nohoranage",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∨ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y)))))",
        );
    }

    #[test]
    fn test_演算子_命題_homu_xor() {
        check(
            "homu peheta sa mi si ri nohapare peheta sa ba si ri nohoranage",
            "((∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∨ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y))))) ∧ ¬(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃y.(∃e2.(nohoranage(e2) ∧ Patient(e2, y)) ∧ ∃e3.(peheta(e3) ∧ (Agent(e3, ba) ∧ Patient(e3, y))))))",
        );
    }

    #[test]
    fn test_演算子_述語_hobu_xor() {
        check(
            "hobu peheta peheta sa mi si ri nohapare",
            "((∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∨ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.(peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))) ∧ ¬(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.(peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))))",
        );
    }

    #[test]
    fn test_全称量化_私は全てのりんごを食べる() {
        check(
            "peheta sa mi si ta ri nohapare",
            "∀x.(∃e.(nohapare(e) ∧ Patient(e, x)) ⇒ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_述語否定_私はりんごを食べない() {
        check(
            "ne peheta sa mi si ri nohapare",
            "¬∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_述語反意化_no_neとは異なる派生atomを作る() {
        // `no`はNotで全体を包まず、独立した新しい原子述語"peheta@anti"を作る。
        check(
            "no peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta@anti(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_述語反意化_noとneは異なる式になる() {
        let no_form = to_fol_string("no peheta sa mi si ri nohapare").unwrap();
        let ne_form = to_fol_string("ne peheta sa mi si ri nohapare").unwrap();
        assert_ne!(no_form, ne_form);
        assert!(!no_form.contains('¬'));
    }

    #[test]
    fn test_述語反意化_hoba配下にネストしても両方の葉に付く() {
        check(
            "no hoba peheta peheta sa mi si ri nohapare",
            "(∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta@anti(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x)))) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.(peheta@anti(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x)))))",
        );
    }

    #[test]
    fn test_転置_りんごは食べられない() {
        check(
            "hapu ta peheta si ri nohapare",
            "□_cir ∃y.∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, y) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_転置と否定_みかんは私に食べられていない() {
        check(
            "ne peheta sa mi si ri nohoranage",
            "¬∃x.(∃e.(nohoranage(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_指示詞_このりんご() {
        check(
            "peheta sa mi si mo ri nohapare",
            "∃x.((mo(x) ∧ ∃e.(nohapare(e) ∧ Patient(e, x))) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_数量化_数のりんご() {
        check(
            "peheta sa mi si tusa ri nohapare",
            "∃_# x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_多量化_多くのりんご() {
        check(
            "peheta sa mi si te ri nohapare",
            "∃_many x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
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
    fn test_tで始まる固有名詞はその語の定項になる() {
        check(
            "parobe sa taro si ba",
            "∃e.(parobe(e) ∧ (Agent(e, taro) ∧ Patient(e, ba)))",
        );
    }

    #[test]
    fn test_du_私はあなたのパートナーを愛している() {
        check(
            "parobe sa mi si ri paparatanera du ba",
            "∃x.((∃e.(paparatanera(e) ∧ Patient(e, x)) ∧ Of(x, ba)) ∧ ∃e1.(parobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_du_量化された項の制限部が残る_床の上() {
        check(
            "peheta sa mi si ri nohapare suni ri zahona du ri duharora",
            "∃y.((∃e1.(zahona(e1) ∧ Patient(e1, y)) ∧ ∃z.(∃e2.(duharora(e2) ∧ Patient(e2, z)) ∧ Of(y, z))) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e3.((peheta(e3) ∧ (Agent(e3, mi) ∧ Patient(e3, x))) ∧ Location(e3, y))))",
        );
    }

    #[test]
    fn test_ru_私は食べる場所でりんごを食べる() {
        check(
            "peheta sa mi si ri nohapare suni ru suni peheta",
            "∃y.(∃e1.(peheta(e1) ∧ Location(e1, y)) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.((peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))) ∧ Location(e2, y))))",
        );
    }

    #[test]
    fn test_ru_duを付けると関係を添える() {
        check(
            "peheta sa mi si ri nohapare sudu ru sudu peheta du mi",
            "∃y.((∃e1.(peheta(e1) ∧ Instrument(e1, y)) ∧ Of(y, mi)) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.((peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))) ∧ Instrument(e2, y))))",
        );
    }

    #[test]
    fn test_節からの抽出_ri_私が食べる物をあなたは愛する() {
        check(
            "perobe sa ba si ri he peheta sa mi ho",
            "∃x.(∃e.(peheta(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ ∃e1.(perobe(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_節からの抽出_ru_私がりんごを食べた場所で食べる() {
        check(
            "peheta sa ba suni ru suni he hita peheta sa mi si ri nohapare ho",
            "∃y.(∃z.(∃e.(nohapare(e) ∧ Patient(e, z)) ∧ ∃e1.(((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, z))) ∧ Location(e1, y)) ∧ τ(e1) < t0)) ∧ ∃x.∃e2.((peheta(e2) ∧ (Agent(e2, ba) ∧ Patient(e2, x))) ∧ Location(e2, y)))",
        );
    }

    #[test]
    fn test_節からの抽出_ra_りんごを食べている者を愛する() {
        check(
            "parobe sa mi si ra he hudida peheta si ri nohapare ho",
            "∃x.(∃tR.∃y.(∃e.(nohapare(e) ∧ Patient(e, y)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, x) ∧ Patient(e1, y))) ∧ tR * τ(e1))) ∧ ∃e2.(parobe(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))",
        );
    }

    #[test]
    fn test_節からの抽出_等位接続の両方のeventに空所が入る() {
        check(
            "perobe sa mi si ri he homa peheta sa mi perobe sa ba ho",
            "∃x.((∃e.(peheta(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ ∃e1.(perobe(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, x)))) ∧ ∃e2.(perobe(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))",
        );
    }

    #[test]
    fn test_節からの抽出_入れ子の名詞化には空所が伝わらない() {
        check(
            "parobe sa mi si ra he karobe si re he peheta si ri nohapare ho ho",
            "∃x.(∃y.(∃x1.∃z.(∃e.(nohapare(e) ∧ Patient(e, z)) ∧ (peheta(y) ∧ (Agent(y, x1) ∧ Patient(y, z)))) ∧ ∃e1.(karobe(e1) ∧ (Agent(e1, x) ∧ Patient(e1, y)))) ∧ ∃e2.(parobe(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))",
        );
    }

    #[test]
    fn test_ru_否定と等位接続は述語ごとに分配される() {
        check(
            "peheta sa mi si ri nohapare suni ru suni hoba peheta ne perobe",
            "∃y.((∃e1.(peheta(e1) ∧ Location(e1, y)) ∧ ¬∃e2.(perobe(e2) ∧ Location(e2, y))) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e3.((peheta(e3) ∧ (Agent(e3, mi) ∧ Patient(e3, x))) ∧ Location(e3, y))))",
        );
    }

    #[test]
    fn test_テンス_過去() {
        check(
            "hita peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ τ(e1) < t0))",
        );
    }

    #[test]
    fn test_テンス_現在() {
        check(
            "hiti peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ τ(e1) * t0))",
        );
    }

    #[test]
    fn test_テンス_未来() {
        check(
            "hitu peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ t0 < τ(e1)))",
        );
    }

    #[test]
    fn test_テンス否定_hineは時間関係だけを否定する() {
        check(
            "hine hita peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ ¬(τ(e1) < t0)))",
        );
    }

    #[test]
    fn test_アスペクト_未然() {
        check(
            "hita huda peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ¬∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ τ_begin(e1) ≤ max(tR))))",
        );
    }

    #[test]
    fn test_アスペクト_開始() {
        check(
            "hita hude peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ tR * τ_begin(e1))))",
        );
    }

    #[test]
    fn test_アスペクト_進行() {
        check(
            "hita hudida peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ tR * τ(e1))))",
        );
    }

    #[test]
    fn test_アスペクト_習慣() {
        check(
            "hita hudidi peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ Hab s.(s ⊆ tR ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ s * τ(e1)))))",
        );
    }

    #[test]
    fn test_アスペクト_反復() {
        check(
            "hita hudidu peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.∃e2.((((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ (peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x)))) ∧ ¬(e1 = e2)) ∧ (tR * τ(e1) ∧ tR * τ(e2)))))",
        );
    }

    #[test]
    fn test_アスペクト_終了() {
        check(
            "hita hudo peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ tR * τ_end(e1))))",
        );
    }

    #[test]
    fn test_アスペクト_事後() {
        check(
            "hita hudu peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ (τ_end(e1) ≤ min(tR) ∧ ResultState(e1, tR)))))",
        );
    }

    #[test]
    fn test_アスペクト否定_huneは参照時のアスペクト命題を否定する() {
        check(
            "hune hudida peheta sa mi si ri nohapare",
            "∃tR.¬∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ tR * τ(e1)))",
        );
    }

    #[test]
    fn test_アスペクト_述語の等位接続は参照時を共有する() {
        check(
            "hita hudida hoba peheta peheta sa mi si ri nohapare",
            "∃tR.(max(tR) < t0 ∧ (∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ tR * τ(e1))) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e2.((peheta(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))) ∧ tR * τ(e2)))))",
        );
    }

    #[test]
    fn test_モダリティ_認識的必然_未来事後() {
        check(
            "hapa ta hitu hudu peheta sa mi si ri nohapare",
            "□_epi ∃tR.(t0 < min(tR) ∧ ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.((peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))) ∧ (τ_end(e1) ≤ min(tR) ∧ ResultState(e1, tR)))))",
        );
    }

    #[test]
    fn test_モダリティ_能力可能() {
        check(
            "hapu to peheta sa mi si ri nohapare",
            "◇_cir ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_モダリティ否定_haneは様相演算子を否定する() {
        check(
            "hane hapi to peheta sa mi si ri nohapare",
            "¬◇_deon ∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_項抽出_raは動作主を抽出する_食べる者を愛する() {
        check(
            "parobe sa mi si ra peheta",
            "∃x.(∃e.(peheta(e) ∧ Agent(e, x)) ∧ ∃e1.(parobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_項抽出_riは被動者を抽出する_食べ物を愛する() {
        check(
            "perobe sa mi si ri peheta",
            "∃x.(∃e.(peheta(e) ∧ Patient(e, x)) ∧ ∃e1.(perobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_項抽出_diで修飾した述語も制限に含まれる_雌の犬() {
        check(
            "parobe sa mi si ri bidoga di bihemare",
            "∃x.((∃e.(bidoga(e) ∧ Patient(e, x)) ∧ ∃e1.(bihemare(e1) ∧ Patient(e1, x))) ∧ ∃e2.(parobe(e2) ∧ (Agent(e2, mi) ∧ Patient(e2, x))))",
        );
    }

    #[test]
    fn test_項抽出_名詞述語のraは定義する者を抽出する() {
        check(
            "parobe sa mi si ra nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Agent(e, x)) ∧ ∃e1.(parobe(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_不定代名詞_誰かがりんごを食べる() {
        check(
            "peheta sa ga si ri nohapare",
            "∃y.∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, y) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_疑問詞_誰がりんごを食べますか() {
        check(
            "ku peheta sa ko si ri nohapare",
            "?y.∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, y) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_疑問詞_複数のkoはそれぞれ別の変数で束縛される() {
        check(
            "ku peheta sa mi si ko sugi ko suni ko",
            "?x.?y.?z.∃e.(((peheta(e) ∧ (Agent(e, mi) ∧ Patient(e, x))) ∧ Location(e, y)) ∧ Time(e, z))",
        );
    }

    #[test]
    fn test_疑問詞の無い疑問文は平叙文と同じ論理式になる() {
        check(
            "ku peheta sa mi si ri nohapare",
            "∃x.(∃e.(nohapare(e) ∧ Patient(e, x)) ∧ ∃e1.(peheta(e1) ∧ (Agent(e1, mi) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_あなたは_私はりんごを食べた_と言った() {
        check(
            "kasa sa ba si re he hita peheta sa mi si ri nohapare ho",
            "∃x.(∃y.(∃e.(nohapare(e) ∧ Patient(e, y)) ∧ ((peheta(x) ∧ (Agent(x, mi) ∧ Patient(x, y))) ∧ τ(x) < t0)) ∧ ∃e1.(kasa(e1) ∧ (Agent(e1, ba) ∧ Patient(e1, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_述語から_私は食べることを経験した() {
        check(
            "kahekasaperinase sa mi si re peheta",
            "∃x.(peheta(x) ∧ ∃e.(kahekasaperinase(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_限定詞が付く_私はこの_あなたが食べたこと_を覚えている() {
        check(
            "karememabera sa mi si mo re he hita peheta sa ba ho",
            "∃x.((mo(x) ∧ ∃y.((peheta(x) ∧ (Agent(x, ba) ∧ Patient(x, y))) ∧ τ(x) < t0)) ∧ ∃e.(karememabera(e) ∧ (Agent(e, mi) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_否定された節() {
        check(
            "kasa sa ba si re he ne peheta sa mi ho",
            "∃x.(¬∃y.(peheta(x) ∧ (Agent(x, mi) ∧ Patient(x, y))) ∧ ∃e.(kasa(e) ∧ (Agent(e, ba) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_等位接続の両方のeventが同じ事象になる() {
        check(
            "kasa sa ba si re he homa peheta sa mi parobe sa mi ho",
            "∃x.((∃y.(peheta(x) ∧ (Agent(x, mi) ∧ Patient(x, y))) ∧ ∃z.(parobe(x) ∧ (Agent(x, mi) ∧ Patient(x, z)))) ∧ ∃e.(kasa(e) ∧ (Agent(e, ba) ∧ Patient(e, x))))",
        );
    }

    #[test]
    fn test_事象名詞化_reなしのhe_hoは項にならない() {
        assert!(to_fol_string("kasa sa ba si he peheta sa mi ho").is_err());
    }
}
