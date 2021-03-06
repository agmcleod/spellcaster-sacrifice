use std::env;
use std::fs::{create_dir, File};
use std::io::prelude::Read;
use std::io::BufReader;
use std::io::Result;
use std::path::{Path, PathBuf};

use dirs;
use gfx::texture::Mipmap;
use image;
use rodio::{decoder::Decoder, Decoder as SoundDecoder};
use serde_json;
use tiled::{parse_file, Map};

use crate::settings::Settings;

pub type Texture<R: gfx::Resources> = gfx::handle::ShaderResourceView<R, [f32; 4]>;

pub fn gfx_load_texture<F, R>(path: &str, factory: &mut F) -> (Texture<R>, u16, u16)
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    use gfx::format::Srgba8;
    let path = get_exe_path().join(path);
    let img = image::open(path).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let (_, view) = factory
        .create_texture_immutable_u8::<Srgba8>(kind, Mipmap::Allocated, &[&img])
        .unwrap();
    (view, width as u16, height as u16)
}

pub fn create_sound(sound_file_path: &str) -> Decoder<BufReader<File>> {
    let audio_file = File::open(&Path::new(&get_exe_path().join(sound_file_path))).unwrap();
    SoundDecoder::new(BufReader::new(audio_file)).unwrap()
}

pub fn load_map(path: &str) -> Map {
    if let Ok(map) = parse_file(get_exe_path().join(path).as_path()) {
        map
    } else {
        panic!("Could not load tiled map at {:?}", path);
    }
}

pub fn read_text_from_file(path: &str) -> Result<String> {
    let path = get_exe_path().join(path);
    read_text_from_path(path)
}

pub fn read_text_from_path(path: PathBuf) -> Result<String> {
    let mut text = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut text)?;
    Ok(text)
}

pub fn get_exe_path() -> PathBuf {
    match env::current_exe() {
        Ok(mut p) => {
            p.pop();
            p
        }
        Err(_) => PathBuf::new(),
    }
}

#[cfg(target_os = "linux")]
pub fn get_settings_path() -> PathBuf {
    if let Some(home_dir) = dirs::home_dir() {
        if !home_dir.join("EnergyGrid").exists() {
            create_dir(home_dir.join("EnergyGrid")).unwrap();
        }
        home_dir.join("EnergyGrid").join("settings.json")
    } else {
        panic!("Could not find $HOME");
    }
}

#[cfg(target_os = "windows")]
pub fn get_settings_path() -> PathBuf {
    get_settings_path_win_mac()
}

#[cfg(target_os = "macos")]
pub fn get_settings_path() -> PathBuf {
    get_settings_path_win_mac()
}

fn get_settings_path_win_mac() -> PathBuf {
    get_exe_path().join("settings.json")
}

pub fn load_settings() -> Settings {
    let settings_path = get_settings_path();
    if settings_path.exists() {
        let settings_text = read_text_from_path(settings_path).unwrap();
        serde_json::from_str(settings_text.as_ref()).unwrap()
    } else {
        Settings::default()
    }
}
