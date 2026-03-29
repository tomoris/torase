pub mod parser;
pub mod tokenizer;
pub mod type_checker;

use parser::{Ast, parse};
use tokenizer::{Range, add_eos_to_last_sentence, tokenize};
use type_checker::type_check;

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
    let mut asts = Vec::new();
    let mut diagnostics = Vec::new();
    for sentence in sentences {
        let parse_result = parse(sentence.clone());
        if let Err(e) = parse_result {
            let diagnostics_item = Diagnostic {
                range: e.error_token.range(),
                message: format!("parse error: {}", e.error_message),
            };
            diagnostics.push(diagnostics_item);
        } else {
            asts.push(parse_result.unwrap());
        }
    }

    // type checking
    for ast in &asts {
        let type_check_result = type_check(ast.clone());
        if let Err(e) = type_check_result {
            let diagnostics_item = Diagnostic {
                range: e.range,
                message: format!("type error: {}", e.message),
            };
            diagnostics.push(diagnostics_item);
        }
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
        let text = "pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べた() {
        let text = "hita pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼はりんごを食べている() {
        let text = "hudida pahete gi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっている() {
        let text = "hudu pahete gi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっていた() {
        let text = "hita hudu pahete gi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを食べたことがある() {
        let text = "hita kaheperihenese ro paparatanera du mi si se pahete ro paparatanera du mi si ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私の妻はりんごを毎日食べる() {
        let text = "hudidi pahete ro paparatanera du mi si ro behapare da [all] rudahi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私と私の妻は昨日りんごを食べた() {
        let text = "hita pahete binary_operator mi ro paparatanera du mi si ro behapare da [昨日]"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私と私の妻は6日前にりんごを食べた() {
        let text = "hita pahete binary_operator mi ro paparatanera du mi si ro behapare da 6日前"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_彼らは明日りんごを食べる() {
        let text = "hitu pahete gu si ro behapare da 明日".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_彼らは6日後にりんごを食べる() {
        let text = "hitu pahete gu si ro behapare da 6日後".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_彼女らは3日間りんごを食べている_食事としてりんごだけを食べている() {
        let text = "hiti hudidi pahete gu si ro behapare da 3日間".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_彼女らはりんごを5分間食べ続けている_りんごを食べる所要時間に5分かかっている() {
        let text = "hiti hudida pahete gu si ro behapare da 5分間".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_彼は常にりんごを食べている_四六時中ずっとりんごを食べている() {
        let text = "hudidu pahete gi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんごが3つある() {
        let text = "存在する number ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんご達がテーブルの上にある() {
        let text = "存在する da 机の上 ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがあのりんごを食べてしまった_その結果あのりんごは今はない() {
        let text = "hudu pahete ga si go ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごを食べそうだ() {
        let text = "hame hitu pahete ga si mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごそのりんごあのりんごを食べたそうだ() {
        let text = "kahehara mi si se pahete ga si binary_operator mo ro behapare binary_operator bo ro behapare go ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんごがひとつもない() {
        let text = "ne [存在する] ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない() {
        let text = "ne pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べられない_りんごがないので状況的に食べられない() {
        let text = "hane hate pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はりんごを床に落とした() {
        let text = "padaropa mi si ro behapare [on 床]".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんごが床に落ちた() {
        let text = "re padaropa ro behapare  [on 床]".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べられない_りんごの性質として可食でなくなった() {
        let text = "hapo re pahete ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたい() {
        let text = "hate pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを買いたい() {
        let text = "hate pabuhi mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_このりんごは汚い() {
        let text = "[x1は汚いもの] mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_このりんごは綺麗ではない() {
        let text = "ne [x1は綺麗なもの] mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あのりんごは食べられそうだ() {
        let text = "hako re padaropa bo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あれはりんごではない() {
        let text = "ne behapare ge".to_string();
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
        let text = "ku pahete ba si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べます() {
        let text = "kaho mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "pahete mi si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べないのですか_疑問() {
        let text = "ku ne pahete ba si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べません() {
        let text = "kaho mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "ne pahete mi si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べたかったですか() {
        let text = "ku hate hita pahete ba si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえ私はみかんを食べたくはありません() {
        let text = "ne kaho mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hate pahete mi si ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごもみかんも食べないのですか() {
        let text = "ku pahete ba si binary_operator ro behapare ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べます() {
        let text = "pahete ba si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はみかんを食べない_だからみかんは食べられない_受け身の否定() {
        let text =
            "ne pahete mi si ro behoranage [接続詞 なので 順接] ne re pahete ro behoranage si mi"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_りんごは食べることができない_なぜなら_りんごは綺麗ではないから() {
        let text =
            "hane hako re pahete ro behapare [接続詞 なぜなら 理由 原因] ne [綺麗] ro behoranage"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんを食べましょうよ_勧誘() {
        let text = "haki pahete ba si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえお断りします() {
        let text = "ne kaho mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_あそこでみかんを食べろ_りんごは食べるな() {
        let text = "haka re pahete ro behoranage [場所] [あそこ]".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "haka ne re pahete ro behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをあげる() {
        let text = "pagibe mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにあげる() {
        let text = "pagibeto mi si ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はあなたにりんごをあげる() {
        let text = "[event の and] pagibe mi si ba pagibeto mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        // 第二項はその他の型を取るようにして、無理やりtupleにする方法もありそう
        let text = "[??give] mi si [tuple] ba pagibeto".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_私はりんごをもらう() {
        let text = "[eventのand] re pagibeto mi , re pagibe ro behapare".to_string();
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
            "kasahi mi si se [eventBinaryOperator] ne pahete ba si ro behapare ne pahete ba si ro behoranage so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごを食べろと言うのですか() {
        let text = "ku kasahi ba si se pahete mi si ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_貴様はりんごを食べたいと言った() {
        let text = "hita kasahi ba si se hate pahete ba si ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたいと言った覚えがない() {
        let text =
            "ne karememabera mi si se hita kasahi mi si se hate pahete mi si ro behapare so so"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは確か私はりんごを食べたいと言ったはずだ() {
        let text = "hame hita kasahi ba si se hate pahete ba si ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はもしかしたらそう言ったかもしれない() {
        let text = "hamo hita kasahi mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_あなたは絶対に私に対してりんごを食べたいと言った() {
        let text = "hame hita kasahi ba si se hate pahete ba si ro behapare so [to] mi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は私にりんごを食べさせる() {
        let text = "kareta gi si se pahete ba si ro behapare so".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない() {
        let text = "ne re pahete ro behoranage si mi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_みかんは私に食べられていない_転置が否定の先にきた場合にエラーなる() {
        let text = "re ne pahete ro behoranage si mi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_りんごは彼によって私に食べさせられた() {
        let text = "re kareta se re pahete ro behapare si ba so si gi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_ここには何もない() {
        let text = "".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあるりんごを食べる() {
        // ∃x (apple(x))
        let text = "pahete mi si ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "pahete mi si mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はそのりんごを食べる() {
        // ∃x (apple(x) ∧ it(x))
        let text = "pahete mi si bo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあのりんごを食べる() {
        // ∃x (apple(x) ∧ that(x))
        let text = "pahete mi si go ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "pahete mi si number ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこの3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "pahete mi si number mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しのりんごを食べる() {
        // ∃=to x (apple(x))
        let text = "pahete mi si to ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しこのりんごを食べる() {
        // ∃=to x (apple(x) ∧ this(x))
        let text = "pahete mi si to mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しの3つのりんごを食べる_がエラーになること() {
        // ∀x ∃=3 (apple(x))
        let text = "pahete mi si number to ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "pahete mi si to number ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私は半分のりんごを食べる() {
        // ∃=ti x (apple(x))
        let text = "pahete mi si ti ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は多くのりんごを食べる() {
        // ∃=ta x (apple(x))
        let text = "pahete mi si te ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのりんごを食べる() {
        // ∀x (apple(x))
        let text = "pahete mi si ta ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのこのりんごを食べる() {
        // ∀x (apple(x) ∧ this(x))
        let text = "pahete mi si ta mo ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全ての3つのりんごを食べる_がエラーになること() {
        // ∀x ∃=3 (apple(x))
        let text = "pahete mi si number ta ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "pahete mi si ta number ro behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }
}
