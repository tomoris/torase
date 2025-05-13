import unittest
from unittest import TestCase

from artificial_language.definition import TruthValue, converter, parser
from compiler.type_system.typed_lambda_calculus import (
    TypedLambdaCalculator,
)


class Test平叙文Parser(TestCase):
    def test_平叙文を受理する(self):
        parser.parse(
            "terminal_declarative_marker:_ terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )

    def test_マーカーなし平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )

    def test_項抽出平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:flower terminal_argument_retriever_end:_"
        )

    def test_限定詞平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_article:the terminal_binary_predicate:flower terminal_argument_retriever_end:_"
        )
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_quantifier:all terminal_binary_predicate:flower terminal_argument_retriever_end:_"
        )

    def test_受動態平叙文を受理する(self):
        parser.parse(
            "terminal_argument_transposition:be-ed terminal_binary_predicate:love terminal_argument:Smith terminal_argument:John"
        )

    def test_過去形や未来形平叙文を受理する(self):
        parser.parse(
            "terminal_tense_marker:will terminal_argument_transposition:be-ed terminal_binary_predicate:love terminal_argument:Smith terminal_argument:John"
        )

    def test_sov平叙文を受理する(self):
        parser.parse(
            "terminal_postfix_binary_predicate_marker:_ terminal_argument:John terminal_argument:Smith terminal_binary_predicate:love"
        )

    def test_orやandを含む平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_operator:or terminal_argument:Smith terminal_argument_operator:or terminal_argument:Wilson terminal_argument:Williams"
        )
        parser.parse(
            "terminal_binary_predicate_operator:or terminal_binary_predicate:love terminal_binary_predicate:like terminal_argument:John terminal_argument:Smith"
        )


class Test疑問文Parser(TestCase):
    def test_真偽疑問文を受理する(self):
        parser.parse(
            "terminal_interrogative_marker:? terminal_binary_predicate:kick terminal_argument:John terminal_argument:ball"
        )

    def test_疑問詞疑問文を受理する(self):
        # who
        parser.parse(
            "terminal_interrogative_marker:who terminal_binary_predicate:love terminal_argument:? terminal_argument:Smith"
        )
        parser.parse(
            "terminal_interrogative_marker:who terminal_binary_predicate:love terminal_argument:John terminal_argument:?"
        )

        # what
        parser.parse(
            "terminal_interrogative_marker:what terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:? terminal_argument_retriever_end:_"
        )

        # when, where, why, how
        # raise NotImplementedError

    def test_選択疑問文を受理する(self):
        # which
        parser.parse(
            "terminal_interrogative_marker:which terminal_binary_predicate:love terminal_argument:John terminal_argument_operator:or terminal_argument:Smith terminal_argument:Wilson"
        )
        parser.parse(
            "terminal_interrogative_marker:which terminal_binary_predicate:love terminal_argument:John terminal_argument_operator:or terminal_argument:Smith terminal_argument_operator:or terminal_argument:Wilson terminal_argument:Williams"
        )
        parser.parse(
            "terminal_interrogative_marker:which terminal_binary_predicate_operator:or terminal_binary_predicate:like terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )
        parser.parse(
            "terminal_interrogative_marker:which terminal_binary_predicate_operator:or terminal_binary_predicate:love terminal_binary_predicate_operator:or terminal_binary_predicate:like terminal_binary_predicate:know terminal_argument:John terminal_argument:Smith"
        )
        parser.parse(
            "terminal_interrogative_marker:which terminal_binary_predicate_operator:or terminal_binary_predicate:love terminal_binary_predicate:like terminal_argument:John terminal_argument_operator:or terminal_argument:Smith terminal_argument:Wilson"
        )


class Test平叙文TypeChecker(TestCase):
    def test_平叙文を型検査が成功する(self):
        calculator = TypedLambdaCalculator()
        ast = parser.parse(
            "terminal_tense_marker:a terminal_aspect_marker:b terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )
        expr = converter.convert(ast)
        self.assertEqual(
            type(calculator.check_type(expr)),
            type(TruthValue()),
        )

        calculator = TypedLambdaCalculator()
        ast = parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )
        expr = converter.convert(ast)
        self.assertEqual(
            type(calculator.check_type(expr)),
            type(TruthValue()),
        )

        calculator = TypedLambdaCalculator()
        ast = parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:flower terminal_argument_retriever_end:_"
        )
        expr = converter.convert(ast)
        self.assertEqual(
            type(calculator.check_type(expr)),
            type(TruthValue()),
        )


if __name__ == "__main__":
    unittest.main()
