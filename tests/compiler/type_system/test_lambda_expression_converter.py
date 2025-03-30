import unittest
from unittest import TestCase

from compiler.parsing.grammar import Grammar
from compiler.lexer import Lexer
from compiler.parsing.parser import Parser
from compiler.type_system.lambda_expression_converter import (
    LambdaExpressionConverter,
)
from compiler.type_system.typed_lambda_calculus import (
    Variable,
    LambdaAbstraction,
    Arrow,
    Entity,
    TruthValue,
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
            ("c",): (0),
            ("d",): (0),
        }
        arg_definitions = {
            "a": Variable("a"),
            "b": Variable("b"),
            "c": Variable("c"),
            "d": Variable("d"),
            "ε": Variable("ε"),
            "A": Variable("A"),
            "B": Variable("B"),
            "S": Variable("S"),
        }
        converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)
        expr = converter.convert(ast)
        self.assertEqual(
            str(expr),
            "(S (a ((A ε) ((B d) b))))",
        )

    def test_ラムダ式の変換に成功2(self):
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
            ("subject", "V", "object"): ((1, 2), 0),
            ("love",): (0),
        }
        arg_definitions = {
            "subject": Variable("john"),
            "object": Variable("mary"),
            "love": LambdaAbstraction(
                "y",
                Arrow(Entity(), Arrow(Entity(), TruthValue())),
                LambdaAbstraction(
                    "x",
                    Arrow(Entity(), TruthValue()),
                    Variable("like(x, y)"),
                ),
            ),
            "john": Variable("john"),
            "mary": Variable("mary"),
            "ε": Variable("ε"),
            "V": Variable("V"),
            "S": Variable("S"),
        }
        converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)
        expr = converter.convert(ast)
        self.assertEqual(
            str(expr),
            "(S (((V (λy:(e → (e → t)).(λx:(e → t).like(x, y)))) mary) john))",
        )


if __name__ == "__main__":
    unittest.main()
