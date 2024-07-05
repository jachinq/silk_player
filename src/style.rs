#![allow(unused)]

use iced::{
    advanced::{graphics::futures::backend::default, widget::operation::Scrollable},
    application,
    border::Radius,
    gradient::{ColorStop, Linear},
    theme::{self, Text},
    widget::{
        self, button,
        container::{self, StyleSheet},
        image::{self, Handle},
        scrollable::{self, Scroller},
        slider, tooltip, Image,
    },
    Background, Border, Color, ContentFit, Length, Padding, Pixels, Radians, Shadow, Theme, Vector,
};

use crate::{util, UPDATE_TIME};

pub fn background_image<Handle>(handle: impl Into<Handle>) -> Image<Handle> {
    Image::new(handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(ContentFit::Cover)
}

pub fn icon(icon: &str, width: f32) -> Image<Handle> {
    Image::new(format!("{}/assets/icon/{}.png", util::current_dir(), icon)).width(width)
}

pub fn transparent() -> Color {
    Color::from_rgba8(0, 0, 0, 0.)
}

pub fn padding(top: f32, right: f32, bottom: f32, left: f32) -> Padding {
    Padding {
        top,
        right,
        bottom,
        left,
    }
}
pub fn padding_top(top: f32) -> Padding {
    padding(top, 0., 0., 0.)
}
pub fn padding_right(right: f32) -> Padding {
    padding(0., right, 0., 0.)
}
pub fn padding_bottom(bottom: f32) -> Padding {
    padding(0., 0., bottom, 0.)
}
pub fn padding_left(left: f32) -> Padding {
    padding(0., 0., 0., left)
}

pub struct SliderStyle(pub bool);

impl slider::StyleSheet for SliderStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            rail: slider::Rail {
                colors: (
                    style.extended_palette().primary.base.color,
                    style.extended_palette().secondary.base.color,
                ),
                width: 5.0,
                border_radius: Radius::from(2.5),
            },
            handle: slider::Handle {
                // 移动点
                shape: slider::HandleShape::Circle { radius: 2.5 },
                color: style.extended_palette().primary.base.color,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 6. },
                color: style.extended_palette().background.strong.color,
                border_width: 1.5,
                border_color: style.extended_palette().primary.strong.color,
            },
            ..Self::active(self, style)
        }
    }

    fn dragging(&self, style: &Self::Style) -> slider::Appearance {
        slider::Appearance {
            ..Self::hovered(self, style)
        }
    }
}

pub struct StyledScrolloableHide;
impl scrollable::StyleSheet for StyledScrolloableHide {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            container: widget::container::Appearance::default(),
            scrollbar: scrollable::Scrollbar {
                background: None,
                border: Border::default(),
                scroller: Scroller {
                    color: Color::default(),
                    border: Border::default(),
                },
            },
            gap: None,
        }
    }

    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> scrollable::Appearance {
        Self::active(self, style)
    }
}

pub enum ButtonType {
    Primary,
    Info,
    Text,
}
impl ButtonType {
    pub fn default(self) -> ButtonStyle {
        ButtonStyle {
            button_type: self,
            ..ButtonStyle::default()
        }
    }
    pub fn cycle(self) -> ButtonStyle {
        ButtonStyle {
            radius: 9999.,
            button_type: self,
            ..ButtonStyle::default()
        }
    }
    pub fn with_radius(self, radius: f32) -> ButtonStyle {
        ButtonStyle {
            radius,
            button_type: self,
            ..ButtonStyle::default()
        }
    }
    pub fn with_opacity(self, opacity: f32) -> ButtonStyle {
        ButtonStyle {
            opacity,
            button_type: self,
            ..ButtonStyle::default()
        }
    }
    pub fn with_radius_opacity(self, radius: f32, opacity: f32) -> ButtonStyle {
        ButtonStyle {
            opacity,
            radius,
            button_type: self,
            ..ButtonStyle::default()
        }
    }
}

pub struct ButtonStyle {
    radius: f32,
    opacity: f32,
    button_type: ButtonType,
}

impl ButtonStyle {
    pub fn default() -> Self {
        Self {
            radius: 4.0,
            opacity: 0.5,
            button_type: ButtonType::Primary,
        }
    }
    fn get_theme_and_opacity<'a>(&'a self, style: &'a Theme) -> (&Theme, f32) {
        let mut opacity = self.opacity;
        if style.extended_palette().is_dark {
            (style, opacity)
        } else {
            (&Theme::Dark, 0.8)
        }
        // match style {
        //     Theme::Light | Theme::SolarizedLight | Theme::GruvboxLight | Theme::TokyoNightLight => {
        //         (&Theme::Dark, 0.8)
        //     }
        //     _ => (style, opacity),
        // }
    }
}
impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let default = button::Appearance::default();

        let (theme, opacity) = self.get_theme_and_opacity(style);
        let extended_palette = theme.extended_palette();

        let mut background = match self.button_type {
            ButtonType::Primary => extended_palette.primary.base.color,
            ButtonType::Info => extended_palette.secondary.base.color,
            ButtonType::Text => style.extended_palette().background.base.color,
        };
        background.a = opacity;

        let border_color = match self.button_type {
            ButtonType::Primary => extended_palette.primary.strong.color,
            ButtonType::Info => extended_palette.secondary.strong.color,
            ButtonType::Text => style.extended_palette().background.base.color,
        };
        let mut text_color = style.palette().text;

        button::Appearance {
            background: Some(background.into()),
            border: Border {
                width: 0.,
                radius: Radius::from(self.radius),
                color: border_color,
            },
            text_color,
            ..default
        }
    }
    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let default = self.active(style);

        let (theme, opacity) = self.get_theme_and_opacity(style);
        let extended_palette = theme.extended_palette();

        let mut background = match self.button_type {
            ButtonType::Primary => extended_palette.primary.strong.color,
            ButtonType::Info => extended_palette.secondary.strong.color,
            ButtonType::Text => style.extended_palette().background.base.color,
        };
        background.a = opacity;

        let mut text_color = style.palette().primary;

        button::Appearance {
            background: Some(background.into()),
            text_color,
            ..default
        }
    }
}

pub enum ContainerStyle {
    Tooltip,
    Border(f32),
    Gradient { time: f32, colors: Vec<Color> },
    ExtraColor(Color),
    BackgroundWithAlpha(f32),
}
impl container::StyleSheet for ContainerStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match self {
            ContainerStyle::Tooltip => self.tooltip(style),
            ContainerStyle::Border(border) => self.border(style, *border),
            ContainerStyle::Gradient { time, colors } => self.gradient(style, *time, colors),
            ContainerStyle::ExtraColor(color) => self.extra_color(style, *color),
            ContainerStyle::BackgroundWithAlpha(a) => {
                let mut color = style.extended_palette().background.base.color;
                color.a = *a;
                self.background_color(color)
            }
        }
    }
}
impl ContainerStyle {
    fn tooltip(&self, style: &Theme) -> container::Appearance {
        container::Appearance {
            background: Some(style.extended_palette().background.weak.color.into()),
            border: Border::with_radius(2),
            ..container::Appearance::default()
        }
    }
    fn border(&self, style: &Theme, width: f32) -> container::Appearance {
        container::Appearance {
            border: Border {
                color: style.extended_palette().secondary.weak.color,
                width,
                radius: Radius::from(8),
            },
            ..container::Appearance::default()
        }
    }

    fn gradient(&self, style: &Theme, time: f32, colors: &Vec<Color>) -> container::Appearance {
        let background = if colors.is_empty() {
            None
        } else {
            if colors.len() > 1 {
                let start = colors[0];
                let end = colors[1];
                // let end = colors[colors.len() - 1];
                let mut liner =
                    iced::gradient::Linear::new(Radians((time / (UPDATE_TIME * 50.)).sin()));

                liner = liner.add_stop(0.0, start);

                // if colors.len() > 2 {
                //     let gap = 1.0 / (colors.len() - 2 + 1) as f32;
                //     for (i, color) in colors.iter().enumerate() {
                //         if i == 0 || i == colors.len() - 1 {
                //             continue;
                //         }
                //         liner = liner.add_stop(i as f32 * gap, *color);
                //     }
                // }
                liner = liner.add_stop(1.0, end);

                let gradient = iced::Gradient::Linear(liner);
                // let gradient = Background::Gradient(gradient);
                Some(Background::Gradient(gradient))
            } else {
                let single = colors[0];
                Some(Background::Color(single))
            }
        };
        container::Appearance {
            // background: Some(gradient),
            background,
            ..container::Appearance::default()
        }
    }

    fn background_color(&self, color: Color) -> container::Appearance {
        container::Appearance {
            background: Some(color.into()),
            ..container::Appearance::default()
        }
    }

    fn extra_color(&self, style: &Theme, color: Color) -> container::Appearance {
        container::Appearance {
            background: Some(color.into()),
            border: Border {
                color: style.palette().text,
                width: 1.0,
                radius: Radius::from(5),
            },
            ..container::Appearance::default()
        }
    }
}

pub struct AppliactionStyle;
impl application::StyleSheet for AppliactionStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }
}
