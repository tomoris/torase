//! ツリービューア共通のレイアウトヘルパー(カテゴリ枠・スペーサー・区切り付き横並び)。

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element};

use crate::app::Message;

pub(crate) fn category_box(
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

fn spacer() -> Element<'static, Message> {
    Space::new().width(iced::Length::Fixed(10.0)).into()
}

/// 要素間に `spacer()` を挟んで横に並べ、中央揃えする。
pub(crate) fn row_with_separator(
    elements: Vec<Element<'static, Message>>,
) -> Element<'static, Message> {
    let mut r = row![];
    for (i, element) in elements.into_iter().enumerate() {
        if i > 0 {
            r = r.push(spacer());
        }
        r = r.push(element);
    }
    r.align_y(Alignment::Center).into()
}
