from compiler.lexer import Lexer
from compiler.parsing.grammar import Grammar
from compiler.parsing.my_ast import (
    EOS_SYMBOL,
    EPSILON_SYMBOL,
    START_SYMBOL,
    AbstractSyntaxTree,
    Start,
    Symbol,
    TerminalSymbol,
    Token,
)


class Parser:
    def __init__(self, grammar: Grammar, lexer: Lexer):
        self.grammar = grammar
        self.lexer = lexer

    def parse(self, sentence: str) -> AbstractSyntaxTree:
        tokens = self.lexer.tokenize(sentence)
        tokens.append(Token(pos=TerminalSymbol(EOS_SYMBOL), string=EOS_SYMBOL))
        stack: list[Symbol | Start] = [EOS_SYMBOL, START_SYMBOL]  # type: ignore[list-item]
        current_position = 0
        tree = AbstractSyntaxTree(START_SYMBOL)  # type: ignore[arg-type]

        while stack:
            tree, stack, current_position = self._parse_tree(  # type: ignore[assignment]
                tree, tokens, stack, current_position
            )

        if current_position != len(tokens) or len(stack) != 0:
            raise Exception(
                f"Invalid syntax. current_position: {current_position}, tokens: {tokens}, stack: {stack}"
            )
        return tree

    def _parse_tree(
        self,
        tree: AbstractSyntaxTree | Token,
        tokens: list[Token],
        stack: list[Symbol | Start],
        current_position: int,
    ) -> tuple[AbstractSyntaxTree | Token, list[Symbol | Start], int]:
        top = stack.pop()
        if top == EOS_SYMBOL:
            if tokens[current_position].pos == EOS_SYMBOL:
                current_position += 1
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid syntax at token {tokens[current_position]}")
        elif top == EPSILON_SYMBOL:
            tree.add_child(EPSILON_SYMBOL)  # type: ignore[union-attr,arg-type]
            return tree, stack, current_position
        elif top in self.grammar.terminals:
            if tokens[current_position].pos == top:
                current_position += 1
                tree = tokens[current_position - 1]
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid syntax at token {tokens[current_position]}")
        elif top in self.grammar.non_terminals:
            production = self.grammar.parse_table[top][tokens[current_position].pos]  # type: ignore[index]
            if production is None:
                raise Exception(f"Invalid syntax at token {tokens[current_position]}")
            elif production == EPSILON_SYMBOL:
                tree.add_child(EPSILON_SYMBOL)  # type: ignore[union-attr,arg-type]
                return tree, stack, current_position
            elif isinstance(production, list):
                stack.extend(reversed(production))
                for child in production:
                    child_tree = AbstractSyntaxTree(child)  # type: ignore[arg-type]
                    child_tree, stack, current_position = self._parse_tree(  # type: ignore[arg-type,assignment]
                        child_tree, tokens, stack, current_position
                    )
                    tree.add_child(child_tree)  # type: ignore[union-attr]
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid production: {production}")
        return tree, stack, current_position
