//! 構文木ビューア。文法ノードごとに`TreeViewer`を実装し、色分けされた入れ子の枠として描画する。

use iced::Element;
use iced::widget::{scrollable, text};

use language::parser::*;

use crate::app::Message;
use crate::layout::{category_box, row_with_separator};

pub(crate) fn asts_view(asts: &[Ast], is_full: bool) -> Element<'static, Message> {
    if asts.is_empty() {
        return text("").into();
    }
    let mut col = iced::widget::column![];
    for (i, ast) in asts.iter().enumerate() {
        col = col.push(category_box(
            format!("Sentence {}", i + 1),
            ast.view(is_full),
            iced::Color::from_rgb8(255, 255, 255),
        ));
    }
    scrollable(col)
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::new(),
            horizontal: scrollable::Scrollbar::new(),
        })
        .into()
}

// 各ノードの背景色は、文法上の意味的なまとまり(ファミリー)ごとに1色相を割り当て、
// ネストが深くなるほどその色相の中で単調に濃くする(白との混合比を下げる)ことで
// 系統立てている。ファミリーと色相の対応:
//   S/DeclarativeSentence/InterrogativeSentence : ニュートラル(文全体の枠)
//   ModalityExpression/ModifiedModality         : blue
//   TenseExpression/ModifiedTense                : yellow
//   AspectExpression/ModifiedAspect              : green
//   PostModifiedMtaEvent〜Event (MTAイベント連鎖) : orange
//   PostModifiedBinaryPredicatePhrase〜BinaryPredicatePhrase (述語句) : aqua
//   ArgumentPhrase/OptionalCaseArgs              : magenta
//   QuantifiedArgument〜ArgumentRetrievalWithArgumentPhrase (項検索連鎖) : violet
//   QuantifierModifier/NumericalExpression/DeterminerModifier/TypeOverrideModifier (修飾子葉) : red
pub(crate) trait TreeViewer {
    fn view(&self, is_full: bool) -> Element<'static, Message>;
    fn is_empty(&self) -> bool;
}

impl TreeViewer for Ast {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let s = self.get_s();
        iced::widget::container(s.view(is_full)).into()
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for S {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 250, 250);
        match self {
            S::One(declarative_sentence) => {
                let category = "s".to_string();
                let content = declarative_sentence.view(is_full);
                category_box(category, content, background)
            }
            S::Two(interrogative_sentence) => {
                let content = interrogative_sentence.view(is_full);
                category_box("s".to_string(), content, background)
            }
            S::Three(universal_sentence) => {
                let content = universal_sentence.view(is_full);
                category_box("s".to_string(), content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for DeclarativeSentence {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 240, 230);
        match self {
            DeclarativeSentence::One(post_modified_mta_event) => {
                let category = "DeclarativeSentence".to_string();
                let content = post_modified_mta_event.view(is_full);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for InterrogativeSentence {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(255, 235, 205);
        match self {
            InterrogativeSentence::One(interrogative_marker, post_modified_mta_event) => {
                let category = "InterrogativeSentence".to_string();
                let content = row_with_separator(vec![
                    text(interrogative_marker.get_string()).into(),
                    post_modified_mta_event.view(is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for UniversalSentence {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(255, 250, 205);
        match self {
            UniversalSentence::One(
                universal_marker,
                quantified_variable_list,
                post_modified_mta_event,
            ) => {
                let category = "UniversalSentence".to_string();
                let mut parts = vec![text(universal_marker.get_string()).into()];
                if is_full || !quantified_variable_list.is_empty() {
                    parts.push(quantified_variable_list.view(is_full));
                }
                parts.push(post_modified_mta_event.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for QuantifiedVariableList {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(248, 215, 228);
        match self {
            QuantifiedVariableList::One(QuantifiedVariable::One(force, arg), rest) => {
                let category = "QuantifiedVariableList".to_string();
                let mut parts = vec![
                    text(force.get_string()).into(),
                    text(arg.get_string()).into(),
                ];
                if is_full || !rest.is_empty() {
                    parts.push(rest.view(is_full));
                }
                category_box(category, row_with_separator(parts), background)
            }
            QuantifiedVariableList::Epsilon => {
                let category = "QuantifiedVariableList".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, QuantifiedVariableList::Epsilon)
    }
}

impl TreeViewer for PostModifiedMtaEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(253, 240, 235);
        match self {
            PostModifiedMtaEvent::One(mta_event) => {
                let category = "PostModifiedMtaEvent".to_string();
                let parts = vec![mta_event.view(is_full)];
                category_box(category, row_with_separator(parts), background)
            }
            PostModifiedMtaEvent::Two(connective_head, role_arg1, role_arg2) => {
                let category = "PostModifiedMtaEvent".to_string();
                let content = row_with_separator(vec![
                    text(connective_head.get_string()).into(),
                    role_arg1.view(is_full),
                    role_arg2.view(is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for RoleArg {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(251, 228, 218);
        match self {
            RoleArg::One(role_label, mta_event) => {
                let category = "RoleArg".to_string();
                let content = row_with_separator(vec![
                    text(role_label.get_string()).into(),
                    mta_event.view(is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ModalityExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(225, 236, 249);
        match self {
            ModalityExpression::One(modality_with_agent) => {
                let category = "ModalityExpression".to_string();
                let content = modality_with_agent.view(is_full);
                category_box(category, content, background)
            }
            ModalityExpression::Epsilon => {
                let category = "ModalityExpression".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, ModalityExpression::Epsilon)
    }
}

impl TreeViewer for ModifiedModality {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(187, 212, 242);
        match self {
            ModifiedModality::One(modality, modality_force) => {
                let category = "ModifiedModality".to_string();
                let content = row_with_separator(vec![
                    text(modality.get_string()).into(),
                    text(modality_force.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
            ModifiedModality::Two(modality_negation, modality, modality_force) => {
                let category = "ModifiedModality".to_string();
                let content = row_with_separator(vec![
                    text(modality_negation.get_string()).into(),
                    text(modality.get_string()).into(),
                    text(modality_force.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for TenseExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(252, 242, 219);
        match self {
            TenseExpression::One(modified_tense) => {
                let category = "TenseExpression".to_string();
                let content = modified_tense.view(is_full);
                category_box(category, content, background)
            }
            TenseExpression::Epsilon => {
                let category = "TenseExpression".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, TenseExpression::Epsilon)
    }
}

impl TreeViewer for ModifiedTense {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(249, 225, 173);
        match self {
            ModifiedTense::One(tense) => {
                let category = "ModifiedTense".to_string();
                let content = text(tense.get_string()).into();
                category_box(category, content, background)
            }
            ModifiedTense::Two(tense_negation, tense) => {
                let category = "ModifiedTense".to_string();
                let content = row_with_separator(vec![
                    text(tense_negation.get_string()).into(),
                    text(tense.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for AspectExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(219, 238, 219);
        match self {
            AspectExpression::One(modified_aspect) => {
                let category = "AspectExpression".to_string();
                let content = modified_aspect.view(is_full);
                category_box(category, content, background)
            }
            AspectExpression::Epsilon => {
                let category = "AspectExpression".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, AspectExpression::Epsilon)
    }
}

impl TreeViewer for ModifiedAspect {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(173, 215, 173);
        match self {
            ModifiedAspect::One(aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = text(aspect.get_string()).into();
                category_box(category, content, background)
            }
            ModifiedAspect::Two(aspect_negation, aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = row_with_separator(vec![
                    text(aspect_negation.get_string()).into(),
                    text(aspect.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for MtaEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 216, 202);
        match self {
            MtaEvent::One(modality_expression, ta_event) => {
                let category = "MtaEvent".to_string();
                let mut parts = vec![];
                if is_full || !modality_expression.is_empty() {
                    parts.push(modality_expression.view(is_full));
                }
                parts.push(ta_event.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
            MtaEvent::Two(proposition_operator, mta_event1, mta_event2) => {
                let category = "MtaEvent".to_string();
                let content = row_with_separator(vec![
                    text(proposition_operator.get_string()).into(),
                    mta_event1.view(is_full),
                    mta_event2.view(is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for TaEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(248, 204, 186);
        match self {
            TaEvent::One(tense_expression, a_event) => {
                let category = "TaEvent".to_string();
                let mut parts = vec![];
                if is_full || !tense_expression.is_empty() {
                    parts.push(tense_expression.view(is_full));
                }
                parts.push(a_event.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for AEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(247, 192, 170);
        match self {
            AEvent::One(aspect_expression, event) => {
                let category = "AEvent".to_string();
                let mut parts = vec![];
                if is_full || !aspect_expression.is_empty() {
                    parts.push(aspect_expression.view(is_full));
                }
                parts.push(event.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for Event {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(245, 180, 154);
        match self {
            Event::One(post_modified_binary_predicate_phrase, optional_case_args) => {
                let category = "Event".to_string();
                let mut parts = vec![post_modified_binary_predicate_phrase.view(is_full)];
                if is_full || !optional_case_args.is_empty() {
                    parts.push(optional_case_args.view(is_full));
                }
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostModifiedBinaryPredicatePhrase {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(221, 243, 235);
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                post_binary_predicate_phrase_expression,
            ) => {
                let category = "PostModifiedBinaryPredicatePhrase".to_string();
                let mut parts = vec![binary_predicate_phrase.view(is_full)];
                if is_full || !post_binary_predicate_phrase_expression.is_empty() {
                    parts.push(post_binary_predicate_phrase_expression.view(is_full));
                }
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostBinaryPredicatePhraseExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(198, 235, 222);
        match self {
            PostBinaryPredicatePhraseExpression::One(
                post_binary_predicate_phrase_modifier,
                binary_predicate_phrase,
            ) => {
                let category = "PostBinaryPredicatePhraseExpression".to_string();
                let content = row_with_separator(vec![
                    text(post_binary_predicate_phrase_modifier.get_string()).into(),
                    binary_predicate_phrase.view(is_full),
                ]);
                category_box(category, content, background)
            }
            PostBinaryPredicatePhraseExpression::Epsilon => {
                let category = "PostBinaryPredicatePhraseExpression".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, PostBinaryPredicatePhraseExpression::Epsilon)
    }
}

impl TreeViewer for BinaryPredicatePhrase {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(175, 227, 208);
        match self {
            BinaryPredicatePhrase::One(binary_predicate) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = text(binary_predicate.get_string()).into();
                category_box(category, content, background)
            }
            BinaryPredicatePhrase::Two(negation, binary_predicate_phrase) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row_with_separator(vec![
                    text(negation.get_string()).into(),
                    binary_predicate_phrase.view(_is_full),
                ]);
                category_box(category, content, background)
            }
            BinaryPredicatePhrase::Three(
                binary_operator,
                binary_predicate_phrase1,
                binary_predicate_phrase2,
            ) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row_with_separator(vec![
                    text(binary_operator.get_string()).into(),
                    binary_predicate_phrase1.view(_is_full),
                    binary_predicate_phrase2.view(_is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentPhrase {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(251, 231, 239);
        match self {
            ArgumentPhrase::One(argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = text(argument.get_string()).into();
                category_box(category, content, background)
            }
            ArgumentPhrase::Two(negation, argument_phrase) => {
                let category = "ArgumentPhrase".to_string();
                let content = row_with_separator(vec![
                    text(negation.get_string()).into(),
                    argument_phrase.view(is_full),
                ]);
                category_box(category, content, background)
            }
            ArgumentPhrase::Three(binary_operator, argument_phrase1, argument_phrase2) => {
                let category = "ArgumentPhrase".to_string();
                let content = row_with_separator(vec![
                    text(binary_operator.get_string()).into(),
                    argument_phrase1.view(is_full),
                    argument_phrase2.view(is_full),
                ]);
                category_box(category, content, background)
            }
            ArgumentPhrase::Four(quantified_argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = row_with_separator(vec![quantified_argument.view(is_full)]);
                category_box(category, content, background)
            }
            ArgumentPhrase::Five(
                mta_event_to_argument_phrase_converter,
                mta_event,
                mta_event_to_argument_phrase_converter_end,
            ) => {
                let category = "ArgumentPhrase".to_string();
                let content = row_with_separator(vec![
                    text(mta_event_to_argument_phrase_converter.get_string()).into(),
                    mta_event.view(is_full),
                    text(mta_event_to_argument_phrase_converter_end.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
            ArgumentPhrase::Six(proper_noun_opener, proper_noun_body, proper_noun_closer) => {
                let category = "ArgumentPhrase".to_string();
                let content = row_with_separator(vec![
                    text(proper_noun_opener.get_string()).into(),
                    text(proper_noun_body.get_string()).into(),
                    text(proper_noun_closer.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
            ArgumentPhrase::Seven(proper_noun) => {
                let category = "ArgumentPhrase".to_string();
                let content = text(proper_noun.get_string()).into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for OptionalCaseArgs {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(248, 215, 228);
        match self {
            OptionalCaseArgs::One(CaseArg::One(label, type_override, arg), rest) => {
                let category = "OptionalCaseArgs".to_string();
                let mut parts = vec![text(label.get_string()).into()];
                if is_full || !type_override.is_empty() {
                    parts.push(type_override.view(is_full));
                }
                parts.push(arg.view(is_full));
                if is_full || !rest.is_empty() {
                    parts.push(rest.view(is_full));
                }
                category_box(category, row_with_separator(parts), background)
            }
            OptionalCaseArgs::Epsilon => {
                let category = "OptionalCaseArgs".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, OptionalCaseArgs::Epsilon)
    }
}

impl TreeViewer for QuantifiedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(228, 225, 242);
        match self {
            QuantifiedArgument::One(quantifier_modifier, determined_argument) => {
                let category = "QuantifiedArgument".to_string();
                let mut parts = vec![];
                if is_full || !quantifier_modifier.is_empty() {
                    parts.push(quantifier_modifier.view(is_full));
                }
                parts.push(determined_argument.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for DeterminedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(215, 212, 236);
        match self {
            DeterminedArgument::One(determiner_modifier, retrieved_argument) => {
                let category = "DeterminedArgument".to_string();
                let mut parts = vec![];
                if is_full || !determiner_modifier.is_empty() {
                    parts.push(determiner_modifier.view(is_full));
                }
                parts.push(retrieved_argument.view(is_full));
                category_box(category, row_with_separator(parts), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for RetrievedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(203, 198, 229);
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                let category = "RetrievedArgument".to_string();
                let content = row_with_separator(vec![
                    text(argument_retriever.get_string()).into(),
                    argument_retrieval_binary_predicate.view(is_full),
                ]);
                category_box(category, content, background)
            }
            RetrievedArgument::Two(
                adjunct_argument_retriever,
                case_label,
                argument_retrieval_binary_predicate,
            ) => {
                let category = "RetrievedArgument".to_string();
                let content = row_with_separator(vec![
                    text(adjunct_argument_retriever.get_string()).into(),
                    text(case_label.get_string()).into(),
                    argument_retrieval_binary_predicate.view(is_full),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentRetrievalBinaryPredicate {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(190, 184, 223);
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                let category = "ArgumentRetrievalBinaryPredicate".to_string();
                let mut parts = vec![post_modified_binary_predicate_phrase.view(is_full)];
                if is_full || !argument_retrieval_with_argument_phrase.is_empty() {
                    parts.push(argument_retrieval_with_argument_phrase.view(is_full));
                }
                category_box(category, row_with_separator(parts), background)
            }
            ArgumentRetrievalBinaryPredicate::Two(
                mta_event_to_argument_phrase_converter,
                mta_event,
                mta_event_to_argument_phrase_converter_end,
            ) => {
                let category = "ArgumentRetrievalBinaryPredicate".to_string();
                let content = row_with_separator(vec![
                    text(mta_event_to_argument_phrase_converter.get_string()).into(),
                    mta_event.view(is_full),
                    text(mta_event_to_argument_phrase_converter_end.get_string()).into(),
                ]);
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentRetrievalWithArgumentPhrase {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(177, 170, 217);
        match self {
            ArgumentRetrievalWithArgumentPhrase::One(
                post_argument_phrase_modifier,
                argument_phrase,
            ) => {
                let category = "ArgumentRetrievalWithArgumentPhrase".to_string();
                let content = row_with_separator(vec![
                    text(post_argument_phrase_modifier.get_string()).into(),
                    argument_phrase.view(is_full),
                ]);
                category_box(category, content, background)
            }
            ArgumentRetrievalWithArgumentPhrase::Epsilon => {
                let category = "ArgumentRetrievalWithArgumentPhrase".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, ArgumentRetrievalWithArgumentPhrase::Epsilon)
    }
}

impl TreeViewer for QuantifierModifier {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(251, 228, 228);
        match self {
            QuantifierModifier::One(quantifier) => {
                let category = "QuantifierModifier".to_string();
                let content = text(quantifier.get_string()).into();
                category_box(category, content, background)
            }
            QuantifierModifier::Two(numerical_expression) => {
                let category = "QuantifierModifier".to_string();
                let content = numerical_expression.view(is_full);
                category_box(category, content, background)
            }
            QuantifierModifier::Epsilon => {
                let category = "QuantifierModifier".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, QuantifierModifier::Epsilon)
    }
}

impl TreeViewer for NumericalExpression {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(247, 200, 200);
        match self {
            NumericalExpression::One(number) => {
                let category = "NumericalExpression".to_string();
                let content = text(number.get_string()).into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for DeterminerModifier {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(251, 228, 228);
        match self {
            DeterminerModifier::One(determiner) => {
                let category = "DeterminerModifier".to_string();
                let content = text(determiner.get_string()).into();
                category_box(category, content, background)
            }
            DeterminerModifier::Epsilon => {
                let category = "DeterminerModifier".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, DeterminerModifier::Epsilon)
    }
}

impl TreeViewer for TypeOverrideModifier {
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(251, 228, 228);
        match self {
            TypeOverrideModifier::One(type_override) => {
                let category = "TypeOverrideModifier".to_string();
                let content = text(type_override.get_string()).into();
                category_box(category, content, background)
            }
            TypeOverrideModifier::Epsilon => {
                let category = "TypeOverrideModifier".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, TypeOverrideModifier::Epsilon)
    }
}
