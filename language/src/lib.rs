pub mod argument_type_checker;
pub mod borrowing;
pub mod formal_semantics;
pub mod parser;
pub mod tokenizer;
pub mod types;

use argument_type_checker::type_check as argument_type_check;
use formal_semantics::{
    ContradictionCheck, check_contradiction, free_variables, is_schema_variable_name, to_fol,
};
use parser::{Ast, S, parse};
use tokenizer::{Range, TerminalType, add_eos_to_last_sentence, tokenize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    pub fn error(range: Range, message: impl Into<String>) -> Self {
        Diagnostic {
            range,
            severity: Severity::Error,
            message: message.into(),
        }
    }

    pub fn warning(range: Range, message: impl Into<String>) -> Self {
        Diagnostic {
            range,
            severity: Severity::Warning,
            message: message.into(),
        }
    }
}

/// エラーが無かった場合の診断結果。警告は残りうる。
#[derive(Debug, Clone)]
pub struct Diagnosed {
    pub asts: Vec<Ast>,
    pub warnings: Vec<Diagnostic>,
}

/// エラーが1つでもあれば全診断を`Err`で、無ければASTと警告を`Ok`で返す。
pub fn diagnose(text: String) -> Result<Diagnosed, Vec<Diagnostic>> {
    let tokenize_result = tokenize(&text);
    match tokenize_result {
        Ok(sentences) => diagnose_tokens(sentences),
        Err(e) => Err(vec![Diagnostic::error(e.range, "tokenize error")]),
    }
}

pub fn diagnose_tokens(sentences: Vec<Vec<TerminalType>>) -> Result<Diagnosed, Vec<Diagnostic>> {
    // parsing
    let sentences = add_eos_to_last_sentence(sentences);
    let mut ast_with_ranges: Vec<(Ast, Range)> = Vec::new();
    let mut diagnostics = Vec::new();
    for sentence in sentences {
        let sentence_range = Range::new(
            sentence[0].range().start(),
            sentence.last().unwrap().range().end(),
        );
        let parse_result = parse(sentence.clone());
        if let Err(e) = parse_result {
            diagnostics.push(Diagnostic::error(
                e.error_token.range(),
                format!("parse error: {}", e.error_message),
            ));
        } else {
            if !matches!(sentence.first(), Some(TerminalType::InterrogativeMarker(_))) {
                for token in &sentence {
                    if let TerminalType::Argument(arg) = token
                        && arg.is_interrogative()
                    {
                        diagnostics.push(Diagnostic::error(
                            token.range(),
                            "interrogative ko can only be used in a question (start the sentence with ku)",
                        ));
                    }
                }
            }
            ast_with_ranges.push((parse_result.unwrap(), sentence_range));
        }
    }
    let asts: Vec<Ast> = ast_with_ranges.iter().map(|(a, _)| a.clone()).collect();

    // argument type checking
    for ast in &asts {
        let argument_type_check_result = argument_type_check(ast);
        if let Err(e) = argument_type_check_result {
            diagnostics.push(Diagnostic::error(
                e.range,
                format!(
                    "argument type error: {} (expected: {:?}, actual: {:?})",
                    e.message, e.expected, e.actual
                ),
            ));
        }
    }

    // formula construction + unbound schema variable check
    let formulas: Vec<_> = ast_with_ranges.iter().map(|(a, _)| to_fol(a)).collect();
    for (formula, (_, range)) in formulas.iter().zip(ast_with_ranges.iter()) {
        let free: Vec<_> = free_variables(formula)
            .into_iter()
            .filter(|name| is_schema_variable_name(name))
            .collect();
        if !free.is_empty() {
            diagnostics.push(Diagnostic::error(
                range.clone(),
                format!(
                    "unbound schema variable(s) (use ze to quantify): {}",
                    free.join(", ")
                ),
            ));
        }
    }

    // contradiction checking (a question asserts nothing, so it is not used as a premise)
    let premise_indices: Vec<usize> = ast_with_ranges
        .iter()
        .enumerate()
        .filter(|(_, (ast, _))| !matches!(ast.get_s(), S::Two(_)))
        .map(|(i, _)| i)
        .collect();
    let premises: Vec<_> = premise_indices
        .iter()
        .map(|&i| formulas[i].clone())
        .collect();
    let sentence_range =
        |premise_idx: usize| ast_with_ranges[premise_indices[premise_idx]].1.clone();
    match check_contradiction(&premises) {
        ContradictionCheck::Consistent => {}
        ContradictionCheck::Contradiction(idx) => {
            diagnostics.push(Diagnostic::error(
                sentence_range(idx),
                "contradiction error",
            ));
        }
        ContradictionCheck::Unknown(idx) => {
            diagnostics.push(Diagnostic::warning(
                sentence_range(idx),
                "contradiction check gave up: consistency from this sentence onward is not verified",
            ));
        }
    }

    finish(asts, diagnostics)
}

fn finish(asts: Vec<Ast>, diagnostics: Vec<Diagnostic>) -> Result<Diagnosed, Vec<Diagnostic>> {
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        Err(diagnostics)
    } else {
        Ok(Diagnosed {
            asts,
            warnings: diagnostics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_私はりんごを食べる() {
        let text = "paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_項の型不一致の診断メッセージに期待型と実際の型が含まれる() {
        let text = "kahoraka sa mi si ba".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("expected") && d.message.contains("actual")),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_私はりんごを食べた() {
        let text = "hita paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_固有名詞を含む文が診断を通る() {
        let text = "parobe sa heba Kanye hoho si ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "expected diagnose to succeed: {result:?}");
    }

    #[test]
    fn test_tで始まる固有名詞を含む文が診断を通る() {
        let text = "pamobe sa mi subo taro".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "expected diagnose to succeed: {result:?}");
    }

    #[test]
    fn test_tで始まる固有名詞の型が合わない文は診断エラーになる() {
        let text = "parobe sa tetitanika si ba".to_string();
        assert!(diagnose(text).is_err());
    }

    #[test]
    fn test_閉じマーカーの無い固有名詞はトークナイズエラーになる() {
        let text = "parobe sa heba Kanye si ba".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(diagnostics.iter().any(|d| d.message == "tokenize error"));
    }

    #[test]
    fn test_彼はりんごを食べている() {
        let text = "hudida paheta sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっている() {
        let text = "hudu paheta sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっていた() {
        let text = "hita hudu paheta sa gi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを食べたことがある() {
        let text = "hita kahekasaperinase sa ri paparatanera du mi si re he paheta sa ri paparatanera du mi si ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを毎日食べる() {
        let text =
            "hudidi paheta sa ri paparatanera du mi si ri behapare sugi ta ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は昨日りんごを食べた() {
        let text = "hita paheta sa hoba mi ri paparatanera du mi si ri behapare sugi ri zuhita du tusa ri zuda"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は6日前にりんごを食べた() {
        let text = "hita paheta sa hoba mi ri paparatanera du mi si ri behapare sugi ri zuhita du tuke ri zuda"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼らは明日りんごを食べる() {
        let text = "hitu paheta sa gu si ri behapare sugi ri zuhitu du tusa ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }
    #[test]
    fn test_彼らは6日後にりんごを食べる() {
        let text = "hitu paheta sa gu si ri behapare sugi ri zuhitu du tuke ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らは3日間りんごを食べている_食事としてりんごだけを食べている() {
        let text =
            "hiti hudidi paheta sa gu si ri behapare suge ri zuhita du tusi ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らはりんごを5分間食べ続けている_りんごを食べる所要時間に5分かかっている() {
        let text = "hiti hudida paheta sa gu si ri behapare suge ri zuhita du tuka ri zuminute"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は常にりんごを食べている_四六時中ずっとりんごを食べている() {
        let text = "hudidu paheta sa gi si ri behapare sugi ri zuharahasa".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが3つある() {
        let text = "pihekasisata sa tusi ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんご達がテーブルの上にある() {
        let text = "pihekasisata sa tusa ri behapare suni ri zahona du ri ditabare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがあのりんごを食べてしまった_その結果あのりんごは今はない() {
        let text = "hudu paheta sa ga si go ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごを食べそうだ() {
        let text = "hapa ta hitu paheta sa ga si mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごそのりんごあのりんごを食べたそうだ() {
        let text = "kasema sa mi si re he paheta sa ga si hoba mo ri behapare hoba bo ri behapare go ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごがひとつもない() {
        let text = "ne pihekasisata sa ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない() {
        let text = "ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない_二重否定がエラーになる() {
        let text = "ne ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_述語反意化_noは文法的に正しい() {
        let text = "no paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_述語反意化_no_noの二重スタックはエラーになる() {
        let text = "no no paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_述語反意化_noとneのスタックもエラーになる() {
        assert!(diagnose("no ne paheta sa mi si ri behapare".to_string()).is_err());
        assert!(diagnose("ne no paheta sa mi si ri behapare".to_string()).is_err());
    }

    #[test]
    fn test_私はりんごを食べるな() {
        let text = "hane hapi ta paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べるな_二重否定がエラーになる() {
        let text = "hane hane hapi ta paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた() {
        let text = "hine hita paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた_二重否定がエラーになる() {
        let text = "hine hine hita paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを食べ始めてない() {
        let text = "hune hude paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べ始めてない_二重否定がエラーになる() {
        let text = "hune hune hude paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べないを開始してないと事実性高く思わない() {
        let text = "hane hapa ta hine hita hune hude ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べられない_りんごがないので状況的に食べられない() {
        let text = "hane hapu to paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを床に落とした() {
        let text = "padaropa sa mi si ri behapare suno ri zahona du ri diharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが床に落ちた() {
        let text = "padaropa si ri behapare suno ri zahona du ri diharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べられない_りんごの性質として可食でなくなった() {
        let text = "hapu ta paheta si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたい() {
        let text = "hapu to paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを買いたい() {
        let text = "hapu to pabu sa mi si ri behapare".to_string();
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
        let text = "hapu to padaropa si bo ri behapare".to_string();
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
    fn test_これはりんごではなくみかんだ() {
        // hehenu(対比)はhesa側を否定し、hesi側を肯定する(¬P ∧ Q)。
        // hesa側(りんご)を明示的にneで否定しなくても、hehenu自体がhesa側を否定する。
        let text = "hehenu hesa behapare si me hesi behoranage si me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehoma() {
        let text = "hehoma hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehomi() {
        let text = "hehomi hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehomu() {
        let text = "hehomu hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehitatu() {
        let text = "hehitatu hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hehitatu hesi paheta sa ba si ri behoranage hesa paheta sa mi si ri behapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehiti() {
        let text = "hehiti hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehito() {
        let text = "hehito hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito() {
        let text = "henohito hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito_過去反実仮想() {
        let text =
            "henohito hesa hita paheta sa mi si ri behapare hesi hita paheta sa ba si ri behoranage"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito_現在反実仮想() {
        let text =
            "henohito hesa hiti paheta sa mi si ri behapare hesi hiti paheta sa ba si ri behoranage"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hoba() {
        let text = "hoba paheta paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hobi() {
        let text = "hobi paheta paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hobu() {
        let text = "hobu paheta paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hoba() {
        let text = "paheta sa hoba mi ri paparatanera du mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hobi() {
        let text = "paheta sa hobi mi ri paparatanera du mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hobu() {
        let text = "paheta sa hobu mi ri paparatanera du mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homa() {
        let text = "homa paheta sa mi si ri behapare paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homi() {
        let text = "homi paheta sa mi si ri behapare paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homu() {
        let text = "homu paheta sa mi si ri behapare paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homaがhe_ho内で使える() {
        let text = "kahekasaperinase sa mi si re he homa paheta sa mi si ri behapare paheta sa ba si ri behoranage ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べるか_疑問() {
        let text = "ku paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べます() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "paheta sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べないのですか_疑問() {
        let text = "ku ne paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べません() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "ne paheta sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べたかったですか() {
        let text = "ku hapu to hita paheta sa ba si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえ私はみかんを食べたくはありません() {
        let text = "ne kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapu to paheta sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごもみかんも食べないのですか() {
        let text = "ku paheta sa ba si hoba ri behapare ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べます() {
        let text = "paheta sa ba si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はみかんを食べない_だからみかんは食べられない_受け身の否定() {
        let text =
            "hesuteto hesa ne paheta sa mi si ri behoranage hesi ne paheta si ri behoranage sa mi"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hesuteto() {
        let text = "hesuteto hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hesuteto hesi paheta sa ba si ri behoranage hesa paheta sa mi si ri behapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henosuteto() {
        let text = "henosuteto hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "henosuteto hesi paheta sa ba si ri behoranage hesa paheta sa mi si ri behapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べることができない_なぜなら_りんごは綺麗ではないから() {
        let text =
            "hane hapu to paheta si ri behapare sute re he hudu ne pakarena sa ri behoranage ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんを食べましょうよ_勧誘() {
        let text = "hapi to paheta sa ba si ri behapare".to_string();
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
        let text = "hapi ta paheta si ri behoranage suni ge".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapi ta ne paheta si ri behoranage".to_string();
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
        let text = "pamobe sa mi subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにりんごをあげる() {
        let text = "pamobe sa mi si ri behapare sube mi subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをもらう() {
        let text = "pamobe subo mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごを頂いても宜しいですか_みかんは差し上げます() {
        let text = "ku pamobe si ri behapare subo mi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "pamobe sa mi si ri behapare subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごは食べるなみかんを食べろと言った() {
        let text =
            "kasa sa mi si re he homa ne paheta sa ba si ri behapare paheta sa ba si ri behoranage ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごを食べろと言うのですか() {
        let text = "ku kasa sa ba si re he paheta sa mi si ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_貴様はりんごを食べたいと言った() {
        let text = "hita kasa sa ba si re he hapu to paheta sa ba si ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたいと言った覚えがない() {
        let text =
            "ne karememabera sa mi si re he hita kasa sa mi si re he hapu to paheta sa mi si ri behapare ho ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは確か私はりんごを食べたいと言ったはずだ() {
        let text =
            "hapa ta hita kasa sa ba si re he hapu to paheta sa ba si ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はもしかしたらそう言ったかもしれない() {
        let text = "hapa to hita kasa sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは絶対に私に対してりんごを食べたいと言った() {
        let text = "hapa ta hita kasa sa ba subo mi si re he kahanata sa ba si re he paheta sa ba si ri behapare ho ho"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は私にりんごを食べさせる() {
        let text = "kareta sa gi si re he paheta sa ba si ri behapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない() {
        let text = "ne paheta sa mi si ri behoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない_転置が否定の先にきた場合にエラーなる() {
        let text = "re ne paheta ri behoranage si mi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_りんごは彼によって私に食べさせられた() {
        let text = "kareta si re he paheta si ri behapare sa ba ho sa gi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ここには何もない() {
        let text =
            "ne puhekasisata sa hobi ri dasinaga hobi ri disinaga ri dusinaga suni me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "ne puhekasisata sa ri hobi dasinaga hobi disinaga dusinaga suni me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあるりんごを食べる() {
        // ∃x (apple(x))
        let text = "paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "paheta sa mi si mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はそのりんごを食べる() {
        // ∃x (apple(x) ∧ it(x))
        let text = "paheta sa mi si bo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあのりんごを食べる() {
        // ∃x (apple(x) ∧ that(x))
        let text = "paheta sa mi si go ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "paheta sa mi si tusi ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこの3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "paheta sa mi si tusi mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しのりんごを食べる() {
        // ∃x (apple(x))
        let text = "paheta sa mi si to ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "paheta sa mi si to mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しの3つのりんごを食べる_がエラーになること() {
        // to(数量詞)とtusi(数)は同じ項に同時指定できずパースエラーになる
        let text = "paheta sa mi si tusi to ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "paheta sa mi si to tusi ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私は半分のりんごを食べる() {
        // ∃=ti x (apple(x))
        let text = "paheta sa mi si ti ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は多くのりんごを食べる() {
        // ∃=te x (apple(x))
        let text = "paheta sa mi si te ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのりんごを食べる() {
        // ∀x (apple(x) -> eat(mi, x))
        let text = "paheta sa mi si ta ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのこのりんごを食べる() {
        // ∀x (apple(x) -> eat(mi, x))
        let text = "paheta sa mi si ta mo ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全ての3つのりんごを食べる_がエラーになること() {
        // ta(数量詞)とtusi(数)は同じ項に同時指定できずパースエラーになる
        let text = "paheta sa mi si tusi ta ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "paheta sa mi si ta tusi ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_seで型が合わない項を比喩として使える() {
        // parobe(x1,x2)はx1,x2ともに生物(bV)を要求するが、didora(人工物,dV)をseで主格に入れられる
        let text = "parobe sa se ri didora si ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_seがないと型が合わない項はエラーになる() {
        let text = "parobe sa ri didora si ba".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_zaとziはzeが無いと未束縛変数エラーになる() {
        let text = "kaho sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_zeとhehezuで定義文が書ける() {
        // ∀za.∀zi. (kagurita(za,zi) ↔ kaho(za,zi))
        let text =
            "ze ta za ta zi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zeの中でも宣言した変数だけが閉じられる() {
        // 単項述語の定義: zeの直後でzaだけ宣言する場合、Forall(zi, ...)は付かない
        // (pi[CV]+は生物x1のみの自動詞なのでx2は無い)。
        let text = "ze ta za hehezu hesa piruna sa za hesi kaho sa za si za".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ze無しの通常の文は今まで通り通る() {
        let text = "hehoma hesa paheta sa mi si ri behapare hesi paheta sa ba si ri behoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_宣言し忘れた変数はzeの中でも未束縛エラーになる() {
        // zi用の宣言(ta zi / to zi)を忘れると、ziは自由変数のまま残りエラーになる。
        let text = "ze ta za hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_za_zi以外を量化子宣言に使うとエラーになる() {
        let text = "ze ta mi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_teやtiを量化子宣言に使うとエラーになる() {
        let text =
            "ze te za to zi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_taとtoでforallとexistsを使い分けられる() {
        use formal_semantics::{Formula, to_fol};

        // ze ta za to zi [本文]: zaは∀、ziは∃で閉じる。
        let text =
            "ze ta za to zi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let tokens = tokenizer::tokenize(&text).unwrap();
        let sentences = tokenizer::add_eos_to_last_sentence(tokens);
        let ast = parser::parse(sentences[0].clone()).unwrap();
        let formula = to_fol(&ast);

        match formula {
            Formula::Forall(x1, body1) => {
                assert_eq!(x1, "za");
                match *body1 {
                    Formula::Exists(x2, body2) => {
                        assert_eq!(x2, "zi");
                        assert!(matches!(*body2, Formula::And(_, _)));
                    }
                    other => panic!("expected nested Exists(zi, ...), got {other:?}"),
                }
            }
            other => panic!("expected Forall(za, ...), got {other:?}"),
        }
    }

    #[test]
    fn test_zeとhehezuの論理式がforallとiffの組み合わせになっている() {
        use formal_semantics::{Formula, to_fol};

        let text =
            "ze ta za ta zi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi".to_string();
        let tokens = tokenizer::tokenize(&text).unwrap();
        let sentences = tokenizer::add_eos_to_last_sentence(tokens);
        let ast = parser::parse(sentences[0].clone()).unwrap();
        let formula = to_fol(&ast);

        match formula {
            Formula::Forall(x1, body1) => {
                assert_eq!(x1, "za");
                match *body1 {
                    Formula::Forall(x2, body2) => {
                        assert_eq!(x2, "zi");
                        assert!(matches!(*body2, Formula::And(_, _)));
                    }
                    other => panic!("expected nested Forall(zi, ...), got {other:?}"),
                }
            }
            other => panic!("expected Forall(za, ...), got {other:?}"),
        }
    }

    #[test]
    fn test_2つの無関係な定義文をまとめて解析しても束縛が独立している() {
        let text = "ze ta za ta zi hehezu hesa kagurita sa za si zi hesi kaho sa za si zi do ze ta za ta zi hehezu hesa guzorata sa za si zi hesi parobe sa za si zi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_認識的必然は現実世界でも成り立つので否定と矛盾する() {
        let text =
            "hapa ta paheta sa mi si ri behapare do ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == "contradiction error")
        );
    }

    #[test]
    fn test_能力的可能は現実世界での否定と矛盾しない() {
        let text =
            "hapu to paheta sa mi si ri behapare do ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_義務的必然は現実世界での違反と矛盾しない() {
        let text =
            "hapi ta paheta sa mi si ri behapare do ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_義務的必然と義務的可能の否定は矛盾する() {
        let text =
            "hapi ta paheta sa mi si ri behapare do hane hapi to paheta sa mi si ri behapare"
                .to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == "contradiction error")
        );
    }

    #[test]
    fn test_過去の出来事と過去に出来事が無いことは矛盾する() {
        let text =
            "hita paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == "contradiction error")
        );
    }

    #[test]
    fn test_テンス省略の出来事と過去に出来事が無いことは矛盾しない() {
        let text = "paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_多くの否定は存在と矛盾しない() {
        let text = "paheta sa mi si ri behapare do ne paheta sa mi si te ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_疑問詞koを含む疑問文が診断を通る() {
        for text in [
            "ku paheta sa ko si ri behapare",
            "ku paheta sa mi si ko",
            "ku paheta sa mi si ko sugi ko suni ko",
            "ku paheta sa ri paparatanera du ko si ri behapare",
        ] {
            let result = diagnose(text.to_string());
            assert!(result.is_ok(), "{text}: {result:?}");
        }
    }

    #[test]
    fn test_kuのない文で疑問詞koを使うとエラーになる() {
        let diagnostics = diagnose("paheta sa ko si ri behapare".to_string()).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message.contains("interrogative ko")),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_疑問詞koは単項述語のx2には置けない() {
        let result = diagnose("ku piruna sa mi si ko".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_疑問文は矛盾判定の前提に含めない() {
        let text = "paheta sa mi si ri behapare do ku ne paheta sa mi si ri behapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_疑問文を除いた前提どうしの矛盾は疑問文の後ろでも検出する() {
        let text = "paheta sa mi si ri behapare do ku paheta sa ko si ri behapare do ne paheta sa mi si ri behapare".to_string();
        let third_sentence_start = text.find("ne paheta").unwrap();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == "contradiction error"
                    && d.range.start().character() == third_sentence_start),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_raで抽出した項とriで抽出した項は矛盾しない() {
        let text = "parobe sa mi si ra paheta do ne parobe sa mi si ri paheta".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_私が食べたことと誰も食べていないことは矛盾する() {
        let text = "paheta sa mi si ri behapare do ne paheta sa ga si ri behapare".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == "contradiction error"),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_警告だけならokで警告を返しエラーがあればerrになる() {
        let range = tokenize("mi").unwrap()[0][0].range();
        let warning = Diagnostic::warning(range.clone(), "warning");
        let diagnosed = finish(vec![], vec![warning.clone()]).unwrap();
        assert_eq!(diagnosed.warnings.len(), 1);

        let error = Diagnostic::error(range, "error");
        let diagnostics = finish(vec![], vec![warning, error]).unwrap_err();
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn test_過去に食べ終えていたことと過去に食べていないことは矛盾する() {
        for text in [
            "hita hudu paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare",
            "hita hudo paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare",
        ] {
            let diagnostics = diagnose(text.to_string()).unwrap_err();
            assert!(
                diagnostics
                    .iter()
                    .all(|d| d.message == "contradiction error"),
                "{text}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn test_過去に食べていた途中や食べ始めたことは過去に食べていないことと矛盾しない() {
        // 進行・開始は出来事が発話時まで続いている可能性があるため、出来事全体が過去とは言えない。
        for text in [
            "hita hudida paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare",
            "hita hude paheta sa mi si ri behapare do hita ne paheta sa mi si ri behapare",
        ] {
            let result = diagnose(text.to_string());
            assert!(result.is_ok_and(|d| d.warnings.is_empty()), "{text}");
        }
    }

    #[test]
    fn test_習慣の否定は1回の出来事と矛盾しない() {
        let text = "hita paheta sa mi si ri behapare do hune hudidi paheta sa mi si ri behapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }
}
