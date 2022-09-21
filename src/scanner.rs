use crate::{util, songentry::SongEntry};
use configparser::ini::Ini;
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf}
};
use walkdir::WalkDir;

const VIDEO_EXTS: [&str; 6] = [".mp4", ".avi", ".webm", ".vp8", ".ogv", ".mpeg"];

fn ini_get_int(c: &Ini, s: &str, k: &str) -> Option<i64> {
    let raw = c.get(s, k);
    if raw.is_none() {
        return None;
    }
    let parsed = raw.unwrap().parse::<i64>();
    if parsed.is_err() {
        return None;
    }
    Some(parsed.unwrap())
}

fn ini_get_bool(c: &Ini, s: &str, k: &str) -> Option<bool> {
    let raw = c.get(s, k);
    if raw.is_none() {
        return None;
    }
    match raw.unwrap().to_lowercase().as_str() {
        "true" | "yes" | "t" | "y" | "1" | "on" => Some(true),
        "false" | "no" | "f" | "n" | "0" | "off" => Some(false),
        _ => None,
    }
}

fn read_ini(song: &mut SongEntry, p: PathBuf) -> bool {
    let mut flag = false;

    let raw_text = util::string_from_file(&p);
    let mut config = Ini::new();
    let i = config.read(raw_text);

    if i.is_err() {
        println!("{}", i.err().unwrap());
        println!("VERYBAD: {:?}", p);
        return flag;
    }

    if config.sections().contains(&String::from("song")) {
        song.metadata[0] = config.get("song", "name").unwrap_or(String::from(""));
        song.metadata[1] = config.get("song", "artist").unwrap_or(String::from(""));
        song.metadata[2] = config.get("song", "album").unwrap_or(String::from(""));
        song.metadata[3] = config.get("song", "genre").unwrap_or(String::from(""));
        song.metadata[4] = config.get("song", "year").unwrap_or(String::from(""));

        song.intensities[8] = ini_get_int(&config, "song", "diff_band").unwrap_or(-1) as i8;
        song.intensities[0] = ini_get_int(&config, "song", "diff_guitar").unwrap_or(-1) as i8;
        song.intensities[2] = ini_get_int(&config, "song", "diff_rhythm").unwrap_or(-1) as i8;
        song.intensities[1] = ini_get_int(&config, "song", "diff_bass").unwrap_or(-1) as i8;
        song.intensities[6] = ini_get_int(&config, "song", "diff_drums").unwrap_or(-1) as i8;
        song.intensities[9] = ini_get_int(&config, "song", "diff_drums_real").unwrap_or(-1) as i8;
        song.intensities[7] = ini_get_int(&config, "song", "diff_keys").unwrap_or(-1) as i8;
        song.intensities[4] = ini_get_int(&config, "song", "diff_guitarghl").unwrap_or(-1) as i8;
        song.intensities[5] = ini_get_int(&config, "song", "diff_bassghl").unwrap_or(-1) as i8;

        song.preview_start = ini_get_int(&config, "song", "preview_start_time").unwrap_or(-1) as i32;
        song.icon_name = config
            .get("song", "icon")
            .unwrap_or(String::from(""))
            .to_lowercase();
        song.playlist_track = ini_get_int(&config, "song", "playlist_track").unwrap_or(16000) as i16;
        song.modchart = ini_get_bool(&config, "song", "modchart").unwrap_or(false);
        song.song_length = ini_get_int(&config, "song", "song_length").unwrap_or(0) as i32;
        song.force_pro_drums = ini_get_bool(&config, "song", "pro_drums").unwrap_or(false);
        song.force_five_lane = ini_get_bool(&config, "song", "five_lane_drums").unwrap_or(false);
        song.top_level_playlist = config
            .get("song", "playlist")
            .unwrap_or(String::from(""))
            .to_lowercase();
        song.sub_playlist = config
            .get("song", "sub_playlist")
            .unwrap_or(String::from(""))
            .to_lowercase();

        song.album_track = ini_get_int(&config, "song", {
            if config.get("song", "album_track").is_some() {
                "album_track"
            } else {
                "track"
            }
        }).unwrap_or(16000) as i16;
        
        song.metadata[5] = config.get("song", {
            if config.get("song", "charter").is_some() {
                "charter"
            } else {
                "frets"
            }
        }).unwrap_or(String::from(""));

        flag = true;
    } else {
        flag = false;
    }

    flag
}

fn read_chart(song: &mut SongEntry, p: PathBuf) -> bool {
    let raw_text = util::string_from_file(&p);

    for line in raw_text.lines() {
        let line = line.trim();
        if line == "}" {
            break;
        }

        let arr_maybe = line.split_once('=');
        if arr_maybe.is_none() {
            continue;
        }
        let arr = arr_maybe.unwrap();

        let key = arr.0.to_lowercase();
        let val = arr.1.replace("\"", "").trim().to_string();

        match key.trim() {
            "charter" => song.metadata[5] = val,
            "artist" => song.metadata[1] = val,
            //"offset" => song
            "genre" => song.metadata[3] = val,
            "album" => song.metadata[2] = val,
            "year" => song.metadata[4] = val.replace(", ", ""),
            "name" => {
                if key.trim() == "TEMPO TRACK" || key.trim() == "" || key.trim() == "midi_export" {
                    return false;
                }
                song.metadata[0] = val;
            },
            _ => {}
        }
    }

    return song.metadata[0] != ""
}

fn create_song_entry(p: &Path) -> Option<SongEntry> {
    let mut song = SongEntry::default();

    let ini_path = p.join("song.ini");
    let chart_path = p.join("notes.chart");

    song.folder_path = p.to_string_lossy().to_string();

    if !read_ini(&mut song, ini_path) {
        if read_chart(&mut song, chart_path) {
            return Some(song);
        }
    } else {
        return Some(song);
    }

    None
}

pub fn scan_folder() -> Vec<SongEntry> {
    let mut aaaa = 0;
    let mut songs = vec![];

    for entry in WalkDir::new("E:\\Spel\\Annat\\Clone Hero\\Songs") {
        let entry = entry.unwrap();

        if entry.path().extension().unwrap_or(&OsStr::new("")) == OsStr::new("sng") {
            // todo
        } else if entry.file_type().is_dir() {
            let mut mid_flag = false;
            let mut chart_flag = false;
            let mut ini_flag = false;
            let mut video_flag = false;
            let mut chart_name: Option<OsString> = None;

            for file in fs::read_dir(entry.path()).unwrap() {
                let file = file.unwrap();
                let name = file
                    .path()
                    .file_stem()
                    .unwrap_or(&OsStr::new(""))
                    .to_string_lossy()
                    .to_lowercase();
                let extension = file
                    .path()
                    .extension()
                    .unwrap_or(&OsStr::new(""))
                    .to_string_lossy()
                    .to_lowercase();
                if name == "notes" {
                    if extension == "mid" {
                        mid_flag = true;
                        chart_name = Some(file.file_name());
                    } else if extension == "chart" {
                        chart_flag = true;
                        chart_name = Some(file.file_name());
                    }
                } else if name == "song" && extension == "ini" {
                    ini_flag = true;
                } else if name == "video"/* && VIDEO_EXTS.contains(&&extension[..])*/ {
                    video_flag = true;
                }
            }

            if !(!mid_flag && !chart_flag) || ini_flag {
                let song = create_song_entry(entry.path());

                if song.is_some() {
                    aaaa = aaaa + 1;
                }

                if song.is_none() {
                    continue;
                }
                let mut song = song.unwrap();

                let num = {
                    if mid_flag {
                        2
                    } else {
                        if chart_flag {
                            1
                        } else {
                            0
                        }
                    }
                };

                song.video_background = video_flag;
                song.chart_name = chart_name.unwrap().to_string_lossy().to_string();

                /*if (num > 0 && this.method_13(songEntry2))
                {*/
                    //songEntry2.method_4();
                    song.date_added = 0;//DateTime.Now.Date;
                    songs.push(song);
                    /*if (this.bool_0 || (num == 1 && !flag3))
                    {
                        songEntry2.method_10(num == 1 && !flag3);
                    }*/
                //}
            }
        }
    }

    println!("{:?}", aaaa);
    songs
}
