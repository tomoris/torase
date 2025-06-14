from compiler.lexer import Lexer
from compiler.parsing.grammar import Grammar
from compiler.parsing.parser import Parser

rules = {
    "S": [
        ["declarative_marker", "tam_event"],  # 平叙文
        ["terminal_interrogative_marker", "tam_event"],  # 疑問文
    ],
    #
    "declarative_marker": [["terminal_declarative_marker"], "ε"],
    #
    "tam_event": [
        ["tense_marker", "aspect_marker", "modality_marker", "event"],
        ["terminal_tam_event_operator", "tam_event", "tam_event"],
    ],  # 時制・相・様相
    "tense_marker": [["terminal_tense_marker"], "ε"],
    "aspect_marker": [["terminal_aspect_marker"], "ε"],
    "modality_marker": [["terminal_modality_marker"], "ε"],
    #
    "event": [
        ["argument_not_ε", "argument", "binary_predicate"],
    ],
    "transposed_binary_predicate": [
        ["terminal_argument_transposition", "terminal_binary_predicate"],
        ["terminal_binary_predicate"],
    ],
    "binary_predicate": [
        # ["terminal_binary_predicate"],
        # ["terminal_argument_transposition", "binary_predicate"],  # 項転置
        # [
        #     "terminal_binary_predicate_operator",
        #     "binary_predicate",
        #     "binary_predicate",
        # ],
        [
            "transposed_binary_predicate",
            "binary_predicate",
        ],
        "ε",
    ],  # 二項述語
    "argument": [
        ["argument_not_ε"],
        "ε",
    ],  # 項
    "argument_not_ε": [
        [
            "terminal_argument_retriever",
            "determiner",
            "argument",
            "binary_predicate",
        ],  # 項抽出
        ["terminal_argument"],
        ["terminal_argument_operator", "argument_not_ε", "argument_not_ε"],
        [
            "terminal_tam_event_to_argument_converter",
            "tam_event",
        ],
    ],  # 項
    "determiner": [
        ["quantifier", "article"],
    ],  # 限定詞
    "article": [["terminal_article"], "ε"],  # 冠詞
    "quantifier": [["terminal_quantifier"], "ε"],  # 量詞
}

terminals = {
    "terminal_declarative_marker",  # 平叙文を示すマーカー
    "terminal_interrogative_marker",  # 疑問文を示すマーカー
    #
    "terminal_tam_event_operator",  # eventの接続詞を示す演算子
    #
    "terminal_tense_marker",  # 時制を示すマーカー
    "terminal_aspect_marker",  # 相を示すマーカー
    "terminal_modality_marker",  # 様相を示すマーカー
    #
    "terminal_binary_predicate",  # 二項述語
    "terminal_argument",  # 項
    "terminal_argument_transposition",  # 項転置
    "terminal_argument_retriever",  # 項抽出
    "terminal_tam_event_to_argument_converter",  # eventを項に変換
    #
    "terminal_article",  # 冠詞
    "terminal_quantifier",  # 量詞
    #
    "terminal_argument_operator",  # 項演算子
    # "terminal_binary_predicate_operator",  # 述語演算子
}


grammar = Grammar(rules, terminals)  # type: ignore[arg-type]
lexer = Lexer(terminals)  # type: ignore[arg-type]

parser = Parser(grammar, lexer)
