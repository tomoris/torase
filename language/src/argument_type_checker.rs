//! 格ラベルごとの選択制限(項の意味的な型)を検証する型チェッカー。
//! `formal_semantics::type_checker` のモンタギュー流の型システムとは別の、格構造に基づく型システムを扱う。

use std::collections::HashSet;
use std::hash::Hash;

use crate::parser::*;
use crate::tokenizer::{CaseLabelKind, Quantifier, Range};
use crate::types::{ArgumentType, PhysicalEntity};

pub struct TypeMismatch {
    pub range: Range,
    pub actual: Vec<ArgumentType>,
    pub expected: Vec<ArgumentType>,
    pub message: String,
}

type TypeMismatchResult = Result<(), TypeMismatch>;

pub fn type_check(ast: &Ast) -> TypeMismatchResult {
    ast.get_s().type_check()
}

impl S {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            S::One(declarative_sentence) => declarative_sentence.type_check(),
            S::Two(interrogative_sentence) => interrogative_sentence.type_check(),
            S::Three(universal_sentence) => universal_sentence.type_check(),
        }
    }
}

impl DeclarativeSentence {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            DeclarativeSentence::One(post_modified_mta_event) => {
                post_modified_mta_event.type_check()
            }
        }
    }
}

impl InterrogativeSentence {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            InterrogativeSentence::One(_interrogative_marker, post_modified_mta_event) => {
                post_modified_mta_event.type_check()
            }
        }
    }
}

impl UniversalSentence {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            UniversalSentence::One(
                _universal_marker,
                quantified_variable_list,
                post_modified_mta_event,
            ) => {
                quantified_variable_list.type_check()?;
                post_modified_mta_event.type_check()
            }
        }
    }
}

/// `ze`の量化子宣言・様相の力として使えるのは`ta`(∀/必然)・`to`(∃/可能)のみ。
/// `te`/`ti`はNP量化詞(`ta X`等)専用で、ここでは意味を持たない。
fn check_universal_existential_quantifier(quantifier: &Quantifier) -> TypeMismatchResult {
    let s = quantifier.get_string();
    if s == "ta" || s == "to" {
        Ok(())
    } else {
        Err(TypeMismatch {
            range: quantifier.get_range(),
            actual: vec![],
            expected: vec![],
            message: "ここで使える量化子はta(∀/必然)/to(∃/可能)のみです".to_string(),
        })
    }
}

impl QuantifiedVariableList {
    /// `ze`直後で宣言された変数は`za`/`zi`(スキーマ変数)でなければならない。
    fn type_check(&self) -> TypeMismatchResult {
        for (force, arg) in self.variables() {
            check_universal_existential_quantifier(force)?;
            if !arg.is_schema_variable() {
                return Err(TypeMismatch {
                    range: arg.get_range(),
                    actual: arg.get_type(),
                    expected: vec![],
                    message: "zeの量化子宣言で使える項はza/ziのみです".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl PostModifiedMtaEvent {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            PostModifiedMtaEvent::One(mta_event) => mta_event.type_check(),
            PostModifiedMtaEvent::Two(_connective_head, role_arg1, role_arg2) => {
                role_arg1.get_mta_event().type_check()?;
                role_arg2.get_mta_event().type_check()?;
                check_role_arg_pair(role_arg1, role_arg2)
            }
        }
    }
}

fn check_role_arg_pair(role_arg1: &RoleArg, role_arg2: &RoleArg) -> TypeMismatchResult {
    if RoleArg::resolve_hesa_hesi(role_arg1, role_arg2).is_some() {
        Ok(())
    } else {
        Err(TypeMismatch {
            range: role_arg2.get_range(),
            actual: vec![],
            expected: vec![],
            message: "接続詞の2つの節はhesa/hesiちょうど1つずつで役割標識されなければなりません"
                .to_string(),
        })
    }
}

impl ModalityExpression {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ModalityExpression::One(modality_with_agent) => modality_with_agent.type_check(),
            ModalityExpression::Epsilon => Ok(()),
        }
    }
}

impl ModifiedModality {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ModifiedModality::One(_modality, force) => {
                check_universal_existential_quantifier(force)
            }
            ModifiedModality::Two(_modality_negation, _modality, force) => {
                check_universal_existential_quantifier(force)
            }
        }
    }
}

impl TenseExpression {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            TenseExpression::One(modified_tense) => modified_tense.type_check(),
            TenseExpression::Epsilon => Ok(()),
        }
    }
}

impl ModifiedTense {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ModifiedTense::One(_tense) => Ok(()),
            ModifiedTense::Two(_tense_negation, _tense) => Ok(()),
        }
    }
}

impl AspectExpression {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            AspectExpression::One(modified_aspect) => modified_aspect.type_check(),
            AspectExpression::Epsilon => Ok(()),
        }
    }
}

impl ModifiedAspect {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ModifiedAspect::One(_aspect) => Ok(()),
            ModifiedAspect::Two(_aspect_negation, _aspect) => Ok(()),
        }
    }
}

impl MtaEvent {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            MtaEvent::One(modality_expression, ta_event) => {
                modality_expression.type_check()?;
                ta_event.type_check()
            }
            MtaEvent::Two(_proposition_operator, mta_event1, mta_event2) => {
                mta_event1.type_check()?;
                mta_event2.type_check()
            }
        }
    }
}

impl TaEvent {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            TaEvent::One(tense_expression, a_event) => {
                tense_expression.type_check()?;
                a_event.type_check()
            }
        }
    }
}

impl AEvent {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            AEvent::One(aspect_expression, event) => {
                aspect_expression.type_check()?;
                event.type_check()
            }
        }
    }
}

fn check_case_arg_type(
    arg: &ArgumentPhrase,
    expected: Vec<ArgumentType>,
    message: &str,
) -> TypeMismatchResult {
    let arg_type = arg.get_type();
    if has_intersection(&expected, &arg_type) {
        Ok(())
    } else {
        Err(TypeMismatch {
            range: arg.get_range().clone(),
            actual: arg_type,
            expected,
            message: message.to_string(),
        })
    }
}

/// 付加詞格の項の型と、型不一致時のメッセージ。`sa`/`si`は述語ごとに型が決まるため`None`。
fn adjunct_case_type(kind: &CaseLabelKind) -> Option<(Vec<ArgumentType>, &'static str)> {
    match kind {
        CaseLabelKind::Sa | CaseLabelKind::Si => None,
        CaseLabelKind::Subu | CaseLabelKind::Sube | CaseLabelKind::Subo => Some((
            vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)],
            "Event type mismatch in a biological (生物系) case argument.",
        )),
        CaseLabelKind::Sute | CaseLabelKind::Suto => Some((
            vec![ArgumentType::Abstraction],
            "Event type mismatch in an abstract (抽象系) case argument.",
        )),
        CaseLabelKind::Sudu => Some((
            vec![
                ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
            ],
            "Event type mismatch in an instrumental (道具系) case argument.",
        )),
        CaseLabelKind::Suni | CaseLabelKind::Sune | CaseLabelKind::Suno => Some((
            vec![ArgumentType::Space],
            "Event type mismatch in a spatial (空間系) case argument.",
        )),
        CaseLabelKind::Sugi | CaseLabelKind::Suge | CaseLabelKind::Sugo => Some((
            vec![ArgumentType::Time],
            "Event type mismatch in a temporal (時間系) case argument.",
        )),
    }
}

impl Event {
    fn type_check(&self) -> TypeMismatchResult {
        let pred = self.get_pred();
        pred.type_check()?;
        let (t1, t2) = pred.get_argument_types();
        // 単項述語のx2スロット(項自体が存在してはいけない)を示す。seが付いていても
        // このアリティ検査だけは引き続き適用し、型不一致のみをバイパスする。
        let x2_argument_slot_absent = t2 == vec![ArgumentType::Epsilon];
        let mut seen_case_kinds: Vec<&CaseLabelKind> = Vec::new();
        for (label, is_type_overridden, arg) in self.case_args_with_override() {
            arg.type_check()?;
            let kind = label.get_kind();
            if seen_case_kinds.contains(&kind) {
                return Err(TypeMismatch {
                    range: arg.get_range().clone(),
                    actual: vec![],
                    expected: vec![],
                    message: format!(
                        "Case label '{}' is applied more than once.",
                        label.get_string()
                    ),
                });
            }
            seen_case_kinds.push(kind);
            let bypass_type_check = is_type_overridden
                && !(matches!(kind, CaseLabelKind::Si) && x2_argument_slot_absent);
            if bypass_type_check {
                continue;
            }
            match kind {
                CaseLabelKind::Sa => {
                    check_case_arg_type(
                        arg,
                        t1.clone(),
                        "Event type mismatch in the first argument.",
                    )?;
                }
                CaseLabelKind::Si => {
                    check_case_arg_type(
                        arg,
                        t2.clone(),
                        "Event type mismatch in the second argument.",
                    )?;
                }
                _ => {
                    if let Some((expected, message)) = adjunct_case_type(kind) {
                        check_case_arg_type(arg, expected, message)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl PostModifiedBinaryPredicatePhrase {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                post_binary_predicate_phrase_expression,
            ) => {
                binary_predicate_phrase.type_check()?;
                post_binary_predicate_phrase_expression.type_check()?;
                if let PostBinaryPredicatePhraseExpression::One(_, modifier) =
                    post_binary_predicate_phrase_expression
                {
                    check_shared_argument_types(
                        binary_predicate_phrase.get_argument_types(),
                        modifier.get_argument_types(),
                        modifier.get_range().clone(),
                        "of the predicate modifier (di)",
                    )?;
                }
                Ok(())
            }
        }
    }
    fn get_argument_types(&self) -> (Vec<ArgumentType>, Vec<ArgumentType>) {
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                _post_binary_predicate_phrase_expression,
            ) => binary_predicate_phrase.get_argument_types(),
        }
    }
}

impl PostBinaryPredicatePhraseExpression {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            PostBinaryPredicatePhraseExpression::One(
                _post_binary_predicate_modifier,
                binary_predicate_phrase,
            ) => binary_predicate_phrase.type_check(),
            PostBinaryPredicatePhraseExpression::Epsilon => Ok(()),
        }
    }
}

/// `P hoba Q`や`P di Q`のように2つの述語が同じ項を共有するとき、x1同士・x2同士の型が重なるか検査する。
fn check_shared_argument_types(
    (t11, t12): (Vec<ArgumentType>, Vec<ArgumentType>),
    (t21, t22): (Vec<ArgumentType>, Vec<ArgumentType>),
    range: Range,
    context: &str,
) -> TypeMismatchResult {
    if !has_intersection(&t11, &t21) {
        return Err(TypeMismatch {
            range,
            actual: t21,
            expected: t11,
            message: format!("Type mismatch in the first argument {context}."),
        });
    }
    if !has_intersection(&t12, &t22) {
        return Err(TypeMismatch {
            range,
            actual: t22,
            expected: t12,
            message: format!("Type mismatch in the second argument {context}."),
        });
    }
    Ok(())
}

impl BinaryPredicatePhrase {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            BinaryPredicatePhrase::One(_binary_predicate) => Ok(()),
            BinaryPredicatePhrase::Two(_negation, _binary_predicate) => Ok(()),
            BinaryPredicatePhrase::Three(
                _binary_operator,
                binary_predicate1,
                binary_predicate2,
            ) => check_shared_argument_types(
                binary_predicate1.get_argument_types(),
                binary_predicate2.get_argument_types(),
                binary_predicate2.get_range().clone(),
                "after binary operation",
            ),
        }
    }
    fn get_argument_types(&self) -> (Vec<ArgumentType>, Vec<ArgumentType>) {
        match self {
            BinaryPredicatePhrase::One(binary_predicate) => binary_predicate.get_argument_types(),
            BinaryPredicatePhrase::Two(_, binary_predicate_phrase) => {
                binary_predicate_phrase.get_argument_types()
            }
            BinaryPredicatePhrase::Three(_, binary_predicate1, _) => {
                binary_predicate1.get_argument_types()
            }
        }
    }
}

impl ArgumentPhrase {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ArgumentPhrase::One(_argument) => Ok(()),
            ArgumentPhrase::Two(_negation, argument_phrase) => argument_phrase.type_check(),
            ArgumentPhrase::Three(_binary_operator, argument_phrase1, argument_phrase2) => {
                argument_phrase1.type_check()?;
                argument_phrase2.type_check()?;
                let t1 = argument_phrase1.get_type();
                let t2 = argument_phrase2.get_type();
                if !has_intersection(&t1, &t2) {
                    return Err(TypeMismatch {
                        range: argument_phrase2.get_range().clone(),
                        actual: t2,
                        expected: t1,
                        message:
                            "Type mismatch between the two argument phrases after binary operation."
                                .to_string(),
                    });
                }
                Ok(())
            }
            ArgumentPhrase::Four(quantified_argument) => quantified_argument.type_check(),
            ArgumentPhrase::Five(_opener, _body, _closer) => Ok(()),
            ArgumentPhrase::Six(_proper_noun) => Ok(()),
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            ArgumentPhrase::One(argument) => argument.get_type(),
            ArgumentPhrase::Two(_, argument) => argument.get_type(),
            // 等位接続した項は、両方の項が満たせる型(共通部分)を持つ。
            ArgumentPhrase::Three(_, argument1, argument2) => {
                intersection(&argument1.get_type(), &argument2.get_type())
            }
            ArgumentPhrase::Four(quantified_argument) => quantified_argument.get_type(),
            ArgumentPhrase::Five(opener, _, _) => vec![opener.get_type()],
            ArgumentPhrase::Six(proper_noun) => vec![proper_noun.get_type()],
        }
    }
}

impl QuantifiedArgument {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            QuantifiedArgument::One(_quantified_modifier, determined_argument) => {
                determined_argument.type_check()
            }
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            QuantifiedArgument::One(_quantified_modifier, determined_argument) => {
                determined_argument.get_type()
            }
        }
    }
}

impl DeterminedArgument {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            DeterminedArgument::One(_determiner_modifier, retrieved_argument) => {
                retrieved_argument.type_check()
            }
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            DeterminedArgument::One(_determiner_modifier, retrieved_argument) => {
                retrieved_argument.get_type()
            }
        }
    }
}

impl RetrievedArgument {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                argument_retrieval_binary_predicate
                    .type_check(argument_retriever.gap_case().as_ref())
            }
            RetrievedArgument::Two(_ru, case_label, argument_retrieval_binary_predicate) => {
                if adjunct_case_type(case_label.get_kind()).is_none() {
                    return Err(TypeMismatch {
                        range: case_label.get_range(),
                        actual: vec![],
                        expected: vec![],
                        message: format!(
                            "'ru {}' is not allowed; use 'ra' or 'ri' to retrieve the first or second argument.",
                            case_label.get_string()
                        ),
                    });
                }
                argument_retrieval_binary_predicate.type_check(Some(case_label.get_kind()))
            }
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                match argument_retriever.gap_case() {
                    Some(gap) => argument_retrieval_binary_predicate.get_type(&gap),
                    None => vec![ArgumentType::Abstraction],
                }
            }
            RetrievedArgument::Two(_ru, case_label, _argument_retrieval_binary_predicate) => {
                adjunct_case_type(case_label.get_kind())
                    .map(|(t, _)| t)
                    .unwrap_or_default()
            }
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    /// `gap`は抽出される格。節の形(`he … ho`)では、節内のEventがこの格を明示してはいけない。
    /// `re`は事象そのものを抽出するので`gap`は`None`になる。
    fn type_check(&self, gap: Option<&CaseLabelKind>) -> TypeMismatchResult {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                post_modified_binary_predicate_phrase.type_check()?;
                argument_retrieval_with_argument_phrase.type_check()
            }
            ArgumentRetrievalBinaryPredicate::Two(_he, mta_event, _ho) => {
                mta_event.type_check()?;
                let Some(gap) = gap else {
                    return Ok(());
                };
                for event in mta_event.events() {
                    if let Some((label, _)) = event
                        .case_args()
                        .into_iter()
                        .find(|(label, _)| label.get_kind() == gap)
                    {
                        return Err(TypeMismatch {
                            range: label.get_range(),
                            actual: vec![],
                            expected: vec![],
                            message: format!(
                                "Case label '{}' is the retrieved gap and must be omitted in the clause.",
                                label.get_string()
                            ),
                        });
                    }
                    let (_, t2) = event.get_pred().get_argument_types();
                    if matches!(gap, CaseLabelKind::Si) && t2 == vec![ArgumentType::Epsilon] {
                        return Err(TypeMismatch {
                            range: event.get_range(),
                            actual: vec![],
                            expected: vec![],
                            message: "Cannot retrieve the second argument of a unary predicate."
                                .to_string(),
                        });
                    }
                }
                Ok(())
            }
        }
    }
    /// `ra`/`ri`で抽出した項の型。`gap`は`sa`か`si`。
    fn get_type(&self, gap: &CaseLabelKind) -> Vec<ArgumentType> {
        let slot_type = |pred: &PostModifiedBinaryPredicatePhrase| {
            let (t1, t2) = pred.get_argument_types();
            match gap {
                CaseLabelKind::Sa => t1,
                _ => t2,
            }
        };
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                _argument_retrieval_with_argument_phrase,
            ) => slot_type(post_modified_binary_predicate_phrase),
            // 等位接続された全Eventに同じ項が入るため、各述語の型の共通部分をとる。
            ArgumentRetrievalBinaryPredicate::Two(_he, mta_event, _ho) => mta_event
                .events()
                .into_iter()
                .map(|event| slot_type(event.get_pred()))
                .reduce(|acc, t| acc.into_iter().filter(|x| t.contains(x)).collect())
                .unwrap_or_default(),
        }
    }
}

impl ArgumentRetrievalWithArgumentPhrase {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ArgumentRetrievalWithArgumentPhrase::One(
                _post_argument_phrase_modifier,
                argument_phrase,
            ) => argument_phrase.type_check(),
            ArgumentRetrievalWithArgumentPhrase::Epsilon => Ok(()),
        }
    }
}

fn has_intersection<T: Eq + Hash>(vec1: &[T], vec2: &[T]) -> bool {
    let set: HashSet<_> = vec1.iter().collect();
    vec2.iter().any(|x| set.contains(x))
}

/// `vec1`のうち`vec2`にも含まれる要素を、`vec1`の順序のまま返す。
fn intersection<T: Eq + Hash + Clone>(vec1: &[T], vec2: &[T]) -> Vec<T> {
    let set: HashSet<_> = vec2.iter().collect();
    vec1.iter().filter(|x| set.contains(x)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::tokenizer;

    #[test]
    fn test_型検査が成功する() {
        let source = "parobe sa ma si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_第一項の型が間違っている場合にerr型を返す() {
        let source = "parobe sa ri didora si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_第二項の型が間違っている場合にerr型を返す() {
        let source = "kahoraka sa mi si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_unary_predicateの型検査が成功する() {
        let source = "pisarepa sa mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_unary_predicateの第一項の型が間違っている場合にerr型を返す() {
        let source = "pusatanada sa ra bahumana eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_unary_predicateで第二項の型が入力された場合にerr型を返す() {
        let source = "pisarepa sa mi si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_項抽出の型検査が成功する() {
        let source = "parobe sa ra bahumana si ri bidoga eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    fn check(source: &str) -> TypeMismatchResult {
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        type_check(&ast)
    }

    #[test]
    fn test_ruで抽出した付加詞の項は格の型を持つ() {
        assert!(check("paheta sa mi si ri behapare suni ru suni paheta eos").is_ok());
        assert!(check("paheta sa mi si ri behapare sudu ru sudu paheta du mi eos").is_ok());
    }

    #[test]
    fn test_ruで抽出した項の型が合わない場合にerr型を返す() {
        assert!(check("parobe sa mi si ru suni paheta eos").is_err());
    }

    #[test]
    fn test_節からの抽出の型検査が成功する() {
        assert!(check("paheta sa mi si ri he paheta sa ba ho eos").is_ok());
        assert!(check("paheta sa mi si ri behapare suni ru suni he paheta sa ba ho eos").is_ok());
        assert!(check("parobe sa mi si ri he homa paheta sa mi parobe sa ba ho eos").is_ok());
    }

    #[test]
    fn test_節からの抽出で空所の格が埋まっている場合にerr型を返す() {
        assert!(check("parobe sa mi si ri he paheta sa mi si ri behapare ho eos").is_err());
        assert!(check("paheta sa mi suni ru suni he paheta suni me ho eos").is_err());
    }

    #[test]
    fn test_節からの抽出で単項述語の第二項を抽出するとerr型を返す() {
        assert!(check("parobe sa mi si ri he pisarepa sa ba ho eos").is_err());
    }

    #[test]
    fn test_節からの抽出で抽出した項の型が合わない場合にerr型を返す() {
        assert!(check("parobe sa mi si ru suni he paheta sa ba ho eos").is_err());
    }

    #[test]
    fn test_ru_sa_ru_siはerr型を返す() {
        assert!(check("parobe sa mi si ru sa paheta eos").is_err());
        assert!(check("paheta sa mi si ru si paheta eos").is_err());
    }

    #[test]
    fn test_項転置したあと項抽出の型検査が成功する() {
        let source = "parobe sa ra patirada si ri patirada eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_生物系の格の型検査が成功する() {
        let source = "pagibe sa mi si ri behapare subu gi sube mi subo ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_随伴格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba subu ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_奪格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba sube ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_与格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba subo ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_抽象系の格の型検査が成功する() {
        let source = "parobe sa mi si ba sute ri kahoraka suto ri kahoraka eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_原因格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba sute ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_目的格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba suto ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_diで修飾する述語と項の型が重なれば型検査が成功する() {
        for source in [
            "parobe sa mi si ri bidoga di bihemare eos",
            "parobe di paherapa sa mi si ba eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(type_check(&ast).is_ok(), "{source}");
        }
    }

    #[test]
    fn test_diで修飾する述語と項の型が合わない場合にerr型を返す() {
        for source in [
            "parobe sa mi si ri bidoga di rebaraka eos",
            "parobe di rebaraka sa mi si ba eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(type_check(&ast).is_err(), "{source}");
        }
    }

    #[test]
    fn test_等位接続した項の型が重なれば型検査が成功する() {
        for source in [
            "paheta sa hoba mi me si ri behapare eos",
            "ku paheta sa hoba mi ko si ri behapare eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(type_check(&ast).is_ok(), "{source}");
        }
    }

    #[test]
    fn test_等位接続した項の型は両方の型の共通部分になる() {
        // hoba me miの型はmiの生物型だけになるので、抽象概念を取るkarobeのx2には置けない。
        let source = "karobe sa mi si hoba me mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        assert!(type_check(&ast).is_err());
    }

    #[test]
    fn test_等位接続や否定の中の項も型検査される() {
        for source in [
            "parobe sa mi si hoba mi ri rumone eos",
            "parobe sa mi si hoba ri bidoga di rebaraka ba eos",
            "parobe sa mi si ne ri bidoga di rebaraka eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(type_check(&ast).is_err(), "{source}");
        }
    }

    #[test]
    fn test_道具系の格の型検査が成功する() {
        let source = "parobe sa mi si ba sudu ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_具格に人工物以外の物理的実体を置いた場合に型検査が成功する() {
        let source = "parobe sa mi si ba sudu ri nihatera eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_具格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba sudu mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_空間系の格の型検査が成功する() {
        let source = "parobe sa mi si ba suni me sune be suno ge eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_空間格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba suni mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_同じ付加詞の格が複数回付与された場合にerr型を返す() {
        let source = "parobe sa mi si ba suni me suni ge eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_同じsaが複数回付与された場合にerr型を返す() {
        let source = "parobe sa mi sa ba si ge eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_同じsiが複数回付与された場合にerr型を返す() {
        let source = "parobe sa mi si ba si ge eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_時間系の格の型検査が成功する() {
        let source = "parobe sa mi si ba sugi me suge be sugo ge eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_時間格の型が間違っている場合にerr型を返す() {
        let source = "parobe sa mi si ba sugi mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_固有名詞_hebaが生物系の格を直接満たす() {
        let source = "parobe sa heba Kanye hoho si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_固有名詞_hebaが人工物専用の格では失敗する() {
        let source = "parobe sa mi si ba sudu heba Kanye hoho eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_固有名詞_heda_hedi_heduが道具系の格を直接満たす() {
        for opener in ["heda", "hedi", "hedu"] {
            let source = format!("parobe sa mi si ba sudu {opener} Ford hoho eos");
            let tokens = tokenizer::tokenize(&source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            let type_checked = type_check(&ast);
            assert!(type_checked.is_ok(), "opener {opener} should satisfy sudu");
        }
    }

    #[test]
    fn test_固有名詞_hezaがza系述語を経由せず空間格を直接満たす() {
        let source = "parobe sa mi si ba suni heza Tokyo hoho eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_固有名詞_heziがzi系述語を経由せず時間格を直接満たす() {
        let source = "parobe sa mi si ba sugi hezi Reiwa hoho eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_固有名詞_hezaは生物系の格では失敗する() {
        let source = "parobe sa heza Tokyo hoho si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_tで始まる固有名詞_母音ごとに対応する格を満たす() {
        for source in [
            "parobe sa taro si ba eos",
            "parobe sa mi si ba sudu tetitanika eos",
            "parobe sa mi si ba suni tikanada eos",
            "parobe sa mi si ba sute tonasa eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(type_check(&ast).is_ok(), "{source} should type-check");
        }
    }

    #[test]
    fn test_tで始まる固有名詞_型の合わない格では失敗する() {
        for source in [
            "parobe sa tetitanika si ba eos",
            "parobe sa mi si ba sudu taro eos",
            "parobe sa mi si ba suni tonasa eos",
            "parobe sa mi si ba sugi tikanada eos",
        ] {
            let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
            let ast = parser::parse(tokens).unwrap();
            assert!(
                type_check(&ast).is_err(),
                "{source} should fail to type-check"
            );
        }
    }

    #[test]
    fn test_se付きで第一項の型不一致をバイパスできる() {
        let source = "parobe sa se ri didora si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_se付きで第二項の型不一致をバイパスできる() {
        let source = "kahoraka sa mi si se ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_se付きで両方の項の型不一致をバイパスできる() {
        let source = "parobe sa se ri didora si se ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_se付きで付加詞格の型不一致をバイパスできる() {
        let source = "parobe sa mi si ba subu se ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_se付きでもseがない別の格の型不一致は検出される() {
        let source = "parobe sa se ri didora si ba subu ri didora eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_se付きでも単項述語の第二項のアリティ検査はバイパスされない() {
        let source = "pisarepa sa mi si se ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
    }
}
