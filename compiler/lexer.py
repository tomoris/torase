from compiler.parsing.my_ast import TerminalSymbol, Token


class Lexer:
    def __init__(self, terminals: set[TerminalSymbol]):
        self.terminals = terminals

    def tokenize(self, sentence: str) -> list[Token]:
        tokens = []
        for s in sentence.split():
            pos, string = s.split(":")
            if pos not in self.terminals:
                raise ValueError(f"Invalid token: {pos}")
            tokens.append(Token(TerminalSymbol(pos), string))

        return tokens
