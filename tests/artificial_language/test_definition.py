import unittest
from unittest import TestCase

from artificial_language.definition import parser


class Test平叙文Parser(TestCase):
    def test_平叙文を受理する(self):
        parser.parse(
            "terminal_declarative_marker:_ terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )

    def test_マーカーなし平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
        )

    def test_マーカーなし複合語の平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate_operator:_ terminal_binary_predicate:love terminal_binary_predicate:true terminal_argument:John terminal_argument:Smith"
        )

    def test_項抽出平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:flower"
        )

    def test_限定詞平叙文を受理する(self):
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_article:the terminal_binary_predicate:flower"
        )
        parser.parse(
            "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_quantifier:all terminal_binary_predicate:flower"
        )

    def test_受動態平叙文を受理する(self):
        parser.parse(
            "terminal_argument_transposition:be-ed terminal_binary_predicate:love terminal_argument:Smith terminal_argument:John"
        )

    def test_過去形や未来形平叙文を受理する(self):
        parser.parse(
            "terminal_tense_marker:will terminal_argument_transposition:be-ed terminal_binary_predicate:love terminal_argument:Smith terminal_argument:John"
        )

    # def test_sov平叙文を受理する(self):
    #     parser.parse(
    #         "terminal_postfix_binary_predicate_marker:_ terminal_argument:John terminal_argument:Smith terminal_binary_predicate:love"
    #     )

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
            "terminal_interrogative_marker:what terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:?"
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


# class Test平叙文TypeChecker(TestCase):
#     def test_平叙文を型検査が成功する(self):
#         calculator = TypedLambdaCalculator()
#         ast = parser.parse(
#             "terminal_tense_marker:a terminal_aspect_marker:b terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
#         )
#         expr = converter.convert(ast)
#         self.assertEqual(
#             type(calculator.check_type(expr)),
#             type(TruthValue()),
#         )

#         calculator = TypedLambdaCalculator()
#         ast = parser.parse(
#             "terminal_binary_predicate:love terminal_argument:John terminal_argument:Smith"
#         )
#         expr = converter.convert(ast)
#         self.assertEqual(
#             type(calculator.check_type(expr)),
#             type(TruthValue()),
#         )

#         calculator = TypedLambdaCalculator()
#         ast = parser.parse(
#             "terminal_binary_predicate:love terminal_argument:John terminal_argument_retriever:_ terminal_binary_predicate:flower"
#         )
#         expr = converter.convert(ast)
#         self.assertEqual(
#             type(calculator.check_type(expr)),
#             type(TruthValue()),
#         )


class Testりんご文Parser(TestCase):
    # https://wiki.conlinguistics.jp/%E3%82%8A%E3%82%93%E3%81%94%E6%96%87

    def test_りんご文を受理する(self):
        # 私はりんごを食べる．
        parser.parse(
            "terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私はりんごを食べた．
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 彼はりんごを食べている．
        parser.parse(
            "terminal_aspect_marker:ing terminal_binary_predicate:eat terminal_argument:He terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 彼女はりんごを食べ終わっている．
        parser.parse(
            "terminal_aspect_marker:ba'o terminal_binary_predicate:eat terminal_argument:She terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 彼女はりんごを食べ終わっていた．
        parser.parse(
            "terminal_tense_marker:ed terminal_aspect_marker:ba'o terminal_binary_predicate:eat terminal_argument:She terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私の妻はりんごを食べたことがある．
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:experience terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_tam_event_to_argument_converter:_ terminal_binary_predicate:eat terminal_argument:it terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_to_argument_converter_end:_"
        )

        # 私の妻はりんごを毎日食べる．
        parser.parse(
            "terminal_binary_predicate:ある命題がarg2のときに成立する terminal_tam_event_to_argument_converter:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_to_argument_converter_end:_ terminal_argument_retriever:_ terminal_quantifier:every terminal_binary_predicate:day"
        )
        parser.parse(
            "terminal_binary_predicate:eat terminal_post_modifier:_ terminal_binary_predicate:everyday terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私と私の妻は昨日りんごを食べた．
        parser.parse(
            "terminal_tam_event_binary_operator:and terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument_operator:and terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_binary_predicate:ago terminal_argument_retriever:_ terminal_binary_predicate:that_event terminal_argument_retriever:_ terminal_binary_predicate_operator:_ terminal_binary_predicate:day terminal_binary_predicate:1"
        )
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument_operator:and terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:ago terminal_argument_retriever:_ terminal_binary_predicate:that_event terminal_argument_retriever:_ terminal_binary_predicate:day terminal_post_modifier:_ terminal_binary_predicate:1"
        )
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument_operator:and terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:wife terminal_post_modified_argument_retriever:_ terminal_argument:me terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:ago terminal_argument_retriever:_ terminal_binary_predicate:that_event terminal_argument_retriever:_ terminal_binary_predicate:day terminal_post_modified_argument_retriever:_ terminal_argument:1"
        )

        # 彼らは明日りんごを食べる．
        parser.parse(
            "terminal_tense_marker:will terminal_binary_predicate:eat terminal_argument:they terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:in terminal_argument:that_event terminal_argument_retriever:_ terminal_quantifier:1 terminal_binary_predicate:day"
        )

        # 彼らは6日後にりんごを食べる．
        parser.parse(
            "terminal_tense_marker:will terminal_binary_predicate:eat terminal_argument:they terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:in terminal_argument:that_event terminal_argument_retriever:_ terminal_quantifier:6 terminal_binary_predicate:day"
        )

        # 彼女らは3日間りんごを食べている．（食事としてりんごだけを食べている）
        parser.parse(
            "terminal_binary_predicate:eat terminal_argument:they terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:during terminal_argument:that_event terminal_argument_retriever:_ terminal_quantifier:3 terminal_binary_predicate:day"
        )

        # 彼女らはりんごを5分間食べ続けている．（りんごを食べる所要時間に5分かかっている）
        parser.parse(
            "terminal_aspect_marker:ca'o terminal_binary_predicate:eat terminal_argument:they terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:during terminal_argument:that_event terminal_argument_retriever:_ terminal_quantifier:5 terminal_binary_predicate:minute"
        )

        # 彼は常にりんごを食べている．（四六時中ずっとりんごを食べている）
        parser.parse(
            "terminal_binary_predicate:eat terminal_argument:they terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:when terminal_binary_predicate:during terminal_argument:that_event terminal_argument_retriever:_ terminal_quantifier:every terminal_binary_predicate:time"
        )

        # りんごが3つある．
        parser.parse(
            "terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_quantifier:3 terminal_binary_predicate:apple"
        )

        # りんご達がテーブルの上にある．
        parser.parse(
            "terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_quantifier:multi terminal_binary_predicate:apple terminal_tam_event_post_modifier:where terminal_binary_predicate:on terminal_argument:that_event terminal_argument_retriever:_ terminal_binary_predicate:table"
        )

        # 誰かがあのりんごを食べてしまった．（その結果，あのりんごは今はない）
        parser.parse(
            "terminal_aspect_marker:ba'o terminal_binary_predicate:eat terminal_argument:someone terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 誰かがこのりんごを食べそうだ．（推量）
        parser.parse(
            "terminal_aspect_marker:pu'o terminal_modality_marker:may terminal_binary_predicate:eat terminal_argument:someone terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 誰かがこのりんご，そのりんご，あのりんごを食べたそうだ．（伝聞）
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:hear terminal_argument:I terminal_tam_event_to_argument_converter:_ terminal_aspect_marker:ba'o terminal_binary_predicate:eat terminal_argument:someone terminal_argument_operator:and terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_post_modified_argument_retriever:_ terminal_argument:this terminal_argument_operator:and terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_post_modified_argument_retriever:_ terminal_argument:it terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_post_modified_argument_retriever:_ terminal_argument:that terminal_tam_event_to_argument_converter_end:_"
        )

        # りんごがひとつもない．
        parser.parse(
            "terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_quantifier:0 terminal_binary_predicate:apple"
        )
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私はりんごを食べない．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私はりんごを食べられない．（りんごがないので状況的に食べられない）
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_modality_marker:can terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私はりんごを床に落とした．
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:drop terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_tam_event_post_modifier:where terminal_binary_predicate:on terminal_argument:that_event terminal_argument_retriever:_ terminal_binary_predicate:floor"
        )

        # りんごが床に落ちた．
        parser.parse(
            "terminal_tense_marker:ed terminal_argument_transposition:_ terminal_binary_predicate:drop terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_argument:null terminal_tam_event_post_modifier:where terminal_binary_predicate:on terminal_argument:that_event terminal_argument_retriever:_ terminal_binary_predicate:floor"
        )

        # りんごは食べられない．（りんごの性質として，可食でなくなった）
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_modality_marker:can terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_argument:null"
        )

        # 私はりんごを食べたい．
        parser.parse(
            "terminal_modality_marker:want terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # 私はりんごを買いたい．
        parser.parse(
            "terminal_modality_marker:want terminal_binary_predicate:buy terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        )

        # このりんごは汚い．
        parser.parse(
            "terminal_argument_transposition:_ terminal_binary_predicate:dirty terminal_argument_retriever:_ terminal_binary_predicate:apple terminal_argument:this"
        )

        # このりんごは綺麗ではない．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:beautiful terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_post_modifier:_ terminal_unary_predicate:this"
        )

        # あのりんごは食べられそうだ．
        # モダリティの主語はどうしよう？
        parser.parse(
            "terminal_modality_marker:may_can terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null"
        )

        # あれはりんごではない．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:apple terminal_argument:that"
        )

        # これはりんごではなくみかんだ．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:apple terminal_argument:that terminal_tam_event_post_modifier:but terminal_unary_predicate:orange terminal_argument:that"
        )

        # あなたはみかんを食べるか？
        parser.parse(
            "terminal_interrogative_marker:? terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # はい，私はみかんを食べます．
        # parser.parse("go'i")

        # あなたはみかんを食べないのですか？
        parser.parse(
            "terminal_interrogative_marker:? terminal_tam_event_unary_operator:not terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # ええ，私はみかんを食べません．
        # parser.parse("go'i")

        # あなたはみかんを食べたかった．ですよね？
        parser.parse(
            "terminal_interrogative_marker:? terminal_tense_marker:ed terminal_modality_marker:want terminal_post_modifier:_ terminal_argument:you terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # いいえ，私はみかんを食べたくはありません．
        # parser.parse("tam_event_operator:not go'i")

        # あなたはりんごもみかんも食べないのですか？
        parser.parse(
            "terminal_interrogative_marker:? terminal_tam_event_unary_operator:not terminal_binary_predicate:eat terminal_argument:you terminal_argument_operator:and terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # いや，私はりんごを食べます．
        # parser.parse(
        #     "tam_event_operator:not go'i .i terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_binary_predicate:apple"
        # )

        # 私はみかんを食べない．だから，みかんは食べられない．（受け身の否定）
        # parser.parse(
        #     "terminal_tam_event_unary_operator:not terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:orange .i 接続詞:ri'a terminal_tam_event_unary_operator:not terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:orange terminal_argument:I"
        # )
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:orange terminal_argument:I"
        )

        # りんごは食べることができない．なぜなら，りんごは綺麗ではないから．
        # parser.parse(
        #     "terminal_tam_event_unary_operator:not terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:apple .i 接続詞:ri'a terminal_tam_event_unary_operator:not terminal_unary_predicate:apple terminal_post_modifier:_ terminal_binary_predicate:clean"
        # )
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:apple terminal_post_modifier:_ terminal_binary_predicate:clean terminal_argument:that"
        )

        # りんごは汚い．一方で，みかんは綺麗だ．
        # parser.parse(
        #     "terminal_tam_event_unary_operator:not terminal_unary_predicate:apple terminal_post_modifier:_ terminal_binary_predicate:clean terminal_argument:that .i 接続詞:ku'i terminal_unary_predicate:orange terminal_post_modifier:_ terminal_binary_predicate:clean terminal_argument:null"
        # )

        # みかんを食べましょうよ？（勧誘）
        parser.parse(
            "terminal_modality_marker:勧誘 terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # いいえ，お断りします．

        # あそこでみかんを食べろ．りんごは食べるな．
        parser.parse(
            "terminal_modality_marker:命令 terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:orange terminal_tam_event_post_modifier:where terminal_binary_predicate:at terminal_argument:that_event terminal_argument:that_place"
        )
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_modality_marker:命令 terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )

        # りんごを頂いても宜しいですか？みかんは差し上げます．
        # parser.parse(
        #     "terminal_interrogative_marker:? terminal_modality_marker:許可 terminal_binary_predicate:貰う terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:apple .i terminal_tam_event_operator:and terminal_binary_predicate:give_to terminal_argument:I terminal_argument:you terminal_binary_predicate:give terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:orange"
        # )
        parser.parse(
            "terminal_interrogative_marker:? terminal_modality_marker:許可 terminal_binary_predicate:貰う terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )
        parser.parse(
            "terminal_tam_event_binary_operator:and terminal_binary_predicate:give_to terminal_argument:I terminal_argument:you terminal_binary_predicate:give terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:orange"
        )

        # 私は「りんごは食べるな，みかんを食べろ」と言った．
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:say terminal_argument:I terminal_tam_event_to_argument_converter:_ terminal_tam_event_binary_operator:and terminal_tam_event_unary_operator:not terminal_modality_marker:命令 terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null terminal_modality_marker:命令 terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:orange terminal_argument:null terminal_tam_event_to_argument_converter_end:_"
        )

        # あなたはりんごを食べろと言うのですか？
        parser.parse(
            "terminal_interrogative_marker:? terminal_binary_predicate:say terminal_argument:you terminal_tam_event_to_argument_converter:_ terminal_modality_marker:命令 terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null terminal_tam_event_to_argument_converter_end:_"
        )

        # 貴様はりんごを食べたいと言った．
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:say terminal_argument:you terminal_tam_event_to_argument_converter:_ terminal_modality_marker:want terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null terminal_tam_event_to_argument_converter_end:_"
        )

        # 私はりんごを食べたいと言った覚えがない．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_binary_predicate:remember terminal_argument:I terminal_tam_event_to_argument_converter:_ terminal_tense_marker:ed terminal_binary_predicate:say terminal_argument:I terminal_tam_event_to_argument_converter:_ terminal_modality_marker:want terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null terminal_tam_event_to_argument_converter_end:_ terminal_tam_event_to_argument_converter_end:_"
        )

        # あなたは確か「私はりんごを食べたい」と言ったはずだ．
        parser.parse(
            "terminal_tense_marker:ed terminal_modality_marker:事実性高い terminal_binary_predicate:say terminal_argument:you terminal_tam_event_to_argument_converter:_ terminal_tense_marker:ed terminal_binary_predicate:say terminal_argument:I terminal_tam_event_to_argument_converter:_ terminal_modality_marker:want terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_argument:null terminal_tam_event_to_argument_converter_end:_ terminal_tam_event_to_argument_converter_end:_"
        )

        # 私はもしかしたらそう言ったかもしれない．
        parser.parse(
            "terminal_tense_marker:ed terminal_modality_marker:事実性低い terminal_binary_predicate:say terminal_argument:you terminal_argument:that"
        )

        # あなたは絶対に私に対して「りんごを食べたい」と言った．
        parser.parse(
            "terminal_tense_marker:ed terminal_modality_marker:事実性確信 terminal_binary_predicate:say terminal_argument:you terminal_tam_event_to_argument_converter:_ terminal_modality_marker:want terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_tam_event_post_modifier:who terminal_binary_predicate:to terminal_argument:that_event terminal_argument:me terminal_tam_event_to_argument_converter_end:_"
        )

        # 彼は私にりんごを食べさせる．
        parser.parse(
            "terminal_binary_predicate:使役(させる) terminal_argument:He terminal_tam_event_to_argument_converter:_ terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_tam_event_to_argument_converter_end:_"
        )

        # みかんは私に食べられていない．
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_argument_transposition:_ terminal_binary_predicate:eat terminal_argument_retriever:_ terminal_unary_predicate:orange terminal_argument:I"
        )

        # りんごは彼によって私に食べさせられた．
        parser.parse(
            "terminal_tense_marker:ed terminal_argument_transposition:_ terminal_binary_predicate:使役(させる) terminal_tam_event_to_argument_converter:_ terminal_binary_predicate:eat terminal_argument:I terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_tam_event_to_argument_converter_end:_ terminal_argument:He"
        )

        # ここには何もない．
        parser.parse(
            "terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_quantifier:0 terminal_binary_predicate:thing"
        )
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_unary_predicate:exist terminal_argument_retriever:_ terminal_binary_predicate:thing"
        )

        # あなたはりんごを食べないだろう
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_modality_marker:推量 terminal_binary_predicate:eat terminal_argument:you terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )

        # あなたは「彼はりんごを食べないだろう」と思ってる
        parser.parse(
            "terminal_binary_predicate:think terminal_argument:you terminal_tam_event_to_argument_converter:_ terminal_modality_marker:推量 terminal_post_modifier:_ terminal_argument:you terminal_predicate_operator:not terminal_binary_predicate:eat terminal_argument:He terminal_argument_retriever:_ terminal_unary_predicate:apple terminal_tam_event_to_argument_converter_end:_"
        )


class Test否定文Parser(TestCase):
    def test_否定文を受理する(self):
        # 太郎はりんごを食べてない(全体の否定)
        parser.parse(
            "terminal_tam_event_unary_operator:not terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument:Taro terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )

        # 太郎はりんごを食べてない(述語の否定)
        parser.parse(
            "terminal_tense_marker:ed terminal_negation:not terminal_binary_predicate:eat terminal_argument:Taro terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )

        # 太郎じゃないはりんごを食べた(項の否定)
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:eat terminal_negation:not terminal_argument:Taro terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )

        # 太郎はりんごじゃないを食べた(項の否定)
        parser.parse(
            "terminal_tense_marker:ed terminal_binary_predicate:eat terminal_argument:Taro terminal_negation:not terminal_argument_retriever:_ terminal_unary_predicate:apple"
        )


if __name__ == "__main__":
    unittest.main()
