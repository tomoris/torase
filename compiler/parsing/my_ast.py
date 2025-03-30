from __future__ import annotations

from typing import Literal, NewType

TerminalSymbol = NewType("TerminalSymbol", str)
NonTerminalSymbol = NewType("NonTerminalSymbol", str)
Symbol = TerminalSymbol | NonTerminalSymbol
START_SYMBOL = "S"
Start = Literal["S"]
EOS_SYMBOL = "$"
Eos = Literal["$"]  # End of sentence
EPSILON_SYMBOL = "ε"
Epsilon = Literal["ε"]
Production = list[Symbol] | Epsilon


class AbstractSyntaxTree:
    def __init__(self, non_terminal_symbol: NonTerminalSymbol):
        self.non_terminal_symbol = non_terminal_symbol
        self.children: list[AbstractSyntaxTree | TerminalSymbol] | Epsilon = []

    def add_child(self, child: AbstractSyntaxTree | TerminalSymbol | Epsilon):
        if child == EPSILON_SYMBOL:  # type: ignore[assignment]
            self.children = EPSILON_SYMBOL  # type: ignore[assignment]
        else:
            self.children.append(child)  # type: ignore[union-attr,arg-type]

    def __repr__(self):
        return f"({self.non_terminal_symbol} -> {self.children})"
