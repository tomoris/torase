from dataclasses import dataclass
from typing import Literal

from compiler.lexer import Lexer
from compiler.parsing.grammar import Grammar
from compiler.parsing.my_ast import (
    EPSILON_SYMBOL,
    AbstractSyntaxTree,
)
from compiler.parsing.parser import Parser
from compiler.type_system.typed_lambda_calculus import (
    MyArgument,
    MyType,
)

rules = {
    "S": [
        ["declarative_marker", "tam_event"],  # 平叙文
        ["terminal_interrogative_marker", "tam_event"],  # 疑問文
    ],
    #
    "declarative_marker": [["terminal_declarative_marker"], "ε"],
    #
    "tam_event": [
        [
            "mta_event",
            "tam_event_post_modifier",
        ],
        ["terminal_tam_event_binary_operator", "tam_event", "tam_event"],
        ["terminal_tam_event_unary_operator", "tam_event"],
    ],  # 時制・相・様相
    "mta_event": [
        ["modality_expr", "ta_event"],
    ],
    "ta_event": [
        ["tense_expr", "a_event"],
    ],
    "a_event": [
        ["aspect_expr", "event"],
    ],
    "tam_event_post_modifier": [
        ["terminal_tam_event_post_modifier", "tam_event"],
        "ε",
    ],  # タグ
    "tense_expr": [
        ["tense_marker"],
        "ε",
    ],
    "tense_marker": [
        ["terminal_tense_marker"],
        ["terminal_tense_negation_begin", "terminal_negation", "tense_marker"],
    ],
    "aspect_expr": [
        ["aspect_marker"],
        "ε",
    ],
    "aspect_marker": [
        ["terminal_aspect_marker"],
        ["terminal_aspect_negation_begin", "terminal_negation", "aspect_marker"],
    ],
    "modality_expr": [
        ["modality_marker_with_agent"],
        "ε",
    ],
    "modality_marker_with_agent": [
        ["modality_marker", "modality_agent"],
    ],
    "modality_marker": [
        ["terminal_modality_marker"],
        [
            "terminal_modality_negation_begin",
            "terminal_negation",
            "modality_marker",
        ],
    ],
    "modality_agent": [
        ["terminal_post_modifier", "argument"],
        "ε",
    ],  # 様相の主語
    #
    "event": [
        [
            "post_modified_binary_predicate",
            "argument",
            "argument_or_ε",
        ],
    ],
    "post_modified_binary_predicate": [
        [
            "binary_predicate",
            "predicate_post_modifier",
        ],
    ],  # 後置修飾された二項述語
    "binary_predicate": [
        ["terminal_binary_predicate"],
        [
            "terminal_argument_transposition",
            "binary_predicate",
        ],  # 項転置
        [
            "terminal_binary_predicate_operator",
            "binary_predicate",
            "binary_predicate",
        ],
        [
            "terminal_negation",
            "binary_predicate",
        ],  # 否定
    ],  # 二項述語
    "predicate_post_modifier": [
        ["terminal_post_modifier", "binary_predicate"],
        "ε",
    ],  # 修飾
    "argument_or_ε": [
        ["terminal_argument_separator", "argument"],
        "ε",
    ],  # 項またはε
    "argument": [
        [
            "terminal_argument_retriever",
            "determiner",
            "post_modified_binary_predicate",
            "post_modified_argument_retriever",
        ],  # 項抽出
        ["terminal_argument"],
        ["terminal_argument_operator", "argument", "argument"],
        [
            "terminal_tam_event_to_argument_converter",
            "tam_event",
            "terminal_tam_event_to_argument_converter_end",
        ],
        ["terminal_negation", "argument"],  # 否定
    ],  # 項
    "post_modified_argument_retriever": [
        [
            "terminal_post_modified_argument_retriever",
            "argument",
        ],
        "ε",
    ],
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
    "terminal_tam_event_binary_operator",  # eventの接続詞を示す２項演算子
    "terminal_tam_event_unary_operator",  # eventの接続詞を示す単項演算子
    "terminal_tam_event_post_modifier",  # eventの後置修飾子
    #
    "terminal_tense_marker",  # 時制を示すマーカー
    "terminal_aspect_marker",  # 相を示すマーカー
    "terminal_modality_marker",  # 様相を示すマーカー
    #
    "terminal_binary_predicate",  # 二項述語
    "terminal_argument",  # 項
    "terminal_argument_separator",  # 項の区切り
    "terminal_argument_transposition",  # 項転置
    "terminal_tam_event_to_argument_converter",  # eventを項に変換
    "terminal_tam_event_to_argument_converter_end",  # eventを項に変換
    #
    "terminal_article",  # 冠詞
    "terminal_quantifier",  # 量詞
    #
    "terminal_argument_operator",  # 項演算子
    "terminal_binary_predicate_operator",  # 述語演算子
    "terminal_post_modifier",  # 述語の後置修飾子
    "terminal_argument_retriever",  # 項抽出
    "terminal_post_modified_argument_retriever",
    #
    "terminal_negation",  # 否定マーカー
    "terminal_tense_negation_begin",
    "terminal_aspect_negation_begin",
    "terminal_modality_negation_begin",
}


def include_type(type1: MyType, type2: MyType) -> bool:
    return True


class TruthValue(MyType):
    def __str__(self):
        return "t"


class Bottom(MyType):
    def __str__(self):
        return "ε"


@dataclass
class Arg(MyArgument):
    arg_type: MyType

    def calc_type(self):
        return self.arg_type


@dataclass
class BinaryPredicate(MyArgument):
    type1: MyType
    type2: MyType

    def calc_type(self):
        raise ValueError("BinaryPredicate should not be used directly")


@dataclass
class Event(MyArgument):
    binary_predicate: BinaryPredicate
    argument_1: Arg
    argument_2: Arg

    def calc_type(self):
        if not include_type(self.binary_predicate.type1, self.argument_1.calc_type()):
            raise ValueError(
                f"Type mismatch: {self.binary_predicate.type1} and {self.argument_1.calc_type()}"
            )
        if isinstance(self.binary_predicate.type2, Bottom):
            if not include_type(
                self.binary_predicate.type2, self.argument_2.calc_type()
            ):
                raise ValueError(
                    f"Type mismatch: {self.binary_predicate.type2} and {self.argument_2.calc_type()}"
                )

        return TruthValue()


@dataclass
class TamEvent(MyArgument):
    tense: MyArgument | None
    aspect: MyArgument | None
    modality: MyArgument | None
    event: Event

    def calc_type(self) -> MyType:
        return self.event.calc_type()


@dataclass
class SentenceType(MyArgument):
    name: Literal["declarative", "interrogative"]

    def calc_type(self) -> MyType:
        raise NotImplementedError("Subclasses must implement calc_type() method")


@dataclass
class S(MyArgument):
    sentence_type: SentenceType
    tam_event: TamEvent

    def calc_type(self):
        return self.tam_event.calc_type()


class MyLambdaExpressionConverter:
    def convert(self, ast: AbstractSyntaxTree) -> MyArgument:
        match ast.non_terminal_symbol:
            case "S":
                return self.convert_S(ast.children)
            case "declarative_marker":
                return self.convert_declarative_marker(ast.children)
            case "tam_event":
                return self.convert_tam_event(ast.children)
            case "tense_marker":
                raise NotImplementedError
            case "aspect_marker":
                raise NotImplementedError
            case "modality_marker":
                raise NotImplementedError
            case "event":
                return self.convert_event(ast.children)
            case "binary_predicate":
                return self.convert_binary_predicate(ast.children)
            case "argument":
                return self.convert_argument(ast.children)
            case "argument_not_ε":
                return self.convert_argument_not_ε(ast.children)
            case _:
                raise ValueError(
                    f"Unknown non-terminal symbol: {ast.non_terminal_symbol}"
                )

    def convert_S(self, children) -> S:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case ("declarative_marker", "tam_event"):
                    sentence_type = self.convert_declarative_marker(
                        children[0].children
                    )
                    return S(
                        sentence_type=sentence_type,
                        tam_event=self.convert_tam_event(children[1].children),
                    )
                case ("terminal_interrogative_marker", "tense_aspect_event"):
                    raise NotImplementedError
                case _:
                    raise ValueError(f"Invalid children for S: {children}")
        else:
            raise ValueError(f"Invalid children for S: {children}")

    def convert_declarative_marker(self, children) -> SentenceType:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case ("terminal_declarative_marker",):
                    return SentenceType(name="declarative")
                case _:
                    raise ValueError(
                        f"Invalid children for declarative_marker: {children}"
                    )
        else:
            if children == EPSILON_SYMBOL:
                return SentenceType(name="declarative")
            else:
                raise ValueError(f"Invalid children for declarative_marker: {children}")

    def convert_tam_event(self, children) -> TamEvent:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case (
                    "tense_marker",
                    "aspect_marker",
                    "modality_marker",
                    "event",
                    "tam_event_post_modifier",
                ):
                    return TamEvent(
                        tense=None,
                        aspect=None,
                        modality=None,
                        event=self.convert_event(children[3].children),
                    )
                case ("terminal_tam_event_operator", "tam_event", "tam_event"):
                    e0 = TamEvent(
                        tense=None,
                        aspect=None,
                        modality=None,
                        event=self.convert_event(children[0].children),
                    )
                    _e1 = TamEvent(
                        tense=None,
                        aspect=None,
                        modality=None,
                        event=self.convert_event(children[0].children),
                    )
                    return e0
                case _:
                    raise ValueError(f"Invalid children for tam_event: {children}")
        else:
            raise ValueError(f"Invalid children for tam_event: {children}")

    def convert_event(self, children) -> Event:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case (
                    "binary_predicate",
                    "binary_predicate_post_modifier",
                    "argument_not_ε",
                    "argument",
                ):
                    return Event(
                        binary_predicate=self.convert_binary_predicate(
                            children[0].children
                        ),
                        argument_1=self.convert_argument_not_ε(children[2].children),
                        argument_2=self.convert_argument(children[3].children),
                    )
                # case (
                #     "terminal_postfix_binary_predicate_marker",
                #     "argument_not_ε",
                #     "argument",
                #     "binary_predicate",
                # ):
                #     return Event(
                #         compound_binary_predicate=self.convert_binary_predicate(
                #             children[3].children
                #         ),
                #         argument_1=self.convert_argument_not_ε(children[1].children),
                #         argument_2=self.convert_argument(children[2].children),
                #     )
                case _:
                    raise ValueError(f"Invalid children for event: {children}")
        else:
            raise ValueError(f"Invalid children for event: {children}")

    def convert_binary_predicate(self, children) -> BinaryPredicate:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case ("terminal_binary_predicate",):
                    return BinaryPredicate(type1=MyType(), type2=MyType())
                case ("terminal_argument_transposition", "binary_predicate"):
                    bp = self.convert_binary_predicate(children[1].children)
                    return BinaryPredicate(
                        type1=bp.type2,
                        type2=bp.type1,
                    )
                case (
                    "terminal_binary_predicate_operator",
                    "binary_predicate",
                    "binary_predicate",
                ):
                    raise NotImplementedError
                case _:
                    raise ValueError(
                        f"Invalid children for binary_predicate: {children}"
                    )
        else:
            raise ValueError(f"Invalid children for binary_predicate: {children}")

    def convert_argument(self, children) -> Arg:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case ("argument_not_ε",):
                    return self.convert_argument_not_ε(children[0].children)
                case _:
                    raise ValueError(f"Invalid children for argument: {children}")
        else:
            if children == EPSILON_SYMBOL:
                return Arg(arg_type=Bottom())
            else:
                raise ValueError(f"Invalid children for argument: {children}")

    def convert_argument_not_ε(self, children) -> Arg:
        if isinstance(children, list):
            production_tuple = tuple(
                [
                    (
                        c.non_terminal_symbol
                        if isinstance(c, AbstractSyntaxTree)
                        else c.pos
                    )
                    for c in children
                ]
            )
            match production_tuple:
                case (
                    "terminal_argument_retriever_without_argument",
                    "determiner",
                    "binary_predicate",
                ):
                    bp = self.convert_binary_predicate(children[2].children)
                    return Arg(arg_type=bp.type1)
                case (
                    "terminal_argument_retriever_with_argument",
                    "determiner",
                    "binary_predicate",
                    "argument_not_ε",
                ):
                    bp = self.convert_binary_predicate(children[2].children)
                    if isinstance(children[3], AbstractSyntaxTree):
                        # arg = self.convert_argument(children[3].children)
                        pass
                    else:
                        assert False, "Invalid children for argument_not_ε"
                    return Arg(arg_type=bp.type1)
                case ("terminal_argument",):
                    return Arg(arg_type=MyType())
                case ("terminal_argument_operator", "argument_not_ε", "argument_not_ε"):
                    raise NotImplementedError
                case _:
                    raise ValueError(f"Invalid children for argument_not_ε: {children}")
        else:
            raise ValueError(f"Invalid children for argument_not_ε: {children}")


rules = {
    "S": [
        ["declarative_sentence"],
        ["interrogative_sentence"],
    ],
    "declarative_sentence": [["post_modified_mta_event"]],
    "interrogative_sentence": [["interrogative_marker", "post_modified_mta_event"]],
    "post_modified_mta_event": [
        ["mta_event", "post_mta_event_expression"],
    ],
    "post_mta_event_expression": [
        ["post_mta_event_modifier", "mta_event"],
        "ε",
    ],
    "modality_expression": [
        ["modality_with_agent"],
        "ε",
    ],
    "modality_with_agent": [
        ["modified_modality", "modality_agent"],
    ],
    "modified_modality": [
        ["modality"],
        [
            "modality_negation_begin",
            "negation",
            "modified_modality",
        ],
    ],
    "modality_agent": [
        ["post_argument_phrase_modifier", "argument_phrase"],
        "ε",
    ],
    "tense_expression": [
        ["modified_tense"],
        "ε",
    ],
    "modified_tense": [
        ["tense"],
        [
            "tense_negation_begin",
            "negation",
            "modified_tense",
        ],
    ],
    "aspect_expression": [
        ["modified_aspect"],
        "ε",
    ],
    "modified_aspect": [
        ["aspect"],
        [
            "aspect_negation_begin",
            "negation",
            "modified_aspect",
        ],
    ],
    "mta_event": [
        ["modality_expression", "ta_event"],
    ],
    "ta_event": [
        ["tense_expression", "a_event"],
    ],
    "a_event": [
        ["aspect_expression", "event"],
    ],
    "event": [
        [
            "post_modified_binary_predicate_phrase",
            "argument_phrase",
            "argument_phrase_or_ε",
        ],
    ],
    "post_modified_binary_predicate_phrase": [
        ["binary_predicate_phrase", "post_binary_predicate_phrase_expression"],
    ],
    "post_binary_predicate_phrase_expression": [
        ["post_binary_predicate_phrase_modifier", "binary_predicate_phrase"],
        "ε",
    ],
    "binary_predicate_phrase": [
        ["binary_predicate"],
        ["argument_phrase_transposer", "binary_predicate_phrase"],
        [
            "binary_operator",
            "binary_predicate_phrase",
            "binary_predicate_phrase",
        ],
        ["negation", "binary_predicate_phrase"],
    ],
    "argument_phrase": [
        ["argument"],
        [
            "argument_retriever",
            "determined_argument_retrieval_binary_predicate",
        ],
        [
            "binary_operator",
            "argument_phrase",
            "argument_phrase",
        ],
        ["negation", "argument_phrase"],
        [
            "mta_event_to_argument_phrase_converter",
            "mta_event",
            "mta_event_to_argument_phrase_converter_end",
        ],
    ],
    "argument_phrase_or_ε": [
        ["argument_phrase_separator", "argument_phrase"],
        "ε",
    ],
    "determined_argument_retrieval_binary_predicate": [
        [
            "quantification",
            "articlized_argument_retrieval_binary_predicate",
        ],
    ],
    "articlized_argument_retrieval_binary_predicate": [
        [
            "article",
            "argument_retrieval_binary_predicate",
        ]
    ],
    "argument_retrieval_binary_predicate": [
        [
            "post_modified_binary_predicate_phrase",
            "argument_retrieval_with_argument_phrase",
        ]
    ],
    "argument_retrieval_with_argument_phrase": [
        ["post_argument_phrase_modifier", "argument_phrase"],
        "ε",
    ],
    "quantification": [
        ["quantifier"],
        "ε",
    ],
    "article": [["articlizer"], "ε"],
}
terminals = {
    "interrogative_marker",
    "modality",
    "tense",
    "aspect",
    "modality_negation_begin",
    "tense_negation_begin",
    "aspect_negation_begin",
    "negation",
    "binary_predicate",
    "argument",
    "argument_phrase_separator",
    "argument_phrase_transposer",
    "binary_operator",
    "argument_retriever",
    "quantifier",
    "articlizer",
    "mta_event_to_argument_phrase_converter",
    "mta_event_to_argument_phrase_converter_end",
    "post_mta_event_modifier",
    "post_binary_predicate_phrase_modifier",
    "post_argument_phrase_modifier",
}

# rules = {
#     "S": [["A"]],
#     "A": [
#         ["A", "a"],
#         ["b"],
#     ],
# }

# terminals = {
#     "a",
#     "b",
# }

grammar = Grammar(rules, terminals)  # type: ignore[arg-type]
lexer = Lexer(terminals)  # type: ignore[arg-type]

parser = Parser(grammar, lexer)
converter = MyLambdaExpressionConverter()

import pprint

pprint.pprint(grammar.parse_table)
