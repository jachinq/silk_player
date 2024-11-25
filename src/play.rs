use std::{collections::HashMap, time::Instant, vec};

use iced::{
    multi_window::Application,
    widget::scrollable::{self, RelativeOffset},
    window, Color, Command,
};
use music_tag::audio::MusicTag;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    util::{self, get_str_value}, Message, SilkPlayer, Tab, PLAY_LIST_SCROLLABLE_ID
};

#[derive(Default, PartialEq, Serialize, Deserialize)]
pub enum PlayMode {
    Single,
    #[default]
    Cycle,
    Random,
}
impl PlayMode {
    pub fn name(&self) -> String {
        match self {
            PlayMode::Single => "单曲循环",
            PlayMode::Cycle => "列表循环",
            PlayMode::Random => "随机播放",
        }
        .to_string()
    }
    pub fn icon(&self) -> String {
        match self {
            PlayMode::Single => "cycle_single",
            PlayMode::Cycle => "cycle_list",
            PlayMode::Random => "cycle_random",
        }
        .to_string()
    }

    pub fn next(&self) -> Self {
        match self {
            PlayMode::Single => PlayMode::Cycle,
            PlayMode::Cycle => PlayMode::Random,
            PlayMode::Random => PlayMode::Single,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum SongControl {
    PlayAll,
    PlayNext(bool),
    PlayClear,
    SnapToCurrentSong, // 定位当前播放的歌曲
    RandomSelect,      // 随机选取n首歌开始播放
    First(MusicInfo),
    Next(MusicInfo),
    Last(MusicInfo),
    List(MusicInfo),
    Remove(MusicInfo),
    PlayOrPause,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tag {
    pub name: String,
    pub path: String,
}
impl Tag {
    pub fn new(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub struct MusicInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_path: String,
    pub album_color: Vec<Color>,
    pub year: String,
    pub time: f32, // millisecons
    pub fmt: MusicFormat,
    pub file_name: String,
    pub path: String,
    pub lyric: Vec<ShowLyric>,
    pub tags: Vec<Tag>,
}
impl Default for MusicInfo {
    fn default() -> Self {
        Self {
            title: Default::default(),
            artist: Default::default(),
            album: Default::default(),
            album_path: format!("{}/assets/default.png", util::current_dir()),
            album_color: Default::default(),
            year: Default::default(),
            time: Default::default(),
            file_name: Default::default(),
            path: Default::default(),
            lyric: Default::default(),
            fmt: Default::default(),
            tags: Default::default(),
        }
    }
}
impl MusicInfo {
    pub fn is_none(&self) -> bool {
        self.path.is_empty()
    }

    pub fn new(path: &str) -> MusicInfo {
        if !util::file_exist(path) {
            util::log_err(format!("file not exist: {}", path));
            return MusicInfo::default();
        }

        match music_tag::audio::MusicTag::read_from_path(path) {
            Err(err) => {
                util::log_err(format!("read tag err path={} err={}", path, err));
                MusicInfo::default()
            }
            Ok(tag) => {
                let title = get_str_value(tag.title(), "");
                let artist = get_str_value(tag.artist(), "");
                let album_path = util::get_album_path_by_tag(&tag);

                let lyric = ShowLyric::build(&tag);

                let mut tags = vec![];
                let path_ = std::path::Path::new(path);
                if let Some(path_) = path_.parent() {
                    if let Some(file_name) = path_.file_name() {
                        tags.push(Tag::new(
                            &file_name.to_string_lossy(),
                            &path_.to_string_lossy(),
                        ))
                    }
                }

                MusicInfo {
                    title,
                    artist,
                    album: get_str_value(tag.album(), ""),
                    album_path,
                    album_color: vec![],
                    year: get_str_value(tag.year(), ""),
                    time: 0.0,
                    file_name: get_str_value(tag.year(), path),
                    path: path.to_string(),
                    lyric,
                    fmt: MusicFormat::from(tag.fmt()),
                    tags,
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MusicFormat {
    M4a,
    Mp3,
    Flac,
    Ogg,
    #[default]
    Unkonw,
}
impl From<music_tag::audio::MusicFormat> for MusicFormat {
    fn from(value: music_tag::audio::MusicFormat) -> Self {
        match value {
            music_tag::audio::MusicFormat::M4a => MusicFormat::M4a,
            music_tag::audio::MusicFormat::Mp3 => MusicFormat::Mp3,
            music_tag::audio::MusicFormat::Flac => MusicFormat::Flac,
            music_tag::audio::MusicFormat::Ogg => MusicFormat::Ogg,
        }
    }
}
impl ToString for MusicFormat {
    fn to_string(&self) -> String {
        match self {
            MusicFormat::M4a => "M4a",
            MusicFormat::Mp3 => "Mp3",
            MusicFormat::Flac => "Flac",
            MusicFormat::Ogg => "Ogg",
            MusicFormat::Unkonw => "Unkonw",
        }
        .to_lowercase()
        .to_string()
    }
}

#[derive(Debug)]
pub struct AppControl {
    pub current_duration: f32,
    pub change_current_duration: bool,
    pub full_screen: bool,
    pub current_lyric_index: usize,
    pub refresh_detail_album: bool,           // 是否刷新专辑封面
    pub hide_status: bool,                    // 是否展示底部播放状态条
    pub hide_status_seconds: Option<Instant>, // 自动隐藏播放条时间
    pub scroll_seconds: Option<Instant>,      // 自动滚动暂停时间
    pub foucus_desktop_lyric: bool,
    pub press_left_mouse_key: bool,
    pub desktop_lyric_win_id: Option<window::Id>,
    pub history_list: Vec<String>, // 播放历史
}
impl Default for AppControl {
    fn default() -> Self {
        Self {
            current_duration: 0.0,
            change_current_duration: false,
            full_screen: false,
            current_lyric_index: Default::default(),
            refresh_detail_album: false,
            hide_status: true,
            hide_status_seconds: Default::default(),
            scroll_seconds: Default::default(),
            foucus_desktop_lyric: false,
            press_left_mouse_key: false,
            desktop_lyric_win_id: None,
            history_list: vec![],
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub struct ShowLyric {
    pub index: usize,
    pub min: u64,
    pub sec: u64,
    pub millisec: u64,
    pub beg: f32,
    pub end: f32,
    pub lyric: String,
}
impl ShowLyric {
    pub fn build(music_tag: &MusicTag) -> Vec<ShowLyric> {
        let mut tmp_lyric_list = vec![];

        if let Some(lyric_value) = music_tag.lyrics() {
            // if !lyric_value.lines().is_empty() {
            //     println!("{:?} = {:?}", music_tag.title(), lyric_value.lines()[lyric_value.lines().len() - 1]);
            // }
            for (index, item) in lyric_value.lines_with_time().into_iter().enumerate() {
                let (item, l) = item;
                let (min, sec, millsec) = if let Some(ld) = item {
                    (ld.minute(), ld.seconds(), ld.milliseconds())
                } else {
                    (0, 0, 0)
                };

                tmp_lyric_list.push((index, min, sec, millsec, l.to_string()))
            }
        }

        let len = tmp_lyric_list.len();
        let mut lyrics = vec![];
        for (index, min, sec, millisec, lyric) in tmp_lyric_list.clone() {
            let secs = (min * 60 + sec) as f32 + millisec as f32 / 100.;

            let beg = if index == 0 { secs - 1. } else { secs };
            let end = if index >= len - 1 {
                secs + 10.
            } else {
                let (_, min, sec, millisec, _) = tmp_lyric_list[index + 1];
                (min * 60 + sec) as f32 + millisec as f32 / 100.
            };

            lyrics.push(ShowLyric {
                index,
                min,
                sec,
                millisec,
                beg,
                end,
                lyric,
            });
        }
        lyrics
    }
}

impl SilkPlayer {
    pub fn clear_play(&mut self) {
        self.play_list.clear();
        self.audio.stop();
        self.current_song = MusicInfo::default();
        self.app_control.history_list.clear();
        self.save_play_status();
    }

    pub fn init_list(&mut self) {
        if &self.setting.monitor == "" {
            return;
        }

        if let Ok(mut all_list) = self.music_list.all_list.try_lock() {
            all_list.clear();
        }

        self.music_list.init_monitor(&self.setting.monitor);

        self.audio.pause();
    }

    /// 尝试读取专辑页面的图片信息
    pub fn init_album_color(&mut self) {
        if self.current_song.album_color.is_empty() {
            let is_dark = self.setting.get_theme().extended_palette().is_dark;

            let colors_vec = util::get_colors_vec(&self.current_song.album_path);
            self.current_song.album_color = colors_vec
                .iter()
                .map(|(r, g, b, _a)| {
                    if is_dark {
                        Color::from_rgba8(*r / 2, *g / 2, *b / 2, 1.0)
                    } else {
                        Color::from_rgba8(*r / 2, *g / 2, *b / 2, 1.0).inverse()
                    }
                })
                .collect();
            util::log_debug(format!(
                "init album color ok;{},colors={:?}",
                self.current_song.album_path,
                self.current_song.album_color.len()
            ));
        }
    }

    pub fn change_play_list(&mut self, play_next: SongControl) -> Command<Message> {
        let all_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            true
        };
        let (filter, play) = match play_next {
            SongControl::PlayAll => {
                let next = all_list_empty;
                let mut map = HashMap::new();
                if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                    for music_info in all_list.iter() {
                        map.insert(music_info.path.to_string(), 1);
                    }
                    let list = 
                    if self.tab == Tab::LikeDetail {
                        &self.get_list_by_tag()
                    } else {
                        &self.music_list.filter_list
                    };
                    for music_info in list {
                        if let None = map.get(&music_info.path) {
                            all_list.push(music_info.clone());
                        }
                    }
                }

                if next {
                    self.next_song();
                }
                (true, false)
            }
            SongControl::PlayNext(next) => {
                if next {
                    self.next_song();
                } else {
                    self.pre_song();
                }
                (false, false)
            }
            SongControl::PlayClear => {
                self.clear_play();
                (true, true)
            }
            SongControl::First(music_info) => {
                let exist = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    all_list.contains(&music_info)
                } else {
                    false
                };
                if exist {
                    let _ = self.update(Message::SongControl(SongControl::List(music_info)));
                    (false, false)
                } else {
                    if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                        all_list.insert(0, music_info.clone());
                    }
                    self.current_song = music_info;
                    (true, true)
                }
            }
            SongControl::Next(music_info) => {
                let exist = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    all_list.contains(&music_info)
                } else {
                    false
                };
                if exist {
                    (false, false)
                } else {
                    let is_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                        all_list.is_empty()
                    } else {
                        true
                    };
                    let index = if is_empty { 0 } else { 1 };
                    if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                        all_list.insert(index, music_info.clone());
                    }
                    self.current_song = music_info;
                    (true, is_empty)
                }
            }
            SongControl::Last(music_info) => {
                let exist = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    all_list.contains(&music_info)
                } else {
                    false
                };
                if exist {
                    (false, false)
                } else {
                    let mut is_single = false;
                    if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                        all_list.push(music_info.clone());
                        is_single = all_list.len() == 1;
                    }
                    if is_single {
                        self.current_song = music_info;
                        (true, true)
                    } else {
                        (true, false)
                    }
                }
            }
            SongControl::List(music_info) => {
                if self.current_song.path == music_info.path {
                    self.audio.toggle_play();
                    (false, false)
                } else {
                    self.current_song = music_info;
                    (true, true)
                }
            }
            SongControl::Remove(music_info) => {
                let current = self.current_song.path == music_info.path;

                if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                    for (index, item) in all_list.to_vec().iter().enumerate() {
                        if item == &music_info {
                            all_list.remove(index);
                            break;
                        }
                    }
                }

                if current {
                    // 当前播放被移出播放列表，则自动播放下一首
                    self.next_song();
                    (true, false)
                } else {
                    (true, false) // 否则只需要刷新列表
                }
            }
            SongControl::SnapToCurrentSong => {
                if !self.current_song.path.is_empty() {
                    let index = self.play_list.pos_current_song(self.current_song.clone());

                    let y = index as f32 / self.play_list.page_list.len() as f32;
                    let offset = if index == 0 {
                        RelativeOffset::START
                    } else if index == self.play_list.page_list.len() - 1 {
                        RelativeOffset::END
                    } else {
                        RelativeOffset { x: 0.0, y }
                    };
                    return scrollable::snap_to(PLAY_LIST_SCROLLABLE_ID.clone(), offset);
                }
                (false, false)
            }
            SongControl::RandomSelect => {
                use ::rand::prelude::*;
                let mut vec = if let Ok(all_list) = self.music_list.all_list.try_lock() {
                    all_list.to_vec()
                } else {
                    vec![]
                };
                let len = vec.len();
                if len > 0 {
                    let mut thread_rng = rand::thread_rng();
                    vec.shuffle(&mut thread_rng);
                    let size = if len > 50 { 50 } else { len };
                    if let Ok(mut all_list) = self.play_list.all_list.try_lock() {
                        for item in &vec[0..size] {
                            all_list.push(item.clone());
                        }
                    }
                    self.current_song = vec[0].clone();
                    // util::log(format!("vec = {}", self.play_list.all_list.len()));
                    (true, true)
                } else {
                    (false, false)
                }
            }
            SongControl::PlayOrPause => {
                self.audio.toggle_play();
                (false, false)
            }
        };
        if filter {
            self.play_list.filter();
        }
        let all_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            true
        };
        if play || all_list_empty {
            self.start_play();
        }

        Command::none()
    }

    pub fn pre_song(&mut self) {
        if self.play_list.page_list.is_empty() {
            return;
        }
        let mut index = 0;

        let mut all_list_len = 0;
        if let Ok(all_list) = self.play_list.all_list.try_lock() {
            for item in all_list.iter() {
                if item.path == self.current_song.path {
                    break;
                }
                index += 1;
            }
            all_list_len = all_list.len();
        }

        index = (index + all_list_len - 1) % all_list_len;
        if let Ok(all_list) = self.play_list.all_list.try_lock() {
            self.current_song = all_list[index].clone();
        }
        self.start_play();
    }

    pub fn next_song(&mut self) {
        let all_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            true
        };
        if all_list_empty {
            return;
        }

        let index_plus = |path: String, list: &Vec<MusicInfo>| {
            let mut hit = false || self.current_song.path.is_empty();
            if list.is_empty() {
                return None;
            }
            for item in list {
                if hit {
                    return Some(item.clone());
                }
                if path == item.path {
                    hit = true;
                }
            }
            if hit {
                Some(list[0].clone())
            } else {
                None
            }
        };

        match self.setting.play_mode {
            PlayMode::Single => {}
            PlayMode::Cycle => {
                if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    if let Some(item) = index_plus(self.current_song.path.to_string(), &all_list) {
                        self.current_song = item;
                    }
                }
            }
            PlayMode::Random => {
                let mut tmp: usize = rand::thread_rng().gen();
                let mut loop_cnt = 0;

                let all_list_len = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                    all_list.len()
                } else {
                    0
                };

                loop {
                    let index = tmp % all_list_len;

                    let music_info = if let Ok(all_list) = self.play_list.all_list.try_lock() {
                        Some(all_list[index].clone())
                    } else {
                        None
                    };

                    if music_info.is_none() {
                        continue;
                    }
                    let music_info = music_info.unwrap();
                    // 随机到同一首就继续生成下一个随机
                    if music_info.path != self.current_song.path
                        && !self.app_control.history_list.contains(&music_info.path)
                    {
                        self.current_song = music_info.clone();
                        break;
                    }

                    if self.app_control.history_list.len() == all_list_len {
                        self.app_control.history_list.clear();
                    }

                    tmp = rand::thread_rng().gen();
                    loop_cnt += 1;
                    if loop_cnt > 10 {
                        if let Ok(all_list) = self.play_list.all_list.try_lock() {
                            if let Some(item) =
                                index_plus(self.current_song.path.to_string(), &all_list)
                            {
                                self.current_song = item;
                            }
                        }
                        break; // 保护，随机过多次
                    }
                }
            }
        }
        self.start_play();
    }

    pub fn start_play(&mut self) {
        let all_list_empty = if let Ok(all_list) = self.play_list.all_list.try_lock() {
            all_list.is_empty()
        } else {
            true
        };
        if all_list_empty {
            self.clear_play();
            util::log_debug(format!("play list is empty"));
            return;
        }

        if self.current_song.path.is_empty() {
            util::log_debug(format!("play path is empty"));
            return;
        }

        let music_info = &self.current_song;
        util::log(format!("now start {:?}", music_info.title));
        self.audio.start_play(&music_info.path, true);
        //
        self.app_control.current_lyric_index = 0;

        self.current_song.time = self.audio.duration();

        self.init_album_img(vec![self.current_song.clone()]);
        self.init_album_color();
        self.app_control.hide_status = true;
        self.app_control.refresh_detail_album = true;
        self.app_control
            .history_list
            .push(self.current_song.path.to_string());
        self.current_song.album_path = format!("{}/assets/default.png", util::current_dir());
    }
}
