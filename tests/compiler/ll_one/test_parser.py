import unittest
from unittest import TestCase

from compiler.ll_one.grammar import Grammar
from compiler.lexer import Lexer
from compiler.ll_one.parser import Parser


class TestParser(TestCase):
    def test_与えられたll1文法で受理できる文の場合に受理される1(self):
        # https://youtu.be/clkHOgZUGWU?t=566
        rules = {
            "S": [["a", "A", "B", "b"]],
            "A": [["c"], "ε"],
            "B": [["d"], "ε"],
        }
        terminals = {"a", "b", "c", "d"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        ast = parser.parse("a:aa d:dd b:bb")
        self.assertEqual(
            str(ast),
            "(S -> ['a', (A -> ε), (B -> ['d']), 'b'])",
        )

    def test_与えられたll1文法で受理できる文の場合に受理される2(self):
        rules = {
            "S": [["a", "A", "B", "b"]],
            "A": [["c"], "ε"],
            "B": [["d"], "ε"],
        }
        terminals = {"a", "b", "c", "d"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        ast = parser.parse("a:aa c:cc b:bb")
        self.assertEqual(
            str(ast),
            "(S -> ['a', (A -> ['c']), (B -> ε), 'b'])",
        )

    def test_与えられたll1文法で受理できる文の場合に受理される3(self):
        rules = {
            "S": [["a", "A", "B", "b"]],
            "A": [["c"], "ε"],
            "B": [["d"], "ε"],
        }
        terminals = {"a", "b", "c", "d"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        ast = parser.parse("a:aa c:cc d:dd b:bb")
        self.assertEqual(
            str(ast),
            "(S -> ['a', (A -> ['c']), (B -> ['d']), 'b'])",
        )

    def test_与えられたll1文法で受理できる文の場合に受理される4(self):
        rules = {
            "S": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        ast = parser.parse("ID:id PLUS:plus ID:id TIMES:times ID:id")
        self.assertEqual(
            str(ast),
            "(S -> [(T -> [(F -> ['ID']), (T_prime -> ε)]), (E_prime -> ['PLUS', (T -> [(F -> ['ID']), (T_prime -> ['TIMES', (F -> ['ID']), (T_prime -> ε)])]), (E_prime -> ε)])])",
        )

    def test_与えられたll1文法で受理できない文の場合に例外が投げられる1(self):
        # https://youtu.be/clkHOgZUGWU?t=566
        rules = {
            "S": [["a", "A", "B", "b"]],
            "A": [["c"], "ε"],
            "B": [["d"], "ε"],
        }
        terminals = {"a", "b", "c", "d"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        with self.assertRaises(Exception):
            parser.parse("a:aa d:dd c:cc b:bb")


if __name__ == "__main__":
    unittest.main()
