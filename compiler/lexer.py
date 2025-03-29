from typing import NewType

from compiler.ll_one.grammar import TerminalSymbol


Stem = NewType("Stem", str)


class Lexer:
    def __init__(self, terminals: set[TerminalSymbol]):
        self.terminals = terminals

    def tokenize(self, sentence: str) -> tuple[list[TerminalSymbol], list[Stem]]:
        poss = []
        stems = []
        for token in sentence.split():
            pos, stem = token.split(":")
            if pos not in self.terminals:
                raise ValueError(f"Invalid token: {pos}")
            poss.append(pos)
            stems.append(stem)

        return poss, stems  # type: ignore[return-value]
