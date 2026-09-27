//! Resolution-based first-order logic contradiction checker.
//!
//! Given a slice of [`Formula`]s, [`is_contradictory`] checks whether their
//! conjunction is unsatisfiable by:
//!
//! 1. Converting each formula to CNF via Skolemization (handles ∀/∃/¬/∧/∨/→).
//! 2. Running binary resolution with Robinson unification until the empty
//!    clause (⊥) is derived or no new clause can be generated.
//!
//! Modal operators are handled by the standard translation: every atom gets an
//! extra world argument, `□_b φ` at `w` becomes `∀v.(R_b(w, v) ⇒ φ@v)` and
//! `◇_b φ` becomes `∃v.(R_b(w, v) ∧ φ@v)`, evaluated from the actual world `w0`.
//! The accessibility relations get the frame conditions in [`frame_axioms`].

use std::collections::{HashMap, HashSet, VecDeque};

use super::event_semantics::{Formula, ModalBase, ModalForce, Term as FolTerm};

// ---------------------------------------------------------------------------
// Term
// ---------------------------------------------------------------------------

/// A first-order term enriched with Skolem function applications.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Term {
    /// Logic variable introduced by a universal quantifier.
    Var(String),
    /// Ground individual constant or Skolem constant.
    Const(String),
    /// Skolem function application  f(y₁, …, yₖ).
    Func(String, Vec<Term>),
}

impl Term {
    /// Fully apply a substitution (chase variable chains).
    fn apply(&self, subst: &HashMap<String, Term>) -> Term {
        match self {
            Term::Var(v) => match subst.get(v) {
                Some(t) => t.apply(subst),
                None => self.clone(),
            },
            Term::Const(_) => self.clone(),
            Term::Func(f, args) => {
                Term::Func(f.clone(), args.iter().map(|t| t.apply(subst)).collect())
            }
        }
    }

    /// Occurs check: does variable `v` appear anywhere in `self`?
    fn contains_var(&self, v: &str) -> bool {
        match self {
            Term::Var(u) => u == v,
            Term::Const(_) => false,
            Term::Func(_, args) => args.iter().any(|t| t.contains_var(v)),
        }
    }

    /// Return a copy with all `Var` names prefixed by `prefix`.
    fn prefix_vars(&self, prefix: &str) -> Term {
        match self {
            Term::Var(v) => Term::Var(format!("{prefix}{v}")),
            Term::Const(_) => self.clone(),
            Term::Func(f, args) => Term::Func(
                f.clone(),
                args.iter().map(|t| t.prefix_vars(prefix)).collect(),
            ),
        }
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(v) => write!(f, "{v}"),
            Term::Const(c) => write!(f, "{c}"),
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

// ---------------------------------------------------------------------------
// Literal  =  signed atom
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Literal {
    pred: String,
    args: Vec<Term>,
    positive: bool,
}

impl Literal {
    fn apply(&self, subst: &HashMap<String, Term>) -> Literal {
        Literal {
            pred: self.pred.clone(),
            args: self.args.iter().map(|t| t.apply(subst)).collect(),
            positive: self.positive,
        }
    }

    fn prefix_vars(&self, prefix: &str) -> Literal {
        Literal {
            pred: self.pred.clone(),
            args: self.args.iter().map(|t| t.prefix_vars(prefix)).collect(),
            positive: self.positive,
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .args
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let neg = if self.positive { "" } else { "¬" };
        if s.is_empty() {
            write!(f, "{neg}{}", self.pred)
        } else {
            write!(f, "{neg}{}({s})", self.pred)
        }
    }
}

/// A clause is a disjunction of literals.
type Clause = Vec<Literal>;

/// Canonical string key for a clause (order-independent).
fn clause_key(c: &[Literal]) -> String {
    let mut lits: Vec<String> = c.iter().map(|l| l.to_string()).collect();
    lits.sort();
    lits.join(" ∨ ")
}

fn clause_prefix_vars(clause: &[Literal], prefix: &str) -> Clause {
    clause.iter().map(|l| l.prefix_vars(prefix)).collect()
}

// ---------------------------------------------------------------------------
// Unification
// ---------------------------------------------------------------------------

/// Try to unify terms `t1` and `t2` under `subst` (Robinson's algorithm).
/// Returns `true` and extends `subst` on success.
/// `subst` may be partially modified on failure — discard it in that case.
fn unify_terms(t1: &Term, t2: &Term, subst: &mut HashMap<String, Term>) -> bool {
    let t1 = t1.apply(subst);
    let t2 = t2.apply(subst);

    if t1 == t2 {
        return true;
    }

    if let Term::Var(v) = &t1 {
        if t2.contains_var(v) {
            return false; // occurs check
        }
        subst.insert(v.clone(), t2.clone());
        return true;
    }

    if let Term::Var(v) = &t2 {
        if t1.contains_var(v) {
            return false; // occurs check
        }
        subst.insert(v.clone(), t1.clone());
        return true;
    }

    if let (Term::Func(f1, args1), Term::Func(f2, args2)) = (&t1, &t2) {
        if f1 != f2 || args1.len() != args2.len() {
            return false;
        }
        // Clone to avoid simultaneous borrow of t1/t2 and subst.
        let pairs: Vec<(Term, Term)> = args1
            .iter()
            .zip(args2.iter())
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect();
        return pairs.iter().all(|(a, b)| unify_terms(a, b, subst));
    }

    false
}

// ---------------------------------------------------------------------------
// Tautology filter
// ---------------------------------------------------------------------------

/// Returns `true` if the clause contains both L and ¬L for some literal L.
fn is_tautology(clause: &[Literal]) -> bool {
    for lit in clause {
        let comp = Literal {
            pred: lit.pred.clone(),
            args: lit.args.clone(),
            positive: !lit.positive,
        };
        if clause.contains(&comp) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// CNF builder  (Skolemization + polarity-driven NNF + distribution)
// ---------------------------------------------------------------------------

struct CnfBuilder {
    counter: usize,
}

impl CnfBuilder {
    fn new() -> Self {
        CnfBuilder { counter: 0 }
    }

    fn fresh(&mut self, prefix: &str) -> String {
        let n = self.counter;
        self.counter += 1;
        format!("{prefix}{n}")
    }

    /// Build a Skolem term for an existential variable in scope of `universals`.
    fn skolem(&mut self, universals: &[Term]) -> Term {
        let name = self.fresh("sk");
        if universals.is_empty() {
            Term::Const(name)
        } else {
            Term::Func(name, universals.to_vec())
        }
    }

    /// Convert a FOL term to a resolution `Term` using the binding environment.
    fn convert_fol_term(&self, t: &FolTerm, env: &HashMap<String, Term>) -> Term {
        match t {
            FolTerm::Var(v) => env
                .get(v)
                .cloned()
                .unwrap_or_else(|| Term::Const(v.clone())),
            FolTerm::Const(c) => Term::Const(c.clone()),
            FolTerm::Func(f, args) => Term::Func(
                f.clone(),
                args.iter().map(|a| self.convert_fol_term(a, env)).collect(),
            ),
        }
    }

    /// Convert a formula to a **conjunction** of CNF clauses.
    ///
    /// - `positive`: `true` = formula in positive position, `false` = negated.
    /// - `universals`: resolution `Term::Var`s for in-scope ∀-variables (Skolem dependencies).
    /// - `env`: maps FOL bound variable names to resolution Terms.
    /// - `world`: the possible world the formula is evaluated at.
    fn build_clauses(
        &mut self,
        formula: &Formula,
        positive: bool,
        universals: &[Term],
        env: &HashMap<String, Term>,
        world: &Term,
    ) -> Vec<Clause> {
        match formula {
            // ---- Atom ----
            Formula::Atom(pred, fol_args) => {
                let mut args: Vec<Term> = fol_args
                    .iter()
                    .map(|t| self.convert_fol_term(t, env))
                    .collect();
                args.push(world.clone());
                vec![vec![Literal {
                    pred: pred.clone(),
                    args,
                    positive,
                }]]
            }

            // ---- Negation: flip polarity ----
            Formula::Not(phi) => self.build_clauses(phi, !positive, universals, env, world),

            // ---- Conjunction ----
            Formula::And(l, r) => {
                if positive {
                    // (A ∧ B)⁺  →  clauses(A) ++ clauses(B)
                    let mut cs = self.build_clauses(l, true, universals, env, world);
                    cs.extend(self.build_clauses(r, true, universals, env, world));
                    cs
                } else {
                    // ¬(A ∧ B)  =  ¬A ∨ ¬B
                    let left = self.build_clauses(l, false, universals, env, world);
                    let right = self.build_clauses(r, false, universals, env, world);
                    cross_or(left, right)
                }
            }

            // ---- Disjunction ----
            Formula::Or(l, r) => {
                if positive {
                    // A ∨ B: distribute into CNF via cross-product
                    let left = self.build_clauses(l, true, universals, env, world);
                    let right = self.build_clauses(r, true, universals, env, world);
                    cross_or(left, right)
                } else {
                    // ¬(A ∨ B)  =  ¬A ∧ ¬B
                    let mut cs = self.build_clauses(l, false, universals, env, world);
                    cs.extend(self.build_clauses(r, false, universals, env, world));
                    cs
                }
            }

            // ---- Implication: A → B  =  ¬A ∨ B ----
            Formula::Implies(l, r) => {
                if positive {
                    let left = self.build_clauses(l, false, universals, env, world);
                    let right = self.build_clauses(r, true, universals, env, world);
                    cross_or(left, right)
                } else {
                    // ¬(A → B)  =  A ∧ ¬B
                    let mut cs = self.build_clauses(l, true, universals, env, world);
                    cs.extend(self.build_clauses(r, false, universals, env, world));
                    cs
                }
            }

            // ---- Universal quantifier ----
            Formula::Forall(x, phi) => {
                if positive {
                    // ∀x.φ  →  introduce fresh logic variable for x
                    let v = Term::Var(self.fresh("u"));
                    let mut new_env = env.clone();
                    new_env.insert(x.clone(), v.clone());
                    let mut new_univ = universals.to_vec();
                    new_univ.push(v);
                    self.build_clauses(phi, true, &new_univ, &new_env, world)
                } else {
                    // ¬∀x.φ  =  ∃x.¬φ  →  Skolemize x
                    let sk = self.skolem(universals);
                    let mut new_env = env.clone();
                    new_env.insert(x.clone(), sk);
                    self.build_clauses(phi, false, universals, &new_env, world)
                }
            }

            // ---- Existential quantifier ----
            // A question `?x.φ` presupposes that some x satisfies φ, so it is treated as ∃x.φ.
            Formula::Exists(x, phi) | Formula::Question(x, phi) => {
                if positive {
                    // ∃x.φ  →  Skolemize x
                    let sk = self.skolem(universals);
                    let mut new_env = env.clone();
                    new_env.insert(x.clone(), sk);
                    self.build_clauses(phi, true, universals, &new_env, world)
                } else {
                    // ¬∃x.φ  =  ∀x.¬φ  →  fresh variable
                    let v = Term::Var(self.fresh("u"));
                    let mut new_env = env.clone();
                    new_env.insert(x.clone(), v.clone());
                    let mut new_univ = universals.to_vec();
                    new_univ.push(v);
                    self.build_clauses(phi, false, &new_univ, &new_env, world)
                }
            }

            // ---- Generalized quantifier ----
            // "many/half/habitually x. φ" entails ∃x.φ, but its negation does not entail
            // ∀x.¬φ, so a negated GenQ is weakened to ⊤ (no clauses).
            Formula::GenQ(_, x, phi) => {
                if positive {
                    let sk = self.skolem(universals);
                    let mut new_env = env.clone();
                    new_env.insert(x.clone(), sk);
                    self.build_clauses(phi, true, universals, &new_env, world)
                } else {
                    Vec::new()
                }
            }

            // ---- Modal operator (standard translation) ----
            Formula::Modal(force, base, phi) => {
                let universal = matches!(
                    (force, positive),
                    (ModalForce::Necessity, true) | (ModalForce::Possibility, false)
                );
                if universal {
                    // □φ / ¬◇φ  →  ∀v.(¬R(w, v) ∨ φ@v)
                    let v = Term::Var(self.fresh("u"));
                    let mut new_univ = universals.to_vec();
                    new_univ.push(v.clone());
                    let not_access = vec![vec![accessibility(*base, world, &v, false)]];
                    let body = self.build_clauses(phi, positive, &new_univ, env, &v);
                    cross_or(not_access, body)
                } else {
                    // ◇φ / ¬□φ  →  R(w, sk) ∧ φ@sk
                    let v = self.skolem(universals);
                    let mut cs = vec![vec![accessibility(*base, world, &v, true)]];
                    cs.extend(self.build_clauses(phi, positive, universals, env, &v));
                    cs
                }
            }
        }
    }
}

fn accessibility(base: ModalBase, from: &Term, to: &Term, positive: bool) -> Literal {
    Literal {
        pred: format!("R_{}", base.label()),
        args: vec![from.clone(), to.clone()],
        positive,
    }
}

const ACTUAL_WORLD: &str = "w0";

/// Frame conditions on the accessibility relations:
/// - epistemic / circumstantial: reflexive (T) — the actual world is compatible with what
///   the speaker knows and with the actual circumstances.
/// - deontic: serial (D) — some world obeys the rules, but the actual one need not.
fn frame_axioms() -> Vec<Clause> {
    let w = Term::Var("fw".to_string());
    vec![
        vec![accessibility(ModalBase::Epistemic, &w, &w, true)],
        vec![accessibility(ModalBase::Circumstantial, &w, &w, true)],
        vec![accessibility(
            ModalBase::Deontic,
            &w,
            &Term::Func("sk_deon".to_string(), vec![w.clone()]),
            true,
        )],
    ]
}

/// Cross-product of two clause sets: each pair is merged into one clause.
/// Tautological clauses are discarded.
fn cross_or(left: Vec<Clause>, right: Vec<Clause>) -> Vec<Clause> {
    let mut result = Vec::new();
    for l in &left {
        for r in &right {
            let mut merged = l.clone();
            merged.extend_from_slice(r);
            // Deduplicate literals.
            let mut seen_lits: HashSet<String> = HashSet::new();
            merged.retain(|lit| seen_lits.insert(lit.to_string()));
            if !is_tautology(&merged) {
                result.push(merged);
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Binary resolution
// ---------------------------------------------------------------------------

/// Enumerate all binary resolvents of `c1` and `c2`.
///
/// For each complementary literal pair (l1 ∈ c1, l2 ∈ c2) that unify,
/// the resolvent is  σ(C₁ \ {l1}  ∪  C₂ \ {l2}).
fn resolve_pair(c1: &[Literal], c2: &[Literal]) -> Vec<Clause> {
    let mut resolvents = Vec::new();

    for (i, l1) in c1.iter().enumerate() {
        for (j, l2) in c2.iter().enumerate() {
            // Complementary polarities + matching predicate/arity
            if l1.positive == l2.positive || l1.pred != l2.pred || l1.args.len() != l2.args.len() {
                continue;
            }

            // Try to unify argument lists
            let mut subst = HashMap::new();
            let pairs: Vec<(Term, Term)> = l1
                .args
                .iter()
                .zip(l2.args.iter())
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect();
            let ok = pairs.iter().all(|(a, b)| unify_terms(a, b, &mut subst));
            if !ok {
                continue;
            }

            // Build resolvent
            let mut resolvent: Clause = c1
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != i)
                .map(|(_, l)| l.apply(&subst))
                .collect();
            resolvent.extend(
                c2.iter()
                    .enumerate()
                    .filter(|(k, _)| *k != j)
                    .map(|(_, l)| l.apply(&subst)),
            );

            // Deduplicate literals
            let mut seen_lits: HashSet<String> = HashSet::new();
            resolvent.retain(|l| seen_lits.insert(l.to_string()));

            if !is_tautology(&resolvent) {
                resolvents.push(resolvent);
            }
        }
    }

    resolvents
}

const MAX_CLAUSES: usize = 10_000;

/// Run binary resolution on the initial set of clauses.
///
/// Returns `true` if the empty clause (⊥) is derived, i.e., the set is
/// unsatisfiable.  Returns `false` if no contradiction is found within
/// [`MAX_CLAUSES`] derived clauses.
fn resolution(initial: Vec<Clause>) -> bool {
    let mut seen: HashSet<String> = HashSet::new();
    let mut worklist: VecDeque<Clause> = VecDeque::new();
    let mut processed: Vec<Clause> = Vec::new();
    let mut rename_id: usize = 0;

    for c in initial {
        if c.is_empty() {
            return true;
        }
        let key = clause_key(&c);
        if seen.insert(key) {
            worklist.push_back(c);
        }
    }

    while let Some(clause) = worklist.pop_front() {
        if seen.len() > MAX_CLAUSES {
            break;
        }

        // Resolve `clause` against every already-processed clause.
        for existing in &processed {
            // Rename vars in `clause` to avoid clashes with `existing`.
            let prefix = format!("r{rename_id}_");
            rename_id += 1;
            let renamed = clause_prefix_vars(&clause, &prefix);

            for resolvent in resolve_pair(&renamed, existing) {
                if resolvent.is_empty() {
                    return true; // ⊥ derived — contradiction!
                }
                let key = clause_key(&resolvent);
                if seen.insert(key) {
                    worklist.push_back(resolvent);
                }
            }
        }

        processed.push(clause);
    }

    false
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns `true` if the conjunction of `formulas` is unsatisfiable
/// (logically contradictory).
fn is_contradictory(formulas: &[Formula]) -> bool {
    let mut builder = CnfBuilder::new();
    let mut all_clauses: Vec<Clause> = frame_axioms();
    let actual = Term::Const(ACTUAL_WORLD.to_string());

    for formula in formulas {
        let clauses = builder.build_clauses(formula, true, &[], &HashMap::new(), &actual);
        all_clauses.extend(clauses);
    }

    resolution(all_clauses)
}

/// Returns `Some(i)` where `i` is the index of the first formula whose addition
/// makes the set contradictory, or `None` if all formulas are consistent.
///
/// Uses prefix checking: finds the smallest prefix `formulas[..i+1]` that is
/// unsatisfiable, and returns `i` as the index of the offending formula.
pub fn find_contradiction_index(formulas: &[Formula]) -> Option<usize> {
    for i in 1..=formulas.len() {
        if is_contradictory(&formulas[..i]) {
            return Some(i - 1);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::event_semantics::{Formula, Term as FolTerm};
    use super::*;

    fn var(name: &str) -> FolTerm {
        FolTerm::Var(name.to_string())
    }

    fn konst(name: &str) -> FolTerm {
        FolTerm::Const(name.to_string())
    }

    fn atom(pred: &str, args: Vec<FolTerm>) -> Formula {
        Formula::Atom(pred.to_string(), args)
    }

    /// ∃x [boy(x) ∧ ∀y (girl(y) → love(x,y))]
    ///
    /// "A boy loves every girl."
    fn a_boy_loves_every_girl() -> Formula {
        Formula::Exists(
            "x".to_string(),
            Box::new(Formula::And(
                Box::new(atom("boy", vec![var("x")])),
                Box::new(Formula::Forall(
                    "y".to_string(),
                    Box::new(Formula::Implies(
                        Box::new(atom("girl", vec![var("y")])),
                        Box::new(atom("love", vec![var("x"), var("y")])),
                    )),
                )),
            )),
        )
    }

    /// ∀x [boy(x) → ¬∀y (girl(y) → love(x,y))]
    ///
    /// "Every boy does not love every girl."
    fn every_boy_does_not_love_every_girl() -> Formula {
        Formula::Forall(
            "x".to_string(),
            Box::new(Formula::Implies(
                Box::new(atom("boy", vec![var("x")])),
                Box::new(Formula::Not(Box::new(Formula::Forall(
                    "y".to_string(),
                    Box::new(Formula::Implies(
                        Box::new(atom("girl", vec![var("y")])),
                        Box::new(atom("love", vec![var("x"), var("y")])),
                    )),
                )))),
            )),
        )
    }

    /// The two formulas
    ///   φ₁ = ∃x [boy(x) ∧ ∀y (girl(y) → love(x,y))]
    ///   φ₂ = ∀x [boy(x) → ¬∀y (girl(y) → love(x,y))]
    /// are contradictory:
    ///   φ₁ gives a boy `a` who loves every girl,
    ///   φ₂ says that very boy `a` does not love every girl.
    ///
    /// Resolution proof sketch:
    ///   CNF(φ₁): {boy(sk0)},  {¬girl(u0), love(sk0, u0)}
    ///   CNF(φ₂): {¬boy(u1), girl(sk1(u1))},  {¬boy(u1), ¬love(u1, sk1(u1))}
    ///
    ///   1. Resolve {boy(sk0)} with {¬boy(u1), girl(sk1(u1))} [u1→sk0]
    ///      → {girl(sk1(sk0))}
    ///   2. Resolve {¬girl(u0), love(sk0,u0)} with {girl(sk1(sk0))} [u0→sk1(sk0)]
    ///      → {love(sk0, sk1(sk0))}
    ///   3. Resolve {boy(sk0)} with {¬boy(u1), ¬love(u1, sk1(u1))} [u1→sk0]
    ///      → {¬love(sk0, sk1(sk0))}
    ///   4. Resolve {love(sk0, sk1(sk0))} with {¬love(sk0, sk1(sk0))}
    ///      → {} (empty clause — ⊥)
    #[test]
    fn test_a_boy_loves_every_girl_contradicts_every_boy_does_not_love_every_girl() {
        let formulas = vec![
            a_boy_loves_every_girl(),
            every_boy_does_not_love_every_girl(),
        ];
        assert!(
            is_contradictory(&formulas),
            "φ₁ ∧ φ₂ should be unsatisfiable"
        );
    }

    /// Each formula alone is satisfiable.
    #[test]
    fn test_each_formula_alone_is_satisfiable() {
        assert!(!is_contradictory(&[a_boy_loves_every_girl()]));
        assert!(!is_contradictory(&[every_boy_does_not_love_every_girl()]));
    }

    /// P(a) ∧ ¬P(a) is an explicit contradiction.
    #[test]
    fn test_explicit_propositional_contradiction() {
        let p = atom("P", vec![konst("a")]);
        let not_p = Formula::Not(Box::new(atom("P", vec![konst("a")])));
        assert!(is_contradictory(&[p, not_p]));
    }

    /// boy(a) alone is not contradictory.
    #[test]
    fn test_single_atom_is_not_contradictory() {
        assert!(!is_contradictory(&[atom("boy", vec![konst("a")])]));
    }

    /// ∀x P(x) and ¬P(a) are contradictory (instantiate universal at a).
    #[test]
    fn test_forall_contradicts_negated_instance() {
        let all_p = Formula::Forall("x".to_string(), Box::new(atom("P", vec![var("x")])));
        let not_pa = Formula::Not(Box::new(atom("P", vec![konst("a")])));
        assert!(is_contradictory(&[all_p, not_pa]));
    }
}
