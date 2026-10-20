pub mod argument_type_checker;
pub mod borrowing;
pub mod dictionary;
pub mod formal_semantics;
pub mod parser;
pub mod tokenizer;
pub mod types;

use argument_type_checker::type_check as argument_type_check;
use formal_semantics::{
    ContradictionCheck, Formula, check_contradiction, free_variables, is_schema_variable_name,
    to_fol,
};
use parser::{Ast, S, parse};
use tokenizer::{Range, TerminalType, add_eos_to_last_sentence, tokenize};
use types::ArgumentType;

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
        Err(e) => Err(vec![Diagnostic::error(
            e.range.clone(),
            format!("字句エラー: {}", e.message()),
        )]),
    }
}

pub fn diagnose_tokens(sentences: Vec<Vec<TerminalType>>) -> Result<Diagnosed, Vec<Diagnostic>> {
    let analysis = analyze_tokens(sentences);
    let contradiction = check_contradictions(&analysis);
    let mut diagnostics = analysis.diagnostics;
    diagnostics.extend(contradiction);
    finish(analysis.asts, diagnostics)
}

/// 矛盾検査を除く診断(構文解析・疑問詞の位置・項の型・スキーマ変数の束縛)の結果。
/// 矛盾検査は重くなりうるため、[`check_contradictions`]で別に行う。
#[derive(Debug, Clone)]
pub struct Analysis {
    pub asts: Vec<Ast>,
    pub diagnostics: Vec<Diagnostic>,
    /// 矛盾検査の前提にする文の論理式と範囲。疑問文は何も主張しないので含めない。
    premises: Vec<(Formula, Range)>,
}

impl Analysis {
    /// 矛盾検査を含まない診断結果を、[`diagnose_tokens`]と同じ形で返す。
    pub fn diagnosed(&self) -> Result<Diagnosed, Vec<Diagnostic>> {
        finish(self.asts.clone(), self.diagnostics.clone())
    }
}

pub fn analyze_tokens(sentences: Vec<Vec<TerminalType>>) -> Analysis {
    let sentences = add_eos_to_last_sentence(sentences);
    let mut asts = Vec::new();
    let mut diagnostics = Vec::new();
    let mut premises = Vec::new();
    for sentence in sentences {
        let sentence_range = Range::new(
            sentence[0].range().start(),
            sentence.last().unwrap().range().end(),
        );
        let ast = match parse(sentence.clone()) {
            Ok(ast) => ast,
            Err(e) => {
                diagnostics.push(Diagnostic::error(
                    e.error_token.range(),
                    format!("構文エラー: {}", e.error_message),
                ));
                continue;
            }
        };
        if !matches!(sentence.first(), Some(TerminalType::InterrogativeMarker(_))) {
            for token in &sentence {
                if let TerminalType::Argument(arg) = token
                    && arg.is_interrogative()
                {
                    diagnostics.push(Diagnostic::error(
                        token.range(),
                        "疑問詞koは疑問文(文頭にkuを置いた文)の中でのみ使えます",
                    ));
                }
            }
        }

        if let Err(e) = argument_type_check(&ast) {
            let mut message = format!("型エラー: {}", e.message);
            if !e.expected.is_empty() && !e.actual.is_empty() {
                message.push_str(&format!(
                    "(期待: {}、実際: {})",
                    join_types(&e.expected),
                    join_types(&e.actual)
                ));
            }
            diagnostics.push(Diagnostic::error(e.range, message));
        }

        let formula = to_fol(&ast);
        let free: Vec<_> = free_variables(&formula)
            .into_iter()
            .filter(|name| is_schema_variable_name(name))
            .collect();
        if !free.is_empty() {
            diagnostics.push(Diagnostic::error(
                sentence_range.clone(),
                format!(
                    "zeで量化されていないスキーマ変数があります: {}",
                    free.join(", ")
                ),
            ));
        }

        if !matches!(ast.get_s(), S::Two(_)) {
            premises.push((formula, sentence_range));
        }
        asts.push(ast);
    }
    Analysis {
        asts,
        diagnostics,
        premises,
    }
}

/// 文を順に前提に加えていき、矛盾が生じた最初の文(または検査を打ち切った文)を診断として返す。
pub fn check_contradictions(analysis: &Analysis) -> Option<Diagnostic> {
    let formulas: Vec<Formula> = analysis.premises.iter().map(|(f, _)| f.clone()).collect();
    let sentence_range = |idx: usize| analysis.premises[idx].1.clone();
    match check_contradiction(&formulas) {
        ContradictionCheck::Consistent => None,
        ContradictionCheck::Contradiction(idx) => Some(Diagnostic::error(
            sentence_range(idx),
            CONTRADICTION_MESSAGE,
        )),
        ContradictionCheck::Unknown(idx) => Some(Diagnostic::warning(
            sentence_range(idx),
            "矛盾検査を打ち切りました。この文以降の無矛盾性は検証されていません",
        )),
    }
}

const CONTRADICTION_MESSAGE: &str = "それまでの文と矛盾しています";

fn join_types(types: &[ArgumentType]) -> String {
    types
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join("・")
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
        let text = "peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_項の型不一致の診断メッセージに期待型と実際の型が含まれる() {
        let text = "kahoraka sa mi si ba".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics.iter().any(|d| d.message
                == "型エラー: 第2項(si)の型が述語と合いません(期待: 抽象・空間・時間、実際: 生物)"),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_未定義の先頭音節を持つ語はpanicせずエラーになる() {
        for text in ["koba sa mi si ba", "keba sa mi si ba", "zoba sa mi si ba"] {
            assert!(diagnose(text.to_string()).is_err(), "{text}");
        }
    }

    #[test]
    fn test_桁位置が重複する数詞はpanicせずエラーになる() {
        assert!(diagnose("parobe sa ta turupinipini si ba".to_string()).is_err());
    }

    #[test]
    fn test_パースエラーは原因のトークンを指し簡潔なメッセージになる() {
        let diagnostics = diagnose("parobe sa ta ba si ba".to_string()).unwrap_err();
        assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
        let range = &diagnostics[0].range;
        assert_eq!(
            (range.start().character(), range.end().character()),
            (13, 15)
        );
        assert_eq!(
            diagnostics[0].message,
            "構文エラー: 'ba'はここに置けません(DeterminedArgumentが必要です)"
        );
    }

    #[test]
    fn test_否定や入れ子の述語句の中の型不一致もエラーになる() {
        for text in [
            "ne hoba parobe kahoraka sa mi si ba",
            "hoba hoba parobe kahoraka parobe sa mi si ba",
            "hoba parobe hoba parobe kahoraka sa mi si ba",
        ] {
            assert!(diagnose(text.to_string()).is_err(), "{text}");
        }
    }

    #[test]
    fn test_0個の項は存在を含意しない() {
        let consistent =
            "peheta sa mi si tuso ri nohapare do ne peheta sa mi si to ri nohapare".to_string();
        assert!(diagnose(consistent).is_ok());
        let contradictory =
            "peheta sa mi si tuso ri nohapare do peheta sa mi si to ri nohapare".to_string();
        let diagnostics = diagnose(contradictory).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == CONTRADICTION_MESSAGE),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_毎日食べる文の矛盾検査が打ち切られない() {
        let text =
            "hudidi peheta sa ri paparatanera du mi si ri nohapare sugi ta ri zuda".to_string();
        let diagnosed = diagnose(text).unwrap();
        assert!(diagnosed.warnings.is_empty(), "{:?}", diagnosed.warnings);
    }

    #[test]
    fn test_矛盾検査はanalyze_tokensとは別に行える() {
        let text = "peheta sa mi si to ri nohapare do ne peheta sa mi si to ri nohapare";
        let analysis = analyze_tokens(tokenize(text).unwrap());
        assert!(analysis.diagnosed().is_ok());
        let contradiction = check_contradictions(&analysis).unwrap();
        assert_eq!(contradiction.severity, Severity::Error);
        assert_eq!(contradiction.range.start().character(), 34);
    }

    #[test]
    fn test_未知の語は字句エラーとしてその語を示す() {
        let diagnostics = diagnose("parobe sa mi si xyz".to_string()).unwrap_err();
        assert_eq!(diagnostics[0].message, "字句エラー: 未知の語です: xyz");
    }

    #[test]
    fn test_期待型か実際の型が空なら型エラーに型を表示しない() {
        let text = "ze ta mi parobe sa za si ba".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "型エラー: zeの量化子宣言で使える項はza/ziのみです"),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_私はりんごを食べた() {
        let text = "hita peheta sa mi si ri nohapare".to_string();
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
        let text = "pemobe sa mi subo taro".to_string();
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
        assert!(
            diagnostics
                .iter()
                .any(|d| d.message == "字句エラー: 固有名詞の終わりを示すhohoがありません"),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_彼はりんごを食べている() {
        let text = "hudida peheta sa gi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっている() {
        let text = "hudu peheta sa gi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女はりんごを食べ終わっていた() {
        let text = "hita hudu peheta sa gi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを食べたことがある() {
        let text = "hita kahekasaperinase sa ri paparatanera du mi si re he peheta sa ri paparatanera du mi si ri nohapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私の妻はりんごを毎日食べる() {
        let text =
            "hudidi peheta sa ri paparatanera du mi si ri nohapare sugi ta ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は昨日りんごを食べた() {
        let text = "hita peheta sa hoba mi ri paparatanera du mi si ri nohapare sugi ri zihita du tusa ri zuda"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私と私の妻は6日前にりんごを食べた() {
        let text = "hita peheta sa hoba mi ri paparatanera du mi si ri nohapare sugi ri zihita du tuke ri zuda"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼らは明日りんごを食べる() {
        let text = "hitu peheta sa gu si ri nohapare sugi ri zihitu du tusa ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }
    #[test]
    fn test_彼らは6日後にりんごを食べる() {
        let text = "hitu peheta sa gu si ri nohapare sugi ri zihitu du tuke ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らは3日間りんごを食べている_食事としてりんごだけを食べている() {
        let text =
            "hiti hudidi peheta sa gu si ri nohapare suge ri zihita du tusi ri zuda".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼女らはりんごを5分間食べ続けている_りんごを食べる所要時間に5分かかっている() {
        let text = "hiti hudida peheta sa gu si ri nohapare suge ri zihita du tuka ri zuminute"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は常にりんごを食べている_四六時中ずっとりんごを食べている() {
        let text = "hudidu peheta sa gi si ri nohapare sugi ri zuharahasa".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが3つある() {
        let text = "puhekasisata sa tusi ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんご達がテーブルの上にある() {
        let text = "puhekasisata sa tusa ri nohapare suni ri zahona du ri databare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがあのりんごを食べてしまった_その結果あのりんごは今はない() {
        let text = "hudu peheta sa ga si go ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごを食べそうだ() {
        let text = "hapa ta hitu peheta sa ga si mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_誰かがこのりんごそのりんごあのりんごを食べたそうだ() {
        let text = "kasema sa mi si re he peheta sa ga si hoba mo ri nohapare hoba bo ri nohapare go ri nohapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごがひとつもない() {
        let text = "ne puhekasisata sa ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない() {
        let text = "ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べない_二重否定がエラーになる() {
        let text = "ne ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_述語反意化_noは文法的に正しい() {
        let text = "no peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_述語反意化_no_noの二重スタックはエラーになる() {
        let text = "no no peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_述語反意化_noとneのスタックもエラーになる() {
        assert!(diagnose("no ne peheta sa mi si ri nohapare".to_string()).is_err());
        assert!(diagnose("ne no peheta sa mi si ri nohapare".to_string()).is_err());
    }

    #[test]
    fn test_私はりんごを食べるな() {
        let text = "hane hapi ta peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べるな_二重否定がエラーになる() {
        let text = "hane hane hapi ta peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた() {
        let text = "hine hita peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べた_二重否定がエラーになる() {
        let text = "hine hine hita peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを食べ始めてない() {
        let text = "hune hude peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べ始めてない_二重否定がエラーになる() {
        let text = "hune hune hude peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私はりんごを過去じゃないときに食べないを開始してないと事実性高く思わない() {
        let text = "hane hapa ta hine hita hune hude ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べられない_りんごがないので状況的に食べられない() {
        let text = "hane hapu to peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを床に落とした() {
        let text = "pedaropa sa mi si ri nohapare suno ri zahona du ri duharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごが床に落ちた() {
        let text = "pedaropa si ri nohapare suno ri zahona du ri duharora".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べられない_りんごの性質として可食でなくなった() {
        let text = "hapu ta peheta si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたい() {
        let text = "hapu to peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを買いたい() {
        let text = "hapu to pebu sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_このりんごは汚い() {
        let text = "hudu pedirati si mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_このりんごは綺麗ではない() {
        let text = "hudu ne pekarena si mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あのりんごは食べられそうだ() {
        let text = "hapu to pedaropa si bo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あれはりんごではない() {
        let text = "ne nohapare sa ge".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_これはりんごではなくみかんだ() {
        // hehenu(対比)はhesa側を否定し、hesi側を肯定する(¬P ∧ Q)。
        // hesa側(りんご)を明示的にneで否定しなくても、hehenu自体がhesa側を否定する。
        let text = "hehenu hesa nohapare si me hesi nohoranage si me".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehoma() {
        let text = "hehoma hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehomi() {
        let text = "hehomi hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehomu() {
        let text = "hehomu hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehitatu() {
        let text = "hehitatu hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hehitatu hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehiti() {
        let text = "hehiti hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hehito() {
        let text = "hehito hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito() {
        let text = "henohito hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito_過去反実仮想() {
        let text =
            "henohito hesa hita peheta sa mi si ri nohapare hesi hita peheta sa ba si ri nohoranage"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henohito_現在反実仮想() {
        let text =
            "henohito hesa hiti peheta sa mi si ri nohapare hesi hiti peheta sa ba si ri nohoranage"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hoba() {
        let text = "hoba peheta peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hobi() {
        let text = "hobi peheta peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_述語_hobu() {
        let text = "hobu peheta peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hoba() {
        let text = "peheta sa hoba mi ri paparatanera du mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hobi() {
        let text = "peheta sa hobi mi ri paparatanera du mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_項_hobu() {
        let text = "peheta sa hobu mi ri paparatanera du mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homa() {
        let text = "homa peheta sa mi si ri nohapare peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homi() {
        let text = "homi peheta sa mi si ri nohapare peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homu() {
        let text = "homu peheta sa mi si ri nohapare peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_演算子_命題_homaがhe_ho内で使える() {
        let text = "kahekasaperinase sa mi si re he homa peheta sa mi si ri nohapare peheta sa ba si ri nohoranage ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べるか_疑問() {
        let text = "ku peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べます() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "peheta sa mi si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べないのですか_疑問() {
        let text = "ku ne peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_はい私はみかんを食べません() {
        let text = "kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "ne peheta sa mi si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはみかんを食べたかったですか() {
        let text = "ku hapu to hita peheta sa ba si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_いいえ私はみかんを食べたくはありません() {
        let text = "ne kaho sa mi si be".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapu to peheta sa mi si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごもみかんも食べないのですか() {
        let text = "ku peheta sa ba si hoba ri nohapare ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べます() {
        let text = "peheta sa ba si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はみかんを食べない_だからみかんは食べられない_受け身の否定() {
        let text =
            "hesuteto hesa ne peheta sa mi si ri nohoranage hesi ne peheta si ri nohoranage sa mi"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_hesuteto() {
        let text = "hesuteto hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hesuteto hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_接続詞_henosuteto() {
        let text = "henosuteto hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "henosuteto hesi peheta sa ba si ri nohoranage hesa peheta sa mi si ri nohapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごは食べることができない_なぜなら_りんごは綺麗ではないから() {
        let text =
            "hane hapu to peheta si ri nohapare sute re he hudu ne pekarena si ri nohoranage ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんを食べましょうよ_勧誘() {
        let text = "hapi to peheta sa ba si ri nohapare".to_string();
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
        let text = "hapi ta peheta si ri nohoranage suni ge".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "hapi ta ne peheta si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをあげる() {
        let text = "pegibe sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにあげる() {
        let text = "pemobe sa mi subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあなたにりんごをあげる() {
        let text = "pemobe sa mi si ri nohapare sube mi subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごをもらう() {
        let text = "pemobe subo mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_りんごを頂いても宜しいですか_みかんは差し上げます() {
        let text = "ku pemobe si ri nohapare subo mi".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());

        let text = "pemobe sa mi si ri nohapare subo ba".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごは食べるなみかんを食べろと言った() {
        let text =
            "kasa sa mi si re he homa ne peheta sa ba si ri nohapare peheta sa ba si ri nohoranage ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたはりんごを食べろと言うのですか() {
        let text = "ku kasa sa ba si re he peheta sa mi si ri nohapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_貴様はりんごを食べたいと言った() {
        let text = "hita kasa sa ba si re he hapu to peheta sa ba si ri nohapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はりんごを食べたいと言った覚えがない() {
        let text =
            "ne karememabera sa mi si re he hita kasa sa mi si re he hapu to peheta sa mi si ri nohapare ho ho"
                .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_あなたは確か私はりんごを食べたいと言ったはずだ() {
        let text =
            "hapa ta hita kasa sa ba si re he hapu to peheta sa ba si ri nohapare ho".to_string();
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
        let text = "hapa ta hita kasa sa ba subo mi si re he kahanata sa ba si re he peheta sa ba si ri nohapare ho ho"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_彼は私にりんごを食べさせる() {
        let text = "kareta sa gi si re he peheta sa ba si ri nohapare ho".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない() {
        let text = "ne peheta sa mi si ri nohoranage".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_みかんは私に食べられていない_転置が否定の先にきた場合にエラーなる() {
        let text = "re ne peheta ri nohoranage si mi".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_りんごは彼によって私に食べさせられた() {
        let text = "kareta si re he peheta si ri nohapare sa ba ho sa gi".to_string();
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
        let text = "peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "peheta sa mi si mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はそのりんごを食べる() {
        // ∃x (apple(x) ∧ it(x))
        let text = "peheta sa mi si bo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はあのりんごを食べる() {
        // ∃x (apple(x) ∧ that(x))
        let text = "peheta sa mi si go ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "peheta sa mi si tusi ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私はこの3つのりんごを食べる() {
        // ∃=3 x (apple(x))
        let text = "peheta sa mi si tusi mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しのりんごを食べる() {
        // ∃x (apple(x))
        let text = "peheta sa mi si to ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しこのりんごを食べる() {
        // ∃x (apple(x) ∧ this(x))
        let text = "peheta sa mi si to mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は少しの3つのりんごを食べる_がエラーになること() {
        // to(数量詞)とtusi(数)は同じ項に同時指定できずパースエラーになる
        let text = "peheta sa mi si tusi to ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "peheta sa mi si to tusi ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_私は半分のりんごを食べる() {
        // ∃=ti x (apple(x))
        let text = "peheta sa mi si ti ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は多くのりんごを食べる() {
        // ∃=te x (apple(x))
        let text = "peheta sa mi si te ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのりんごを食べる() {
        // ∀x (apple(x) -> eat(mi, x))
        let text = "peheta sa mi si ta ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全てのこのりんごを食べる() {
        // ∀x (apple(x) -> eat(mi, x))
        let text = "peheta sa mi si ta mo ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_私は全ての3つのりんごを食べる_がエラーになること() {
        // ta(数量詞)とtusi(数)は同じ項に同時指定できずパースエラーになる
        let text = "peheta sa mi si tusi ta ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_err());

        let text = "peheta sa mi si ta tusi ri nohapare".to_string();
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
        let text = "hehoma hesa peheta sa mi si ri nohapare hesi peheta sa ba si ri nohoranage"
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
            "hapa ta peheta sa mi si ri nohapare do ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == CONTRADICTION_MESSAGE)
        );
    }

    #[test]
    fn test_能力的可能は現実世界での否定と矛盾しない() {
        let text =
            "hapu to peheta sa mi si ri nohapare do ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_義務的必然は現実世界での違反と矛盾しない() {
        let text =
            "hapi ta peheta sa mi si ri nohapare do ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_義務的必然と義務的可能の否定は矛盾する() {
        let text =
            "hapi ta peheta sa mi si ri nohapare do hane hapi to peheta sa mi si ri nohapare"
                .to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == CONTRADICTION_MESSAGE)
        );
    }

    #[test]
    fn test_過去の出来事と過去に出来事が無いことは矛盾する() {
        let text =
            "hita peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        let diagnostics = result.unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == CONTRADICTION_MESSAGE)
        );
    }

    #[test]
    fn test_テンス省略の出来事と過去に出来事が無いことは矛盾しない() {
        let text = "peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_多くの否定は存在と矛盾しない() {
        let text = "peheta sa mi si ri nohapare do ne peheta sa mi si te ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_疑問詞koを含む疑問文が診断を通る() {
        for text in [
            "ku peheta sa ko si ri nohapare",
            "ku peheta sa mi si ko",
            "ku peheta sa mi si ko sugi ko suni ko",
            "ku peheta sa ri paparatanera du ko si ri nohapare",
        ] {
            let result = diagnose(text.to_string());
            assert!(result.is_ok(), "{text}: {result:?}");
        }
    }

    #[test]
    fn test_kuのない文で疑問詞koを使うとエラーになる() {
        let diagnostics = diagnose("peheta sa ko si ri nohapare".to_string()).unwrap_err();
        assert!(
            diagnostics.iter().any(|d| d.message.contains("疑問詞ko")),
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
        let text = "peheta sa mi si ri nohapare do ku ne peheta sa mi si ri nohapare".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_疑問文を除いた前提どうしの矛盾は疑問文の後ろでも検出する() {
        let text = "peheta sa mi si ri nohapare do ku peheta sa ko si ri nohapare do ne peheta sa mi si ri nohapare".to_string();
        let third_sentence_start = text.find("ne peheta").unwrap();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == CONTRADICTION_MESSAGE
                    && d.range.start().character() == third_sentence_start),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn test_raで抽出した項とriで抽出した項は矛盾しない() {
        let text = "parobe sa mi si ra peheta do ne parobe sa mi si se ri peheta".to_string();
        let result = diagnose(text);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn test_私が食べたことと誰も食べていないことは矛盾する() {
        let text = "peheta sa mi si ri nohapare do ne peheta sa ga si ri nohapare".to_string();
        let diagnostics = diagnose(text).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|d| d.message == CONTRADICTION_MESSAGE),
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
            "hita hudu peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare",
            "hita hudo peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare",
        ] {
            let diagnostics = diagnose(text.to_string()).unwrap_err();
            assert!(
                diagnostics
                    .iter()
                    .all(|d| d.message == CONTRADICTION_MESSAGE),
                "{text}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn test_過去に食べていた途中や食べ始めたことは過去に食べていないことと矛盾しない() {
        // 進行・開始は出来事が発話時まで続いている可能性があるため、出来事全体が過去とは言えない。
        for text in [
            "hita hudida peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare",
            "hita hude peheta sa mi si ri nohapare do hita ne peheta sa mi si ri nohapare",
        ] {
            let result = diagnose(text.to_string());
            assert!(result.is_ok_and(|d| d.warnings.is_empty()), "{text}");
        }
    }

    #[test]
    fn test_習慣の否定は1回の出来事と矛盾しない() {
        let text = "hita peheta sa mi si ri nohapare do hune hudidi peheta sa mi si ri nohapare"
            .to_string();
        let result = diagnose(text);
        assert!(result.is_ok());
    }
}
