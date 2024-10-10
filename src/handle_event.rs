use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use device_query::{DeviceEvents, DeviceState, Keycode};
use iced::{
    advanced::graphics::core::SmolStr,
    futures::lock::Mutex,
    keyboard::{self, key::Named, Key, Modifiers},
    mouse::{self, Button},
    multi_window::Application,
    window, Command, Event,
};

use crate::{config::ConfigMessage, Message, MyCommand, SilkPlayer, SongControl, Status, Tab};

impl SilkPlayer {
    // 注册全局热键
    pub fn register_global_hotkeys(&mut self) {
        use std::thread;
        let command = self.command.clone();
        let _ = thread::spawn(move || {
            let device_state = DeviceState::new();

            let global_hot_keys = GolbalHotKey::defaut_list();

            let input = Arc::new(Mutex::new(Input::new()));
            let input_press = input.clone();
            let input_up = input.clone();
            let _guard = device_state.on_key_down(move |key| {
                if let Some(mut keys) = input_press.try_lock() {
                    keys.push(*key);
                    // println!("Down: {:#?} keys={:?}", key, keys);

                    for ele in &global_hot_keys {
                        // let target = [Keycode::Numpad5, Keycode::LControl];
                        if keys.check_key_press(&ele.keys) {
                            if let Some(mut command) = command.try_lock() {
                                command.push(MyCommand::Cmd(ele.message.clone()));
                                // println!("cmd={:?}", command);
                            }
                        }
                    }
                };
            });
            let _guard = device_state.on_key_up(move |key| {
                // println!("Up: {:#?}", key);
                if let Some(mut keys) = input_up.try_lock() {
                    keys.remove(*key);
                }
            });

            loop {
                thread::sleep(Duration::from_secs(3600));
            }
        });
    }

    pub fn handle_key(&mut self, key: Key<SmolStr>, modifiers: Modifiers) -> Option<Message> {
        let key = key.as_ref();
        let is_ctrl = modifiers == Modifiers::CTRL;

        if let Key::Character(key) = key {
            match key {
                "a" => Some(Message::PlayDetail),
                "f" => Some(Message::ChangeTab(Tab::Like)),
                "l" => Some(Message::ChangeTab(Tab::List)),
                "s" => Some(Message::ChangeTab(Tab::Option)),
                "h" => Some(Message::ChangeTab(Tab::Home)),
                _ => None
            }
        } else if let Key::Named(n) = key {
            match n {
                Named::ArrowUp if is_ctrl => Some(Message::ChangeConfig(
                    ConfigMessage::ChangeVolume(self.audio.volume() + 0.01),
                )),
                Named::ArrowDown if is_ctrl => Some(Message::ChangeConfig(
                    ConfigMessage::ChangeVolume(self.audio.volume() - 0.01),
                )),
                Named::ArrowRight if is_ctrl => {
                    Some(Message::SongControl(SongControl::PlayNext(true)))
                }
                Named::ArrowLeft if is_ctrl => {
                    Some(Message::SongControl(SongControl::PlayNext(false)))
                }
                Named::Space => Some(Message::SongControl(SongControl::PlayOrPause)),
                Named::Escape => Some(Message::ToggleEsc),
                Named::F11 => Some(Message::ToggleFullScreen),
                Named::Enter => Some(Message::ToggleEnter),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn toggl_esc(&mut self) -> Command<Message> {
        match self.status {
            Status::Tab => {
                if self.tab == Tab::Home {
                    self.update(Message::RequestExit)
                } else {
                    self.update(Message::ChangeTab(Tab::Home))
                }
            }
            Status::PlayDetial => self.update(Message::PlayDetail),
        }
    }
    pub fn toggl_full_screen(&mut self) -> Command<Message> {
        let id = window::Id::MAIN;
        // window::fetch_mode(id, |v| {
        //     match v {
        //         window::Mode::Windowed => window::change_mode(id, window::Mode::Fullscreen),
        //         window::Mode::Fullscreen => window::change_mode(id, window::Mode::Windowed),
        //         window::Mode::Hidden => Command::none(),
        //     }
        // });
        if self.app_control.full_screen {
            self.app_control.full_screen = false;
            window::change_mode(id, window::Mode::Windowed)
        } else {
            self.app_control.full_screen = true;
            window::change_mode(id, window::Mode::Fullscreen)
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Command<Message> {
        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(btn) => {
                    if btn == Button::Left {
                        self.app_control.press_left_mouse_key = true;
                    } else if btn == Button::Middle {
                        return self.change_desktop_lyric_decorations();
                    }
                }
                mouse::Event::ButtonReleased(btn) => {
                    if btn == Button::Left {
                        self.app_control.press_left_mouse_key = false;
                    }
                }
                mouse::Event::CursorMoved { position } => {
                    if self.app_control.hide_status && self.status == Status::PlayDetial {
                        self.app_control.hide_status_seconds = Some(Instant::now());
                        self.app_control.hide_status = false;
                    }
                    return self.move_desktop_lyric(position);
                }
                _ => {}
            },
            Event::Window(id, win_event) => match win_event {
                window::Event::Focused => self.is_focuse_desktop_lyric(id),
                window::Event::Resized { width, height } => {
                    self.setting.resize_windown(id, width, height)
                }
                window::Event::Closed => {
                    return self.try_close_desktop_lyric();
                }
                window::Event::Moved { x, y } => {
                    if let Some(lyric_id) = self.app_control.desktop_lyric_win_id {
                        if lyric_id == id {
                            self.setting.change_desktop_lyric_position(x, y);
                        }
                    }
                }
                _ => {}
            },
            Event::Touch(_) => {}
            Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key,
                    location: _,
                    modifiers,
                    text: _,
                } => {
                    if !self.key_modify.contains(&modifiers) {
                        self.key_modify.push(modifiers);
                    }

                    if let Some(msg) = self.handle_key(key, modifiers) {
                        return self.update(msg);
                    }
                }
                keyboard::Event::KeyReleased {
                    key: _,
                    location: _,
                    modifiers: _,
                } => self.key_modify.clear(),
                keyboard::Event::ModifiersChanged(_) => {}
            },
        }

        Command::none()
    }
}

#[derive(Debug)]
pub struct Input {
    // command: Arc<Mutex<Vec<MyCommand>>>,
    keys: Vec<Keycode>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            // command,
            keys: vec![],
        }
    }
    pub fn push(&mut self, key: Keycode) {
        if self.keys.contains(&key) {
            return;
        }
        self.keys.push(key);
    }

    pub fn remove(&mut self, key: Keycode) {
        if !self.keys.contains(&key) {
            return;
        }
        let mut index = -1;
        for (i, ele) in self.keys.to_vec().iter().enumerate() {
            if *ele == key {
                index = i as isize;
                break;
            }
        }

        if index >= 0 {
            self.keys.remove(index as usize);
        }
    }

    pub fn check_key_press(&self, target: &Vec<Keycode>) -> bool {
        let mut hash_map = HashMap::with_capacity(target.len());
        for key in target {
            hash_map.insert(key, false);
        }

        for key in &self.keys {
            if let Some(_) = hash_map.get(&key) {
                hash_map.insert(key, true);
            }
        }

        for key in hash_map.keys() {
            if let Some(hit) = hash_map.get(key) {
                if !hit {
                    return false;
                }
            }
        }
        // println!("hit {:?}", target);
        return true;
    }
}

#[derive(Debug, Clone)]
pub struct GolbalHotKey {
    pub keys: Vec<Keycode>,
    pub message: Message,
}

impl GolbalHotKey {
    pub fn new(message: Message, keys: Vec<Keycode>) -> Self {
        Self { keys, message }
    }

    pub fn defaut_list() -> Vec<Self> {
        let mut vec = vec![
            Self::new(
                Message::SongControl(SongControl::PlayNext(false)),
                vec![Keycode::LControl, Keycode::Numpad4],
            ),
            Self::new(
                Message::SongControl(SongControl::PlayNext(true)),
                vec![Keycode::LControl, Keycode::Numpad6],
            ),
            Self::new(
                Message::SongControl(SongControl::PlayOrPause),
                vec![Keycode::LControl, Keycode::Numpad5],
            ),
            Self::new(
                Message::DesktopLyricWindow,
                vec![Keycode::LControl, Keycode::NumpadDivide],
            ),
        ];

        let mut copy = vec![];
        for ele in &vec {
            let mut ele = ele.clone();
            let mut hit = false;
            for key in &mut ele.keys {
                match key {
                    Keycode::LControl => {
                        *key = Keycode::RControl;
                        hit = true;
                    },
                    Keycode::LAlt => {
                        *key = Keycode::RAlt;
                        hit = true;
                    },
                    Keycode::LShift => {
                        *key = Keycode::RShift;
                        hit = true;
                    },
                    Keycode::LMeta => {
                        *key = Keycode::RMeta;
                        hit = true;
                    },
                    Keycode::LOption => {
                        *key = Keycode::ROption;
                        hit = true;
                    },
                    _ => {}
                }
            }
            if hit {
                copy.push(ele);
            }
        }
        vec.append(&mut copy);

        vec
    }
}
