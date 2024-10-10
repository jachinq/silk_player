#![allow(unused)]

use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{DirEntry, File, OpenOptions},
    io::{self, BufWriter, Read, Write},
    os::windows::fs::MetadataExt,
    path::Path,
    result::Result,
    time::{Duration, SystemTime},
};

use chrono::Local;
use iced::{
    advanced::graphics::{core::SmolStr, text::cache::Key},
    keyboard::Modifiers,
    widget::shader::wgpu::util,
    window::icon,
};
use image::{DynamicImage, GenericImageView, Pixel};
use music_tag::audio::MusicTag;
use serde::{Deserialize, Serialize};
use std::{fs, path};

use crate::{MusicInfo, Setting};

static ICON: &[u8] = include_bytes!("../assets/icon.ico");

pub fn current_dir() -> String {
    match std::env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(_) => ".".to_string(),
    }
}

pub fn data_dir() -> String {
    format!("{}/data", current_dir())
}
pub fn cache_dir() -> String {
    format!("{}/cache", data_dir())
}
pub fn log_dir() -> String {
    format!("{}/log", data_dir())
}

/// 将图片转为窗口图标
pub fn app_icon() -> Option<icon::Icon> {
    match image::load_from_memory(ICON) {
        Ok(img) => {
            let (w1, h1) = img.dimensions();
            let img_file = img.to_rgba8();
            let ico = icon::from_rgba(img_file.to_vec(), w1, h1);
            let ico_file = match ico {
                Ok(file) => file,
                Err(e) => panic!("error is {}", e),
            };
            Some(ico_file)
        }
        Err(err) => {
            log_err(format!("load icon error;err={}", err));
            None
        }
    }
}

/// 文件是否存在 可以判断 路径是否存在，文件、文件夹都可以
pub fn file_exist(path: &str) -> bool {
    path::Path::new(path).exists()
}

/// 读取目录
pub fn read_dir(path: &str) -> Vec<DirEntry> {
    let mut vec = vec![];
    if !file_exist(path) {
        return vec;
    }

    if let Ok(paths) = fs::read_dir(path) {
        for path in paths {
            if let Ok(path) = path {
                log(format!("init music dir: {}", path.path().display()));
                vec.push(path);
            }
        }
    }
    vec
}

/// 读取文件
pub fn read_file(path: String) {
    if !file_exist(&path) {
        return;
    }
    if let Ok(file) = fs::File::open(path) {}
}

//遍历dir目录，找出修改日期距离当前超过age天的文件名称，存入file_list中
fn visit_dir(dir: &Path, file_list: &mut Vec<String>, age: u64) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, file_list, age)?;
            } else {
                let file_matedata = fs::metadata(entry.path())?;
                let modify_time = file_matedata.modified()?;
                if modify_time + Duration::from_secs(age * 24 * 60 * 60) < SystemTime::now() {
                    file_list.push(entry.path().to_str().unwrap().to_string());
                }
            }
        }
    }
    Ok(())
}

//遍历dir目录，找出空目录（内部无文件，无目录）
fn get_empty_dir(dir: &Path, dir_list: &mut Vec<String>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let read_dir = fs::read_dir(dir)?;
    let cnt = read_dir.count();
    if cnt == 0 {
        dir_list.push(dir.to_str().unwrap().to_owned());
        return Ok(());
    }

    let read_dir = fs::read_dir(dir)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            get_empty_dir(path.as_path(), dir_list)?;
        }
    }
    Ok(())
}

//遍历dir目录，找出空目录（内部无文件，无目录）
pub fn get_files(dir: &str, file_list: &mut Vec<String>) -> io::Result<()> {
    let dir = path::Path::new(dir);
    if !dir.is_dir() {
        return Ok(());
    }
    let read_dir = fs::read_dir(dir)?;
    let cnt = read_dir.count();
    if cnt == 0 {
        return Ok(());
    }

    let read_dir = fs::read_dir(dir)?;
    for entry in read_dir {
        let path = entry?.path();
        if path.is_dir() {
            let path = path.as_path().to_str().unwrap().to_string();
            // log_debug(format!("path: {}", path));
            get_files(&path, file_list)?;
        } else {
            let file_name = path.file_name().unwrap().to_owned().into_string().unwrap();
            if !file_name.ends_with(".mp3")
                && !file_name.ends_with(".flac")
                && !file_name.ends_with(".m4a")
            // && !file_name.ends_with(".ogg")
            {
                continue;
            }
            let path = path.to_str().unwrap().to_owned();
            file_list.push(path);
        }
    }
    Ok(())
}

pub fn log_time(arg: impl Debug) {
    let time = chrono::Local::now();
    println!("{} {:?}", time.format("%m-%d %H:%M:%S%.3f"), arg);
}
pub fn log(arg: impl Debug) {
    do_log("infos", arg);
}
pub fn log_debug(arg: impl Debug) {
    do_log("debug", arg);
}

pub fn log_err(arg: impl Debug) {
    do_log("error", arg);
}

const TIME_FMT: &str = "%m-%d %H:%M:%S";
fn do_log(level: &str, arg: impl Debug) {
    // let fmt = "%Y年%m月%d日 %H:%M:%S";
    let time = chrono::Local::now();
    let mut arg = format!("[{}] {} {:?}", level, time.format(TIME_FMT), arg);
    if level.eq("info") {
        arg = arg
            .replace("\\\\", "/")
            .replace("\\\"", "")
            .replace("\"", "");
    }

    let fmt = "%Y-%m";
    let now = Local::now().format(fmt);
    let path = format!("{}/{}.txt", log_dir(), now);
    if !file_exist(&path) {
        let _create = std::fs::File::create(&path);
    }
    if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
        let _ = writeln!(file, "{}", arg);
    }
}

pub fn play_time(secs: f32) -> String {
    let (hour, minute, second) = {
        let time = secs as u64;
        (time / (60 * 60), time / 60 % 60, time % 60)
    };

    if hour > 0 {
        return format!("{:0>2}:{:0>2}:{:0>2}", hour, minute, second);
    }
    format!("{:0>2}:{:0>2}", minute, second)
}

/// 检查路径是否存在，不存在则创建路径
pub fn check_dir_and_create(path: &str) {
    if file_exist(path) {
        return;
    }

    if let Err(err) = fs::create_dir(path) {
        log_err(format!("create path {} error: {}", path, err));
    } else {
        log(format!("create path {} ok", path));
    }
}

pub fn save_file_from_buffer(path: String, buf: &[u8]) -> std::io::Result<()> {
    if file_exist(&path) {
        gen_album_thumbnail(path.clone());
        return Ok(());
    }
    let file = File::create(path.clone())?;
    BufWriter::new(file).write_all(&buf)?;
    gen_album_thumbnail(path.clone());
    Ok(())
}
pub fn write_file(path: String, data: String) -> std::io::Result<()> {
    if file_exist(&path) {
        return Ok(());
    }

    let file = File::create(path.clone())?;
    BufWriter::new(file).write_all(&data.as_bytes())?;
    Ok(())
}

fn get_rgbau8_key(channels: &[u8], map: &HashMap<String, usize>) -> String {
    if channels.len() < 4 {
        return String::new();
    }
    let [r, g, b, a] = [channels[0], channels[1], channels[2], channels[3]];

    for key in map.keys() {
        let color = Color::new(key, 0);
        let r1 = color.r;
        let g1 = color.g;
        let b1 = color.b;
        if range_channel(r, r1) == r1 && range_channel(g, g1) == g1 && range_channel(b, b1) == b1 {
            return key.to_string();
        }
    }

    // 0-255 10

    format!("{r}_{g}_{b}_{a}")
}
fn range_channel(new_val: u8, old_val: u8) -> u8 {
    let fuzzy = 50; // 误差在 (-fuzzy, +fuzzy] 之间，看作同个颜色
    if old_val < fuzzy {
        // 边界处理 左区间 o=4 n=8
        if new_val <= old_val + fuzzy {
            return old_val;
        } else {
            return new_val;
        }
    }
    if old_val >= (255 - fuzzy) {
        // 边界处理 右区间 o=254 n=250
        if new_val > old_val - fuzzy {
            return old_val;
        } else {
            return new_val;
        }
    }
    if old_val - fuzzy < new_val && new_val <= old_val + fuzzy {
        old_val
    } else {
        new_val
    }
}

pub fn get_colors_vec(album: &str) -> Vec<(u8, u8, u8, u8)> {
    let path = get_color_path(album);
    if !file_exist(&path) {
        extract_album_color(album.to_string());
        return vec![];
    }
    if let Ok(data) = std::fs::read_to_string(&path) {
        let result: Result<Vec<Color>, serde_json::Error> = serde_json::from_str(&data);
        if let Ok(data) = result {
            return data
                .iter()
                .map(|item| (item.r, item.g, item.b, item.a))
                .collect();
        }
    }
    return vec![];
}

#[derive(Debug, Serialize, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    c: usize,
}
impl Color {
    fn new(key: &str, count: usize) -> Self {
        let rgba: Vec<u8> = key
            .split("_")
            .into_iter()
            .map(|v| if let Ok(value) = v.parse() { value } else { 0 })
            .collect();

        Color {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
            c: count,
        }
    }
}

pub fn get_str_value<P: ToString>(value: Option<P>, default: &str) -> String {
    if let Some(value) = value {
        value.to_string()
    } else {
        default.to_string()
    }
}

pub fn batch_list<T: Clone>(list: &Vec<T>, task_num: usize) -> Vec<Vec<T>> {
    // let task_num = 10;
    let len = list.len();
    let single_task_num = len / task_num;
    let mut result = Vec::with_capacity(task_num);
    let mut index = 0;
    for i in 0..task_num {
        let mut task_list = Vec::with_capacity(single_task_num);
        for j in 0..single_task_num {
            task_list.push(list[index].clone());
            index += 1;
        }
        result.push(task_list);
    }
    if len % task_num != 0 {
        for value in &list[index..len] {
            result[task_num - 1].push(value.clone())
        }
    }
    result
}

fn remove_special_char(mut value: String) -> String {
    // \ / : * ? " < > |
    let mut special_char = Vec::with_capacity(8);
    special_char.push("\\");
    special_char.push("/");
    special_char.push(":");
    special_char.push("*");
    special_char.push("?");
    special_char.push("<");
    special_char.push(">");
    special_char.push("|");
    special_char.push("\"");
    for char in special_char {
        if value.contains(char) {
            value = value.replace(char, "");
        }
    }
    value
}

/// 获取专辑封面图片路径
pub fn get_album_path(path: &str) -> String {
    if let Ok(tag) = music_tag::audio::MusicTag::read_from_path(&path) {
        get_album_path_by_tag(&tag)
    } else {
        format!("{}/assets/default.png", current_dir())
    }
}
/// 获取专辑封面图片路径
pub fn get_album_path_by_tag(tag: &MusicTag) -> String {
    let title = get_str_value(tag.title(), "");
    let artist = get_str_value(tag.artist(), "");
    let mut album = get_str_value(tag.album(), "");
    if album.is_empty() {
        album = title.clone();
    }
    album = remove_special_char(album);

    if let Some(artwork) = tag.artwork() {
        let fmt = &artwork.fmt;
        let format = match fmt {
            music_tag::audio::ImgFmt::JPEG => image::ImageFormat::Jpeg,
            music_tag::audio::ImgFmt::PNG => image::ImageFormat::Png,
        };

        let fmt = &format.to_mime_type().replace("image/", "");
        let file_path = format!("{}.{}", album, fmt);
        let path = format!("{}/{}", cache_dir(), file_path);
        album = path.to_string();
    } else {
        album = format!("{}/assets/default.png", current_dir());
    }
    album
}

/// 获取专辑封面对应的模糊图片路径
pub fn get_blur_path(album: &str) -> String {
    format!("{}_blur.png", album)
}

/// 获取专辑封面对应的缩略图路径
pub fn get_thumbnail_path(album: &str) -> String {
    format!("{}_thumbnail.png", album)
}
/// 获取专辑封面对应的颜色提取数据路径
pub fn get_color_path(album: &str) -> String {
    format!("{}_color.json", album)
}

/// 生成专辑封面缩略图
pub fn gen_album_thumbnail(album: String) {
    let album450 = album.clone();
    std::thread::spawn(move || {
        if !file_exist(&album450) {
            return;
        }

        let thumbnail_size = 450;
        if let Ok(open) = image::open(&album450) {
            if open.dimensions().0 <= thumbnail_size {
                return;
            }
            let _ = open
                .thumbnail(thumbnail_size, thumbnail_size)
                .save(album450);
        }
    });

    let tb_path_128 = get_thumbnail_path(&album);
    if file_exist(&tb_path_128) {
        return;
    }
    std::thread::spawn(move || {
        do_gen_album_thumbnail(album);
    });
}

/// 保存专辑封面缩略图
fn do_gen_album_thumbnail(album_path: String) -> image::ImageResult<()> {
    let tb_path = get_thumbnail_path(&album_path);
    if album_path.is_empty() || file_exist(&tb_path) {
        return Ok(());
    }
    let thumbnail_size = 128;
    let open = image::open(album_path)?;
    open.thumbnail(thumbnail_size, thumbnail_size).save(tb_path)
}

/// 提取专辑封面缩略图
pub fn extract_album_color(album: String) {
    let color_path = get_color_path(&album);
    if album.is_empty() || file_exist(&color_path) {
        return;
    }

    std::thread::spawn(move || {
        let tb_path = get_thumbnail_path(&album);
        match do_gen_album_thumbnail(album) {
            Ok(_) => {
                if let Ok(di) = image::open(tb_path) {
                    do_extract_album_color(color_path, di);
                }
            }
            Err(err) => log_err(format!(
                "do gen album error;path={} error = {}",
                tb_path, err
            )),
        }
    });
}

/// 执行提取并保存
fn do_extract_album_color(file_path: String, thumbnail: DynamicImage) {
    let mut count: HashMap<String, usize> = HashMap::new();
    // 读取像素点，转换为rgba颜色通道
    for pixel in thumbnail.to_rgba8().pixels() {
        let channels = pixel.channels();
        let key = get_rgbau8_key(channels, &count);
        if let Some(value) = count.get(&key) {
            count.insert(key, value + 1);
        } else {
            count.insert(key, 1);
        }
    }
    let mut top = vec![];
    for key in count.keys() {
        let value = count.get(key).unwrap();
        top.push((key, value))
    }
    top.sort_by(|a, b| {
        if a.1 == b.1 {
            std::cmp::Ordering::Equal
        } else if a.1 < b.1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    // top.sort_by(|a, b| a.1.cmp(b.1));
    let mut colors = Vec::with_capacity(8);
    for (i, item) in top.iter().enumerate() {
        if i >= 8 {
            break;
        }
        colors.push(Color::new(item.0, *item.1))
    }
    if let Ok(to_string) = serde_json::to_string(&colors) {
        write_file(file_path, to_string);
    }
}

/// 生成专辑封面模糊背景图
pub fn gen_album_blur(album: String) {
    let blur_path = get_blur_path(&album);
    if file_exist(&blur_path) {
        return;
    }

    std::thread::spawn(move || {
        let tb_path = get_thumbnail_path(&album);
        if let Ok(_) = do_gen_album_thumbnail(album) {
            if let Ok(di) = image::open(tb_path) {
                di.blur(10.0).save(blur_path);
            }
        }
    });
}

pub fn get_title(music_info: &MusicInfo) -> String {
    let title = if music_info.title.is_empty() {
        &music_info.file_name
    } else {
        &music_info.title
    };
    title.to_string()
}

pub fn get_parent_path(path: &str) -> String {
    if file_exist(path) {
        let dir = path::Path::new(path);
        if let Some(parent) = dir.parent() {
            if let Some(path) = parent.to_str() {
                return path.to_string();
            }
        }
    }


    path.to_string()
}
