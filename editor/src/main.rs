use iced::widget::{Space, button, column, container, row, scrollable, text, text_editor};
use iced::{Alignment, Element, Task};

use language::{Diagnostic, diagnose, parser::*};

fn main() -> iced::Result {
    iced::run("Artificial Language Editor", Editor::update, Editor::view)
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    ToFull,
    ToCompact,
    Diagnosing(Result<Vec<Ast>, Vec<Diagnostic>>),
}

enum Viewer {
    Full(Vec<Ast>),
    Compact(Vec<Ast>),
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
            viewer: Viewer::Compact(asts),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                let text = self.content.text();
                Task::perform(async { diagnose(text) }, Message::Diagnosing)
            }
            Message::ToFull => {
                if let Viewer::Compact(asts) = &self.viewer {
                    self.viewer = Viewer::Full(asts.clone());
                }
                Task::none()
            }
            Message::ToCompact => {
                if let Viewer::Full(asts) = &self.viewer {
                    self.viewer = Viewer::Compact(asts.clone());
                }
                Task::none()
            }
            Message::Diagnosing(diagnostics_result) => match diagnostics_result {
                Err(diagnostics) => {
                    self.viewer = Viewer::Error(diagnostics);
                    Task::none()
                }
                Ok(asts) => {
                    self.viewer = Viewer::Compact(asts);
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = text_editor(&self.content).on_action(Message::Edit);
        let viewer = match &self.viewer {
            Viewer::Full(asts) => column![
                row![
                    text("Full View"),
                    Space::new(iced::Length::Fill, iced::Length::Fixed(0.0)),
                    button("Compact View").on_press(Message::ToCompact),
                ],
                scrollable(asts[0].view(true)).direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::new(),
                    horizontal: scrollable::Scrollbar::new(),
                }),
            ]
            .into(),
            Viewer::Compact(asts) => column![
                row![
                    button("Full View").on_press(Message::ToFull),
                    Space::new(iced::Length::Fill, iced::Length::Fixed(0.0)),
                    text("Compact View"),
                ],
                scrollable(asts[0].view(false)).direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::new(),
                    horizontal: scrollable::Scrollbar::new(),
                }),
            ]
            .into(),
            Viewer::Error(diagnostics) => {
                let mut error_string = String::new();
                for diagnostic in diagnostics {
                    let message = format!(
                        "{} ({}:{}) - ({}:{})\n",
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
    }
}

fn viewer(s: String) -> Element<'static, Message> {
    container(text(s)).style(container::bordered_box).into()
}

fn category_box(
    category: String,
    main_content: Element<'static, Message>,
    background: iced::Color,
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
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(background)),
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
    fn view(&self, is_full: bool) -> Element<'static, Message>;
    fn is_empty(&self) -> bool;
}

impl TreeViewer for Ast {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let s = self.get_s();
        container(s.view(is_full)).into()
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
                let content = row![
                    text(interrogative_marker.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    post_modified_mta_event.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostModifiedMtaEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(255, 220, 185);
        match self {
            PostModifiedMtaEvent::One(mta_event, post_mta_event_expression) => {
                let category = "PostModifiedMtaEvent".to_string();
                let mut content = row![mta_event.view(is_full),];
                if is_full || !post_mta_event_expression.is_empty() {
                    content = content
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ))
                        .push(post_mta_event_expression.view(is_full))
                }
                content = content.align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostMtaEventExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(255, 200, 160);
        match self {
            PostMtaEventExpression::One(post_mta_event_modifier, mta_event) => {
                let category = "PostMtaEventExpression".to_string();
                let content = row![
                    text(post_mta_event_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    mta_event.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            PostMtaEventExpression::Epsilon => {
                let category = "PostMtaEventExpression".to_string();
                let content = text("ε").into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, PostMtaEventExpression::Epsilon)
    }
}

impl TreeViewer for ModalityExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(0, 190, 255);
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
        let background = iced::Color::from_rgb8(0, 255, 255);
        match self {
            ModifiedModality::One(modality) => {
                let category = "ModifiedModality".to_string();
                let content = text(modality.get_string()).into();
                category_box(category, content, background)
            }
            ModifiedModality::Two(modality_negation, modified_modality) => {
                let category = "ModifiedModality".to_string();
                let content = row![
                    text(modality_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_modality.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
        let background = iced::Color::from_rgb8(250, 240, 160);
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
        let background = iced::Color::from_rgb8(240, 230, 140);
        match self {
            ModifiedTense::One(tense) => {
                let category = "ModifiedTense".to_string();
                let content = text(tense.get_string()).into();
                category_box(category, content, background)
            }
            ModifiedTense::Two(tense_negation, modified_tense) => {
                let category = "ModifiedTense".to_string();
                let content = row![
                    text(tense_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_tense.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
        let background = iced::Color::from_rgb8(155, 250, 155);
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
        let background = iced::Color::from_rgb8(140, 250, 140);
        match self {
            ModifiedAspect::One(aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = text(aspect.get_string()).into();
                category_box(category, content, background)
            }
            ModifiedAspect::Two(aspect_negation, modified_aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = row![
                    text(aspect_negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    modified_aspect.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
        let background = iced::Color::from_rgb8(135, 205, 250);
        match self {
            MtaEvent::One(modality_expression, ta_event) => {
                let category = "MtaEvent".to_string();
                let mut content = row![];
                if is_full || !modality_expression.is_empty() {
                    content = content
                        .push(modality_expression.view(is_full))
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ));
                }
                content = content
                    .push(ta_event.view(is_full))
                    .align_y(Alignment::Center)
                    .align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for TaEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 245, 170);
        match self {
            TaEvent::One(tense_expression, a_event) => {
                let category = "TaEvent".to_string();
                let mut content = row![];
                if is_full || !tense_expression.is_empty() {
                    content = content
                        .push(tense_expression.view(is_full))
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ));
                }
                content = content
                    .push(a_event.view(is_full))
                    .align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for AEvent {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(170, 250, 170);
        match self {
            AEvent::One(aspect_expression, event) => {
                let category = "AEvent".to_string();
                let mut content = row![];
                if is_full || !aspect_expression.is_empty() {
                    content = content
                        .push(aspect_expression.view(is_full))
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ));
                }
                content = content.push(event.view(is_full)).align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for Event {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(240, 255, 240);
        match self {
            Event::One(
                post_modified_binary_predicate_phrase,
                argument_phrase,
                argument_phrase_or_epsilon,
            ) => {
                let category = "Event".to_string();
                let mut content = row![
                    post_modified_binary_predicate_phrase.view(is_full),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(is_full),
                ];
                if is_full || !argument_phrase_or_epsilon.is_empty() {
                    content = content
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ))
                        .push(argument_phrase_or_epsilon.view(is_full));
                }
                content = content.align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostModifiedBinaryPredicatePhrase {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(175, 225, 230);
        match self {
            PostModifiedBinaryPredicatePhrase::One(
                binary_predicate_phrase,
                post_binary_predicate_phrase_expression,
            ) => {
                let category = "PostModifiedBinaryPredicatePhrase".to_string();
                let mut content = row![binary_predicate_phrase.view(is_full)];
                if is_full || !post_binary_predicate_phrase_expression.is_empty() {
                    content = content
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ))
                        .push(post_binary_predicate_phrase_expression.view(is_full));
                }
                content = content.align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for PostBinaryPredicatePhraseExpression {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(175, 240, 240);
        match self {
            PostBinaryPredicatePhraseExpression::One(
                post_binary_predicate_phrase_modifier,
                binary_predicate_phrase,
            ) => {
                let category = "PostBinaryPredicatePhraseExpression".to_string();
                let content = row![
                    text(post_binary_predicate_phrase_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
        let background = iced::Color::from_rgb8(225, 255, 255);
        match self {
            BinaryPredicatePhrase::One(binary_predicate) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = text(binary_predicate.get_string()).into();
                category_box(category, content, background)
            }
            BinaryPredicatePhrase::Two(negation, binary_predicate_phrase) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            BinaryPredicatePhrase::Three(argument_phrase_transposer, binary_predicate_phrase) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(argument_phrase_transposer.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
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
                    binary_predicate_phrase1.view(_is_full),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    binary_predicate_phrase2.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
        let background = iced::Color::from_rgb8(240, 230, 170);
        match self {
            ArgumentPhrase::One(argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = text(argument.get_string()).into();
                category_box(category, content, background)
            }
            ArgumentPhrase::Two(negation, argument_phrase) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(negation.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            ArgumentPhrase::Three(binary_operator, argument_phrase1, argument_phrase2) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(binary_operator.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase1.view(is_full),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase2.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            ArgumentPhrase::Four(argument_retriever, quantified_argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![
                    text(argument_retriever.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    quantified_argument.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
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
                    mta_event.view(is_full),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    text(mta_event_to_argument_phrase_converter_end.get_string()),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentPhraseOrEpsilon {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 240, 205);
        match self {
            ArgumentPhraseOrEpsilon::One(argument_phrase_separator, argument_phrase) => {
                let category = "ArgumentPhraseOrEpsilon".to_string();
                let content = row![
                    text(argument_phrase_separator.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(is_full)
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            ArgumentPhraseOrEpsilon::Epsilon => text("ε").into(),
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, ArgumentPhraseOrEpsilon::Epsilon)
    }
}

impl TreeViewer for QuantifiedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 50, 50);
        match self {
            QuantifiedArgument::One(quantifier_modifier, determined_argument) => {
                let category = "QuantifiedArgument".to_string();
                let mut content = row![];
                if is_full || !quantifier_modifier.is_empty() {
                    content = content
                        .push(quantifier_modifier.view(is_full))
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ));
                }
                content = content
                    .push(determined_argument.view(is_full))
                    .align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for DeterminedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 50, 50);
        match self {
            DeterminedArgument::One(determiner_modifier, argument_retrieval_binary_predicate) => {
                let category = "DeterminedArgument".to_string();
                let mut content = row![];
                if is_full || !determiner_modifier.is_empty() {
                    content = content
                        .push(determiner_modifier.view(is_full))
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ));
                }
                content = content
                    .push(argument_retrieval_binary_predicate.view(is_full))
                    .align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentRetrievalBinaryPredicate {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 50, 50);
        match self {
            ArgumentRetrievalBinaryPredicate::One(
                post_modified_binary_predicate_phrase,
                argument_retrieval_with_argument_phrase,
            ) => {
                let category = "ArgumentRetrievalBinaryPredicate".to_string();
                let mut content = row![post_modified_binary_predicate_phrase.view(is_full)];
                if is_full || !argument_retrieval_with_argument_phrase.is_empty() {
                    content = content
                        .push(Space::new(
                            iced::Length::Fixed(1.0),
                            iced::Length::Fixed(0.0),
                        ))
                        .push(argument_retrieval_with_argument_phrase.view(is_full));
                }
                content = content.align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for ArgumentRetrievalWithArgumentPhrase {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 0, 50);
        match self {
            ArgumentRetrievalWithArgumentPhrase::One(
                post_argument_phrase_modifier,
                argument_phrase,
            ) => {
                let category = "ArgumentRetrievalWithArgumentPhrase".to_string();
                let content = row![
                    text(post_argument_phrase_modifier.get_string()),
                    Space::new(iced::Length::Fixed(1.0), iced::Length::Fixed(0.0)),
                    argument_phrase.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
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
    fn view(&self, _is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 50, 0);
        match self {
            QuantifierModifier::One(quantifier) => {
                let category = "QuantifierModifier".to_string();
                let content = text(quantifier.get_string()).into();
                category_box(category, content, background)
            }
            QuantifierModifier::Two(numerical_expression) => {
                let category = "QuantifierModifier".to_string();
                let content = numerical_expression.view(_is_full);
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
        let background = iced::Color::from_rgb8(0, 50, 0);
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
        let background = iced::Color::from_rgb8(0, 50, 0);
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

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
