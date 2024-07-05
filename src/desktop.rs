use iced::{
    theme,
    widget::{column, container, text},
    window::{self, settings::PlatformSpecific, Level, Position},
    Command, Length, Point, Size,
};

use crate::{config::ConfigDesktopLyricLine, util, Message, SilkPlayer, View};

impl SilkPlayer {
    pub fn is_focuse_desktop_lyric(&mut self, id: window::Id) {
        self.app_control.foucus_desktop_lyric = Some(id) == self.app_control.desktop_lyric_win_id;
    }

    pub fn try_close_desktop_lyric(&mut self) -> Command<Message> {
        if let Some(id) = self.app_control.desktop_lyric_win_id {
            self.app_control.desktop_lyric_win_id = None;
            window::close(id)
        } else {
            Command::none()
        }
    }

    pub fn move_desktop_lyric(&self, _position: iced::Point) -> Command<Message> {
        if self.app_control.foucus_desktop_lyric && self.app_control.press_left_mouse_key {
            if let Some(id) = self.app_control.desktop_lyric_win_id {
                return window::drag(id);
            }
        }
        Command::none()
    }

    pub fn change_desktop_lyric_decorations(&self) -> Command<Message> {
        if self.app_control.foucus_desktop_lyric {
            if let Some(id) = self.app_control.desktop_lyric_win_id {
                return window::toggle_decorations(id);
            }
        }
        Command::none()
    }

    // pub fn get_desktop_window_position(&self) -> Command<Message> {
    //     // if let Some(id) = self.app_control.desktop_lyric_win_id {
    //     //     return window::fetch_size(id, |size| {
    //     //         util::log_debug(format!("{size:?}"));
    //     //         Message::PlayOrPause
    //     //     });
    //     // }
    //     // Command::none()
    //     return window::fetch_size(window::Id::MAIN, |size| {
    //         util::log_debug(format!("{size:?}"));
    //         Message::PlayOrPause
    //     });
    // }

    pub fn new_destop_window(&mut self) -> Command<Message> {
        if let Some(_) = self.app_control.desktop_lyric_win_id {
            return self.try_close_desktop_lyric();
        }

        let position = if self.setting.desktop_lyric.x > 0.0 && self.setting.desktop_lyric.x > 0.0 {
            Position::Specific(Point::new(
                self.setting.desktop_lyric.x,
                self.setting.desktop_lyric.y,
            ))
        } else {
            Position::Centered
        };

        let (id, w) = window::spawn(window::Settings {
            size: Size::new(
                self.setting.desktop_lyric.width,
                self.setting.desktop_lyric.height,
            ),
            position,
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: false, // 传统窗口栏
            transparent: true,  // 透明窗口
            level: Level::AlwaysOnTop,
            icon: util::app_icon(),
            exit_on_close_request: true,
            platform_specific: PlatformSpecific {
                parent: None,
                drag_and_drop: true,
                skip_taskbar: true,
            },
            ..Default::default()
        });

        self.app_control.desktop_lyric_win_id = Some(id);

        return w;
    }

    pub fn desktop_lyric_view(&self, _window: iced::window::Id) -> View<'static> {
        let mut lyrics = vec![];
        // first
        let mut step = 0;
        let mut vec = self.current_song.lyric.to_vec();
        vec.reverse();
        for show_lyric in vec {
            let current_index = self.app_control.current_lyric_index as i32 + step;
            if current_index == show_lyric.index as i32 {
                if show_lyric.lyric.is_empty() {
                    step -= 1;
                    continue;
                }
                lyrics.push(show_lyric.lyric.to_string());
                break;
            }
        }

        // next
        if self.setting.desktop_lyric.line == ConfigDesktopLyricLine::Two {
            let mut step = 1;
            for show_lyric in self.current_song.lyric.iter() {
                if self.app_control.current_lyric_index + step == show_lyric.index {
                    if show_lyric.lyric.is_empty() {
                        step += 1;
                        continue;
                    }
                    lyrics.push(show_lyric.lyric.to_string());
                    break;
                }
            }
        }

        let mut list = column!()
            .width(Length::Fill)
            .padding(10)
            .align_items(iced::Alignment::Center);
        if lyrics.is_empty() {
            // 无歌词时显示歌曲名
            let var_name = format!("{}-{}", self.current_song.artist, self.current_song.title);
            list = list.push(text(var_name));
        }

        for lyric in lyrics {
            list = list.push(
                text(lyric).size(self.setting.desktop_lyric.font_size.parse::<u16>().unwrap()),
            );
        }
        // let list = column!("fasdfafds");

        container(list)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Transparent)
            .into()
    }
}
