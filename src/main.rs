// use the sub command to hide bash window
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod components;
mod config;
mod data;
mod desktop;
mod handle_event;
mod play;
mod style;
mod util;
mod view;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use audio::Audio;
use config::{ConfigMessage, Setting};

use data::PlayStatus;
use iced::{
    event, executor, futures::lock::Mutex, keyboard::Modifiers, multi_window::Application, widget::{column, container, scrollable}, window::{self, settings::PlatformSpecific, Level, Position}, Command, Event, Font, Length, Pixels, Settings, Size, Subscription, Theme
};
use util::{log, log_err};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    GetCurrentProcess, SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS,
};

use ::silk_player::ThreadPool;
use once_cell::sync::Lazy;
use play::*;
// use thread_priority::*;
use view::{DetailTab, PageInfo};

const UPDATE_TIME: f32 = 0.5;
// const RERRESH: f32 = 1.0;
static LYRIC_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
static PLAY_LIST_SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

fn main() -> iced::Result {
    // match set_current_thread_priority(ThreadPriority::Os(WinAPIThreadPriority::AboveNormal.into())) {
    //     Ok(_) => util::log("set thread priority success: AboveNormal"),
    //     Err(e) => util::log_err(format!("set thread priority failed: {}", e)),
    // }

    #[cfg(target_os = "windows")]
    {    
        // 获取当前进程的句柄
        let process = unsafe { GetCurrentProcess() };

        // 设置进程的优先级为高于普通的优先级
        match unsafe { SetPriorityClass(process, ABOVE_NORMAL_PRIORITY_CLASS) } {
            Ok(_) => log("set process priority success: AboveNormal"),
            Err(e) => log_err(format!("set process priority failed: {}", e)),
        }
    }

    // 创建数据存放路径
    util::check_dir_and_create(&util::cache_dir());
    util::check_dir_and_create(&util::data_dir());
    util::check_dir_and_create(&util::log_dir());

    // 用 include_bytes 如果路径错误，还会提示的
    let fonts = vec![include_bytes!("../assets/LXGWWenKaiGBScreenR.ttf").into()];
    // let xqfont: Font = Font::with_name("霞鹜文楷");
    let xqfont: Font = Font::with_name("霞鹜文楷 GB 屏幕阅读版 R");
    // let xqfont: Font = Font::with_name("Maple Mono NF CN");

    
    let min_width = 800.0;
    let min_height = 580.0;
    let windows = Setting::default().windows;

    let run = SilkPlayer::run(Settings {
        id: None,
        window: window::Settings {
            size: Size::new(windows.width, windows.height),
            position: Position::Centered,
            min_size: Some(Size::new(min_width, min_height)),
            max_size: None,
            visible: true,
            resizable: true,
            decorations: windows.decorations, // 传统窗口栏
            transparent: false,               // 透明窗口
            level: Level::default(),
            icon: util::app_icon(),
            exit_on_close_request: true,
            platform_specific: PlatformSpecific::default(),
        },
        flags: Default::default(),
        fonts,
        default_font: xqfont,
        default_text_size: Pixels(24.0),
        antialiasing: false, // 抗锯齿
    });
    run
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Tab {
    #[default]
    Home,
    Fave,
    List,
    Like,
    LikeDetail,
    Option,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event), // 订阅 iced 的事件转发给 handle_event 处理
    UpdateTime(Instant),
    ScrollLyric(scrollable::Viewport),

    MoveWindow(bool), // 是否开始移动窗口

    ChangeTab(Tab),
    ChangeTag(Tag),
    ChangPage { page: usize, is_play_list: bool },
    Filter(String),
    PlayDetail,

    SongControl(SongControl),

    ChangeDetail(DetailTab),
    UpdateSongTime(f32),
    UpdateSongTimeRelease,
    ChangeConfig(ConfigMessage),
    DesktopLyricWindow,

    OpenWith(bool, String, String),
    ToggleEsc,
    ToggleFullScreen,
    ToggleEnter,
    ConfirmExit,
    RequestExit,
}

#[derive(Default, PartialEq)]
pub enum Status {
    #[default]
    Tab,
    PlayDetial,
}

#[derive(Default)]
pub struct SilkPlayer {
    status: Status,
    tab: Tab,
    tag: Tag,
    detail_tab: DetailTab,
    current_song: MusicInfo,
    audio: Audio,
    setting: Setting,
    show_exit_confirm: bool,
    music_list: PageInfo,
    play_list: PageInfo,
    app_control: AppControl,             // 控制应用相关行为
    key_modify: Vec<Modifiers>,          // 复杂按键控制
    thread_pool: ThreadPool,             // 使用线程池处理耗时任务
    command: Arc<Mutex<Vec<MyCommand>>>, // 多线程命令
    album_map: HashMap<String, bool>,
}

#[derive(Debug)]
pub enum MyCommand {
    Cmd(Message),
}

impl Application for SilkPlayer {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let mut app = Self::default();
        app.init_list();
        let volume = app.setting.volume;
        app.audio.set_volume(volume);

        app.play_list.size = 50;

        PlayStatus::load().init(&mut app);

        app.register_global_hotkeys();

        (app, Command::none())
    }

    fn title(&self, window: window::Id) -> String {
        if window == window::Id::MAIN {
            "Silk Player"
        } else {
            "Desktop Lyric"
        }
        .to_string()
    }

    fn theme(&self, _window: window::Id) -> iced::Theme {
        self.setting.get_theme()
    }

    fn style(&self) -> <Self::Theme as iced::application::StyleSheet>::Style {
        iced::theme::Application::Custom(Box::new(style::AppliactionStyle))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let mut msgs = vec![];
        if let Some(mut msg) = self.command.try_lock() {
            if let Some(msg) = msg.pop() {
                match msg {
                    MyCommand::Cmd(msg) => msgs.push(msg),
                }
            }
        }
        if !msgs.is_empty() {
            return self.update(msgs.get(0).unwrap().clone());
        }

        match message {
            Message::EventOccurred(event) => return self.handle_event(event),
            Message::ChangeTab(tab) => {
                if self.status == Status::PlayDetial {
                    let _ = self.change_status();
                }
                self.tab = tab;
            }
            Message::MoveWindow(start) => {
                if start { // 移动窗口
                    return window::drag(window::Id::MAIN);
                }
            }
            Message::ChangeTag(tag) => {
                self.tag = tag;
                return self.update(Message::ChangeTab(Tab::LikeDetail));
            }
            Message::ChangPage { page, is_play_list } => {
                if is_play_list {
                    let is_ctrl = self.key_modify.contains(&Modifiers::CTRL);
                    self.play_list.change_page(is_ctrl, page);
                } else {
                    let is_ctrl = self.key_modify.contains(&Modifiers::CTRL);
                    self.music_list.change_page(is_ctrl, page);
                    if !self.music_list.page_list.is_empty() {
                        self.init_album_img(self.music_list.page_list.to_vec());
                    }
                }
            }
            Message::Filter(value) => {
                self.music_list.search = value;
                self.music_list.filter();
            }
            Message::PlayDetail => return self.change_status(),
            Message::ChangeDetail(value) => self.detail_tab = value,
            Message::ChangeConfig(config) => {
                return config.change(self);
            }
            Message::SongControl(play_next) => {
                return self.change_play_list(play_next);
            }
            Message::OpenWith(is_dir, mut path, app) => {
                if is_dir {
                    path = util::get_parent_path(&path);
                }
                let _ = if app.is_empty() {
                    open::that(path)
                } else {
                    open::with(path, app)
                };
            }
            Message::ToggleEsc => return self.toggl_esc(),
            Message::ToggleFullScreen => return self.toggl_full_screen(),
            Message::ToggleEnter => {
                if self.show_exit_confirm {
                    return self.update(Message::ConfirmExit);
                }
            }
            Message::UpdateSongTime(value) => {
                self.app_control.change_current_duration = true;
                self.app_control.current_duration = value;
            }
            Message::UpdateSongTimeRelease => {
                self.audio.seek(self.app_control.current_duration);
                self.app_control.change_current_duration = false;
            }
            Message::UpdateTime(t) => {
                return self.update_time(t);
            }
            Message::ScrollLyric(_viewport) => {
                self.app_control.scroll_seconds = Some(Instant::now());
            }
            Message::DesktopLyricWindow => {
                return self.new_destop_window();
            }

            Message::ConfirmExit => {
                return Command::batch([
                    self.try_close_desktop_lyric(),
                    iced::window::close(window::Id::MAIN),
                ]);
            }
            Message::RequestExit => self.show_exit_confirm = !self.show_exit_confirm,
        }

        Command::none()
    }

    fn view(&self, window: window::Id) -> View {
        if window != window::Id::MAIN {
            return self.desktop_lyric_view(window);
        }
        if self.show_exit_confirm {
            return self.exit_view();
        }

        let content = match self.status {
            Status::Tab => {
                let tab_container = column!()
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .spacing(20)
                    .push(self.top_status_view())
                    .push(self.tab_container());
                let tab_container = container(tab_container).style(iced::theme::Container::Custom(
                    Box::new(style::ContainerStyle::BackgroundWithAlpha(1.0)),
                ));
                tab_container.into()
            }
            Status::PlayDetial => self.playing_detail(),
        };
        // let content = container(content).padding(ui::padding_bottom(120.));

        let play_status = if self.app_control.hide_status {
            column!().into()
        } else {
            self.bottom_status_view()
        };
        column!(content, play_status).into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let every = iced::time::every(std::time::Duration::from_secs_f32(UPDATE_TIME))
            .map(|v| Message::UpdateTime(v));

        // 把系统事件转给 EventOccurred 消息进行处理
        let map = event::listen().map(Message::EventOccurred);

        Subscription::batch([
            every, //press, release,
            map,
        ])
    }
}

type View<'a> = iced::Element<'a, Message>;

impl SilkPlayer {
    /// 页面刷新
    fn update_time(&mut self, _msg_time: Instant) -> Command<Message> {
        // init done?
        // if !self.music_list.check_init_done() {
        self.music_list.filter();
        self.music_list.page();
        // }
        self.init_album_img(self.music_list.page_list.to_vec());

        if !self.current_song.is_none() && self.audio.is_play() {
            if !self.app_control.change_current_duration {
                self.app_control.current_duration = self.audio.position();
            }
            if self.audio.position() > 0.0 {
                // per 10s save once
                if (self.audio.position() * 1000.0) as u64 / 1000 % 10 == 0 {
                    self.save_play_status();
                }
            }

            // 自动下一曲
            if self.audio.is_play()
                && (self.audio.is_over() || self.audio.position().ceil() >= self.audio.duration())
            {
                self.next_song();
            }
        }

        // 播放条自动隐藏处理
        if let Some(time) = self.app_control.hide_status_seconds {
            if time.elapsed().as_secs() > 5 {
                self.app_control.hide_status_seconds = None;
            }
        }
        if self.audio.is_play()
            && self.status == Status::PlayDetial
            && self.app_control.hide_status_seconds.is_none()
        {
            self.app_control.hide_status = true;
        }

        if self.app_control.hide_status {
            if !self.current_song.path.is_empty() {
                self.init_album_color();
                if self.app_control.hide_status && self.status == Status::Tab {
                    let path = util::get_thumbnail_path(&self.current_song.album_path);
                    if util::file_exist(&path) {
                        self.app_control.hide_status = false;
                    }
                }
            }
        }

        // 自动刷新专辑封面
        if self.app_control.refresh_detail_album {
            let album_path = util::get_album_path(&self.current_song.path);
            if util::file_exist(&album_path) {
                self.app_control.refresh_detail_album = false;
                self.current_song.album_path = album_path;
                self.app_control.hide_status = true;
                self.current_song.album_color.clear();
            }
        }

        // 歌词滚动处理
        for show_lyric in &self.current_song.lyric {
            let lyric_ahead = self.setting.desktop_lyric.ahead;
            if show_lyric.beg - lyric_ahead <= self.audio.position()
                && self.audio.position() < show_lyric.end - lyric_ahead
            {
                if self.app_control.current_lyric_index != show_lyric.index {
                    self.app_control.current_lyric_index = show_lyric.index;
                }
                break;
            }
        }
        if let Some(time) = self.app_control.scroll_seconds {
            if time.elapsed().as_secs() > 5 {
                self.app_control.scroll_seconds = None;
            }
        }

        self.scroll_lyric()
    }

    /// 缓存专辑封面
    fn init_album_img(&mut self, vec: Vec<MusicInfo>) {
        let init = |music_info: &MusicInfo| {
            if let Ok(tag) = music_tag::audio::MusicTag::read_from_path(&music_info.path) {
                util::log(format!("album : {:?}", tag.title()));
                if let Some(artwork) = tag.artwork() {
                    let buf = artwork.data.to_vec();
                    let title = util::get_str_value(tag.title(), "unknow");
                    let album_path = music_info.album_path.clone();
                    if let Err(err) = util::save_file_from_buffer(album_path.clone(), &buf) {
                        util::log_err(format!(
                            "save error. title={} path:{} err:{}",
                            title, album_path, err
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        };

        // 这里做过滤，已经提交过的任务不再重复提交
        let mut task = vec![];
        for music_info in vec {
            if self.album_map.get(&music_info.path).is_some() {
                continue;
            }
            self.album_map.insert(music_info.path.to_string(), true);
            task.push(music_info);
        }

        // let vec = self.music_list.all_list.clone();
        self.thread_pool.execute(move || {
            for music_info in task {
                init(&music_info);
            }
        });
    }
}
