from compiler.ll_one.grammar import Grammar
from compiler.lexer import Lexer
from compiler.ll_one.parser import Parser

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
        ["binary_predicate", "argument_not_ε", "argument"],
        [
            "terminal_postfix_binary_predicate_marker",
            "argument_not_ε",
            "argument",
            "binary_predicate",
        ],
    ],
    "binary_predicate": [
        ["terminal_binary_predicate"],
        ["terminal_argument_transposition", "binary_predicate"],  # 項転置
        ["terminal_binary_predicate_operator", "binary_predicate", "binary_predicate"],
    ],  # 二項述語
    "argument": [
        ["argument_not_ε"],
        "ε",
    ],  # 項
    "argument_not_ε": [
        ["determiner", "retrieved_argument"],
        ["terminal_argument"],
        ["terminal_argument_operator", "argument_not_ε", "argument_not_ε"],
    ],  # 項
    "retrieved_argument": [
        [
            "terminal_argument_retrieval",
            "argument",
            "binary_predicate",
        ],
    ],  # 項抽出
    "determiner": [["article", "quantifier"]],  # 限定詞
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
    "terminal_argument_retrieval",  # 項抽出
    #
    "terminal_article",  # 冠詞
    "terminal_quantifier",  # 量詞
    #
    "terminal_argument_operator",  # 項演算子
    "terminal_binary_predicate_operator",  # 述語演算子
    #
    "terminal_postfix_binary_predicate_marker",  # 二項述語を後置するマーカー
}

grammar = Grammar(rules, terminals)  # type: ignore[arg-type]
lexer = Lexer(terminals)  # type: ignore[arg-type]

parser = Parser(grammar, lexer)
