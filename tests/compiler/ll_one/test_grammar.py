import unittest
from unittest import TestCase

from compiler.ll_one.grammar import Grammar


class TestGrammar(TestCase):
    def test_ll1言語で受理できる文法の場合に受理される1(self):
        # https://www.youtube.com/watch?v=DT-cbznw9aY
        rules = {
            "S": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"ID", "LPAREN"},
                "E_prime": {"ε", "PLUS"},
                "T": {"ID", "LPAREN"},
                "T_prime": {"ε", "TIMES"},
                "F": {"ID", "LPAREN"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$", "RPAREN"},
                "E_prime": {"$", "RPAREN"},
                "T": {"PLUS", "$", "RPAREN"},
                "T_prime": {"PLUS", "$", "RPAREN"},
                "F": {"PLUS", "$", "RPAREN", "TIMES"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "ID": ["T", "E_prime"],
                    "LPAREN": ["T", "E_prime"],
                    "RPAREN": None,
                    "PLUS": None,
                    "TIMES": None,
                    "$": None,
                },
                "E_prime": {
                    "ID": None,
                    "LPAREN": None,
                    "RPAREN": "ε",
                    "PLUS": ["PLUS", "T", "E_prime"],
                    "TIMES": None,
                    "$": "ε",
                },
                "T": {
                    "ID": ["F", "T_prime"],
                    "LPAREN": ["F", "T_prime"],
                    "RPAREN": None,
                    "PLUS": None,
                    "TIMES": None,
                    "$": None,
                },
                "T_prime": {
                    "ID": None,
                    "LPAREN": None,
                    "RPAREN": "ε",
                    "PLUS": "ε",
                    "TIMES": ["TIMES", "F", "T_prime"],
                    "$": "ε",
                },
                "F": {
                    "ID": ["ID"],
                    "LPAREN": ["LPAREN", "S", "RPAREN"],
                    "RPAREN": None,
                    "PLUS": None,
                    "TIMES": None,
                    "$": None,
                },
            },
        )

    def test_ll1言語で受理できない文法の場合に例外処理が投げられる1(self):
        # https://youtu.be/5P7ehgZ6EIs?t=335
        rules = {
            "S": [["a", "S", "b", "S"], ["b", "S", "a", "S"], "ε"],
        }
        terminals = {"a", "b"}
        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_ll1言語で受理できる文法の場合に受理される2(self):
        # https://youtu.be/5P7ehgZ6EIs?t=493
        rules = {
            "S": [["(", "S", ")"], "ε"],
        }
        terminals = {"(", ")"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"(", "ε"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$", ")"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "(": ["(", "S", ")"],
                    ")": "ε",
                    "$": "ε",
                },
            },
        )

    def test_ll1言語で受理できる文法の場合に受理される3(self):
        # https://youtu.be/5P7ehgZ6EIs?t=784
        rules = {
            "S": [["A", "a", "A", "b"], ["B", "b", "B", "a"]],
            "A": "ε",
            "B": "ε",
        }
        terminals = {"a", "b"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"a", "b"},
                "A": {"ε"},
                "B": {"ε"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$"},
                "A": {"a", "b"},
                "B": {"b", "a"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "a": ["A", "a", "A", "b"],
                    "b": ["B", "b", "B", "a"],
                    "$": None,
                },
                "A": {
                    "a": "ε",
                    "b": "ε",
                    "$": None,
                },
                "B": {
                    "a": "ε",
                    "b": "ε",
                    "$": None,
                },
            },
        )

    def test_ll1言語で受理できない文法の場合に例外処理が投げられる2(self):
        # https://youtu.be/5P7ehgZ6EIs?t=895
        rules = {
            "S": [["A"], ["a"]],
            "A": [["a"]],
        }
        terminals = {"a"}
        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_ll1言語で受理できる文法の場合に受理される4(self):
        # https://youtu.be/5P7ehgZ6EIs?t=1150
        rules = {
            "S": [["a", "B"], "ε"],
            "B": [["b", "C"], "ε"],
            "C": [["c", "S"], "ε"],
        }
        terminals = {"a", "b", "c"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"a", "ε"},
                "B": {"b", "ε"},
                "C": {"c", "ε"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$"},
                "B": {"$"},
                "C": {"$"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "a": ["a", "B"],
                    "b": None,
                    "c": None,
                    "$": "ε",
                },
                "B": {
                    "a": None,
                    "b": ["b", "C"],
                    "c": None,
                    "$": "ε",
                },
                "C": {
                    "a": None,
                    "b": None,
                    "c": ["c", "S"],
                    "$": "ε",
                },
            },
        )

    def test_ll1言語で受理できる文法の場合に受理される5(self):
        # https://youtu.be/INbA4Wr6dXs?t=364
        rules = {
            "S": [["A", "B"]],
            "A": [["a"], "ε"],
            "B": [["b"], "ε"],
        }
        terminals = {"a", "b"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"a", "b", "ε"},
                "A": {"a", "ε"},
                "B": {"b", "ε"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$"},
                "A": {"b", "$"},
                "B": {"$"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "a": ["A", "B"],
                    "b": ["A", "B"],
                    "$": ["A", "B"],
                },
                "A": {
                    "a": ["a"],
                    "b": "ε",
                    "$": "ε",
                },
                "B": {
                    "a": None,
                    "b": ["b"],
                    "$": "ε",
                },
            },
        )

    def test_ll1言語で受理できない文法の場合に例外処理が投げられる3(self):
        # https://youtu.be/INbA4Wr6dXs?t=588
        rules = {
            "S": [["a", "S", "A"], "ε"],
            "A": [["c"], "ε"],
        }
        terminals = {"a", "c"}
        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_ll1言語で受理できる文法の場合に受理される6(self):
        # https://youtu.be/INbA4Wr6dXs?t=908
        rules = {
            "S": [["A"]],
            "A": [["B", "b"], ["C", "d"]],
            "B": [["a", "B"], "ε"],
            "C": [["c", "C"], "ε"],
        }
        terminals = {"a", "b", "c", "d"}
        grammar = Grammar(rules=rules, terminals=terminals)

        self.assertEqual(
            grammar.first_sets,
            {
                "S": {"a", "b", "c", "d"},
                "A": {"a", "b", "c", "d"},
                "B": {"a", "ε"},
                "C": {"c", "ε"},
            },
        )
        self.assertEqual(
            grammar.follow_sets,
            {
                "S": {"$"},
                "A": {"$"},
                "B": {"b"},
                "C": {"d"},
            },
        )
        self.assertEqual(
            grammar.parse_table,
            {
                "S": {
                    "a": ["A"],
                    "b": ["A"],
                    "c": ["A"],
                    "d": ["A"],
                    "$": None,
                },
                "A": {
                    "a": ["B", "b"],
                    "b": ["B", "b"],
                    "c": ["C", "d"],
                    "d": ["C", "d"],
                    "$": None,
                },
                "B": {
                    "a": ["a", "B"],
                    "b": "ε",
                    "c": None,
                    "d": None,
                    "$": None,
                },
                "C": {
                    "a": None,
                    "b": None,
                    "c": ["c", "C"],
                    "d": "ε",
                    "$": None,
                },
            },
        )

    def test_ll1言語で受理できない文法の場合に例外処理が投げられる4(self):
        # https://youtu.be/INbA4Wr6dXs?t=1100
        rules = {
            "S": [["a", "A", "a"], "ε"],
            "A": [["a", "b", "S"], "ε"],
        }
        terminals = {"a", "c"}
        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_不必要な終端記号がある場合に例外が投げられる(self):
        rules = {
            "S": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID", "蛇足"}

        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_開始記号がない場合に例外が投げられる(self):
        rules = {
            "E": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID"}

        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_空文字の記号が終端記号に含まれる場合に例外が投げられる(self):
        rules = {
            "S": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID", "ε"}

        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_空文字の記号がルールの左辺に含まれる場合に例外が投げられる(self):
        rules = {
            "S": [["T", "E_prime"]],
            "E_prime": [["PLUS", "T", "E_prime"], "ε"],
            "T": [["F", "T_prime"]],
            "T_prime": [["TIMES", "F", "T_prime"], "ε"],
            "F": [["LPAREN", "S", "RPAREN"], ["ID"]],
            "ε": [["PLUS"]],
        }
        terminals = {"PLUS", "TIMES", "LPAREN", "RPAREN", "ID"}

        with self.assertRaises(ValueError):
            Grammar(rules=rules, terminals=terminals)

    def test_artificial_language(self):

        rules = {
            "S": [["subject", "verb_phrase", "object"]],
            "subject": [["noun_phrase"], ["S_preposition", "S"]],
            "verb_phrase": [["verb"], ["auxiliary_verb", "verb"]],
            "object": [["noun_phrase"], ["S_preposition", "S"]],
            "noun_phrase": [["compound_preposition", "noun", "noun_phrase"], ["noun"]],
            "noun": [["nominalization_preposition", "verb"]],
        }
        terminals = {
            "compound_preposition",
            "nominalization_preposition",
            "S_preposition",
            "verb",
            "auxiliary_verb",
        }

        Grammar(rules=rules, terminals=terminals)


if __name__ == "__main__":
    unittest.main()
