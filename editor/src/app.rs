//! アプリケーション本体: 状態(`Editor`)・メッセージ・更新・購読・トップレベルの描画。

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{Space, button, canvas, column, container, row, stack, text, text_editor};
use iced::{Element, Font, Subscription, Task};

use language::parser::Ast;
use language::tokenizer::{TerminalType, tokenize};
use language::{Diagnosed, Diagnostic, diagnose_tokens};

use crate::overlay::{DiagnosticOverlay, LINE_HEIGHT_RATIO};
use crate::tree_view::asts_view;

// フォントサイズの初期値・最小・最大
const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 48.0;
const FONT_SIZE_STEP: f32 = 2.0;

pub(crate) fn run() -> iced::Result {
    iced::application(Editor::new, Editor::update, Editor::view)
        .title("Artificial Language Editor")
        .subscription(Editor::subscription)
        .run()
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Edit(text_editor::Action),
    ToFull,
    ToCompact,
    Diagnosing(Result<Diagnosed, Vec<Diagnostic>>),
    IncreaseFontSize,
    DecreaseFontSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewMode {
    Full,
    Compact,
}

enum Viewer {
    Asts { asts: Vec<Ast>, mode: ViewMode },
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
        match tokenize(text) {
            Ok(sentences) => {
                let tokens: Vec<TerminalType> = sentences.iter().flatten().cloned().collect();
                let (viewer, diagnostics) = match diagnose_tokens(sentences) {
                    Ok(Diagnosed { asts, warnings }) => (
                        Viewer::Asts {
                            asts,
                            mode: ViewMode::Compact,
                        },
                        warnings,
                    ),
                    Err(diagnostics) => {
                        eprintln!(
                            "warning: default editor text failed to diagnose: {diagnostics:?}"
                        );
                        (Viewer::Error(diagnostics.clone()), diagnostics)
                    }
                };
                Self {
                    content: text_editor::Content::with_text(text),
                    viewer,
                    diagnostics,
                    font_size: DEFAULT_FONT_SIZE,
                    tokens,
                }
            }
            Err(e) => {
                eprintln!("warning: default editor text failed to tokenize: {e:?}");
                Self {
                    content: text_editor::Content::with_text(text),
                    viewer: Viewer::Error(vec![]),
                    diagnostics: vec![],
                    font_size: DEFAULT_FONT_SIZE,
                    tokens: vec![],
                }
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                let text = self.content.text();
                match tokenize(&text) {
                    Ok(sentences) => {
                        self.tokens = sentences.iter().flatten().cloned().collect();
                        Task::perform(async { diagnose_tokens(sentences) }, Message::Diagnosing)
                    }
                    Err(e) => {
                        self.tokens = vec![];
                        let diagnostics = vec![Diagnostic::error(e.range, "tokenize error")];
                        self.diagnostics = diagnostics.clone();
                        self.viewer = Viewer::Error(diagnostics);
                        Task::none()
                    }
                }
            }
            Message::ToFull => {
                if let Viewer::Asts { mode, .. } = &mut self.viewer {
                    *mode = ViewMode::Full;
                }
                Task::none()
            }
            Message::ToCompact => {
                if let Viewer::Asts { mode, .. } = &mut self.viewer {
                    *mode = ViewMode::Compact;
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
                Ok(Diagnosed { asts, warnings }) => {
                    self.diagnostics = warnings;
                    self.viewer = Viewer::Asts {
                        asts,
                        mode: ViewMode::Compact,
                    };
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
        let line_height = font_size * LINE_HEIGHT_RATIO;
        let content = text_editor(&self.content)
            .on_action(Message::Edit)
            .font(Font::MONOSPACE)
            .size(font_size);
        let viewer = match &self.viewer {
            Viewer::Asts { asts, mode } => {
                let is_full = matches!(mode, ViewMode::Full);
                column![view_toggle_row(*mode), asts_view(asts, is_full)].into()
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
                error_viewer(error_string)
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

fn view_toggle_row(mode: ViewMode) -> Element<'static, Message> {
    match mode {
        ViewMode::Full => row![
            text("Full View"),
            Space::new().width(iced::Length::Fill),
            button("Compact View").on_press(Message::ToCompact),
        ]
        .into(),
        ViewMode::Compact => row![
            button("Full View").on_press(Message::ToFull),
            Space::new().width(iced::Length::Fill),
            text("Compact View"),
        ]
        .into(),
    }
}

fn error_viewer(s: String) -> Element<'static, Message> {
    container(text(s)).style(container::bordered_box).into()
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
