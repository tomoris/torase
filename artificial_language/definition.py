from compiler.lexer import Lexer
from compiler.parsing.grammar import Grammar
from compiler.parsing.parser import Parser
from compiler.type_system.lambda_expression_converter import (
    LambdaExpressionConverter,
)
from compiler.type_system.typed_lambda_calculus import (
    Arrow,
    Constant,
    Entity,
    LambdaAbstraction,
    TruthValue,
    Variable,
)

rules = {
    "S": [
        ["declarative_sentence"],
        ["interrogative_sentence"],
        ["imperative_sentence"],
    ],
    #
    "declarative_sentence": [
        ["declarative_marker", "tense_marker", "aspect_marker", "event"]
    ],  # 平叙文
    "interrogative_sentence": [
        ["terminal_interrogative_marker", "tense_marker", "aspect_marker", "event"]
    ],  # 疑問文
    "imperative_sentence": [["terminal_imperative_marker", "event"]],  # 命令文
    #
    "declarative_marker": [["terminal_declarative_marker"], "ε"],
    #
    "tense_marker": [["terminal_tense_marker"], "ε"],
    "aspect_marker": [["terminal_aspect_marker"], "ε"],
    #
    "event": [
        ["app_binary_predicate", "argument"],
        [
            "terminal_postfix_binary_predicate_marker",
            "postfix_event",
        ],
    ],
    "postfix_binary_predicate": [["empty_argument_transposition", "binary_predicate"]],
    "app_postfix_binary_predicate": [
        [
            "argument",
            "postfix_binary_predicate",
        ],
    ],
    "postfix_event": [
        [
            "argument_not_ε",
            "app_postfix_binary_predicate",
        ],
    ],
    "app_binary_predicate": [["binary_predicate", "argument_not_ε"]],
    "binary_predicate": [
        ["terminal_binary_predicate"],
        ["terminal_argument_transposition", "binary_predicate"],  # 項転置
        ["binary_predicate_operator", "binary_predicate"],
    ],  # 二項述語
    "binary_predicate_operator": [
        ["terminal_binary_predicate_operator", "binary_predicate"]
    ],
    "empty_argument_transposition": [
        "ε",
    ],
    "argument": [
        ["argument_not_ε"],
        "ε",
    ],  # 項
    "argument_not_ε": [
        ["argument_retriever", "binary_predicate"],  # 項抽出
        ["terminal_argument"],
        ["argument_operator", "argument_not_ε"],
    ],  # 項
    "argument_operator": [["terminal_argument_operator", "argument_not_ε"]],
    "argument_retriever_with_quantifier": [
        [
            "terminal_argument_retriever",
            "quantifier",
        ],
    ],
    "argument_retriever_with_quantifier_and_article": [
        [
            "argument_retriever_with_quantifier",
            "article",
        ]
    ],
    "argument_retriever": [
        [
            "argument_retriever_with_quantifier_and_article",
            "argument",
        ]
    ],
    "article": [["terminal_article"], "ε"],  # 冠詞
    "quantifier": [["terminal_quantifier"], "ε"],  # 量詞
}

terminals = {
    "terminal_declarative_marker",  # 平叙文を示すマーカー
    "terminal_interrogative_marker",  # 疑問文を示すマーカー
    "terminal_imperative_marker",  # 命令文を示すマーカー
    #
    "terminal_tense_marker",  # 時制を示すマーカー
    "terminal_aspect_marker",  # 相を示すマーカー
    #
    "terminal_binary_predicate",  # 二項述語
    "terminal_argument",  # 項
    "terminal_argument_transposition",  # 項転置
    "terminal_argument_retriever",  # 項抽出
    #
    "terminal_article",  # 冠詞
    "terminal_quantifier",  # 量詞
    #
    "terminal_argument_operator",  # 項演算子
    "terminal_binary_predicate_operator",  # 述語演算子
    #
    "terminal_postfix_binary_predicate_marker",  # 二項述語を後置するマーカー
}

app_priorities = {
    ("declarative_marker", "tense_marker", "aspect_marker", "event"): (0, (1, (2, 3))),
    ("terminal_interrogative_marker", "tense_marker", "aspect_marker", "event"): (
        0,
        (1, (2, 3)),
    ),
    ("terminal_imperative_marker", "event"): (0, 1),
    ("app_binary_predicate", "argument"): (0, 1),
    ("binary_predicate", "argument_not_ε"): (0, 1),
    ("terminal_postfix_binary_predicate_marker", "postfix_event"): (0, 1),
    ("empty_argument_transposition", "binary_predicate"): (0, 1),
    ("argument_not_ε", "app_postfix_binary_predicate"): (1, 0),
    ("argument", "postfix_binary_predicate"): (1, 0),
    ("terminal_argument_transposition", "binary_predicate"): (0, 1),
    ("binary_predicate_operator", "binary_predicate"): (0, 1),
    ("terminal_binary_predicate_operator", "binary_predicate"): (0, 1),
    ("argument_operator", "argument_not_ε"): (0, 1),
    ("terminal_argument_operator", "argument_not_ε"): (0, 1),
    ("argument_retriever", "binary_predicate"): (0, 1),
    ("argument_retriever_with_quantifier_and_article", "argument"): (0, 1),
    ("argument_retriever_with_quantifier", "article"): (0, 1),
    ("terminal_argument_retriever", "quantifier"): (0, 1),
}
arg_definitions = {
    "S": LambdaAbstraction("S__x", TruthValue(), Variable("S__x")),
    "declarative_sentence": LambdaAbstraction(
        "declarative_sentence__x",
        Entity(),
        Constant(TruthValue()),
    ),
    "interrogative_sentence": LambdaAbstraction(
        "interrogative_sentence__x",
        Entity(),
        Constant(TruthValue()),
    ),
    "imperative_sentence": Variable("?"),
    "declarative_marker": LambdaAbstraction(
        "declarative_marker__x",
        Entity(),
        LambdaAbstraction(
            "declarative_marker__y",
            Entity(),
            Variable("declarative_marker__y"),
        ),
    ),
    "tense_marker": LambdaAbstraction(
        "tense_marker__x",
        Entity(),
        LambdaAbstraction(
            "tense_marker__y",
            Entity(),
            Variable("tense_marker__y"),
        ),
    ),
    "aspect_marker": LambdaAbstraction(
        "aspect_marker__x",
        Entity(),
        LambdaAbstraction(
            "aspect_marker__y",
            Entity(),
            Variable("aspect_marker__y"),
        ),
    ),
    "event": LambdaAbstraction("event__x", Entity(), Constant(Entity())),
    "postfix_event": Constant(Entity()),
    "postfix_binary_predicate": Constant(Entity()),
    "app_postfix_binary_predicate": Constant(Entity()),
    "empty_argument_transposition": Constant(Entity()),
    "app_binary_predicate": LambdaAbstraction(
        "app_binary_predicate__x",
        Arrow(Entity(), Entity()),
        LambdaAbstraction(
            "app_binary_predicate__y",
            Entity(),
            Variable("app_binary_predicate__y"),
        ),
    ),
    "binary_predicate": LambdaAbstraction(
        "binary_predicate__x",
        Arrow(Entity(), Arrow(Entity(), Entity())),
        LambdaAbstraction(
            "binary_predicate__y",
            Entity(),
            Variable("binary_predicate__y"),
        ),
    ),
    "argument_not_ε": LambdaAbstraction(
        "argument_not_ε__x",
        Entity(),
        Variable("argument_not_ε__x"),
    ),
    "argument": LambdaAbstraction(
        "argument__x",
        Entity(),
        Variable("argument__x"),
    ),
    "retrieved_argument": LambdaAbstraction(
        "retrieved_argument__x",
        Entity(),
        Variable("retrieved_argument__x"),
    ),
    "article": LambdaAbstraction(
        "article__x",
        Entity(),
        Variable("article__x"),
    ),
    "quantifier": LambdaAbstraction(
        "quantifier__x",
        Entity(),
        Variable("quantifier__x"),
    ),
    # terminal
    "terminal_declarative_marker": Constant("?"),
    "terminal_interrogative_marker": Constant("?"),
    "terminal_imperative_marker": Constant("?"),
    "terminal_tense_marker": Constant(Entity()),
    "terminal_aspect_marker": Constant(Entity()),
    # "terminal_binary_predicate": LambdaAbstraction(
    #     "x",
    #     Arrow(
    #         Arrow(Entity(), Arrow(Entity(), Entity())),
    #         Arrow(Entity(), Arrow(Entity(), Entity())),
    #     ),
    #     Variable("x"),
    # ),
    "terminal_binary_predicate": LambdaAbstraction(
        "terminal_binary_predicate__x",
        Entity(),
        LambdaAbstraction(
            "terminal_binary_predicate__y",
            Entity(),
            Variable("terminal_binary_predicate__y"),
        ),
    ),
    # "terminal_argument": LambdaAbstraction(
    #     "x",
    #     Arrow(Entity(), Entity()),
    #     Variable("x"),
    # ),
    "terminal_argument": Constant(Entity()),
    "terminal_argument_transposition": LambdaAbstraction(
        "terminal_argument_transposition__x",
        Arrow(Entity(), Arrow(Entity(), Entity())),
        Constant(Arrow(Entity(), Arrow(Entity(), Entity()))),
    ),
    # "terminal_argument_retriever": LambdaAbstraction(
    #     "terminal_argument_retriever__x",
    #     Arrow(Entity(), Arrow(Entity(), Entity())),
    #     LambdaAbstraction(
    #         "terminal_argument_retriever__y",
    #         Entity(),
    #         Variable("terminal_argument_retriever__y"),
    #     ),
    # ),
    "argument_retriever": LambdaAbstraction(
        "argument_retriever__x",
        Entity(),
        LambdaAbstraction(
            "argument_retriever__y",
            Arrow(Entity(), Arrow(Entity(), Entity())),
            Constant(Entity()),
        ),
    ),
    "argument_retriever_with_quantifier_and_article": LambdaAbstraction(
        "terminal_argument_retriever_with_quantifier_and_article__x",
        Entity(),
        LambdaAbstraction(
            "terminal_argument_retriever_with_quantifier_and_article__y",
            Entity(),
            Variable("terminal_argument_retriever_with_quantifier_and_article__y"),
        ),
    ),
    "argument_retriever_with_quantifier": LambdaAbstraction(
        "terminal_argument_retriever_with_quantifier__x",
        Entity(),
        LambdaAbstraction(
            "terminal_argument_retriever_with_quantifier__y",
            Entity(),
            Variable("terminal_argument_retriever_with_quantifier__y"),
        ),
    ),
    "terminal_argument_retriever": LambdaAbstraction(
        "terminal_argument_retriever__x",
        Entity(),
        Variable("terminal_argument_retriever__x"),
    ),
    "terminal_article": Constant("?"),
    "terminal_quantifier": Constant("?"),
    "argument_operator": LambdaAbstraction(
        "argument_operator__x",
        Entity(),
        LambdaAbstraction(
            "argument_operator__y",
            Entity(),
            Constant(Arrow(Entity(), Entity())),
        ),
    ),
    "binary_predicate_operator": LambdaAbstraction(
        "binary_predicate_operator__x",
        Entity(),
        LambdaAbstraction(
            "binary_predicate_operator__y",
            Entity(),
            Constant(Arrow(Entity(), Entity())),
        ),
    ),
    "terminal_argument_operator": LambdaAbstraction(
        "terminal_argument_operator__x",
        Entity(),
        LambdaAbstraction(
            "terminal_argument_operator__y",
            Entity(),
            Constant(Arrow(Entity(), Entity())),
        ),
    ),
    "terminal_binary_predicate_operator": Constant("?"),
    "terminal_postfix_binary_predicate_marker": Constant("?"),
    "ε": Constant(Entity()),  # Constant("ε"),
}

grammar = Grammar(rules, terminals)  # type: ignore[arg-type]
lexer = Lexer(terminals)  # type: ignore[arg-type]

parser = Parser(grammar, lexer)
converter = LambdaExpressionConverter(app_priorities, arg_definitions, grammar)  # type: ignore[arg-type]
