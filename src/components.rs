use iced::{
    theme,
    widget::{button, text, tooltip, Tooltip},
    Element,
};

use crate::style::{self, ButtonStyle};

/// create a text tooltip
pub fn tooltip_text<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    text_name: &str,
    position: tooltip::Position,
) -> Tooltip<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer + 'a,
    Theme: iced::widget::container::StyleSheet + 'a,
    Theme: iced::widget::text::StyleSheet + 'a,
    <Theme as iced::widget::container::StyleSheet>::Style: From<iced::theme::Container>,
{
    tooltip(content, text(text_name).size(16), position)
        .gap(5)
        .style(theme::Container::Custom(Box::new(
            style::ContainerStyle::Tooltip,
        )))
}

/// create a button with icon
pub fn button_icon<'a, Message, Theme, Renderer>(
    icon: &str,
    icon_width: f32,
    msg: Message,
    style: ButtonStyle,
) -> button::Button<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Renderer: iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
    Theme: button::StyleSheet,
    <Theme as iced::widget::button::StyleSheet>::Style: From<iced::theme::Button>,
{
    // let btn_size = icon_width * 2.0;
    button(style::icon(icon, icon_width))
        // .width(btn_size)
        // .height(btn_size)
        .on_press(msg)
        .style(theme::Button::Custom(Box::new(style)))
}
