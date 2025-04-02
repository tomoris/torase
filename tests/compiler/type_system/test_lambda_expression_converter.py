import unittest
from unittest import TestCase

from compiler.lexer import Lexer
from compiler.parsing.grammar import Grammar
from compiler.parsing.parser import Parser
from compiler.type_system.lambda_expression_converter import (
    LambdaExpressionConverter,
)
from compiler.type_system.typed_lambda_calculus import (
    Constant,
    Entity,
    LambdaAbstraction,
    TruthValue,
    Variable,
)


class TestLambdaExpressionConverter(TestCase):
    def test_ラムダ式の変換に成功1(self):
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

        app_priorities = {
            ("a", "A", "B", "b"): (0, (1, (2, 3))),
        }
        arg_definitions = {
            "a": Variable("a"),
            "b": Variable("b"),
            "c": Variable("c"),
            "d": Variable("d"),
            "A": Variable("A"),
            "B": Variable("B"),
        }
        converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)
        expr = converter.convert(ast)
        self.assertEqual(
            str(expr),
            "(a (A (d b)))",
        )

    def test_ラムダ式の変換に成功2(self):
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

        app_priorities = {
            ("a", "A", "B", "b"): (0, (1, (3, 2))),
        }
        arg_definitions = {
            "a": Variable("a"),
            "b": Variable("b"),
            "c": Variable("c"),
            "d": Variable("d"),
            "A": Variable("A"),
            "B": Variable("B"),
        }
        converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)
        expr = converter.convert(ast)
        self.assertEqual(
            str(expr),
            "(a (A (b d)))",
        )

    def test_ラムダ式の変換に成功3(self):
        rules = {
            "S": [["subject", "V", "object"]],
            "V": [["love"], "ε"],
        }
        terminals = {"subject", "object", "love"}
        grammar = Grammar(rules, terminals)
        lexer = Lexer(terminals)

        parser = Parser(grammar, lexer)
        ast = parser.parse("subject:john love:愛 object:mary")

        app_priorities = {
            ("subject", "V", "object"): ((1, 2), 0),  # ((V object) subject)
        }
        arg_definitions = {
            "subject": Constant(Entity()),
            "object": Constant(Entity()),
            "love": LambdaAbstraction(
                "y",
                Entity(),
                LambdaAbstraction(
                    "x",
                    Entity(),
                    Constant(TruthValue()),
                ),
            ),
            "V": LambdaAbstraction(
                "y",
                Entity(),
                LambdaAbstraction(
                    "x",
                    Entity(),
                    Constant(TruthValue()),
                ),
            ),
        }
        converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)
        expr = converter.convert(ast)
        self.assertEqual(
            str(expr),
            "(((λy:e.(λx:e.t)) e) e)",
        )


if __name__ == "__main__":
    unittest.main()
