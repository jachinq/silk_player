use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use iced::{
    theme, widget::{
        button, column, container, row, scrollable, text, text_input, Column, Image, MouseArea, Scrollable, Slider, Text
    }, Alignment, Command, Length
};
// use iced_aw::FloatingElement;

use crate::{
    components::{self, button_icon, tooltip_text},
    config::ConfigMessage,
    style::{self, ButtonType},
    util, Message, MusicInfo, ShowLyric, SilkPlayer, SongControl, Status, Tab, View,
    LYRIC_SCROLLABLE_ID, PLAY_LIST_SCROLLABLE_ID,
};

const BOTTOM_STATUS_HEIGHT: f32 = 120.;

#[derive(Debug, Default, PartialEq)]
pub enum InitState {
    #[default]
    Loading,
    // WaitRefresh,
    InitDone,
}

pub struct PageInfo {
    pub all_list: Arc<Mutex<Vec<MusicInfo>>>, // 保存全部的数据
    pub filter_list: Vec<MusicInfo>,          // 保存筛选数据
    pub page_list: Vec<MusicInfo>,            // 保存展示数据
    pub current: usize,
    pub total: usize,
    pub total_page: usize,
    pub size: usize, // 筛选字段
    pub search: String,
    pub init_state: Arc<Mutex<InitState>>,
}
impl Default for PageInfo {
    fn default() -> Self {
        Self {
            all_list: Default::default(),
            filter_list: Default::default(),
            page_list: Default::default(),
            current: 1,
            total: Default::default(),
            total_page: Default::default(),
            size: 10,
            search: Default::default(),
            init_state: Default::default(),
        }
    }
}
impl PageInfo {
    pub fn clear(&mut self) {
        *self = Self {
            size: self.size,
            search: self.search.to_string(),
            ..PageInfo::default()
        };
    }

    // pub fn check_init_done(&mut self) -> bool {
    //     if let Ok(mut state) = self.init_state.try_lock() {
    //         if *state == InitState::WaitRefresh {
    //             *state = InitState::InitDone;
    //             util::log("init local file done.");
    //             return true;
    //         }
    //         util::log("loading local file...")
    //     }
    //     return false;
    // }

    pub fn init_list(&mut self, list: Vec<MusicInfo>) {
        if let Ok(mut all_list) = self.all_list.try_lock() {
            *all_list = list;
        }
        self.filter();
        self.page();
    }

    pub fn init_monitor(&mut self, monitor: &str) {
        use std::thread;
        use std::time::Duration;

        let init_state = self.init_state.clone();
        let all_list: Arc<Mutex<Vec<MusicInfo>>> = self.all_list.clone();
        let monitor = monitor.to_string();
        let _ = thread::spawn(move || {
            let mut file_list = vec![];
            if let Ok(_) = util::get_files(&monitor, &mut file_list) {
                let file_len = file_list.len();
                // 用多线程来处理，加快初始化速度
                let task_num = if let Ok(task_num) = std::thread::available_parallelism() {
                    task_num.try_into().unwrap_or(1)
                } else {
                    1
                };
                util::log(format!("local file len={};task_num={}", file_len, task_num));

                let batch_list = util::batch_list(&file_list, task_num);

                let counter = all_list.clone();
                for index in 0..task_num {
                    let counter = Arc::clone(&counter);
                    let task = batch_list[index].clone();
                    let _ = thread::spawn(move || {
                        // for path in task {
                        for path in task {
                            let music_info = MusicInfo::new(&path);
                            counter.lock().unwrap().push(music_info);
                        }
                    });
                }

                {
                    let _ = std::thread::spawn(move || loop {
                        thread::sleep(Duration::from_secs_f32(1.5));
                        if let Ok(mut list) = Arc::clone(&counter).lock() {
                            util::log(format!("loading len={}", list.len()));
                            if list.len() == file_len {
                                util::log(format!("final len={}", list.len()));
                                list.sort_by(|a, b| a.title.cmp(&b.title));
                                if let Ok(mut init_state) = init_state.try_lock() {
                                    *init_state = InitState::InitDone;
                                    break;
                                }
                            }
                        }
                    });
                }
            }
        });
    }

    pub fn page(&mut self) {
        if self.total <= 0 {
            self.page_list = vec![];
            return;
        }

        let beg = (self.current - 1) * self.size;
        if beg > self.total {
            return;
        }

        let mut end = beg + self.size;
        if self.total < end {
            end = self.total;
        }

        let vec = self.filter_list[beg..end].to_vec();
        self.page_list = vec;
    }

    pub fn change_page(&mut self, is_ctrl: bool, page: usize) {
        let factor = if is_ctrl { 5 } else { 1 };
        match page {
            0 => {
                if self.current <= factor {
                    return;
                }
                self.current -= factor;
            }
            1 => {
                if self.current + factor > self.total_page {
                    return;
                }
                self.current += factor;
            }
            value => {
                self.current = value;
            }
        }
        self.page();
    }

    pub fn filter(&mut self) {
        let all_list_len = if let Ok(all_list) = self.all_list.try_lock() {
            all_list.len()
        } else {
            0
        };

        self.total = all_list_len;
        if self.total <= 0 {
            // self.clear(); // todo
            return;
        }

        self.filter_list = if self.search.is_empty() {
            if let Ok(all_list) = self.all_list.try_lock() {
                all_list.clone()
            } else {
                vec![]
            }
        } else {
            let search = self.search.to_lowercase();
            let mut result = vec![];
            if let Ok(all_list) = self.all_list.try_lock() {
                for item in all_list.iter() {
                    if item.title.to_lowercase().contains(&search) {
                        result.push(item.clone());
                        continue;
                    }
                    if item.artist.to_lowercase().contains(&search) {
                        result.push(item.clone());
                        continue;
                    }
                }
            }
            self.current = 1;
            result
        };

        // util::log_debug(format!("s = {} f = {}", self.search, self.filter_list.len()));
        self.total = self.filter_list.len();
        let total_page = self.total as f64 / self.size as f64;
        self.total_page = total_page.ceil() as usize;
        self.page();
    }

    pub fn pos_current_song(&mut self, music_info: MusicInfo) -> usize {
        let mut index = 0;
        if let Ok(all_list) = self.all_list.try_lock() {
            for (i, item) in all_list.iter().enumerate() {
                if item.path == music_info.path {
                    index = i;
                    break;
                }
                if i == all_list.len() - 1 {
                    // self.page();
                    return 0;
                }
            }
        } else {
            return 0;
        }

        // index = 187 size = 10; page = ?
        // index = 2 size = 10; page = ?
        let page = if index == 0 {
            1
        } else {
            index / self.size + if index % self.size > 0 { 1 } else { 0 }
        };
        // util::log_debug(format!("page={}", page));
        self.current = page;
        self.page();
        index % self.size
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum DetailTab {
    #[default]
    Lyric,
    Info,
}

impl SilkPlayer {
    pub fn change_status(&mut self) -> Command<Message> {
        match self.status {
            Status::Tab => {
                self.status = Status::PlayDetial;
                self.app_control.hide_status = false;
                self.app_control.hide_status_seconds = Some(Instant::now());

                self.app_control.scroll_seconds = None;
                return self.scroll_lyric();
            }
            Status::PlayDetial => {
                self.status = Status::Tab;
            }
        }
        Command::none()
    }

    /// 顶部状态条
    pub fn top_status_view(&self) -> View {
        let page_control = self.page_control();

        let logo = MouseArea::new(Image::new(format!("{}/assets/icon.ico", util::current_dir())).width(32.)).on_press(Message::MoveWindow(true)).on_release(Message::MoveWindow(false));

        // let logo =
            // button(Image::new(format!("{}/assets/icon.ico", util::current_dir())).width(32.)).on_press(Message::MoveWindow(window::Id::MAIN));

        let home = row!(logo, button(text("Silk 播放器")).on_press(Message::ChangeTab(Tab::Home))
            .style(theme::Button::Custom(Box::new(
                style::ButtonType::Text.default(),
            )))).spacing(5)
            .align_items(Alignment::Center)
            ;

        let icon_size = 18.0;

        let mut control = row!(home)
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(20)
            .padding(style::padding(15.0, 10.0, 0.0, 10.0));

        match self.tab {
            Tab::Home => {
                let play_all = button(style::icon("play_all", icon_size))
                    .style(theme::Button::Custom(Box::new(
                        style::ButtonType::Primary.default(),
                    )))
                    .on_press(Message::SongControl(SongControl::PlayAll));
                let play_all = components::tooltip_text(
                    play_all,
                    "播放全部",
                    iced::widget::tooltip::Position::Bottom,
                );

                let search = text_input("搜索", &self.music_list.search)
                    .on_input(|value| Message::Filter(value))
                    .width(Length::Fixed(250.));

                let setting = tooltip_text(
                    button_icon(
                        "settings",
                        icon_size,
                        Message::ChangeTab(Tab::Option),
                        style::ButtonType::Info.default(),
                    ),
                    "设置",
                    iced::widget::tooltip::Position::Bottom,
                );

                let all_list_empty = if let Ok(all_list) = self.music_list.all_list.try_lock() {
                    all_list.is_empty()
                } else {
                    false
                };
                let play_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    all_list.is_empty()
                } else {
                    false
                };

                let list: View = if !all_list_empty && play_list_empty {
                    tooltip_text(
                        button(text("go!").size(14.5))
                            .on_press(Message::SongControl(SongControl::RandomSelect))
                            .style(theme::Button::Custom(Box::new(
                                ButtonType::Primary.default(),
                            ))),
                        "随机听歌",
                        iced::widget::tooltip::Position::Bottom,
                    )
                    .into()
                } else {
                    tooltip_text(
                        button_icon(
                            "play_list",
                            icon_size,
                            Message::ChangeTab(Tab::List),
                            style::ButtonType::Info.default(),
                        ),
                        "播放列表",
                        iced::widget::tooltip::Position::Bottom,
                    )
                    .into()
                };

                let like: View = tooltip_text(
                    button_icon(
                        "like",
                        icon_size,
                        Message::ChangeTab(Tab::Like),
                        style::ButtonType::Info.default(),
                    ),
                    "歌单",
                    iced::widget::tooltip::Position::Bottom,
                )
                .into();

                control = control.push(search);
                control = control.push(page_control);
                control = control.push(play_all);
                control = control.push(list);
                control = control.push(like);
                control = control.push(setting);
            }
            Tab::Fave => {}
            Tab::List => {
                let remove_all = tooltip_text(
                    button_icon(
                        "remove_all",
                        icon_size,
                        Message::SongControl(SongControl::PlayClear),
                        style::ButtonType::Primary.default(),
                    ),
                    "清空播放列表",
                    iced::widget::tooltip::Position::Bottom,
                );

                let pos_current_song = tooltip_text(
                    button_icon(
                        "focus",
                        icon_size,
                        Message::SongControl(SongControl::SnapToCurrentSong),
                        style::ButtonType::Primary.default(),
                    ),
                    "定位当前播放",
                    iced::widget::tooltip::Position::Bottom,
                );
                control = control.push(remove_all);
                control = control.push(pos_current_song);
                control = control.push(page_control);
            }
            Tab::Like | Tab::LikeDetail => {
                let play_all = button(style::icon("play_all", icon_size))
                    .style(theme::Button::Custom(Box::new(
                        style::ButtonType::Primary.default(),
                    )))
                    .on_press(Message::SongControl(SongControl::PlayAll));
                let play_all = components::tooltip_text(
                    play_all,
                    "播放全部",
                    iced::widget::tooltip::Position::Bottom,
                );

                let play_list: View = tooltip_text(
                    button_icon(
                        "play_list",
                        icon_size,
                        Message::ChangeTab(Tab::List),
                        style::ButtonType::Info.default(),
                    ),
                    "播放列表",
                    iced::widget::tooltip::Position::Bottom,
                )
                .into();

                let like: View = tooltip_text(
                    button_icon(
                        "like",
                        icon_size,
                        Message::ChangeTab(Tab::Like),
                        style::ButtonType::Info.default(),
                    ),
                    "歌单",
                    iced::widget::tooltip::Position::Bottom,
                )
                .into();

                control = control.push(play_all);
                control = control.push(play_list);
                control = control.push(like);
            }
            Tab::Option => {}
        }

        control.into()
    }

    /// 底部播放工具条
    pub fn bottom_status_view(&self) -> View {
        if self.current_song.is_none() {
            return column!().into();
        }

        let left_time = Text::new(util::play_time(self.audio.position()));
        let right_time = Text::new(util::play_time(self.current_song.time));
        let progess_slider = Slider::new(
            0.0..=self.audio.duration(),
            self.app_control.current_duration,
            Message::UpdateSongTime,
        )
        .on_release(Message::UpdateSongTimeRelease)
        .style(theme::Slider::Custom(Box::new(style::SliderStyle(false))));

        let volume_icon = style::icon("volume", 20.);
        let volume_slider = Slider::new(0.0..=1.0, self.audio.volume(), |value| {
            Message::ChangeConfig(ConfigMessage::ChangeVolume(value))
        })
        .style(theme::Slider::Custom(Box::new(style::SliderStyle(true))))
        .step(0.01)
        .width(Length::Fixed(100.));
        let volume_setting = row!(volume_icon, volume_slider)
            .spacing(2)
            .padding(style::padding_right(20.));

        let pre = button_icon(
            "back",
            20.,
            Message::SongControl(SongControl::PlayNext(false)),
            style::ButtonType::Primary.cycle(),
        );

        let play = button_icon(
            if self.audio.is_play() {
                "pause"
            } else {
                "play"
            },
            30.,
            Message::SongControl(SongControl::PlayOrPause),
            style::ButtonType::Primary.cycle(),
        );

        let next = button_icon(
            "go",
            20.,
            Message::SongControl(SongControl::PlayNext(true)),
            style::ButtonType::Primary.cycle(),
        );

        let play_mode = tooltip_text(
            button_icon(
                &self.setting.play_mode.icon(),
                20.,
                Message::ChangeConfig(crate::config::ConfigMessage::ChangePlayMode),
                ButtonType::Primary.default(),
            ),
            &self.setting.play_mode.name(),
            iced::widget::tooltip::Position::Top,
        );
        let desktop_lyric = tooltip_text(
            button_icon(
                "destop_lyric",
                20.,
                Message::DesktopLyricWindow,
                ButtonType::Primary.default(),
            ),
            "桌面歌词",
            iced::widget::tooltip::Position::Top,
        );

        match self.status {
            Status::Tab => {
                let tb_album = if self.app_control.hide_status {
                    format!("{}/assets/default.png", util::current_dir())
                } else {
                    util::get_thumbnail_path(&self.current_song.album_path)
                };

                let album = button(
                    Image::new(tb_album)
                        .height(BOTTOM_STATUS_HEIGHT - 20.)
                        .width(BOTTOM_STATUS_HEIGHT - 20.),
                )
                .on_press(Message::PlayDetail);

                let progess = row!(left_time, progess_slider, right_time)
                    .align_items(Alignment::Center)
                    .spacing(2);
                let control = row!(pre, play, next, progess)
                    .spacing(5)
                    .align_items(Alignment::Center);
                let title = column!(Text::new(&self.current_song.title),);
                let middle = column!(title, control).spacing(5);

                let mut container = container(
                    row!(album, middle, play_mode, desktop_lyric, volume_setting)
                        .width(Length::Fill)
                        .spacing(20)
                        .align_items(Alignment::Center),
                );

                if self.current_song.album_color.is_empty() || self.status == Status::Tab {
                    container = container.style(theme::Container::Custom(Box::new(
                        style::ContainerStyle::BackgroundWithAlpha(0.7),
                    )));
                } else {
                    container = container.style(theme::Container::Custom(Box::new(
                        style::ContainerStyle::Gradient {
                            time: self.audio.position() + 15.705,
                            colors: self.current_song.album_color.clone(),
                        },
                    )));
                }

                container.into()
            }
            Status::PlayDetial => {
                let time = row!(left_time, "/", right_time)
                    .align_items(Alignment::Center)
                    .spacing(2);

                let unfold = button_icon(
                    "unfold",
                    20.,
                    Message::PlayDetail,
                    style::ButtonType::Primary.cycle(),
                );

                let control = row!(
                    unfold,
                    time,
                    pre,
                    play,
                    next,
                    play_mode,
                    desktop_lyric,
                    volume_setting
                )
                .spacing(15)
                .align_items(Alignment::Center);

                let mut bottom_status = container(
                    column!(progess_slider, control)
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                )
                .height(BOTTOM_STATUS_HEIGHT - 20.)
                .width(Length::Fill)
                .center_y();

                if self.current_song.album_color.is_empty() || self.status == Status::Tab {
                    bottom_status = bottom_status.style(theme::Container::Custom(Box::new(
                        style::ContainerStyle::BackgroundWithAlpha(0.7),
                    )));
                } else {
                    bottom_status = bottom_status.style(theme::Container::Custom(Box::new(
                        style::ContainerStyle::Gradient {
                            time: self.audio.position() + 15.705,
                            colors: self.current_song.album_color.clone(),
                        },
                    )));
                }

                bottom_status.into()
            }
        }
    }

    pub fn tab_container(&self) -> View {
        let tab_container = match self.tab {
            Tab::Home => self.home_view(),
            Tab::Fave => Text::new("Fave").into(),
            Tab::List => self.list_view(),
            Tab::Like => self.like_view(),
            Tab::LikeDetail => self.like_detail_view(),
            Tab::Option => self.option_view(),
        };

        // let padding = if self.current_song.is_none() {
        //     Padding::ZERO
        // } else {
        //     ui::padding_bottom(BOTTOM_STATUS_HEIGHT)
        // };
        container(tab_container)
            .padding(style::padding_bottom(10.))
            .width(Length::Fill)
            .into()
    }

    fn pack_album(&self, music_info: &MusicInfo) -> View {
        let album_size = 64.0;
        let thumbnail_path = util::get_thumbnail_path(&music_info.album_path);
        Image::new(&thumbnail_path)
            .height(album_size)
            .width(album_size)
            .into()
    }
    fn pack_music_info_list(&self, music_info: &MusicInfo, with_album: bool) -> View {
        let content = if with_album {
            row![
                self.pack_album(music_info),
                self.show_name(false, music_info)
            ]
            .spacing(20)
            .align_items(Alignment::Center)
            .into()
        } else {
            self.show_name(false, music_info)
        };

        // row
        button(content)
            // .width(Length::Fill)
            .on_press(Message::SongControl(SongControl::First(music_info.clone())))
            .style(theme::Button::Custom(Box::new(
                style::ButtonType::Text.default(),
            )))
            .into()
    }

    pub fn home_view(&self) -> View {
        let mut list = column!().spacing(15);
        for music_info in &self.music_list.page_list {
            list = list.push(self.pack_music_info_list(music_info, true));
        }

        container(Scrollable::new(list).width(Length::Fill))
            .padding(style::padding_left(50.0))
            .into()
    }

    pub fn list_view(&self) -> View {
        let all_list_empty = if let Ok(all_list) = self.music_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            false
        };
        let play_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            false
        };

        if all_list_empty {
            container("播放列表为空，先去设置一下本地路径吧~")
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else if play_list_empty {
            container(
                column!(
                    "播放列表为空，开启随机听歌？",
                    button("go!").on_press(Message::SongControl(SongControl::RandomSelect))
                )
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            let mut list = column!().spacing(15);
            for music_info in &self.play_list.page_list {
                list = list.push(self.pack_music_info_list(music_info, false));
            }

            container(
                Scrollable::new(list)
                    .width(Length::Fill)
                    .id(PLAY_LIST_SCROLLABLE_ID.clone()),
            )
            .padding(style::padding_left(50.0))
            .into()
        }
    }

    pub fn like_view(&self) -> View {
        let mut tags = vec![];
        let mut tag_names = vec![];
        if let Ok(all_list) = self.music_list.all_list.try_lock() {
            for music_info in all_list.iter() {
                for tag in music_info.tags.iter() {
                    if tag_names.contains(&tag.path) {
                        continue;
                    }
                    tag_names.push(tag.path.clone());
                    if !tags.contains(tag) {
                        tags.push(tag.clone());
                    }
                }
            }
        }

        // let icon_size = 18.0;
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        let mut list = column!().spacing(15);
        for tag in tags {
            let show_name = column!(text(&tag.name).size(22)).spacing(5);

            list = list.push(button(show_name).on_press(Message::ChangeTag(tag)).style(
                theme::Button::Custom(Box::new(style::ButtonType::Text.default())),
            ));
        }
        container(
            Scrollable::new(list)
                .width(Length::Fill)
                .id(PLAY_LIST_SCROLLABLE_ID.clone()),
        )
        .padding(style::padding_left(50.0))
        .into()
    }

    pub fn get_list_by_tag(&self) -> Vec<MusicInfo> {
        let mut list = vec![];
        // let mut list = column!(detail).spacing(15);
        if let Ok(all_list) = self.music_list.all_list.try_lock() {
            for music_info in all_list.iter() {
                let tag_names: Vec<_> = music_info
                    .tags
                    .iter()
                    .filter(|item| item.name.eq(&self.tag.name))
                    .collect();
                if tag_names.is_empty() {
                    continue;
                }
                list.push(music_info.clone());
            }
        }
        list
    }

    pub fn like_detail_view(&self) -> View {
        let list = self.get_list_by_tag();
        let info = column!(text(&self.tag.name), text(&self.tag.path).size(16)).spacing(15);
        let detail: View = if !list.is_empty() {
            row!(self.pack_album(&list[0]), info).spacing(15).into()
        } else {
            info.into()
        };

        let mut show_list = column!(detail).spacing(15);
        for music_info in list {
            show_list = show_list.push(self.pack_music_info_list(&music_info, false));
        }

        container(Scrollable::new(show_list).width(Length::Fill))
            .padding(style::padding_left(50.0))
            .into()
    }

    fn show_name(&self, is_play_list: bool, music_info: &MusicInfo) -> View {
        let play_icon = if self.current_song.path == music_info.path && self.audio.is_play() {
            "pause"
        } else {
            "play"
        };

        let icon_size = 14.;

        let play_btn = button_icon(
            play_icon,
            icon_size,
            if is_play_list {
                Message::SongControl(SongControl::List(music_info.clone()))
            } else {
                Message::SongControl(SongControl::First(music_info.clone()))
            },
            style::ButtonType::Primary.cycle(),
        );

        let list_btn = if is_play_list {
            button_icon(
                "close",
                icon_size,
                Message::SongControl(SongControl::Remove(music_info.clone())),
                style::ButtonType::Info.default(),
            )
        } else {
            button_icon(
                "plus",
                icon_size,
                Message::SongControl(SongControl::Last(music_info.clone())),
                style::ButtonType::Info.default(),
            )
        };

        let title = util::get_title(music_info);

        column!(
            text(title).size(22),
            row![play_btn, list_btn, text(&music_info.artist).size(16),]
                .align_items(Alignment::Center)
                .spacing(5),
        )
        .spacing(5)
        .into()
    }

    fn page_control(&self) -> View {
        let is_play_list = match self.tab {
            Tab::List => true,
            _ => false,
        };
        let prev_page = button_icon(
            "back",
            18.,
            Message::ChangPage {
                page: 0,
                is_play_list,
            },
            style::ButtonType::Primary.default(),
        );

        let next_page = button_icon(
            "go",
            18.,
            Message::ChangPage {
                page: 1,
                is_play_list,
            },
            style::ButtonType::Primary.default(),
        );

        let page_info = if is_play_list {
            &self.play_list
        } else {
            &self.music_list
        };
        let current_page = container(
            column![
                text(format!(
                    "{:0>2}/{:0>2}",
                    page_info.current, page_info.total_page
                ))
                .size(14),
                text(format!("共 {} 条", page_info.total)).size(14),
            ]
            .align_items(Alignment::Center),
        )
        .padding([0, 3]);

        row!(prev_page, current_page, next_page)
            .spacing(5)
            .align_items(Alignment::Center)
            .into()
    }

    pub fn playing_detail(&self) -> View {
        let img_width = self.setting.windows.width / 3.5;
        let img_height = self.setting.windows.width / 3.5;
        let img_size = img_height.min(img_width);

        let img = self.current_song.album_path.clone();
        let img = Image::new(img)
            .height(img_size)
            .width(img_size)
            .content_fit(iced::ContentFit::Cover);
        let img = container(img)
            .padding(2)
            .style(theme::Container::Custom(Box::new(
                style::ContainerStyle::Border(2.0),
            )));

        let left_album = container(img)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .center_y()
            .center_x();

        let title = util::get_title(&self.current_song);
        let title = Text::new(title).size(26);
        let control = row!(
            button(text("歌词").size(14))
                .on_press(Message::ChangeDetail(DetailTab::Lyric))
                .style(theme::Button::Custom(Box::new(ButtonType::Info.default()))),
            button(text("信息").size(14))
                .on_press(Message::ChangeDetail(DetailTab::Info))
                .style(theme::Button::Custom(Box::new(ButtonType::Info.default()))),
        )
        .padding(10);
        let detail = match self.detail_tab {
            DetailTab::Lyric => self.detail_lyrics(&self.current_song.lyric),
            DetailTab::Info => self.detail_info(),
        };
        let detail = container(detail)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill);

        let right_info = column!(title, control, detail,)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .align_items(Alignment::Center);

        let mut container = container(row!(left_album, right_info).spacing(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(style::padding(50., 20., 50., 0.))
            // .center_x()
            // .center_y()
            ;

        if self.current_song.album_color.is_empty() {
            container = container.style(theme::Container::Custom(Box::new(
                style::ContainerStyle::BackgroundWithAlpha(1.0),
            )));
        } else {
            container = container.style(theme::Container::Custom(Box::new(
                style::ContainerStyle::Gradient {
                    time: self.audio.position(),
                    colors: self.current_song.album_color.clone(),
                },
            )));
        }

        // let background = util::get_blur_path(&self.current_song.album);
        // let background = ui::background_image(background);
        container.into()

        // FloatingElement::new(
        //     container,
        //     background,
        // )
        //     .offset(0.)
        //     .into()
    }

    fn detail_info(&self) -> View {
        let text_size = 18.0;
        let label_width = Length::Fixed(text_size * 4.0);
        let gap = 5;
        let mut colors = row![text("专辑颜色").width(label_width).size(text_size)]
            .spacing(gap)
            .align_items(Alignment::Center);
        for (_i, color) in self.current_song.album_color.iter().enumerate() {
            colors = colors.push(container("").width(25).height(25).style(
                theme::Container::Custom(Box::new(style::ContainerStyle::ExtraColor(*color))),
            ));
        }

        let album_name = row![
            text("专辑").width(label_width).size(text_size),
            text(&self.current_song.album).size(text_size),
        ]
        .spacing(gap);
        let date = row![
            text("发行时间").width(label_width).size(text_size),
            text(&self.current_song.year).size(text_size),
        ]
        .spacing(gap);
        let artist = row![
            text("歌手").width(label_width).size(text_size),
            text(&self.current_song.artist).size(text_size),
        ]
        .spacing(gap);
        let file_fmt = row![
            text("文件类型").width(label_width).size(text_size),
            text(&self.current_song.fmt.to_string()).size(text_size),
        ]
        .spacing(gap);
        let file_path = row![
            text("文件路径").width(label_width).size(text_size),
            button(text(&self.current_song.path).size(text_size))
                .on_press(Message::OpenWith(
                    true,
                    self.current_song.path.to_string(),
                    "".to_string()
                ))
                .style(theme::Button::Custom(Box::new(ButtonType::Text.default()))),
        ]
        .align_items(Alignment::Center)
        .spacing(gap);

        let info = column!(artist, album_name, colors, date, file_fmt, file_path).spacing(15);

        container(info)
            .padding(30)
            .style(theme::Container::Custom(Box::new(
                style::ContainerStyle::BackgroundWithAlpha(0.2),
            )))
            .into()
    }

    fn detail_lyrics(&self, lyrics: &Vec<ShowLyric>) -> View {
        let mut col = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center);

        for _ in 0..10 {
            col = col.push(text(""));
        }
        for show_lyric in lyrics.iter() {
            let mut text = Text::new(show_lyric.lyric.clone()).shaping(text::Shaping::Advanced);
            if show_lyric.index == self.app_control.current_lyric_index {
                text = text
                    .size(32)
                    .line_height(2.)
                    .style(self.setting.get_theme().palette().primary)
                // .shaping(text::Shaping::Advanced);
            } else {
                // let color = self.setting.get_theme().palette().text;
                // let color = if self.setting.get_theme().extended_palette().is_dark {
                //     let f = 1.2;
                //     Color::from_rgb(color.r / f, color.g / f, color.b / f)
                // } else {
                //     let f = 3.0;
                //     Color::from_rgb(color.r * f, color.g * f, color.b * f)
                // };
                text = text.size(24) //.style(color);
            }
            col = col.push(text);
        }

        if lyrics.is_empty() {
            col = col.push("暂无歌词");
        }
        for _ in 0..10 {
            col = col.push(text(""));
        }

        let content = Scrollable::new(col)
            .on_scroll(Message::ScrollLyric)
            .style(theme::Scrollable::Custom(Box::new(
                style::StyledScrolloableHide,
            )))
            .id(LYRIC_SCROLLABLE_ID.clone());

        container(content).padding(10).into()
    }

    pub fn scroll_lyric(&self) -> Command<Message> {
        if self.audio.is_play()
            && self.current_song.lyric.len() > 0
            && self.app_control.scroll_seconds.is_none()
        {
            // util::log_time(format!("snappp {}", self.current_time, ));

            // let y = self.slider_value / self.current_song.time;
            let y =
                self.app_control.current_lyric_index as f32 / self.current_song.lyric.len() as f32;
            scrollable::snap_to(
                LYRIC_SCROLLABLE_ID.clone(),
                scrollable::RelativeOffset { x: 0., y },
            )
        } else {
            Command::none()
        }
    }

    pub fn exit_view(&self) -> View {
        let content = column!(
            text("确定退出应用吗？"),
            text("确定/Enter退出，取消/esc 返回应用").size(20),
            row!(
                button("确定")
                    .padding([10, 20])
                    .on_press(Message::ConfirmExit),
                button("取消")
                    .padding([10, 20])
                    .on_press(Message::ToggleEsc)
            )
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .spacing(10)
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .style(theme::Container::Custom(Box::new(
                style::ContainerStyle::BackgroundWithAlpha(0.5),
            )))
            .into()
    }
}
