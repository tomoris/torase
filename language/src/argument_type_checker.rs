//! 格ラベルごとの選択制限(項の意味的な型)を検証する型チェッカー。
//! `formal_semantics::type_checker` のモンタギュー流の型システムとは別の、格構造に基づく型システムを扱う。

use std::collections::HashSet;
use std::hash::Hash;

use crate::parser::*;
use crate::tokenizer::{
    ArgumentRetrieverKind, ArgumentType, CaseLabelKind, PhysicalEntity, Quantifier, Range,
    RoleLabelKind,
};

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
    let kinds = [
        role_arg1.get_label().get_kind(),
        role_arg2.get_label().get_kind(),
    ];
    let has_hesa = kinds.iter().any(|k| matches!(k, RoleLabelKind::Hesa));
    let has_hesi = kinds.iter().any(|k| matches!(k, RoleLabelKind::Hesi));
    if has_hesa && has_hesi {
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

impl Event {
    fn type_check(&self) -> TypeMismatchResult {
        let pred = self.get_pred();
        pred.type_check()?;
        let (t1, t2) = pred.get_argument_types();
        // 単項述語のx2スロット(項自体が存在してはいけない)を示す。seが付いていても
        // このアリティ検査だけは引き続き適用し、型不一致のみをバイパスする。
        let x2_argument_slot_absent = t2 == vec![ArgumentType::Epsilon];
        let mut seen_adjunct_kinds: Vec<&CaseLabelKind> = Vec::new();
        for (label, is_type_overridden, arg) in self.case_args_with_override() {
            arg.type_check()?;
            let kind = label.get_kind();
            if !matches!(kind, CaseLabelKind::Sa | CaseLabelKind::Si) {
                if seen_adjunct_kinds.contains(&kind) {
                    return Err(TypeMismatch {
                        range: arg.get_range().clone(),
                        actual: vec![],
                        expected: vec![],
                        message: format!(
                            "Adjunct case label '{}' is applied more than once.",
                            label.get_string()
                        ),
                    });
                }
                seen_adjunct_kinds.push(kind);
            }
            let bypass_type_check = is_type_overridden
                && !(matches!(kind, CaseLabelKind::Si) && x2_argument_slot_absent);
            if bypass_type_check {
                continue;
            }
            match kind {
                CaseLabelKind::Sa => {
                    let arg_type = arg.get_type();
                    if !has_intersection(&t1, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected: t1.clone(),
                            message: "Event type mismatch in the first argument.".to_string(),
                        });
                    }
                }
                CaseLabelKind::Si => {
                    let arg_type = arg.get_type();
                    if !has_intersection(&t2, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected: t2.clone(),
                            message: "Event type mismatch in the second argument.".to_string(),
                        });
                    }
                }
                CaseLabelKind::Subu | CaseLabelKind::Sube | CaseLabelKind::Subo => {
                    let expected = vec![ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing)];
                    let arg_type = arg.get_type();
                    if !has_intersection(&expected, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected,
                            message: "Event type mismatch in a biological (生物系) case argument."
                                .to_string(),
                        });
                    }
                }
                CaseLabelKind::Sute | CaseLabelKind::Suto => {
                    let expected = vec![ArgumentType::Abstraction];
                    let arg_type = arg.get_type();
                    if !has_intersection(&expected, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected,
                            message: "Event type mismatch in an abstract (抽象系) case argument."
                                .to_string(),
                        });
                    }
                }
                CaseLabelKind::Sudu => {
                    let expected = vec![ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact)];
                    let arg_type = arg.get_type();
                    if !has_intersection(&expected, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected,
                            message:
                                "Event type mismatch in an instrumental (道具系) case argument."
                                    .to_string(),
                        });
                    }
                }
                CaseLabelKind::Suni | CaseLabelKind::Sune | CaseLabelKind::Suno => {
                    let expected = vec![ArgumentType::Space];
                    let arg_type = arg.get_type();
                    if !has_intersection(&expected, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected,
                            message: "Event type mismatch in a spatial (空間系) case argument."
                                .to_string(),
                        });
                    }
                }
                CaseLabelKind::Sugi | CaseLabelKind::Suge | CaseLabelKind::Sugo => {
                    let expected = vec![ArgumentType::Time];
                    let arg_type = arg.get_type();
                    if !has_intersection(&expected, &arg_type) {
                        return Err(TypeMismatch {
                            range: arg.get_range().clone(),
                            actual: arg_type,
                            expected,
                            message: "Event type mismatch in a temporal (時間系) case argument."
                                .to_string(),
                        });
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
                post_binary_predicate_phrase_expression.type_check()
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

impl BinaryPredicatePhrase {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            BinaryPredicatePhrase::One(_binary_predicate) => Ok(()),
            BinaryPredicatePhrase::Two(_negation, _binary_predicate) => Ok(()),
            BinaryPredicatePhrase::Three(
                _binary_operator,
                binary_predicate1,
                binary_predicate2,
            ) => {
                let (t11, t12) = binary_predicate1.get_argument_types();
                let (t21, t22) = binary_predicate2.get_argument_types();
                if !has_intersection(&t11, &t21) {
                    return Err(TypeMismatch {
                        range: binary_predicate2.get_range().clone(),
                        actual: t21,
                        expected: t11,
                        message: "Type mismatch in the first argument after binary operation."
                            .to_string(),
                    });
                } else if !has_intersection(&t12, &t22) {
                    return Err(TypeMismatch {
                        range: binary_predicate2.get_range().clone(),
                        actual: t22,
                        expected: t12,
                        message: "Type mismatch in the second argument after binary operation."
                            .to_string(),
                    });
                }
                Ok(())
            }
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
            ArgumentPhrase::Two(_negation, _argument) => Ok(()),
            ArgumentPhrase::Three(_binary_operator, argument_phrase1, argument_phrase2) => {
                let t1 = argument_phrase1.get_type();
                let t2 = argument_phrase2.get_type();
                if t1 != t2 {
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
            ArgumentPhrase::Five(
                _mta_event_to_argument_phrase_converter,
                mta_event,
                _mta_event_to_argument_phrase_converter_end,
            ) => mta_event.type_check(),
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            ArgumentPhrase::One(argument) => argument.get_type(),
            ArgumentPhrase::Two(_, argument) => argument.get_type(),
            ArgumentPhrase::Three(_, argument1, _) => argument1.get_type(),
            ArgumentPhrase::Four(quantified_argument) => quantified_argument.get_type(),
            ArgumentPhrase::Five(_, _, _) => {
                vec![ArgumentType::Abstraction]
            }
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
            RetrievedArgument::One(_argument_retriever, argument_retrieval_binary_predicate) => {
                argument_retrieval_binary_predicate.type_check()
            }
        }
    }
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                argument_retrieval_binary_predicate.get_type(argument_retriever.get_kind())
            }
        }
    }
}

impl ArgumentRetrievalBinaryPredicate {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                post_modified_binary_predicate_phrase.type_check()?;
                argument_retrieval_with_argument_phrase.type_check()
            }
        }
    }
    fn get_type(&self, kind: &ArgumentRetrieverKind) -> Vec<ArgumentType> {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                _argument_retrieval_with_argument_phrase,
            ) => {
                let (t1, t2) = post_modified_binary_predicate_phrase.get_argument_types();
                match kind {
                    ArgumentRetrieverKind::Nominative => t1,
                    ArgumentRetrieverKind::Accusative => t2,
                }
            }
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
        let source = "pusatanada sa ra bahomo eos";
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
        let source = "parobe sa ra bahomo si ri bidoga eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_ok());
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
    #[ignore]
    fn test_unary_predicateを項転置しないとerr型を返す() {
        let source = "parobe sa mi si ro re pisarepa eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(&ast);
        assert!(type_checked.is_err());
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
    fn test_道具系の格の型検査が成功する() {
        let source = "parobe sa mi si ba sudu ri didora eos";
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
