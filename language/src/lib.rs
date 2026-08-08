pub mod argument_type_checker;
pub mod formal_semantics;
pub mod parser;
pub mod tokenizer;

use argument_type_checker::type_check as argument_type_check;
use formal_semantics::{find_contradiction_index, to_fol, type_check as formal_type_check};
use parser::{Ast, parse};
use tokenizer::{Range, add_eos_to_last_sentence, tokenize};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
}

pub fn diagnose(text: String) -> Result<Vec<Ast>, Vec<Diagnostic>> {
    // tokenizing
    let tokenize_result = tokenize(&text);
    if let Err(e) = tokenize_result {
        let diagnostics = vec![Diagnostic {
            range: e.range,
            message: "tokenize failed".to_string(),
        }];
        return Err(diagnostics);
    }

    // parsing
    let sentences = add_eos_to_last_sentence(tokenize_result.unwrap());
    let mut ast_with_ranges: Vec<(Ast, Range)> = Vec::new();
    let mut diagnostics = Vec::new();
    for sentence in sentences {
        let sentence_range = Range::new(
            sentence[0].range().start(),
            sentence.last().unwrap().range().end(),
        );
        let parse_result = parse(sentence.clone());
        if let Err(e) = parse_result {
            let diagnostics_item = Diagnostic {
                range: e.error_token.range(),
                message: format!("parse error: {}", e.error_message),
            };
            diagnostics.push(diagnostics_item);
        } else {
            ast_with_ranges.push((parse_result.unwrap(), sentence_range));
        }
    }
    let asts: Vec<Ast> = ast_with_ranges.iter().map(|(a, _)| a.clone()).collect();

    // argument type checking
    for ast in &asts {
        let argument_type_check_result = argument_type_check(ast.clone());
        if let Err(e) = argument_type_check_result {
            let diagnostics_item = Diagnostic {
                range: e.range,
                message: format!("argument type error: {}", e.message),
            };
            diagnostics.push(diagnostics_item);
        }
    }

    // formal semantics type checking
    for ast in &asts {
        let formal_type_check_result = formal_type_check(ast.clone());
        if let Err(e) = formal_type_check_result {
            let diagnostics_item = Diagnostic {
                range: e.range,
                message: format!("formal type error: {}", e.message),
            };
            diagnostics.push(diagnostics_item);
        }
    }

    // contradiction checking
    let formulas: Vec<_> = ast_with_ranges.iter().map(|(a, _)| to_fol(a)).collect();
    if let Some(idx) = find_contradiction_index(&formulas) {
        diagnostics.push(Diagnostic {
            range: ast_with_ranges[idx].1.clone(),
            message: "contradiction".to_string(),
        });
    }

    if !diagnostics.is_empty() {
        Err(diagnostics)
    } else {
        Ok(asts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_私はりんごを食べる() {
        let text = "pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べた() {
        let text = "hita pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼はりんごを食べている() {
        let text = "hudida pahete sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっている() {
        let text = "hudu pahete sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっていた() {
        let text = "hita hudu pahete sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを食べたことがある() {
        let text = "hita kaheperihenese sa ri paparatanera du mi si se pahete sa ri paparatanera du mi si ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私の妻はりんごを毎日食べる() {
        let text = "hudidi pahete ri paparatanera du mi si ri behapare da [all] zudahi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は昨日りんごを食べた() {
        let text = "hita pahete sa binary_operator mi ri paparatanera du mi si ri behapare hegi ri zuhita du number ri zudahi"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は6日前にりんごを食べた() {
        let text = "hita pahete sa binary_operator mi ri paparatanera du mi si ri behapare hegi ri zuhita du number ri zudahi"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼らは明日りんごを食べる() {
        let text =
            "hitu pahete sa gu si ri behapare hegi ri zuhitu du number ri zudahi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼らは6日後にりんごを食べる() {
        let text =
            "hitu pahete sa gu si ri behapare hegi ri zuhitu du number ri zudahi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らは3日間りんごを食べている_食事としてりんごだけを食べている() {
        let text = "hiti hudidi pahete sa gu si ri behapare hege ri zuhita du number ri zudahi"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らはりんごを5分間食べ続けている_りんごを食べる所要時間に5分かかっている() {
        let text = "hiti hudida pahete sa gu si ri behapare hege ri zuhita du number ri zuminite"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は常にりんごを食べている_四六時中ずっとりんごを食べている() {
        let text = "hudidu pahete sa gi si ri behapare hegi ri zuharahahi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが3つある() {
        let text = "pihezisata sa number ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんご達がテーブルの上にある() {
        let text = "pihezisata sa number ri behapare heni ri zahatano du ri ditabare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがあのりんごを食べてしまった_その結果あのりんごは今はない() {
        let text = "hudu pahete sa ga si go ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごを食べそうだ() {
        let text = "hapa □ hitu pahete sa ga si mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごそのりんごあのりんごを食べたそうだ() {
        let text = "kahehara sa mi si se pahete sa ga si binary_operator mo ri behapare binary_operator bo ri behapare go ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごがひとつもない() {
        let text = "ne pihezisata sa ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない() {
        let text = "ne pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない_二重否定がエラーになる() {
        let text = "ne ne pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを食べるな() {
        let text = "hane hapi □ pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べるな_二重否定がエラーになる() {
        let text = "hane hane hapi □ pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた() {
        let text = "hine hita pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた_二重否定がエラーになる() {
        let text = "hine hine hita pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを食べ始めてない() {
        let text = "hune hude pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べ始めてない_二重否定がエラーになる() {
        let text = "hune hune hude pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べないを開始してないと事実性高く思わない() {
        let text = "hane hapa □ hine hita hune hude ne pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べられない_りんごがないので状況的に食べられない() {
        let text = "hane hapu ◇ pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを床に落とした() {
        let text = "padaropa sa mi si ri behapare heno ri zahatano du ri diharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが床に落ちた() {
        let text = "padaropa si ri behapare heno ri zahatano du ri diharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べられない_りんごの性質として可食でなくなった() {
        let text = "hapu □ pahete si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたい() {
        let text = "hapu ◇ pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを買いたい() {
        let text = "hapu ◇ pabuhi sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_このりんごは汚い() {
        let text = "hudu padirati si mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_このりんごは綺麗ではない() {
        let text = "hudu ne pakarena sa mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あのりんごは食べられそうだ() {
        let text = "hapu ◇ padaropa si bo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あれはりんごではない() {
        let text = "ne behapare sa ge".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_これはりんごではなくみかんだ() {
        let text = "behoranage me da [文分割] ne behapare me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べるか_疑問() {
        let text = "ku pahete sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べます() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "pahete sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べないのですか_疑問() {
        let text = "ku ne pahete sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べません() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "ne pahete sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べたかったですか() {
        let text = "ku hapu ◇ hita pahete sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえ私はみかんを食べたくはありません() {
        let text = "ne kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapu ◇ pahete sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごもみかんも食べないのですか() {
        let text = "ku pahete sa ba si binary_operator ri behapare ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べます() {
        let text = "pahete sa ba si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はみかんを食べない_だからみかんは食べられない_受け身の否定() {
        let text =
            "ne pahete mi si ri behoranage [接続詞 なので 順接] ne re pahete ri behoranage si mi"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べることができない_なぜなら_りんごは綺麗ではないから() {
        let text = "hane hapu ◇ pahete si ri behapare hete se hudu ne pakarena sa ri behoranage so"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんを食べましょうよ_勧誘() {
        let text = "hapi ◇ pahete sa ba si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえお断りします() {
        let text = "ne kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あそこでみかんを食べろ_りんごは食べるな() {
        let text = "hapi □ pahete si ri behoranage heni ge".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapi □ ne pahete si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをあげる() {
        let text = "pagibe sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにあげる() {
        let text = "pagibeto sa mi si ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにりんごをあげる() {
        // 所有を移す
        let text = "pamobe sa mi si ri behapare hebe mi hebo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをもらう() {
        let text = "pamobe hebo mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんごを頂いても宜しいですか_みかんは差し上げます() {
        let text = "????".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "????".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はりんごは食べるなみかんを食べろと言った() {
        let text =
            "kasahi mi si se [eventBinaryOperator] ne pahete ba si ri behapare ne pahete ba si ri behoranage so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごを食べろと言うのですか() {
        let text = "ku kasahi sa ba si se pahete sa mi si ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_貴様はりんごを食べたいと言った() {
        let text = "hita kasahi sa ba si se hapu ◇ pahete sa ba si ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたいと言った覚えがない() {
        let text =
            "ne karememabera sa mi si se hita kasahi sa mi si se hapu ◇ pahete sa mi si ri behapare so so"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは確か私はりんごを食べたいと言ったはずだ() {
        let text =
            "hapa □ hita kasahi sa ba si se hapu ◇ pahete sa ba si ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はもしかしたらそう言ったかもしれない() {
        let text = "hapa ◇ hita kasahi sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは絶対に私に対してりんごを食べたいと言った() {
        let text = "hapa □ hita kasahi sa ba hebo mi si se kahanata sa ba si se pahete sa ba si ri behapare so so"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は私にりんごを食べさせる() {
        let text = "kareta sa gi si se pahete sa ba si ri behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない() {
        let text = "ne pahete sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない_転置が否定の先にきた場合にエラーなる() {
        let text = "re ne pahete ri behoranage si mi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_りんごは彼によって私に食べさせられた() {
        let text = "kareta si se pahete si ri behapare sa ba so sa gi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_ここには何もない() {
        let text = "ne pihezisata sa 何か heni me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあるりんごを食べる() {
        // ∃x (apple(x))
        let text = "pahete sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "pahete sa mi si mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はそのりんごを食べる() {
        // ∃x (apple(x) ∧ it(x))
        let text = "pahete sa mi si bo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあのりんごを食べる() {
        // ∃x (apple(x) ∧ that(x))
        let text = "pahete sa mi si go ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "pahete sa mi si number ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこの3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "pahete sa mi si number mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しのりんごを食べる() {
        // ∃=to x (apple(x))
        let text = "pahete sa mi si to ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しこのりんごを食べる() {
        // ∃=to x (apple(x) ∧ this(x))
        let text = "pahete sa mi si to mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しの3つのりんごを食べる_がエラーになること() {
        // ∀x ∃=3 (apple(x))
        let text = "pahete sa mi si number to ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "pahete sa mi si to number ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私は半分のりんごを食べる() {
        // ∃=ti x (apple(x))
        let text = "pahete sa mi si ti ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は多くのりんごを食べる() {
        // ∃=ta x (apple(x))
        let text = "pahete sa mi si te ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのりんごを食べる() {
        // ∀x (apple(x))
        let text = "pahete sa mi si ta ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのこのりんごを食べる() {
        // ∀x (apple(x) ∧ this(x))
        let text = "pahete sa mi si ta mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全ての3つのりんごを食べる_がエラーになること() {
        // ∀x ∃=3 (apple(x))
        let text = "pahete sa mi si number ta ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "pahete sa mi si ta number ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }
}
