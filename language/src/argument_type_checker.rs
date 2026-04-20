use std::collections::HashSet;
use std::hash::Hash;

use crate::parser::*;
use crate::tokenizer::{ArgumentType, PhysicalEntity, Range};

pub struct TypeMismatch {
    pub range: Range,
    pub actual: Vec<ArgumentType>,
    pub expected: Vec<ArgumentType>,
    pub message: String,
}

type TypeMismatchResult = Result<(), TypeMismatch>;

pub fn type_check(ast: Ast) -> TypeMismatchResult {
    ast.get_s().type_check()
}

impl S {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            S::One(declarative_sentence) => declarative_sentence.type_check(),
            S::Two(interrogative_sentence) => interrogative_sentence.type_check(),
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

impl PostModifiedMtaEvent {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            PostModifiedMtaEvent::One(mta_event, post_mta_event_expression) => {
                mta_event.type_check()?;
                post_mta_event_expression.type_check()
            }
        }
    }
}

impl PostMtaEventExpression {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            PostMtaEventExpression::One(_post_mta_event_modifier, mta_event) => {
                mta_event.type_check()
            }
            PostMtaEventExpression::Epsilon => Ok(()),
        }
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
            ModifiedModality::One(_modality) => Ok(()),
            ModifiedModality::Two(_modality_negation, _modality) => Ok(()),
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
        match self {
            Event::One(
                post_modified_binary_predicate_phrase,
                _nominative_preposition,
                argument_phrase,
                argument_phrase_or_epsilon,
            ) => {
                post_modified_binary_predicate_phrase.type_check()?;
                argument_phrase.type_check()?;
                argument_phrase_or_epsilon.type_check()?;
                let (t1, t2) = post_modified_binary_predicate_phrase.get_argument_types();
                let arg1_type = argument_phrase.get_type();
                if !has_intersection(&t1, &arg1_type) {
                    return Err(TypeMismatch {
                        range: argument_phrase.get_range().clone(),
                        actual: arg1_type,
                        expected: t1,
                        message: "Event type mismatch in the first argument.".to_string(),
                    });
                }
                match argument_phrase_or_epsilon {
                    ArgumentPhraseOrEpsilon::One(_, argument_phrase2) => {
                        let arg2_type = argument_phrase2.get_type();
                        if !has_intersection(&t2, &arg2_type) {
                            return Err(TypeMismatch {
                                range: argument_phrase2.get_range().clone(),
                                actual: arg2_type,
                                expected: t2,
                                message: "Event type mismatch in the second argument.".to_string(),
                            });
                        }
                    }
                    ArgumentPhraseOrEpsilon::Epsilon => return Ok(()),
                }
                Ok(())
            }
        }
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
            BinaryPredicatePhrase::Three(_argument_phrase_transposer, binary_predicate) => {
                let (_, t2) = binary_predicate.get_argument_types();
                if t2 == vec![ArgumentType::Epsilon] {
                    return Err(TypeMismatch {
                        range: binary_predicate.get_range().clone(),
                        actual: t2,
                        expected: vec![
                            ArgumentType::PhysicalEntity(PhysicalEntity::BLivingThing),
                            ArgumentType::PhysicalEntity(PhysicalEntity::DArtifact),
                            ArgumentType::PhysicalEntity(PhysicalEntity::NOther),
                            ArgumentType::Abstraction,
                        ],
                        message: "Cannot transpose arguments of a unary predicate.".to_string(),
                    });
                }
                Ok(())
            }
            BinaryPredicatePhrase::Four(_binary_operator, binary_predicate1, binary_predicate2) => {
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
            BinaryPredicatePhrase::Three(_, binary_predicate) => {
                let (t1, t2) = binary_predicate.get_argument_types();
                (t2, t1)
            }
            BinaryPredicatePhrase::Four(_, binary_predicate1, _) => {
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

impl ArgumentPhraseOrEpsilon {
    fn type_check(&self) -> TypeMismatchResult {
        match self {
            ArgumentPhraseOrEpsilon::One(_argument_phrase_separator, argument_phrase) => {
                argument_phrase.type_check()
            }
            ArgumentPhraseOrEpsilon::Epsilon => Ok(()),
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
            RetrievedArgument::One(_argument_retriever, argument_retrieval_binary_predicate) => {
                argument_retrieval_binary_predicate.get_type()
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
    fn get_type(&self) -> Vec<ArgumentType> {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                _argument_retrieval_with_argument_phrase,
            ) => {
                let (_, t2) = post_modified_binary_predicate_phrase.get_argument_types();
                t2
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
        let type_checked = type_check(ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_第一項の型が間違っている場合にerr型を返す() {
        let source = "parobe sa ro didora si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_第二項の型が間違っている場合にerr型を返す() {
        let source = "kahoraka sa mi si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_unary_predicateの型検査が成功する() {
        let source = "pisarepa sa mi eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_unary_predicateの第一項の型が間違っている場合にerr型を返す() {
        let source = "pusatanada sa ro bahomo eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_unary_predicateで第二項の型が入力された場合にerr型を返す() {
        let source = "pisarepa sa mi si ba eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_err());
    }

    #[test]
    fn test_項抽出の型検査が成功する() {
        let source = "parobe sa ro bahomo si ro bidoga eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_項転置したあと項抽出の型検査が成功する() {
        let source = "parobe sa ro re patirada si ro patirada eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_ok());
    }

    #[test]
    fn test_unary_predicateを項転置しないとerr型を返す() {
        let source = "parobe sa mi si ro re pisarepa eos";
        let tokens = tokenizer::tokenize(source).unwrap()[0].clone();
        let ast = parser::parse(tokens).unwrap();
        let type_checked = type_check(ast);
        assert!(type_checked.is_err());
    }
}
