use std::vec;

use iced::widget::{Space, column, container, row, scrollable, text, text_editor};
use iced::{Alignment, Element, Task};

use language::parser::*;
use language::tokenizer::{Range, add_eos_to_last_sentence, tokenize};
use language::type_checker::type_check;

fn main() -> iced::Result {
    iced::run("Artificial Language Editor", Editor::update, Editor::view)
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Diagnosing(Result<Vec<Ast>, Vec<Diagnostic>>),
}

enum Viewer {
    Rich(Vec<Ast>),
    Error(Vec<Diagnostic>),
}

struct Editor {
    content: text_editor::Content,
    viewer: Viewer,
}

impl Editor {
    fn new() -> Self {
        let text = "parobe mi si ba";
        let asts = diagnose(text.to_string()).unwrap();
        Self {
            content: text_editor::Content::with_text(text),
            viewer: Viewer::Rich(asts),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                let text = self.content.text();
                Task::perform(async { diagnose(text) }, Message::Diagnosing)
            }
            Message::Diagnosing(diagnostics_result) => match diagnostics_result {
                Err(diagnostics) => {
                    self.viewer = Viewer::Error(diagnostics);
                    Task::none()
                }
                Ok(asts) => {
                    self.viewer = Viewer::Rich(asts);
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = text_editor(&self.content).on_action(Message::Edit);
        let viewer = match &self.viewer {
            Viewer::Rich(asts) => scrollable(asts[0].view())
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::new(),
                    horizontal: scrollable::Scrollbar::new(),
                })
                .into(),
            Viewer::Error(diagnostics) => {
                let mut error_string = String::new();
                for diagnostic in diagnostics {
                    let severity = match diagnostic.severity {
                        DiagnosticSeverity::Error => "Error",
                    };
                    let message = format!(
                        "[{}] {} ({}:{}) - ({}:{})\n",
                        severity,
                        diagnostic.message,
                        diagnostic.range.start().line(),
                        diagnostic.range.start().character(),
                        diagnostic.range.end().line(),
                        diagnostic.range.end().character()
                    );
                    error_string.push_str(&message);
                }
                viewer(error_string)
            }
        };

        container(row![
            container(content)
                .padding(20)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            container(viewer)
                .padding(20)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
        ])
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
        // scrollable(column![
        //     "Scroll me!",
        //     Space::new(iced::Length::Fixed(3000.0), iced::Length::Fixed(3000.0)),
        //     "You did it!",
        // ])
        // .into()
    }
}

fn viewer(s: String) -> Element<'static, Message> {
    container(text(s)).style(container::bordered_box).into()
}

fn category_box(
    category: String,
    main_content: Element<'static, Message>,
) -> Element<'static, Message>
where
    Message: 'static + Clone,
{
    // 左上にカテゴリ名
    let header = row![text(category).size(16)].align_y(Alignment::Start);

    // main コンテンツ（中央より少し右に配置）
    let content = row![
        Space::new(iced::Length::Fixed(50.0), iced::Length::Fixed(0.0)), // 左に空白
        main_content
    ]
    .align_y(Alignment::Center);

    // 全体を縦に積む
    let column = column![
        header,
        Space::with_height(iced::Length::Fixed(1.0)),
        content
    ];

    // 枠付き Container
    container(column)
        .padding(10)
        .width(iced::Length::Shrink)
        .height(iced::Length::Shrink)
        .style(|_| container::Style {
            border: iced::Border {
                color: iced::Color::from_rgb8(200, 200, 200),
                width: 2.0,
                ..iced::Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

trait TreeViewer {
    fn view(&self) -> Element<'static, Message>;
}

impl TreeViewer for Ast {
    fn view(&self) -> Element<'static, Message> {
        let s = self.get_s();
        container(s.view()).into()
    }
}

impl TreeViewer for S {
    fn view(&self) -> Element<'static, Message> {
        match self {
            S::One(declarative_sentence) => {
                let category = "s".to_string();
                let content = declarative_sentence.view();
                category_box(category, content)
            }
            S::Two(interrogative_sentence) => {
                let content = interrogative_sentence.view();
                category_box("s".to_string(), content)
            }
        }
    }
}

impl TreeViewer for DeclarativeSentence {
    fn view(&self) -> Element<'static, Message> {
        match self {
            DeclarativeSentence::One(post_modified_mta_event) => {
                let category = "PostModifiedMtaEvent".to_string();
                let content = post_modified_mta_event.view();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for InterrogativeSentence {
    fn view(&self) -> Element<'static, Message> {
        match self {
            InterrogativeSentence::One(interrogative_marker, post_modified_mta_event) => {
                let category = "InterrogativeSentence".to_string();
                let content = row![
                    text(interrogative_marker.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    post_modified_mta_event.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for PostModifiedMtaEvent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            PostModifiedMtaEvent::One(mta_event, post_mta_event_expression) => {
                let category = "PostModifiedMtaEvent".to_string();
                let content = row![
                    mta_event.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    post_mta_event_expression.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for PostMtaEventExpression {
    fn view(&self) -> Element<'static, Message> {
        match self {
            PostMtaEventExpression::One(post_mta_event_modifier, mta_event) => {
                let category = "PostMtaEventExpression".to_string();
                let content = row![
                    text(post_mta_event_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    mta_event.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            PostMtaEventExpression::Epsilon => {
                let category = "PostMtaEventExpression".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModalityExpression {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModalityExpression::One(modality_with_agent) => {
                let category = "ModalityExpression".to_string();
                let content = modality_with_agent.view();
                category_box(category, content)
            }
            ModalityExpression::Epsilon => {
                let category = "ModalityExpression".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModalityWithAgent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModalityWithAgent::One(modified_modality, modality_agent) => {
                let category = "ModalityWithAgent".to_string();
                let content = row![
                    modified_modality.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modality_agent.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModifiedModality {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModifiedModality::One(modality) => {
                let category = "ModifiedModality".to_string();
                let content = text(modality.get_string()).into();
                category_box(category, content)
            }
            ModifiedModality::Two(modality_negation, modified_modality) => {
                let category = "ModifiedModality".to_string();
                let content = row![
                    text(modality_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_modality.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModalityAgent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModalityAgent::One(post_argument_phrase_modifier, argument_phrase) => {
                let category = "ModalityAgent".to_string();
                let content = row![
                    text(post_argument_phrase_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ModalityAgent::Epsilon => {
                let category = "ModalityAgent".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for TenseExpression {
    fn view(&self) -> Element<'static, Message> {
        match self {
            TenseExpression::One(modified_tense) => {
                let category = "TenseExpression".to_string();
                let content = modified_tense.view();
                category_box(category, content)
            }
            TenseExpression::Epsilon => {
                let category = "TenseExpression".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModifiedTense {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModifiedTense::One(tense) => {
                let category = "ModifiedTense".to_string();
                let content = text(tense.get_string()).into();
                category_box(category, content)
            }
            ModifiedTense::Two(tense_negation, modified_tense) => {
                let category = "ModifiedTense".to_string();
                let content = row![
                    text(tense_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_tense.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for AspectExpression {
    fn view(&self) -> Element<'static, Message> {
        match self {
            AspectExpression::One(modified_aspect) => {
                let category = "AspectExpression".to_string();
                let content = modified_aspect.view();
                category_box(category, content)
            }
            AspectExpression::Epsilon => {
                let category = "AspectExpression".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ModifiedAspect {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ModifiedAspect::One(aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = text(aspect.get_string()).into();
                category_box(category, content)
            }
            ModifiedAspect::Two(aspect_negation, modified_aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = row![
                    text(aspect_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_aspect.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for MtaEvent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            MtaEvent::One(modality_expression, ta_event) => {
                let category = "MtaEvent".to_string();
                let content = row![
                    modality_expression.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    ta_event.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for TaEvent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            TaEvent::One(tense_expression, a_event) => {
                let category = "TaEvent".to_string();
                let content = row![
                    tense_expression.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    a_event.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for AEvent {
    fn view(&self) -> Element<'static, Message> {
        match self {
            AEvent::One(aspect_expression, event) => {
                let category = "AEvent".to_string();
                let content = row![
                    aspect_expression.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    event.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for Event {
    fn view(&self) -> Element<'static, Message> {
        match self {
            Event::One(
                post_modified_binary_predicate_phrase,
                argument_phrase,
                argument_phrase_or_epsilon,
            ) => {
                let category = "Event".to_string();
                let content = row![
                    post_modified_binary_predicate_phrase.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase_or_epsilon.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for PostModifiedBinaryPredicatePhrase {
    fn view(&self) -> Element<'static, Message> {
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                post_binary_predicate_phrase_expression,
            ) => {
                let category = "PostModifiedBinaryPredicatePhrase".to_string();
                let content = row![
                    binary_predicate_phrase.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    post_binary_predicate_phrase_expression.view(),
                ]
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for PostBinaryPredicatePhraseExpression {
    fn view(&self) -> Element<'static, Message> {
        match self {
            PostBinaryPredicatePhraseExpression::One(
                post_binary_predicate_phrase_modifier,
                binary_predicate_phrase,
            ) => {
                let category = "PostBinaryPredicatePhraseExpression".to_string();
                let content = row![
                    text(post_binary_predicate_phrase_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            PostBinaryPredicatePhraseExpression::Epsilon => {
                let category = "PostBinaryPredicatePhraseExpression".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for BinaryPredicatePhrase {
    fn view(&self) -> Element<'static, Message> {
        match self {
            BinaryPredicatePhrase::One(binary_predicate) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = text(binary_predicate.get_string()).into();
                category_box(category, content)
            }
            BinaryPredicatePhrase::Two(negation, binary_predicate_phrase) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            BinaryPredicatePhrase::Three(argument_phrase_transposer, binary_predicate_phrase) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(argument_phrase_transposer.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            BinaryPredicatePhrase::Four(
                binary_operator,
                binary_predicate_phrase1,
                binary_predicate_phrase2,
            ) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(binary_operator.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase1.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase2.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ArgumentPhrase {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ArgumentPhrase::One(argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = text(argument.get_string()).into();
                category_box(category, content)
            }
            ArgumentPhrase::Two(negation, argument_phrase) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ArgumentPhrase::Three(binary_operator, argument_phrase1, argument_phrase2) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(binary_operator.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase1.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase2.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ArgumentPhrase::Four(
                argument_retriever,
                determined_argument_retrieval_binary_predicate,
            ) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(argument_retriever.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    determined_argument_retrieval_binary_predicate.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ArgumentPhrase::Five(
                mta_event_to_argument_phrase_converter,
                mta_event,
                mta_event_to_argument_phrase_converter_end,
            ) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(mta_event_to_argument_phrase_converter.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    mta_event.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    text(mta_event_to_argument_phrase_converter_end.get_string()),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ArgumentPhraseOrEpsilon {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ArgumentPhraseOrEpsilon::One(argument_phrase_separator, argument_phrase) => {
                let category = "ArgumentPhraseOrEpsilon".to_string();
                let content = row![
                    text(argument_phrase_separator.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view()
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ArgumentPhraseOrEpsilon::Epsilon => text("ε").into(),
        }
    }
}

impl TreeViewer for DeterminedArgumentRetrievalBinaryPredicate {
    fn view(&self) -> Element<'static, Message> {
        match self {
            DeterminedArgumentRetrievalBinaryPredicate::One(
                quantification,
                articlized_argument_retrieval_binary_predicate,
            ) => {
                let category = "DeterminedArgumentRetrievalBinaryPredicate".to_string();
                let content = row![
                    quantification.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    articlized_argument_retrieval_binary_predicate.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ArticlizedArgumentRetrievalBinaryPredicate {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ArticlizedArgumentRetrievalBinaryPredicate::One(
                article,
                argument_retrieval_binary_predicate,
            ) => {
                let category = "DeterminedArgumentRetrievalBinaryPredicate".to_string();
                let content = row![
                    article.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_retrieval_binary_predicate.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ArgumentRetrievalBinaryPredicate {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                let category = "ArgumentRetrievalBinaryPredicate".to_string();
                let content = row![
                    post_modified_binary_predicate_phrase.view(),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_retrieval_with_argument_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for ArgumentRetrievalWithArgumentPhrase {
    fn view(&self) -> Element<'static, Message> {
        match self {
            ArgumentRetrievalWithArgumentPhrase::One(
                post_argument_phrase_modifier,
                argument_phrase,
            ) => {
                let category = "ArgumentRetrievalWithArgumentPhrase".to_string();
                let content = row![
                    text(post_argument_phrase_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content)
            }
            ArgumentRetrievalWithArgumentPhrase::Epsilon => {
                let category = "ArgumentRetrievalWithArgumentPhrase".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for Quantification {
    fn view(&self) -> Element<'static, Message> {
        match self {
            Quantification::One(quantifier) => {
                let category = "Quantification".to_string();
                let content = text(quantifier.get_string()).into();
                category_box(category, content)
            }
            Quantification::Epsilon => {
                let category = "Quantification".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

impl TreeViewer for Article {
    fn view(&self) -> Element<'static, Message> {
        match self {
            Article::One(articlizer) => {
                let category = "Article".to_string();
                let content = text(articlizer.get_string()).into();
                category_box(category, content)
            }
            Article::Epsilon => {
                let category = "Article".to_string();
                let content = text("ε").into();
                category_box(category, content)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Diagnostic {
    range: Range,
    severity: DiagnosticSeverity,
    message: String,
}

#[derive(Debug, Clone)]
enum DiagnosticSeverity {
    Error,
}

fn diagnose(text: String) -> Result<Vec<Ast>, Vec<Diagnostic>> {
    // tokenizing
    let tokenize_result = tokenize(&text);
    if let Err(e) = tokenize_result {
        let diagnostics = vec![Diagnostic {
            range: e.range,
            severity: DiagnosticSeverity::Error,
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
                severity: DiagnosticSeverity::Error,
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
                severity: DiagnosticSeverity::Error,
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

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
