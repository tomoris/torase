import unittest
from unittest import TestCase

from compiler.type_system.typed_lambda_calculus import (
    Arrow,
    Constant,
    FunctionApplication,
    If,
    LambdaAbstraction,
    TBool,
    TInt,
    TypedLambdaCalculator,
    TypedLambdaTypeError,
    Variable,
)


class TestTypedLambdaCalculus(TestCase):
    def test_与えられたラムダ項の型検査に成功(self):
        calculator = TypedLambdaCalculator()
        expr = FunctionApplication(
            LambdaAbstraction("x", TInt(), Variable("x")),
            Constant(1),
        )
        self.assertEqual(calculator.check_type(expr), TInt())

        calculator = TypedLambdaCalculator()
        expr = LambdaAbstraction(
            "x",
            TBool(),
            If(
                Variable("x"),
                Constant(False),
                Constant(True),
            ),
        )
        self.assertEqual(calculator.check_type(expr), Arrow(TBool(), TBool()))

    def test_与えられたラムダ項の型検査に失敗(self):
        calculator = TypedLambdaCalculator()
        expr = FunctionApplication(
            LambdaAbstraction("x", TInt(), Variable("x")),
            Constant(True),
        )
        with self.assertRaises(TypedLambdaTypeError):
            calculator.check_type(expr)

        calculator = TypedLambdaCalculator()
        expr = LambdaAbstraction(
            "x",
            TInt(),
            FunctionApplication(
                Variable("x"),
                Constant(1),
            ),
        )
        with self.assertRaises(TypedLambdaTypeError):
            calculator.check_type(expr)

    def test_ラムダ項の評価に成功(self):
        calculator = TypedLambdaCalculator()
        expr = FunctionApplication(
            LambdaAbstraction("x", TInt(), Variable("x")),
            Constant(1),
        )
        result = calculator.evaluate(expr)
        self.assertEqual(result, Constant(1))

        calculator = TypedLambdaCalculator()
        expr = FunctionApplication(
            LambdaAbstraction(
                "x",
                TBool(),
                If(
                    Variable("x"),
                    Constant(False),
                    Constant(True),
                ),
            ),
            Constant(True),
        )

        result = calculator.evaluate(expr)
        self.assertEqual(result, Constant(False))

    def test_ラムダ項の評価に失敗(self):
        calculator = TypedLambdaCalculator()
        expr = FunctionApplication(
            LambdaAbstraction(
                "x",
                TInt(),
                If(
                    Variable("x"),
                    Constant(False),
                    Constant(True),
                ),
            ),
            Variable("y"),
        )
        with self.assertRaises(TypedLambdaTypeError):
            calculator.evaluate(expr)


if __name__ == "__main__":
    unittest.main()
