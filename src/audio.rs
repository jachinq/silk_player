use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::{FromFileError, PlaybackState};
use kira::tween::Tween;

use kira::manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings};

use crate::util;

pub struct Audio {
    duration: f32,
    volume: f32,
    status: PlaybackState,
    manager: AudioManager,
    sound_handle: Option<StreamingSoundHandle<FromFileError>>,
}


impl Default for Audio {
    fn default() -> Self {
        Audio::new()
    }
}

impl Audio {
    pub fn new() -> Audio {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).unwrap();
        Audio {
            duration: 0.,
            volume: 1.0,
            manager,
            status: PlaybackState::Stopped,
            sound_handle: None,
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.stop(Tween::default());
            self.sound_handle = None;
            self.status = PlaybackState::Stopped;
        }
    }

    pub fn start_play(&mut self, path: &String, go_play: bool) {
        self.stop();
        if let Some(sound_handle) = &self.sound_handle {
            if sound_handle.state() == PlaybackState::Playing {
                return;
            }
        }

        if let Ok(sound_data) = StreamingSoundData::from_file(path) {
            // self.sound_data = Some(sound_data);
            self.duration = sound_data.duration().as_secs_f32();

            let mut play = self.manager.play(sound_data).unwrap();
            play.set_volume(self.volume as f64, Tween::default());
            self.sound_handle = Some(play);

            if go_play {
                self.status = PlaybackState::Playing;
            } else {
                self.pause()
            }

            util::log(format!("sink append {}", path));
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.pause(Tween::default());
            self.status = PlaybackState::Paused;
            util::log("pause sink");
        }
    }

    pub fn toggle_play(&mut self) {
        if self.is_play() {
            self.pause();
        } else {
            if let Some(ref mut sound_handle) = self.sound_handle {
                sound_handle.resume(Tween::default());
                self.status = PlaybackState::Playing;
            }
        }
    }

    pub fn seek(&mut self, pos: f32) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.seek_to(pos as f64);
        }
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn is_play(&self) -> bool {
        self.status == PlaybackState::Playing
    }
    pub fn is_over(&self) -> bool {
        if let Some(ref sound_handle) = self.sound_handle {
            sound_handle.state() == PlaybackState::Stopped
        } else {
            true
        }
    }

    pub fn position(&self) -> f32 {
        if let Some(ref sound_handle) = self.sound_handle {
            return sound_handle.position() as f32;
        }
        0.0
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, volume: f32) {
        if self.volume > 1. {
            return;
        }
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(volume as f64, Tween::default())
        }
        self.volume = volume;
    }
}
