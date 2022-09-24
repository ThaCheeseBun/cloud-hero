use crate::{songentry::SongEntry, util};
use configparser::ini::Ini;
use midly::{MetaMessage, Smf};
use std::fs::File;
use std::io::prelude::*;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const VIDEO_EXTS: [&str; 6] = ["mp4", "avi", "webm", "vp8", "ogv", "mpeg"];
const METADATA_DEFAULTS: [&str; 7] = [
    "Unknown Name",
    "Unknown Artist",
    "Unknown Album",
    "Unknown Genre",
    "Unknown Year",
    "Unknown Charter",
    "Unknown Playlist",
];

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

fn read_ini(song: &mut SongEntry, p: &PathBuf) -> bool {
    let mut flag = false;

    let raw_text = util::string_from_file(&p);
    let mut config = Ini::new();
    let i = config.read(raw_text);
    if i.is_err() {
        println!("Could not parse \"song.ini\" at {:?}", p);
        return flag;
    }

    // if we have a proper song section
    if config.sections().contains(&String::from("song")) {
        // get all metadata
        song.metadata[0] = config.get("song", "name").unwrap_or(String::from(""));
        song.metadata[1] = config.get("song", "artist").unwrap_or(String::from(""));
        song.metadata[2] = config.get("song", "album").unwrap_or(String::from(""));
        song.metadata[3] = config.get("song", "genre").unwrap_or(String::from(""));
        song.metadata[4] = config.get("song", "year").unwrap_or(String::from(""));
        song.metadata[5] = config
            .get("song", {
                if config.get("song", "charter").is_some() {
                    "charter"
                } else {
                    "frets"
                }
            })
            .unwrap_or(String::from(""));

        // store all intensities
        song.intensities[8] = ini_get_int(&config, "song", "diff_band").unwrap_or(-1) as i8;
        song.intensities[0] = ini_get_int(&config, "song", "diff_guitar").unwrap_or(-1) as i8;
        song.intensities[2] = ini_get_int(&config, "song", "diff_rhythm").unwrap_or(-1) as i8;
        song.intensities[1] = ini_get_int(&config, "song", "diff_bass").unwrap_or(-1) as i8;
        song.intensities[6] = ini_get_int(&config, "song", "diff_drums").unwrap_or(-1) as i8;
        song.intensities[9] = ini_get_int(&config, "song", "diff_drums_real").unwrap_or(-1) as i8;
        song.intensities[7] = ini_get_int(&config, "song", "diff_keys").unwrap_or(-1) as i8;
        song.intensities[4] = ini_get_int(&config, "song", "diff_guitarghl").unwrap_or(-1) as i8;
        song.intensities[5] = ini_get_int(&config, "song", "diff_bassghl").unwrap_or(-1) as i8;

        song.preview_start =
            ini_get_int(&config, "song", "preview_start_time").unwrap_or(-1) as i32;
        song.icon_name = config
            .get("song", "icon")
            .unwrap_or(String::from(""))
            .to_lowercase();
        song.playlist_track =
            ini_get_int(&config, "song", "playlist_track").unwrap_or(16000) as i16;
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
        })
        .unwrap_or(16000) as i16;

        flag = true;
    } else {
        flag = false;
    }

    flag
}

/*const INST_2: [&str; 10] = [
    "none", // -1
    "guitar", // 0
    "bass", // 1
    "rhythm", // 2
    "guitar coop", // 3
    "guitar ghl", // 4
    "bass ghl", // 5
    "drums / drum", // 6
    "keys", // 7
    "band", // 8
];*/
fn read_mid(song: &mut SongEntry, buf: &[u8]) {
    let smf = Smf::parse(buf).unwrap();
    for i in 0..smf.tracks.len() {
        let mut diff_str = String::new();
        let mut diff_arr = [false; 4];
        for j in 0..smf.tracks[i].len() {
            match smf.tracks[i][j].kind {midly::TrackEventKind::Meta(MetaMessage::TrackName(m)) => {
                    let str = String::from_utf8_lossy(m).to_lowercase();
                    println!("{:?}", str);
                    match str.as_str() {
                        "part vocals" => {
                            song.lyrics = true;
                            break;
                        }
                        "part keys" | "part drum" | "part rhythm" | "part bass ghl"
                        | "part guitar coop" | "part bass" | "part guitar ghl" | "part guitar"
                        | "part drums" => {
                            diff_str = str;
                        }
                        _ => {
                            break;
                        }
                    }
                }
                midly::TrackEventKind::Midi {
                    channel: _,
                    message: midly::MidiMessage::NoteOn { key, vel: _ },
                } => {
                    match key.as_int() {
                        58..=66 => diff_arr[0] = true,
                        70..=78 => diff_arr[1] = true,
                        82..=90 => diff_arr[2] = true,
                        94..=102 => diff_arr[3] = true,
                        _ => {}
                    }
                }
                _ => {}
            };
        }
        if diff_str != String::new() && diff_arr != [true, true, true, true] {
            println!("{:?}: {:?}, {:?}", diff_str, diff_arr, song.folder_path);
        }
        let index = {
            match diff_str.as_str() {
                "part guitar" => 0,
                "part bass" => 1,
                "part rhythm" => 2,
                "part guitar coop" => 3,
                "part guitar ghl" => 4,
                "part bass ghl" => 5,
                "part drums" | "part drum" => 6,
                "part keys" => 7,
                _ => -1
            }
        };
        for i in 0..diff_arr.len() {
            if !diff_arr[i] {
                continue;
            }
            let num = 1i64 << (index * 4 + (i as i64));
            if (song.charts & num) == num {
                continue;
            }
            song.charts |= num;
        }
    }
}

const INST: [&str; 10] = [
    "none",
    "single",
    "doublebass",
    "doublerhythm",
    "doubleguitar",
    "ghlguitar",
    "ghlbass",
    "drums",
    "keyboard",
    "band",
];
const DIFF: [&str; 4] = ["easy", "medium", "hard", "expert"];

fn read_chart(song: &mut SongEntry, buf: &[u8]) {
    let raw_text = util::string_from_bytes(buf);

    // quickly check to see if lyrics are present
    if raw_text.find("phrase_start").is_some() {
        song.lyrics = true;
    }

    // loop through all lines
    let mut section = String::new();
    for line in raw_text.lines() {
        let line = line.trim();

        // extract current section
        if line.starts_with('[') {
            section = line
                .get(1..line.len() - 1)
                .unwrap()
                .to_string()
                .to_lowercase();

            for d in 0..DIFF.len() {
                if section.starts_with(DIFF[d]) {
                    for i in 0..INST.len() {
                        if section.ends_with(INST[i]) {
                            let num = 1i64 << ((i - 1) * 4 + d);
                            if (song.charts & num) == num {
                                break;
                            }
                            song.charts |= num;
                            break;
                        }
                    }
                    break;
                }
            }
        }

        // if metadata section
        if section == "song" {
            // split key and value, ignore errors
            let arr_maybe = line.split_once('=');
            if arr_maybe.is_none() {
                continue;
            }
            let arr = arr_maybe.unwrap();

            // properly format key and value
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
                    if key.trim() == "TEMPO TRACK"
                        || key.trim() == ""
                        || key.trim() == "midi_export"
                    {
                        return;
                    }
                    song.metadata[0] = val;
                }
                _ => {}
            }
        }
    }
}

pub fn scan_folder(p: &Path) -> Vec<SongEntry> {
    let mut songs = vec![];
    let mut checksums = vec![];

    for entry in WalkDir::new(p) {
        let entry = entry.unwrap();

        if entry.path().extension().unwrap_or(&OsStr::new("")) == OsStr::new("sng") {
            // todo
        } else if entry.file_type().is_dir() {
            let mut mid_flag = false;
            let mut chart_flag = false;
            let mut ini_flag = false;
            let mut video_flag = false;
            let mut chart_name = String::new();

            // scan current folder
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
                        chart_name = file.file_name().to_string_lossy().to_string();
                    } else if extension == "chart" {
                        chart_flag = true;
                        chart_name = file.file_name().to_string_lossy().to_string();
                    }
                } else if name == "song" && extension == "ini" {
                    ini_flag = true;
                } else if name == "video" && VIDEO_EXTS.contains(&&extension[..]) {
                    video_flag = true;
                }
            }

            // idk what this does but it works :tm:
            if !(!mid_flag && !chart_flag) || ini_flag {
                let s_path = entry.path();
                let mut song = SongEntry::default();

                song.folder_path = s_path.to_string_lossy().to_string();

                // skip if song.ini is invalid
                if !read_ini(&mut song, &s_path.join("song.ini")) {
                    continue;
                }

                // read all of the note data and metadata
                let notes_data = {
                    let mut f = File::open(s_path.join(&chart_name)).unwrap();
                    let mut d = vec![];
                    f.read_to_end(&mut d).unwrap();
                    d
                };

                // calcute md5 checksum for the data
                let check = md5::compute(&notes_data);
                // check for duplicates
                if checksums.contains(&check.0) {
                    println!("duplicate {:?}", s_path);
                    continue;
                }
                song.checksum = check.0;
                checksums.push(check.0);

                // reuse the data to read all needed metadata
                if mid_flag {
                    read_mid(&mut song, &notes_data);
                    //break;
                } else if chart_flag {
                    read_chart(&mut song, &notes_data);
                }

                // add some stuffs
                song.video_background = video_flag;
                song.chart_name = chart_name;
                song.date_added = 0; //DateTime.Now.Date;

                // fix empty metadata
                for m in 0..song.metadata.len() {
                    if song.metadata[m].trim() == "" {
                        song.metadata[m] = String::from(METADATA_DEFAULTS[m]);
                    }
                }

                // set last metadata element and top_level_playlist
                if song.top_level_playlist == String::from("") {
                    // populate element
                    let mut tempdata = song.folder_path.clone();
                    if tempdata.bytes().nth(tempdata.len() - 1).unwrap() == '\\' as u8 {
                        tempdata.remove(tempdata.len() - 1);
                    }
                    tempdata =
                        String::from(tempdata.get(p.to_string_lossy().len()..).unwrap_or(""));
                    let mut num = -1;
                    if tempdata.len() > 0 {
                        tempdata.remove(0);
                        num = tempdata.rfind("\\").unwrap_or(0) as i32;
                    }
                    song.metadata[6] = {
                        if num == -1 {
                            String::from("")
                        } else {
                            String::from(tempdata.get(..num as usize).unwrap_or(""))
                        }
                    };
                    // create top_level_playlist
                    if song.metadata[6] != String::from("") {
                        let temppos = song.metadata[6].find('\\');
                        song.top_level_playlist = {
                            if temppos.is_none() {
                                song.metadata[6].clone()
                            } else {
                                String::from(song.metadata[6].get(..temppos.unwrap()).unwrap())
                            }
                        };
                    }
                    song.sub_playlist = String::from("");
                } else {
                    song.metadata[6] = format!("{}{}", song.top_level_playlist, {
                        if song.sub_playlist != String::from("") {
                            format!("\\{}", song.sub_playlist)
                        } else {
                            String::from("")
                        }
                    })
                }

                songs.push(song);
                //println!("{:?}", songs.len());
            }
        }
    }

    println!("{:?}", songs.len());
    songs
}
