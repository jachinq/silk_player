use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use iced::{
    widget::{button, checkbox, column, pick_list, radio, row, text, text_input, Scrollable}, window, Alignment, Command, Length, Theme
};

use crate::{util, Message, PlayMode, SilkPlayer, View};

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    SelectMonitor,
    ChangeMonitor(String),
    SubmitMonitor,
    ChangeTheme(ThemeConfig),
    ChangePlayMode,
    ChangeVolume(f32),
    ChangeDesktopLyric(ChangeDesktopLyric),
    ChangeWinMode(bool),
    ChangeAutoPlay(bool),
}
impl ConfigMessage {
    pub fn change(&self, app: &mut SilkPlayer) -> Command<Message> {
        match self {
            ConfigMessage::SelectMonitor => {
                if let Some(a) = FileDialog::new().pick_folder() {
                    // app.setting.monitor = a;
                    app.setting.monitor = a.as_path().as_os_str().to_str().unwrap().to_string();
                }
            }
            ConfigMessage::ChangeMonitor(monitor) => {
                app.setting.monitor = monitor.to_string();
                app.setting.save();
            }
            ConfigMessage::SubmitMonitor => {
                app.clear_play();
                app.init_list();
            }
            ConfigMessage::ChangeTheme(theme) => {
                app.setting.theme = Some(*theme);
                app.setting.save();
            }
            ConfigMessage::ChangePlayMode => {
                app.setting.play_mode = app.setting.play_mode.next();
                app.setting.save();
            }
            ConfigMessage::ChangeVolume(value) => {
                app.audio.set_volume(*value);
                app.setting.volume = app.audio.volume();
                app.setting.save();
            }
            ConfigMessage::ChangeDesktopLyric(change) => match change {
                ChangeDesktopLyric::Line(value) => {
                    app.setting.desktop_lyric.line = *value;
                    app.setting.save();
                }
                ChangeDesktopLyric::FontSize(value) => {
                    if let Ok(num) = value.parse::<f32>() {
                        if num < 120. && num > 2. {
                            app.setting.desktop_lyric.font_size = value.to_string();
                            app.setting.save();
                        }
                    }
                }
                ChangeDesktopLyric::Ahead(value) => {
                    if let Ok(num) = value.parse::<f32>() {
                        if num < 3600. && num > 0. {
                            app.setting.desktop_lyric.ahead = num;
                            app.setting.save();
                        }
                    }
                }
            },
            ConfigMessage::ChangeWinMode(mode) => {
                app.setting.windows.decorations = *mode;
                app.setting.save();
                return window::toggle_decorations(window::Id::MAIN);
            }
            ConfigMessage::ChangeAutoPlay(auto_play) => {
                app.setting.auto_play = *auto_play;
                app.setting.save();
            }
        }
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub enum ChangeDesktopLyric {
    Line(ConfigDesktopLyricLine),
    FontSize(String),
    Ahead(String),
}

#[derive(Deserialize, Serialize)]
pub struct Setting {
    pub monitor: String,
    pub auto_play: bool,
    pub theme: Option<ThemeConfig>,
    pub play_mode: PlayMode, // 播放模式
    pub volume: f32,
    pub desktop_lyric: DesktopLyric,
    pub windows: Windows,
}

#[derive(Deserialize, Serialize)]
pub struct Windows {
    pub decorations: bool,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Serialize)]
pub struct DesktopLyric {
    pub line: ConfigDesktopLyricLine,
    pub font_size: String,
    pub height: f32,
    pub width: f32,
    pub x: f32,
    pub y: f32,
    pub font_color: Vec<u8>,
    pub ahead: f32, // 歌词提前显示，单位：s
}

impl Default for Setting {
    fn default() -> Self {
        load_config()
    }
}
impl Setting {
    pub fn new() -> Self {
        let current_dir = util::current_dir();
        println!("{current_dir}");
        Self {
            monitor: format!("{}/music", current_dir),
            // monitor: "D:/Jachin/我的文件/音乐/华语".to_string(),
            theme: Some(ThemeConfig::Dark),
            play_mode: Default::default(),
            volume: 1.0,
            desktop_lyric: DesktopLyric {
                line: ConfigDesktopLyricLine::Two,
                font_size: 22.0.to_string(),
                height: 77.5,
                width: 1024.5,
                x: 0.0,
                y: 0.0,
                font_color: vec![],
                ahead: 0.0,
            },
            windows: Windows {
                decorations: true,
                width: 1024.0,
                height: 697.0,
            },
            auto_play: true,
        }
    }

    fn save(&self) {
        match serde_json::to_string(self) {
            Err(err) => util::log_err(format!("save config error {}", err)),
            Ok(data) => save_config(data),
        }
    }

    pub fn get_theme(&self) -> Theme {
        if let Some(theme) = self.theme {
            theme.into()
        } else {
            Theme::Dark
        }
    }

    pub fn resize_windown(&mut self, id: window::Id, width: u32, height: u32) {
        if id == window::Id::MAIN {
            self.windows.width = width as f32;
            self.windows.height = height as f32;
        } else {
            self.desktop_lyric.width = width as f32;
            self.desktop_lyric.height = height as f32;
        }
        self.save();
    }

    pub fn change_desktop_lyric_position(&mut self, x: i32, y: i32) {
        self.desktop_lyric.x = x as f32;
        self.desktop_lyric.y = y as f32;
        self.save();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum ThemeConfig {
    Light,
    #[default]
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
}

impl ThemeConfig {
    const ALL: [Self; 21] = [
        ThemeConfig::Light,
        ThemeConfig::Dark,
        ThemeConfig::Dracula,
        ThemeConfig::Nord,
        ThemeConfig::SolarizedLight,
        ThemeConfig::SolarizedDark,
        ThemeConfig::GruvboxLight,
        ThemeConfig::GruvboxDark,
        ThemeConfig::CatppuccinLatte,
        ThemeConfig::CatppuccinFrappe,
        ThemeConfig::CatppuccinMacchiato,
        ThemeConfig::CatppuccinMocha,
        ThemeConfig::TokyoNight,
        ThemeConfig::TokyoNightStorm,
        ThemeConfig::TokyoNightLight,
        ThemeConfig::KanagawaWave,
        ThemeConfig::KanagawaDragon,
        ThemeConfig::KanagawaLotus,
        ThemeConfig::Moonfly,
        ThemeConfig::Nightfly,
        ThemeConfig::Oxocarbon,
    ];
}

impl Into<Theme> for ThemeConfig {
    fn into(self) -> Theme {
        match self {
            ThemeConfig::Light => Theme::Light,
            ThemeConfig::Dark => Theme::Dark,
            ThemeConfig::Dracula => Theme::Dracula,
            ThemeConfig::Nord => Theme::Nord,
            ThemeConfig::SolarizedLight => Theme::SolarizedLight,
            ThemeConfig::SolarizedDark => Theme::SolarizedDark,
            ThemeConfig::GruvboxLight => Theme::GruvboxLight,
            ThemeConfig::GruvboxDark => Theme::GruvboxDark,
            ThemeConfig::CatppuccinLatte => Theme::CatppuccinLatte,
            ThemeConfig::CatppuccinFrappe => Theme::CatppuccinFrappe,
            ThemeConfig::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            ThemeConfig::CatppuccinMocha => Theme::CatppuccinMocha,
            ThemeConfig::TokyoNight => Theme::TokyoNight,
            ThemeConfig::TokyoNightStorm => Theme::TokyoNightStorm,
            ThemeConfig::TokyoNightLight => Theme::TokyoNightLight,
            ThemeConfig::KanagawaWave => Theme::KanagawaWave,
            ThemeConfig::KanagawaDragon => Theme::KanagawaDragon,
            ThemeConfig::KanagawaLotus => Theme::KanagawaLotus,
            ThemeConfig::Moonfly => Theme::Moonfly,
            ThemeConfig::Nightfly => Theme::Nightfly,
            ThemeConfig::Oxocarbon => Theme::Oxocarbon,
        }
    }
}

impl std::fmt::Display for ThemeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ThemeConfig::Light => "Light",
                ThemeConfig::Dark => "Dark",
                ThemeConfig::Dracula => "Dracula",
                ThemeConfig::Nord => "Nord",
                ThemeConfig::SolarizedLight => "SolarizedLight",
                ThemeConfig::SolarizedDark => "SolarizedDark",
                ThemeConfig::GruvboxLight => "GruvboxLight",
                ThemeConfig::GruvboxDark => "GruvboxDark",
                ThemeConfig::CatppuccinLatte => "CatppuccinLatte",
                ThemeConfig::CatppuccinFrappe => "CatppuccinFrappe",
                ThemeConfig::CatppuccinMacchiato => "CatppuccinMacchiato",
                ThemeConfig::CatppuccinMocha => "CatppuccinMocha",
                ThemeConfig::TokyoNight => "TokyoNight",
                ThemeConfig::TokyoNightStorm => "TokyoNightStorm",
                ThemeConfig::TokyoNightLight => "TokyoNightLight",
                ThemeConfig::KanagawaWave => "KanagawaWave",
                ThemeConfig::KanagawaDragon => "KanagawaDragon",
                ThemeConfig::KanagawaLotus => "KanagawaLotus",
                ThemeConfig::Moonfly => "Moonfly",
                ThemeConfig::Nightfly => "Nightfly",
                ThemeConfig::Oxocarbon => "Oxocarbon",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ConfigDesktopLyricLine {
    One,
    Two,
}

impl SilkPlayer {
    pub fn option_view(&self) -> View {
        let gap = 10;
        let monitor = row!(
            "路径",
            text_input("", &self.setting.monitor)
                .on_input(|value| Message::ChangeConfig(ConfigMessage::ChangeMonitor(value)))
                .on_submit(Message::ChangeConfig(ConfigMessage::SubmitMonitor))
                .width(500),
            button("选择").on_press(Message::ChangeConfig(ConfigMessage::SelectMonitor))
        )
        .spacing(gap)
        .align_items(Alignment::Center);

        let theme_pick_list = pick_list(&ThemeConfig::ALL[..], self.setting.theme, |value| {
            Message::ChangeConfig(ConfigMessage::ChangeTheme(value))
        })
        .placeholder("Choose a theme...");

        let theme = row!("主题", theme_pick_list)
            .spacing(gap)
            .align_items(Alignment::Center);

        let win_mode = checkbox("窗口模式", self.setting.windows.decorations)
            .on_toggle(|v| Message::ChangeConfig(ConfigMessage::ChangeWinMode(v)));

        let auto_play = checkbox("启动时恢复播放", self.setting.auto_play)
            .on_toggle(|v| Message::ChangeConfig(ConfigMessage::ChangeAutoPlay(v)));

        let general = column!("常规设置", monitor, theme, win_mode, auto_play).spacing(5);

        let desktop_lyric = column!(
            "",
            "桌面歌词",
            form_item(
                "歌词行数",
                row!(
                    radio(
                        "单行",
                        ConfigDesktopLyricLine::One,
                        Some(self.setting.desktop_lyric.line),
                        |value| {
                            Message::ChangeConfig(ConfigMessage::ChangeDesktopLyric(
                                ChangeDesktopLyric::Line(value),
                            ))
                        }
                    ),
                    radio(
                        "双行",
                        ConfigDesktopLyricLine::Two,
                        Some(self.setting.desktop_lyric.line),
                        |value| {
                            Message::ChangeConfig(ConfigMessage::ChangeDesktopLyric(
                                ChangeDesktopLyric::Line(value),
                            ))
                        }
                    ),
                )
                .spacing(20)
                .into()
            ),
            form_item(
                "字体大小",
                text_input("", &self.setting.desktop_lyric.font_size)
                    .on_input(
                        |value| Message::ChangeConfig(ConfigMessage::ChangeDesktopLyric(
                            ChangeDesktopLyric::FontSize(value)
                        ))
                    )
                    .width(150)
                    .into(),
            ),
            form_item(
                "歌词提前",
                text_input("", &self.setting.desktop_lyric.ahead.to_string())
                    .on_input(
                        |value| Message::ChangeConfig(ConfigMessage::ChangeDesktopLyric(
                            ChangeDesktopLyric::Ahead(value)
                        ))
                    )
                    .width(150)
                    .into(),
            ),
        )
        .spacing(5);

        let key = column!(
            "",
            "快捷键",
            form_item("Esc ------ ", text("返回上一个界面/退出应用").into()),
            form_item("a ------ ", text("进入/退出播放页").into()),
            form_item("s ------ ", text("进入设置页").into()),
            form_item("f ------ ", text("进入歌单页").into()),
            form_item("h ------ ", text("进入主页").into()),
            form_item("l ------ ", text("进入播放列表页").into()),
            form_item("space ------ ", text("播放/暂停").into()),
            form_item("Ctrl+up/down ------ ", text("音量增加/减小").into()),
            form_item("Ctrl+left/right ------ ", text("上一首/下一首").into()),
            "",
            "全局快捷键",
            form_item("Ctrl+num8/num2 ------ ", text("音量增加/减小").into()),
            form_item("Ctrl+num4/num6 ------ ", text("上一首/下一首").into()),
            form_item("Ctrl+num5 ------ ", text("暂停/播放").into()),
        )
        .spacing(5);

        // column!("常规设置", monitor, theme, wim_mode, desktop_lyric)
        Scrollable::new(
            column!(general, desktop_lyric, key)
                .padding([10, 50])
                .spacing(gap),
        )
        .width(Length::Fill)
        .into()
    }
}

fn form_item<'a>(label: &'a str, content: View<'a>) -> View<'a> {
    row!(label, content)
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
}

/// 加载配置文件
fn load_config() -> Setting {
    let data_dir = &util::data_dir();
    if !util::file_exist(&data_dir) {
        if let Err(err) = std::fs::create_dir(data_dir) {
            util::log_err(format!("create data dir errolr: {}", err));
            return Setting::new();
        }
    }
    let config_path = format!("{}/config.json", data_dir);
    match std::fs::read_to_string(&config_path) {
        Err(err) => {
            util::log_err(format!("load config file {} errolr: {}", &config_path, err));
            Setting::new()
        }
        Ok(data) => {
            let result: Result<Setting, serde_json::Error> = serde_json::from_str(&data);
            match result {
                Err(err) => {
                    util::log_err(format!("parse config data error {};data={}", err, &data));
                    Setting::new()
                }
                Ok(data) => data,
            }
        }
    }
}

fn save_config(data: String) {
    let data_dir = &util::data_dir();
    if !util::file_exist(&data_dir) {
        if let Err(err) = std::fs::create_dir(data_dir) {
            util::log_err(format!("save config;Create data dir errolr: {}", err));
            return;
        }
    }
    let config_path = format!("{}/config.json", data_dir);
    if let Err(err) = std::fs::write(&config_path, &data) {
        util::log_err(format!(
            "save config error {};path:{}, data:{}",
            err, &config_path, &data
        ));
    }
}
