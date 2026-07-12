use iced::advanced::graphics::text::Paragraph as ConcreteParagraph;
use iced::advanced::text::{self as adv_text, Paragraph as ParagraphTrait, Shaping, Wrapping};
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::mouse;
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::widget::{
    Space, button, canvas, column, container, row, scrollable, stack, text, text_editor,
};
use iced::{
    Alignment, Color, Element, Font, Pixels, Point, Rectangle, Renderer, Size, Subscription, Task,
    Theme,
};

use language::tokenizer::{TerminalType, tokenize};
use language::{Diagnostic, diagnose, parser::*};

// text_editor のデフォルト内部パディング
const TEXT_EDITOR_PADDING: f32 = 5.0;
// 波線の振幅・波長
const WAVE_AMPLITUDE: f32 = 1.5;
const WAVE_LENGTH: f32 = 5.0;
// フォントサイズの初期値・最小・最大
const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 48.0;
const FONT_SIZE_STEP: f32 = 2.0;

fn main() -> iced::Result {
    iced::application(Editor::new, Editor::update, Editor::view)
        .title("Artificial Language Editor")
        .subscription(Editor::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    ToFull,
    ToCompact,
    Diagnosing(Result<Vec<Ast>, Vec<Diagnostic>>),
    IncreaseFontSize,
    DecreaseFontSize,
}

enum Viewer {
    Full(Vec<Ast>),
    Compact(Vec<Ast>),
    Error(Vec<Diagnostic>),
}

struct Editor {
    content: text_editor::Content,
    viewer: Viewer,
    diagnostics: Vec<Diagnostic>,
    font_size: f32,
    tokens: Vec<TerminalType>,
}

impl Editor {
    fn new() -> Self {
        let text = "parobe sa mi si ba";
        let asts = diagnose(text.to_string()).unwrap();
        let tokens = tokenize(text)
            .map(|sentences| sentences.into_iter().flatten().collect())
            .unwrap_or_default();
        Self {
            content: text_editor::Content::with_text(text),
            viewer: Viewer::Compact(asts),
            diagnostics: vec![],
            font_size: DEFAULT_FONT_SIZE,
            tokens,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                let text = self.content.text();
                self.tokens = tokenize(&text)
                    .map(|sentences| sentences.into_iter().flatten().collect())
                    .unwrap_or_default();
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
            Message::IncreaseFontSize => {
                self.font_size = (self.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                Task::none()
            }
            Message::DecreaseFontSize => {
                self.font_size = (self.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                Task::none()
            }
            Message::Diagnosing(diagnostics_result) => match diagnostics_result {
                Err(diagnostics) => {
                    self.diagnostics = diagnostics.clone();
                    self.viewer = Viewer::Error(diagnostics);
                    Task::none()
                }
                Ok(asts) => {
                    self.diagnostics = vec![];
                    self.viewer = Viewer::Compact(asts);
                    Task::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::keyboard::listen().filter_map(|event| match event {
            iced::keyboard::Event::KeyPressed { key, modifiers, .. }
                if modifiers.contains(Modifiers::CTRL) =>
            {
                match &key {
                    Key::Character(c) if c.as_str() == "+" || c.as_str() == "=" => {
                        Some(Message::IncreaseFontSize)
                    }
                    Key::Character(c) if c.as_str() == "-" => Some(Message::DecreaseFontSize),
                    Key::Named(Named::ArrowUp) => Some(Message::IncreaseFontSize),
                    Key::Named(Named::ArrowDown) => Some(Message::DecreaseFontSize),
                    _ => None,
                }
            }
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let font_size = self.font_size;
        let line_height = font_size * 1.3;
        let content = text_editor(&self.content)
            .on_action(Message::Edit)
            .font(Font::MONOSPACE)
            .size(font_size);
        let viewer = match &self.viewer {
            Viewer::Full(asts) => {
                let body: Element<_> = if let Some(ast) = asts.first() {
                    scrollable(ast.view(true))
                        .direction(scrollable::Direction::Both {
                            vertical: scrollable::Scrollbar::new(),
                            horizontal: scrollable::Scrollbar::new(),
                        })
                        .into()
                } else {
                    text("").into()
                };
                column![
                    row![
                        text("Full View"),
                        Space::new().width(iced::Length::Fill),
                        button("Compact View").on_press(Message::ToCompact),
                    ],
                    body,
                ]
                .into()
            }
            Viewer::Compact(asts) => {
                let body: Element<_> = if let Some(ast) = asts.first() {
                    scrollable(ast.view(false))
                        .direction(scrollable::Direction::Both {
                            vertical: scrollable::Scrollbar::new(),
                            horizontal: scrollable::Scrollbar::new(),
                        })
                        .into()
                } else {
                    text("").into()
                };
                column![
                    row![
                        button("Full View").on_press(Message::ToFull),
                        Space::new().width(iced::Length::Fill),
                        text("Compact View"),
                    ],
                    body,
                ]
                .into()
            }
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

        let lines: Vec<String> = (0..self.content.line_count())
            .filter_map(|i| self.content.line(i).map(|l| l.text.to_string()))
            .collect();

        let overlay = canvas(DiagnosticOverlay {
            diagnostics: self.diagnostics.clone(),
            tokens: self.tokens.clone(),
            lines,
            font_size,
            line_height,
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

        let editor_stack = stack![content, overlay]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        container(row![
            container(editor_stack)
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

// トークン種別ごとの強調色（半透明）
fn token_color(token: &TerminalType) -> Option<Color> {
    match token {
        TerminalType::BinaryPredicate(_) => Some(Color::from_rgba8(80, 180, 80, 0.25)),
        TerminalType::Argument(_) => Some(Color::from_rgba8(80, 130, 220, 0.25)),
        TerminalType::Tense(_) => Some(Color::from_rgba8(160, 80, 220, 0.25)),
        TerminalType::Aspect(_) => Some(Color::from_rgba8(220, 80, 160, 0.25)),
        TerminalType::ModalityBase(_) => Some(Color::from_rgba8(60, 200, 200, 0.25)),
        TerminalType::Negation(_) => Some(Color::from_rgba8(220, 60, 60, 0.25)),
        TerminalType::ModalityNegation(_)
        | TerminalType::TenseNegation(_)
        | TerminalType::AspectNegation(_) => Some(Color::from_rgba8(230, 120, 60, 0.25)),
        TerminalType::InterrogativeMarker(_) => Some(Color::from_rgba8(210, 210, 50, 0.25)),
        TerminalType::CaseLabel(_) => Some(Color::from_rgba8(60, 160, 180, 0.25)),
        TerminalType::BinaryOperator(_) => Some(Color::from_rgba8(130, 130, 130, 0.25)),
        TerminalType::ArgumentRetriever(_) => Some(Color::from_rgba8(160, 60, 160, 0.25)),
        TerminalType::Quantifier(_) => Some(Color::from_rgba8(60, 170, 170, 0.25)),
        TerminalType::Number(_) => Some(Color::from_rgba8(120, 120, 210, 0.25)),
        TerminalType::Determiner(_) => Some(Color::from_rgba8(170, 120, 60, 0.25)),
        TerminalType::MtaEventToArgumentPhraseConverter(_)
        | TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
            Some(Color::from_rgba8(210, 60, 210, 0.25))
        }
        TerminalType::PostMtaEventModifier(_)
        | TerminalType::PostBinaryPredicatePhraseModifier(_)
        | TerminalType::PostArgumentPhraseModifier(_) => Some(Color::from_rgba8(60, 60, 180, 0.25)),
        TerminalType::ModalityForce(_) => Some(Color::from_rgba8(60, 200, 200, 0.25)),
        TerminalType::Eos(_) => None,
    }
}

// トークン強調と Diagnostic 波線を描画するキャンバスオーバーレイ
struct DiagnosticOverlay {
    diagnostics: Vec<Diagnostic>,
    tokens: Vec<TerminalType>,
    lines: Vec<String>,
    font_size: f32,
    line_height: f32,
}

impl canvas::Program<Message> for DiagnosticOverlay {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());

        // 等幅フォントの実際の文字幅を計測（grapheme_position はスペースをカウントしないためズレる）
        let char_width = ConcreteParagraph::with_text(adv_text::Text {
            content: "A",
            bounds: Size::INFINITE,
            size: Pixels(self.font_size),
            line_height: adv_text::LineHeight::Relative(1.3),
            font: Font::MONOSPACE,
            align_x: adv_text::Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            shaping: Shaping::Basic,
            wrapping: Wrapping::None,
        })
        .min_bounds()
        .width;

        // トークン種別ごとに背景色矩形を描画
        for token in &self.tokens {
            if let Some(color) = token_color(token) {
                let start = token.start();
                let end = token.end();
                if start.line() == end.line() {
                    let x = TEXT_EDITOR_PADDING + start.character() as f32 * char_width;
                    let y = TEXT_EDITOR_PADDING + start.line() as f32 * self.line_height;
                    let width = (end.character() - start.character()) as f32 * char_width;
                    frame.fill_rectangle(
                        Point::new(x, y),
                        Size::new(width, self.line_height),
                        color,
                    );
                }
            }
        }

        for diagnostic in &self.diagnostics {
            let start = diagnostic.range.start();
            let end = diagnostic.range.end();

            // 開始行と終了行それぞれに波線を描画
            let ranges: &[(usize, usize, usize)] = if start.line() == end.line() {
                &[(start.line(), start.character(), end.character())]
            } else {
                &[
                    (start.line(), start.character(), usize::MAX),
                    (end.line(), 0, end.character()),
                ]
            };

            for &(line_idx, col_start, col_end) in ranges {
                // 末尾の改行を除いた行テキストで paragraph を作成
                let line_text = self
                    .lines
                    .get(line_idx)
                    .map(|s| s.trim_end_matches('\n'))
                    .unwrap_or("");
                let char_count = line_text.chars().count();
                let col_end = col_end.min(char_count);

                if col_end <= col_start {
                    continue;
                }

                let x_start = TEXT_EDITOR_PADDING + col_start as f32 * char_width;
                let x_end = TEXT_EDITOR_PADDING + col_end as f32 * char_width;

                // y は LINE_HEIGHT ベースで計算（grapheme_position の y は glyph トップ座標で小さすぎる）
                // 行底端の少し手前 = ちょうど文字の下に来る
                let y = TEXT_EDITOR_PADDING + (line_idx as f32 + 1.0) * self.line_height - 2.0;

                if x_end <= x_start {
                    continue;
                }

                let path = Path::new(|builder| {
                    builder.move_to(Point::new(x_start, y));
                    let mut x = x_start;
                    let mut going_up = true;
                    while x < x_end {
                        let next_x = (x + WAVE_LENGTH).min(x_end);
                        let control_x = x + (next_x - x) / 2.0;
                        let control_y = if going_up {
                            y - WAVE_AMPLITUDE
                        } else {
                            y + WAVE_AMPLITUDE
                        };
                        builder.quadratic_curve_to(
                            Point::new(control_x, control_y),
                            Point::new(next_x, y),
                        );
                        x = next_x;
                        going_up = !going_up;
                    }
                });

                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color::from_rgb(1.0, 0.0, 0.0))
                        .with_width(1.5),
                );
            }
        }

        vec![frame.into_geometry()]
    }
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
        Space::new().width(iced::Length::Fixed(50.0)), // 左に空白
        main_content
    ]
    .align_y(Alignment::Center);

    // 全体を縦に積む
    let column = column![
        header,
        Space::new().height(iced::Length::Fixed(10.0)),
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
                    Space::new().width(iced::Length::Fixed(10.0)),
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)))
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
                    Space::new().width(iced::Length::Fixed(10.0)),
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
            ModifiedModality::One(modality, modality_force) => {
                let category = "ModifiedModality".to_string();
                let content = row![
                    text(modality.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    text(modality_force.get_string()),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            ModifiedModality::Two(modality_negation, modality, modality_force) => {
                let category = "ModifiedModality".to_string();
                let content = row![
                    text(modality_negation.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    text(modality.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    text(modality_force.get_string()),
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
            ModifiedTense::Two(tense_negation, tense) => {
                let category = "ModifiedTense".to_string();
                let content = row![
                    text(tense_negation.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    text(tense.get_string()),
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
            ModifiedAspect::Two(aspect_negation, aspect) => {
                let category = "ModifiedAspect".to_string();
                let content = row![
                    text(aspect_negation.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    text(aspect.get_string()),
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)));
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)));
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)));
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
            Event::One(post_modified_binary_predicate_phrase, optional_case_args) => {
                let category = "Event".to_string();
                let mut content = row![post_modified_binary_predicate_phrase.view(is_full)];
                if is_full || !optional_case_args.is_empty() {
                    content = content
                        .push(Space::new().width(iced::Length::Fixed(10.0)))
                        .push(optional_case_args.view(is_full));
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)))
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
                    Space::new().width(iced::Length::Fixed(10.0)),
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
                    Space::new().width(iced::Length::Fixed(10.0)),
                    binary_predicate_phrase.view(_is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            BinaryPredicatePhrase::Three(
                binary_operator,
                binary_predicate_phrase1,
                binary_predicate_phrase2,
            ) => {
                let category = "BinaryPredicatePhrase".to_string();
                let content = row![
                    text(binary_operator.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    binary_predicate_phrase1.view(_is_full),
                    Space::new().width(iced::Length::Fixed(10.0)),
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
                    Space::new().width(iced::Length::Fixed(10.0)),
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
                    Space::new().width(iced::Length::Fixed(10.0)),
                    argument_phrase1.view(is_full),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    argument_phrase2.view(is_full),
                ]
                .align_y(Alignment::Center)
                .into();
                category_box(category, content, background)
            }
            ArgumentPhrase::Four(quantified_argument) => {
                let category = "ArgumentPhrase".to_string();
                let content = row![quantified_argument.view(is_full),]
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
                    Space::new().width(iced::Length::Fixed(10.0)),
                    mta_event.view(is_full),
                    Space::new().width(iced::Length::Fixed(10.0)),
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

impl TreeViewer for OptionalCaseArgs {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(250, 240, 205);
        match self {
            OptionalCaseArgs::One(CaseArg::One(label, arg), rest) => {
                let category = "OptionalCaseArgs".to_string();
                let mut content = row![
                    text(label.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    arg.view(is_full),
                ];
                if is_full || !rest.is_empty() {
                    content = content
                        .push(Space::new().width(iced::Length::Fixed(10.0)))
                        .push(rest.view(is_full));
                }
                category_box(
                    category,
                    content.align_y(Alignment::Center).into(),
                    background,
                )
            }
            OptionalCaseArgs::Epsilon => text("ε").into(),
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self, OptionalCaseArgs::Epsilon)
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)));
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
            DeterminedArgument::One(determiner_modifier, retrieved_argument) => {
                let category = "DeterminedArgument".to_string();
                let mut content = row![];
                if is_full || !determiner_modifier.is_empty() {
                    content = content
                        .push(determiner_modifier.view(is_full))
                        .push(Space::new().width(iced::Length::Fixed(10.0)));
                }
                content = content
                    .push(retrieved_argument.view(is_full))
                    .align_y(Alignment::Center);
                category_box(category, content.into(), background)
            }
        }
    }
    fn is_empty(&self) -> bool {
        false
    }
}

impl TreeViewer for RetrievedArgument {
    fn view(&self, is_full: bool) -> Element<'static, Message> {
        let background = iced::Color::from_rgb8(50, 50, 50);
        match self {
            RetrievedArgument::One(argument_retriever, argument_retrieval_binary_predicate) => {
                let category = "RetrievedArgument".to_string();
                let content = row![
                    text(argument_retriever.get_string()),
                    Space::new().width(iced::Length::Fixed(10.0)),
                    argument_retrieval_binary_predicate.view(is_full),
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
                        .push(Space::new().width(iced::Length::Fixed(10.0)))
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
                    Space::new().width(iced::Length::Fixed(10.0)),
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
