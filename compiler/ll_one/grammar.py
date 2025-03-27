from copy import deepcopy

from compiler.my_ast import (
    TerminalSymbol,
    NonTerminalSymbol,
    Symbol,
    START_SYMBOL,
    EOS_SYMBOL,
    Eos,
    EPSILON_SYMBOL,
    Epsilon,
    Production,
)


class Grammar:
    def __init__(
        self,
        rules: dict[NonTerminalSymbol, list[Production]],
        terminals: set[TerminalSymbol],
    ):
        self.rules = rules
        self.terminals = terminals
        self.non_terminals: set[NonTerminalSymbol] = set()
        self.first_sets: dict[NonTerminalSymbol, set[Symbol | Epsilon]] = {}
        self.follow_sets: dict[NonTerminalSymbol, set[Symbol | Eos]] = {}
        self.parse_table: dict[
            NonTerminalSymbol,
            dict[TerminalSymbol | Eos, Production | None],
        ] = {}
        self.build()

    def build(self):
        for lhs, rhs in self.rules.items():
            if lhs in self.non_terminals:
                raise ValueError(f"非終端記号 {lhs} が複数回定義されています")
            self.non_terminals.add(lhs)
            if lhs in self.terminals:
                raise ValueError(f"非終端記号 {lhs} が終端記号として定義されています")
        for lhs, rhs in self.rules.items():
            for production in rhs:
                if production == EPSILON_SYMBOL:
                    continue
                elif isinstance(production, list):
                    for symbol in production:
                        if symbol not in self.terminals:
                            if symbol not in self.non_terminals:
                                raise ValueError(
                                    f"記号 {symbol} は非終端記号としてルールの左辺に出現させるか終端記号として定義する必要があります"
                                )
                else:
                    raise ValueError(f"Invalid production: {production}")
        if START_SYMBOL not in self.rules:
            raise ValueError(f"開始記号 {START_SYMBOL} が定義されていません")
        if START_SYMBOL not in self.non_terminals:
            raise ValueError(f"開始記号 {START_SYMBOL} が定義されていません")
        if EPSILON_SYMBOL in self.terminals or EPSILON_SYMBOL in self.non_terminals:
            raise ValueError("ε は終端記号や非終端記号に含まれてはいけません")
        for terminal in self.terminals:
            is_existed = False
            for _, rhs in self.rules.items():
                for production in rhs:
                    if isinstance(production, list):
                        if terminal in production:
                            is_existed = True
            if not is_existed:
                raise ValueError(f"終端記号 {terminal} が使われていません")

        self.compute_first_sets()
        self.compute_follow_sets()
        self.build_parse_table()

    def compute_first_sets(self):
        # Implementation of FIRST set computation
        for non_terminal in self.non_terminals:
            self.first_sets[non_terminal] = set()
        old_first_sets = deepcopy(self.first_sets)
        while True:
            for lhs, rhs in self.rules.items():
                for production in rhs:
                    if production == EPSILON_SYMBOL:
                        self.first_sets[lhs].add(production)
                    elif isinstance(production, list):
                        self.first_sets[lhs].update(self.first(production))
                    else:
                        raise ValueError(f"Invalid production: {production}")
            if old_first_sets == self.first_sets:
                break
            old_first_sets = deepcopy(self.first_sets)

        for non_terminal in self.non_terminals:
            if len(self.first_sets[non_terminal]) == 0:
                raise ValueError(f"FIRST set for {non_terminal} is empty")

    def first(self, symbols: list[Symbol]) -> set[Symbol | Epsilon]:
        first_set = set()
        for symbol in symbols:
            if symbol in self.terminals:
                first_set.add(symbol)
                return first_set
            elif symbol in self.non_terminals:
                first_set.update(self.first_sets[symbol])
                if EPSILON_SYMBOL not in self.first_sets[symbol]:
                    return first_set
                else:
                    first_set.remove(EPSILON_SYMBOL)
            else:
                raise ValueError(f"Invalid symbol {symbol}")
        first_set.add(EPSILON_SYMBOL)
        return first_set

    def compute_follow_sets(self):
        # Implementation of FOLLOW set computation
        for non_terminal in self.non_terminals:
            self.follow_sets[non_terminal] = set()
        self.follow_sets["S"].add(EOS_SYMBOL)
        old_follow_sets = deepcopy(self.follow_sets)
        while True:
            for lhs, rhs in self.rules.items():
                for production in rhs:
                    if production == EPSILON_SYMBOL:
                        continue
                    elif isinstance(production, list):
                        for i, symbol in enumerate(production):
                            if symbol in self.non_terminals:
                                if i == len(production) - 1:
                                    self.follow_sets[symbol].update(
                                        self.follow_sets[lhs]
                                    )
                                else:
                                    first_set = self.first(production[i + 1 :])
                                    if EPSILON_SYMBOL in first_set:
                                        first_set.remove(EPSILON_SYMBOL)
                                        self.follow_sets[symbol].update(first_set)
                                        self.follow_sets[symbol].update(
                                            self.follow_sets[lhs]
                                        )
                                    else:
                                        self.follow_sets[symbol].update(first_set)
            if old_follow_sets == self.follow_sets:
                break
            old_follow_sets = deepcopy(self.follow_sets)

        for non_terminal in self.non_terminals:
            if len(self.follow_sets[non_terminal]) == 0:
                raise ValueError(f"FOLLOW set for {non_terminal} is empty")

    def follow(self, non_terminal: NonTerminalSymbol) -> set[Symbol | Eos]:
        return self.follow_sets[non_terminal]

    def build_parse_table(self) -> None:
        # Implementation of parse table construction
        for non_terminal in self.non_terminals:
            self.parse_table[non_terminal] = {}
            for terminal in self.terminals:
                self.parse_table[non_terminal][terminal] = None
            self.parse_table[non_terminal][EOS_SYMBOL] = None
        for lhs, rhs in self.rules.items():
            for production in rhs:
                if production == EPSILON_SYMBOL:
                    follow_set = self.follow(lhs)
                    for terminal in follow_set:
                        if self.parse_table[lhs][terminal] is not None:
                            raise ValueError(
                                f"Conflict in parse table for {lhs} and {terminal}. {self.parse_table[lhs][terminal]} and {production} cannot be in the same cell"
                            )
                        self.parse_table[lhs][terminal] = production
                elif isinstance(production, list):
                    first_set = self.first(production)
                    for terminal in first_set:
                        if terminal != EPSILON_SYMBOL:
                            if self.parse_table[lhs][terminal] is not None:
                                raise ValueError(
                                    f"Conflict in parse table for {lhs} and {terminal}. {self.parse_table[lhs][terminal]} and {production} cannot be in the same cell"
                                )
                            self.parse_table[lhs][terminal] = production
                    if EPSILON_SYMBOL in first_set:
                        follow_set = self.follow(lhs)
                        for terminal in follow_set:
                            if self.parse_table[lhs][terminal] is not None:
                                raise ValueError(
                                    f"Conflict in parse table for {lhs} and {terminal}. {self.parse_table[lhs][terminal]} and {production} cannot be in the same cell"
                                )
                            self.parse_table[lhs][terminal] = production
                else:
                    raise ValueError(f"Invalid production: {production}")
