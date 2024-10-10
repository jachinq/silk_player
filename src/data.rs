use serde::{Deserialize, Serialize};

use crate::{util, MusicInfo, SilkPlayer};

#[derive(Serialize, Deserialize)]
pub struct PlayStatus {
    play_path_list: Vec<String>,
    history_path_list: Vec<String>,
    current_path: String,
    time: f32,
    is_play: bool,
}
impl PlayStatus {
    pub fn init(&self, app: &mut SilkPlayer) {
        if self.play_path_list.is_empty() {
            return;
        }

        let mut play_list = vec![];
        let mut current_song = MusicInfo::default();
        for path in &self.play_path_list {
            let music_info = MusicInfo::new(&path);
            if music_info.path.is_empty() {
                continue;
            }
            if self.current_path.eq(path) {
                current_song = music_info.clone();
            }
            play_list.push(music_info);
        }

        app.play_list.init_list(play_list);
        app.app_control.history_list = self.history_path_list.to_vec();
        app.current_song = current_song;
        if app.setting.auto_play && self.is_play {
            util::log(format!(
                "init play status start play {}",
                app.current_song.title
            ));
            app.start_play();
        } else {
            app.audio.start_play(&app.current_song.path, false);
            app.current_song.time = app.audio.duration();
        }
        app.audio.seek(self.time);
    }
    pub fn default() -> Self {
        Self {
            play_path_list: Default::default(),
            history_path_list: Default::default(),
            current_path: Default::default(),
            time: Default::default(),
            is_play: Default::default(),
        }
    }
    pub fn load() -> Self {
        load_data()
    }
    pub fn save(play_list: &Vec<MusicInfo>, history_list: &Vec<String>, current_song: &MusicInfo, is_play: bool, time: f32) {
        let mut play_path_list = Vec::with_capacity(play_list.len());
        for item in play_list {
            play_path_list.push(item.path.to_string());
        }

        let play_status = Self {
            play_path_list,
            history_path_list: history_list.to_vec(),
            current_path: current_song.path.to_string(),
            time,
            is_play,
        };
        match serde_json::to_string(&play_status) {
            Err(err) => util::log_err(format!("save data error {}", err)),
            Ok(data) => save_data(data),
        }
    }
}

impl SilkPlayer {
    pub fn save_play_status(&self) {
        if let Ok(all_list) = self.play_list.all_list.try_lock() {
            PlayStatus::save(
                &all_list,
                &self.app_control.history_list,
                &self.current_song,
                self.audio.is_play(),
                self.audio.position(),
            );
        }
    }
}


const DATA_PATH: &str = "data.json";
/// 加载数据文件
fn load_data() -> PlayStatus {
    let data_dir = &util::data_dir();
    if !util::file_exist(&data_dir) {
        if let Err(err) = std::fs::create_dir(data_dir) {
            util::log_err(format!("create data dir errolr: {}", err));
            return PlayStatus::default();
        }
    }
    let data_path = format!("{}/{}", data_dir, DATA_PATH);
    match std::fs::read_to_string(&data_path) {
        Err(err) => {
            util::log_err(format!("load data file {} errolr: {}", &data_path, err));
            PlayStatus::default()
        }
        Ok(data) => {
            let result: Result<PlayStatus, serde_json::Error> = serde_json::from_str(&data);
            match result {
                Err(err) => {
                    util::log_err(format!("parse data data error {};data={}", err, &data));
                    PlayStatus::default()
                }
                Ok(data) => data,
            }
        }
    }
}

fn save_data(data: String) {
    let data_dir = &util::data_dir();
    if !util::file_exist(&data_dir) {
        if let Err(err) = std::fs::create_dir(data_dir) {
            util::log_err(format!("save data;Create data dir errolr: {}", err));
            return;
        }
    }
    let data_path = format!("{}/{}", data_dir, DATA_PATH);
    if let Err(err) = std::fs::write(&data_path, &data) {
        util::log_err(format!(
            "save data error {};path:{}, data:{}",
            err, &data_path, &data
        ));
    }
}
