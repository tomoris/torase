use core::panic;
use std::collections::{HashMap, HashSet};

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Terminal(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NonTerminal(String);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Rhs {
    Sequence(Vec<Symbol>),
    Epsilon,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum FirstElement {
    Terminal(Terminal),
    Epsilon,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum FollowElement {
    Terminal(Terminal),
    Eos,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Lookahead {
    Terminal(Terminal),
    Eos,
}

pub struct LL1Grammar {
    start_symbol: NonTerminal,
    non_terminals: HashSet<NonTerminal>,
    terminals: HashSet<Terminal>,
    rules: HashMap<NonTerminal, Vec<Rhs>>,
    first_sets: HashMap<NonTerminal, HashSet<FirstElement>>,
    follow_sets: HashMap<NonTerminal, HashSet<FollowElement>>,
    parse_table: HashMap<(NonTerminal, Lookahead), Rhs>,
}

impl LL1Grammar {
    pub fn new(rules: HashMap<NonTerminal, Vec<Rhs>>) -> Self {
        let start_symbol = NonTerminal("S".to_string());
        let non_terminals = LL1Grammar::build_non_terminals(&rules, &start_symbol);
        let terminals = LL1Grammar::build_terminals(&rules);
        let first_sets = LL1Grammar::build_first_sets(&rules, &non_terminals);
        let follow_sets = LL1Grammar::build_follow_sets(
            &start_symbol,
            &rules,
            &non_terminals,
            &first_sets,
        );
        let parse_table = LL1Grammar::build_parse_table(
            &rules,
            &non_terminals,
            &terminals,
            &first_sets,
            &follow_sets,
        );
        LL1Grammar { 
            start_symbol,
            non_terminals,
            terminals,
            rules,
            first_sets,
            follow_sets,
            parse_table,
        }

    }

    fn build_non_terminals(rules: &HashMap<NonTerminal, Vec<Rhs>>, start_symbol: &NonTerminal) -> HashSet<NonTerminal> {
        let non_terminals: HashSet<NonTerminal> = rules.keys().cloned().collect();
        if !non_terminals.contains(start_symbol) {
            panic!("Start symbol {:?} is not defined in the grammar rules", start_symbol);
        }
        non_terminals
    }

    fn build_terminals(rules: &HashMap<NonTerminal, Vec<Rhs>>) -> HashSet<Terminal> {
        let mut terminals: HashSet<Terminal> = HashSet::new();
        for rhs_vec in rules.values() {
            for rhs in rhs_vec {
                match rhs {
                    Rhs::Sequence(symbols) => {
                        for symbol in symbols {
                            if let Symbol::Terminal(terminal) = symbol {
                                terminals.insert(terminal.clone());
                            }
                        }
                    }
                    Rhs::Epsilon => {}
                }
            }
        }
        terminals
    }

    fn build_first_sets(
        rules: &HashMap<NonTerminal, Vec<Rhs>>,
        non_terminals: &HashSet<NonTerminal>,
    ) -> HashMap<NonTerminal, HashSet<FirstElement>> {
        let mut first_sets: HashMap<NonTerminal, HashSet<FirstElement>> = HashMap::new();
        for non_terminal in non_terminals {
            first_sets.insert(non_terminal.clone(), HashSet::new());
        }
        let mut changed = true;
        while changed {
            changed = false;
            for (lhs, rhs_vec) in rules {
                let mut nullable = true;
                for rhs in rhs_vec {
                    match rhs {
                        Rhs::Sequence(symbols) => {
                            if let Some(first_symbol) = symbols.first() {
                                match first_symbol {
                                    Symbol::Terminal(t) => {
                                        changed |= first_sets
                                            .get_mut(lhs)
                                            .unwrap()
                                            .insert(FirstElement::Terminal(t.clone()));
                                        nullable = false;
                                    }
                                    Symbol::NonTerminal(nt) => {
                                        match first_sets.get(nt).cloned() {
                                            Some(first_set) => {
                                                for element in &first_set {
                                                    if element != &FirstElement::Epsilon {
                                                        changed |= first_sets
                                                            .get_mut(lhs)
                                                            .unwrap()
                                                            .insert(element.clone());
                                                    }
                                                }
                                                if !first_set.contains(&FirstElement::Epsilon) {
                                                    nullable = false;
                                                    break;
                                                }
                                            }
                                            None => {
                                                panic!("Non-terminal {:?} not found in first sets", nt);
                                            }
                                        }
                                    }
                                }
                            } else {
                                panic!("sequenceが空の場合は想定してません non-terminal: {:?}", lhs);
                            }
                        }
                        Rhs::Epsilon => {
                            changed |= first_sets
                                .get_mut(lhs)
                                .unwrap()
                                .insert(FirstElement::Epsilon);
                            break;
                        }
                    }
                }
                if nullable {
                    changed |= first_sets
                        .get_mut(lhs)
                        .unwrap()
                        .insert(FirstElement::Epsilon);
                }
            }
        }
        first_sets
    }

    fn first(symbols: &[Symbol], first_sets: &HashMap<NonTerminal, HashSet<FirstElement>>) -> HashSet<FirstElement> {
        let mut result: HashSet<FirstElement> = HashSet::new();
        for symbol in symbols {
            match symbol {
                Symbol::Terminal(t) => {
                    result.insert(FirstElement::Terminal(t.clone()));
                    return result;
                }
                Symbol::NonTerminal(nt) => {
                    match first_sets.get(&nt) {
                        Some(first_set) => {
                            for element in first_set {
                                result.insert(element.clone());
                            }
                            if !first_set.contains(&FirstElement::Epsilon) {
                                return result;
                            } else {
                                result.remove(&FirstElement::Epsilon);
                            }
                        }
                        None => {
                            panic!("Non-terminal {:?} not found in first sets", nt);
                        }
                    }
                }
            }
        }
        result.insert(FirstElement::Epsilon);
        result
    }

    fn build_follow_sets(
        start_symbol: &NonTerminal,
        rules: &HashMap<NonTerminal, Vec<Rhs>>,
        non_terminals: &HashSet<NonTerminal>,
        first_sets: &HashMap<NonTerminal, HashSet<FirstElement>>,
    ) -> HashMap<NonTerminal, HashSet<FollowElement>> {
        let mut follow_sets: HashMap<NonTerminal, HashSet<FollowElement>> = HashMap::new();
        follow_sets.insert(start_symbol.clone(), HashSet::from([FollowElement::Eos]));
        for non_terminal in non_terminals {
            follow_sets.insert(non_terminal.clone(), HashSet::new());
        }
        let mut changed = true;
        while changed {
            changed = false;

            for (lhs, rhs_vec) in rules {
                for rhs in rhs_vec {
                    match rhs {
                        Rhs::Sequence(symbols) => {
                            for i in 0..symbols.len() {
                                let symbol = &symbols[i];
                                if let Symbol::NonTerminal(nt) = symbol {
                                    // Follow setの更新
                                    if i == symbols.len() - 1 {
                                        for follow_element in follow_sets.get(lhs).unwrap().clone() {
                                            changed |= follow_sets
                                                .get_mut(nt)
                                                .unwrap()
                                                .insert(follow_element.clone());
                                        }
                                    } else {
                                        let mut first_set = LL1Grammar::first(&symbols[i + 1..], first_sets).clone();
                                        if first_set.contains(&FirstElement::Epsilon) {
                                            first_set.remove(&FirstElement::Epsilon);
                                            for first_element in first_set {
                                                match first_element {
                                                    FirstElement::Terminal(t) => {
                                                        changed |= follow_sets
                                                            .get_mut(nt)
                                                            .unwrap()
                                                            .insert(FollowElement::Terminal(t.clone()));
                                                    }
                                                    FirstElement::Epsilon => {}
                                                }
                                            }
                                            for follow_element in follow_sets.get(lhs).unwrap().clone() {
                                                changed |= follow_sets
                                                    .get_mut(nt)
                                                    .unwrap()
                                                    .insert(follow_element.clone());
                                            }
                                        } else {
                                            for first_element in first_set {
                                                match first_element {
                                                    FirstElement::Terminal(t) => {
                                                        changed |= follow_sets
                                                            .get_mut(nt)
                                                            .unwrap()
                                                            .insert(FollowElement::Terminal(t.clone()));
                                                    }
                                                    FirstElement::Epsilon => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Rhs::Epsilon => {}
                    }
                }
            }
        }

        follow_sets
    }

    fn build_parse_table(
        rules: &HashMap<NonTerminal, Vec<Rhs>>,
        non_terminals: &HashSet<NonTerminal>,
        terminals: &HashSet<Terminal>,
        first_sets: &HashMap<NonTerminal, HashSet<FirstElement>>,
        follow_sets: &HashMap<NonTerminal, HashSet<FollowElement>>,
    ) -> HashMap<(NonTerminal, Lookahead), Rhs> {
        let mut parse_table: HashMap<(NonTerminal, Lookahead), Rhs> = HashMap::new();
        for (lhs, rhs_vec) in rules {
            for rhs in rhs_vec {
                match rhs {
                    Rhs::Sequence(symbols) => {
                        let first_set = LL1Grammar::first(symbols, first_sets);
                        for first_element in &first_set {
                            match first_element {
                                FirstElement::Terminal(t) => {
                                    if parse_table.contains_key(&(lhs.clone(), Lookahead::Terminal(t.clone()))) {
                                        panic!("Conflict in parse table for lhs={:?}, t={:?},  parse_table[(lhs, t)]={:?} and rhs={:?}", lhs, t, parse_table.get(&(lhs.clone(), Lookahead::Terminal(t.clone()))).unwrap(), rhs);
                                    }
                                    parse_table.insert((lhs.clone(), Lookahead::Terminal(t.clone())), rhs.clone());
                                }
                                FirstElement::Epsilon => {}
                            }
                        }
                        if first_set.contains(&FirstElement::Epsilon) {
                            for follow_element in follow_sets.get(lhs).unwrap() {
                                match follow_element {
                                    FollowElement::Terminal(t) => {
                                        if parse_table.contains_key(&(lhs.clone(), Lookahead::Terminal(t.clone()))) {
                                            panic!("Conflict in parse table for lhs={:?}, t={:?},  parse_table[(lhs, t)]={:?} and rhs={:?}", lhs, t, parse_table.get(&(lhs.clone(), Lookahead::Terminal(t.clone()))).unwrap(), rhs);
                                        }
                                        parse_table.insert((lhs.clone(), Lookahead::Terminal(t.clone())), rhs.clone());
                                    }
                                    FollowElement::Eos => {
                                        if parse_table.contains_key(&(lhs.clone(), Lookahead::Eos)) {
                                            panic!("Conflict in parse table for lhs={:?}, t={:?},  parse_table[(lhs, t)]={:?} and rhs={:?}", lhs, Lookahead::Eos, parse_table.get(&(lhs.clone(), Lookahead::Eos)).unwrap(), rhs);
                                        }
                                        parse_table.insert((lhs.clone(), Lookahead::Eos), rhs.clone());
                                    }
                                }
                            }
                        }
                    }
                    Rhs::Epsilon => {
                        for follow_element in follow_sets.get(lhs).unwrap() {
                            match follow_element {
                                FollowElement::Terminal(t) => {
                                    if parse_table.contains_key(&(lhs.clone(), Lookahead::Terminal(t.clone()))) {
                                        panic!("Conflict in parse table for lhs={:?}, t={:?},  parse_table[(lhs, t)]={:?} and rhs={:?}", lhs, t, parse_table.get(&(lhs.clone(), Lookahead::Terminal(t.clone()))).unwrap(), rhs);
                                    }
                                    parse_table.insert((lhs.clone(), Lookahead::Terminal(t.clone())), rhs.clone());
                                }
                                FollowElement::Eos => {
                                    if parse_table.contains_key(&(lhs.clone(), Lookahead::Eos)) {
                                        panic!("Conflict in parse table for lhs={:?}, t={:?},  parse_table[(lhs, t)]={:?} and rhs={:?}", lhs, Lookahead::Eos, parse_table.get(&(lhs.clone(), Lookahead::Eos)).unwrap(), rhs);
                                    }
                                    parse_table.insert((lhs.clone(), Lookahead::Eos), rhs.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        parse_table
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_与えられたll1文法で受理できる文の場合に受理される1() {
        // https://youtu.be/clkHOgZUGWU?t=566

        let terminal_a = Terminal("a".to_string());
        let terminal_b = Terminal("b".to_string());
        let terminal_c = Terminal("c".to_string());
        let terminal_d = Terminal("d".to_string());
        let non_terminal_a = NonTerminal("A".to_string());
        let non_terminal_b = NonTerminal("B".to_string());

        let mut rules = HashMap::new();
        rules.insert(
            NonTerminal("S".to_string()),
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(terminal_a),
                Symbol::NonTerminal(non_terminal_a.clone()),
                Symbol::NonTerminal(non_terminal_b.clone()),
                Symbol::Terminal(terminal_b),
            ])],
        );
        rules.insert(
            non_terminal_a,
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(terminal_c),
            ]), Rhs::Epsilon],
        );
        rules.insert(
            non_terminal_b,
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(terminal_d),
            ]), Rhs::Epsilon],
        );

        LL1Grammar::new(rules);
    }

    #[test]
    fn test_与えられたll1文法で受理できる文の場合に受理される4() {

        let plus = Terminal("PLUS".to_string());
        let times = Terminal("TIMES".to_string());
        let lparen = Terminal("LPAREN".to_string());
        let rparen = Terminal("RPAREN".to_string());
        let id = Terminal("ID".to_string());
        let e_prime = NonTerminal("E_prime".to_string());
        let t = NonTerminal("T".to_string());
        let t_prime = NonTerminal("T_PRIME".to_string());
        let f = NonTerminal("F".to_string());
        let s = NonTerminal("S".to_string());

        let mut rules = HashMap::new();
        rules.insert(
            s.clone(),
            vec![Rhs::Sequence(vec![
                Symbol::NonTerminal(t.clone()),
                Symbol::NonTerminal(e_prime.clone()),
            ])],
        );
        rules.insert(
            e_prime.clone(),
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(plus.clone()),
                Symbol::NonTerminal(t.clone()),
                Symbol::NonTerminal(e_prime.clone()),
            ]), Rhs::Epsilon],
        );
        rules.insert(
            t.clone(),
            vec![Rhs::Sequence(vec![
                Symbol::NonTerminal(f.clone()),
                Symbol::NonTerminal(t_prime.clone()),
            ])],
        );
        rules.insert(
            t_prime.clone(),
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(times.clone()),
                Symbol::NonTerminal(f.clone()),
                Symbol::NonTerminal(t_prime.clone()),
            ]), Rhs::Epsilon],
        );
        rules.insert(
            f.clone(),
            vec![Rhs::Sequence(vec![
                Symbol::Terminal(id.clone()),
            ]), Rhs::Sequence(vec![
                Symbol::Terminal(lparen.clone()),
                Symbol::NonTerminal(s.clone()),
                Symbol::Terminal(rparen.clone()),
            ])],
        );

        LL1Grammar::new(rules);
    }
}
