//! テキストエディタの上に重ねて描く、トークン種別のハイライトと診断の赤波線オーバーレイ。

use iced::advanced::graphics::text::Paragraph as ConcreteParagraph;
use iced::advanced::text::{self as adv_text, Paragraph as ParagraphTrait, Shaping, Wrapping};
use iced::mouse;
use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::{Color, Font, Pixels, Point, Rectangle, Renderer, Size, Theme};

use language::Diagnostic;
use language::tokenizer::TerminalType;

use crate::app::Message;

// text_editor のデフォルト内部パディング
pub(crate) const TEXT_EDITOR_PADDING: f32 = 5.0;
// 波線の振幅・波長
const WAVE_AMPLITUDE: f32 = 1.5;
const WAVE_LENGTH: f32 = 5.0;
// テキストの行間比率
pub(crate) const LINE_HEIGHT_RATIO: f32 = 1.3;

pub(crate) fn token_color(token: &TerminalType) -> Option<Color> {
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
        TerminalType::InterrogativeMarker(_) | TerminalType::UniversalMarker(_) => {
            Some(Color::from_rgba8(210, 210, 50, 0.25))
        }
        TerminalType::CaseLabel(_) => Some(Color::from_rgba8(60, 160, 180, 0.25)),
        TerminalType::RoleLabel(_) => Some(Color::from_rgba8(220, 170, 90, 0.25)),
        TerminalType::TypeOverride(_) => Some(Color::from_rgba8(60, 200, 120, 0.25)),
        TerminalType::PredicateOperator(_)
        | TerminalType::ArgumentOperator(_)
        | TerminalType::MtaEventOperator(_) => Some(Color::from_rgba8(130, 130, 130, 0.25)),
        TerminalType::ArgumentRetriever(_) => Some(Color::from_rgba8(160, 60, 160, 0.25)),
        TerminalType::Quantifier(_) => Some(Color::from_rgba8(60, 170, 170, 0.25)),
        TerminalType::Number(_) => Some(Color::from_rgba8(120, 120, 210, 0.25)),
        TerminalType::Determiner(_) => Some(Color::from_rgba8(170, 120, 60, 0.25)),
        TerminalType::MtaEventToArgumentPhraseConverter(_)
        | TerminalType::MtaEventToArgumentPhraseConverterEnd(_) => {
            Some(Color::from_rgba8(210, 60, 210, 0.25))
        }
        TerminalType::ProperNounOpener(_) | TerminalType::ProperNounCloser(_) => {
            Some(Color::from_rgba8(210, 60, 210, 0.25))
        }
        TerminalType::ProperNounBody(_) => Some(Color::from_rgba8(140, 140, 140, 0.25)),
        TerminalType::PostBinaryPredicatePhraseModifier(_)
        | TerminalType::PostArgumentPhraseModifier(_) => Some(Color::from_rgba8(60, 60, 180, 0.25)),
        TerminalType::ConnectiveHead(_) => Some(Color::from_rgba8(220, 140, 40, 0.25)),
        TerminalType::Eos(_) => None,
    }
}

/// 等幅フォントの実際の文字幅を計測する(`grapheme_position`はスペースをカウントしないためズレる)。
fn measure_char_width(font_size: f32) -> f32 {
    ConcreteParagraph::with_text(adv_text::Text {
        content: "A",
        bounds: Size::INFINITE,
        size: Pixels(font_size),
        line_height: adv_text::LineHeight::Relative(LINE_HEIGHT_RATIO),
        font: Font::MONOSPACE,
        align_x: adv_text::Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    })
    .min_bounds()
    .width
}

pub(crate) struct DiagnosticOverlay {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) tokens: Vec<TerminalType>,
    pub(crate) lines: Vec<String>,
    pub(crate) font_size: f32,
    pub(crate) line_height: f32,
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
        let char_width = measure_char_width(self.font_size);

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

            let ranges: &[(usize, usize, usize)] = if start.line() == end.line() {
                &[(start.line(), start.character(), end.character())]
            } else {
                &[
                    (start.line(), start.character(), usize::MAX),
                    (end.line(), 0, end.character()),
                ]
            };

            for &(line_idx, col_start, col_end) in ranges {
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
