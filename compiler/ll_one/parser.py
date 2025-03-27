from compiler.ll_one.grammar import Grammar
from compiler.lexer import Lexer
from compiler.my_ast import (
    AbstractSyntaxTree,
    EOS_SYMBOL,
    EPSILON_SYMBOL,
    Start,
    START_SYMBOL,
    Symbol,
    TerminalSymbol,
)


class Parser:
    def __init__(self, grammar: Grammar, lexer: Lexer):
        self.grammar = grammar
        self.lexer = lexer

    def parse(self, sentence: str) -> AbstractSyntaxTree:
        poss, _ = self.lexer.tokenize(sentence)
        poss.append(EOS_SYMBOL)
        stack: list[Symbol | Start] = [EOS_SYMBOL, START_SYMBOL]
        current_position = 0
        tree = AbstractSyntaxTree(START_SYMBOL)

        while stack:
            tree, stack, current_position = self._parse_tree(
                tree, poss, stack, current_position
            )

        if current_position != len(poss) or len(stack) != 0:
            raise Exception(
                f"Invalid syntax. current_position: {current_position}, poss: {poss}, stack: {stack}"
            )
        return tree

    def _parse_tree(
        self,
        tree: AbstractSyntaxTree,
        poss: list[TerminalSymbol],
        stack: list[Symbol | Start],
        current_position: int,
    ) -> tuple[AbstractSyntaxTree | TerminalSymbol, list[Symbol | Start], int]:
        top = stack.pop()
        if top == EOS_SYMBOL:
            if poss[current_position] == EOS_SYMBOL:
                current_position += 1
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid syntax at token {poss[current_position]}")
        elif top == EPSILON_SYMBOL:
            tree.add_child(EPSILON_SYMBOL)
            return tree, stack, current_position
        elif top in self.grammar.terminals:
            if poss[current_position] == top:
                current_position += 1
                tree = top
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid syntax at token {poss[current_position]}")
        elif top in self.grammar.non_terminals:
            production = self.grammar.parse_table[top][poss[current_position]]
            if production is None:
                raise Exception(f"Invalid syntax at token {poss[current_position]}")
            elif production == EPSILON_SYMBOL:
                tree.add_child(EPSILON_SYMBOL)
                return tree, stack, current_position
            elif isinstance(production, list):
                stack.extend(reversed(production))
                for child in production:
                    child_tree = AbstractSyntaxTree(child)
                    child_tree, stack, current_position = self._parse_tree(
                        child_tree, poss, stack, current_position
                    )
                    tree.add_child(child_tree)
                return tree, stack, current_position
            else:
                raise Exception(f"Invalid production: {production}")
        return tree, stack, current_position
