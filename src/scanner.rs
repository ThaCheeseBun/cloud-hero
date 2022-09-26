use crate::{songentry::SongEntry, util};
use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};
use std::fs::File;
use std::io::prelude::*;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const VIDEO_EXTS: [&str; 6] = ["mp4", "avi", "webm", "vp8", "ogv", "mpeg"];
const IMAGE_EXTS: [&str; 3] = ["png", "jpg", "jpeg"];
const AUDIO_EXTS: [&str; 4] = ["ogg", "mp3", "wav", "opus"];
const METADATA_DEFAULTS: [&str; 7] = [
    "Unknown Name",
    "Unknown Artist",
    "Unknown Album",
    "Unknown Genre",
    "Unknown Year",
    "Unknown Charter",
    "Unknown Playlist",
];
const AUDIO_FILES: [&str; 14] = ["guitar", "bass", "rhythm", "vocals", "vocals_1", "vocals_2", "drums", "drums_1", "drums_2", "drums_3",
"drums_4", "keys", "song", "crowd"];

#[derive(Copy, Clone, PartialEq)]
enum Instrument {
    None = -1,
    Guitar = 0,
    Bass = 1,
    Rhythm = 2,
    GuitarCoop = 3,
    GHLGuitar = 4,
    GHLBass = 5,
    Drums = 6,
    Keys = 7,
    Band = 8,
    ProDrums = 9,
}

fn ini_get_bool(v: &str) -> Option<bool> {
    match v.to_lowercase().as_str() {
        "true" | "yes" | "t" | "y" | "1" | "on" => Some(true),
        "false" | "no" | "f" | "n" | "0" | "off" => Some(false),
        _ => None,
    }
}

fn read_ini(song: &mut SongEntry, p: &PathBuf) -> bool {
    let mut flag = false;

    let raw_text = util::string_from_file(&p);
    let mut section = String::new();

    for line in raw_text.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            let end_pos = line.find(']').unwrap_or(line.len() - 1);
            section = line.get(1..end_pos).unwrap().to_lowercase();
            continue;
        }

        // split key and value, ignore errors
        let arr_ = line.split_once('=');
        if arr_.is_none() {
            continue;
        }
        let arr = arr_.unwrap();

        // properly format key and value
        let key = arr.0.trim().to_lowercase();
        let val = arr.1.trim().to_string();

        if section == "song" {
            flag = true;

            match key.as_str() {
                "name" => song.metadata[0] = val,
                "artist" => song.metadata[1] = val,
                "album" => song.metadata[2] = val,
                "genre" => song.metadata[3] = val,
                "year" => song.metadata[4] = val,
                "charter" | "frets" => song.metadata[5] = val,

                "diff_band" => song.intensities[8] = val.parse::<i8>().unwrap_or(-1),
                "diff_guitar" => song.intensities[0] = val.parse::<i8>().unwrap_or(-1),
                "diff_rhythm" => song.intensities[2] = val.parse::<i8>().unwrap_or(-1),
                "diff_bass" => song.intensities[1] = val.parse::<i8>().unwrap_or(-1),
                "diff_drums" => song.intensities[6] = val.parse::<i8>().unwrap_or(-1),
                "diff_drums_real" => song.intensities[9] = val.parse::<i8>().unwrap_or(-1),
                "diff_keys" => song.intensities[7] = val.parse::<i8>().unwrap_or(-1),
                "diff_guitarghl" => song.intensities[4] = val.parse::<i8>().unwrap_or(-1),
                "diff_bassghl" => song.intensities[5] = val.parse::<i8>().unwrap_or(-1),

                "preview_start_time" => song.preview_start = val.parse::<i32>().unwrap_or(-1),
                "icon" => song.icon_name = val.to_lowercase(),
                "playlist_track" => song.playlist_track = val.parse::<i16>().unwrap_or(16000),
                "modchart" => song.modchart = ini_get_bool(&val).unwrap_or(false),
                "song_length" => song.song_length = val.parse::<i32>().unwrap_or(0),
                "pro_drums" => song.force_pro_drums = ini_get_bool(&val).unwrap_or(false),
                "five_lane_drums" => song.force_five_lane = ini_get_bool(&val).unwrap_or(false),
                "playlist" => song.top_level_playlist = val.to_lowercase(),
                "sub_playlist" => song.sub_playlist = val.to_lowercase(),

                "album_track" | "track" => song.album_track = val.parse::<i16>().unwrap_or(16000),
                _ => {}
            }
    
            // fix intensities
            song.intensities[3] = 0;
            if song.intensities[9] == -1 {
                song.intensities[9] = song.intensities[6];
            }
        }
    }

    flag
}

fn apply_charts(song: &mut SongEntry, inst: Instrument, flag: bool, diff: i64) {
    if inst == Instrument::Drums && (flag || song.force_pro_drums || song.force_five_lane) {
        let num = 1 << (Instrument::ProDrums as i64 * Instrument::GHLGuitar as i64 + diff);
        if !((song.charts & num) == num) {
            song.charts |= num;
        }
    }
    let num = 1 << (inst as i64 * Instrument::GHLGuitar as i64 + diff);
    if !((song.charts & num) == num) {
        song.charts |= num;
    }
}

fn read_midi(song: &mut SongEntry, buf: &[u8]) {
    let smf = Smf::parse(buf).unwrap();
    for i in 0..smf.tracks.len() {
        let mut inst = Instrument::None;
        let mut diff = [false; 4];
        let mut flag = false;

        for j in 0..smf.tracks[i].len() {
            match smf.tracks[i][j].kind {
                TrackEventKind::Meta(MetaMessage::TrackName(m)) => {
                    let str = String::from_utf8_lossy(m).to_lowercase();
                    match str.as_str() {
                        "part vocals" => {
                            song.lyrics = true;
                            break;
                        }
                        "part guitar" | "t1 gems" => inst = Instrument::Guitar,
                        "part bass" => inst = Instrument::Bass,
                        "part rhythm" => inst = Instrument::Rhythm,
                        "part guitar coop" => inst = Instrument::GuitarCoop,
                        "part guitar ghl" => inst = Instrument::GHLGuitar,
                        "part bass ghl" => inst = Instrument::GHLBass,
                        "part drums" | "part drum" => inst = Instrument::Drums,
                        "part keys" => inst = Instrument::Keys,
                        _ => {
                            break;
                        }
                    }
                }
                TrackEventKind::Midi {
                    channel: _,
                    message: MidiMessage::NoteOn { key, vel: _ },
                } => {
                    match key.as_int() {
                        58..=66 => diff[0] = true,
                        70..=78 => diff[1] = true,
                        82..=90 => diff[2] = true,
                        94..=102 => diff[3] = true,
                        110..=112 => flag = true,
                        _ => {}
                    }
                    if key.as_int() == 101 {
                        flag = true;
                    }
                }
                _ => {}
            };
        }

        if inst != Instrument::None && diff != [false, false, false, false] {
            if diff != [true, true, true, true] {
                println!("{:?}: {:?}, {:?}", inst as i64, diff, song.folder_path);
            }
            for d in 0..diff.len() {
                if diff[d] {
                    apply_charts(song, inst, flag, d as i64);
                }
            }
        }
    }
}

const DIFF: [&str; 4] = ["easy", "medium", "hard", "expert"];
fn read_chart(song: &mut SongEntry, buf: &[u8], full: bool) {
    let raw_text = util::string_from_bytes(buf);

    let mut section = String::new();
    let mut inst = Instrument::None;
    let mut diff = -1i64;
    let mut notes_flag = false;
    let mut drums_flag = false;

    for line in raw_text.lines() {
        let line = line.trim();

        // skip this one no one cares
        if line == "{" {
            continue;
        }

        // on new section
        if line.starts_with('[') {
            section = line.get(1..line.len() - 1).unwrap().to_lowercase();

            // get inst and diff berforehand
            for d in 0..DIFF.len() {
                if section.starts_with(DIFF[d]) {
                    diff = d as i64;
                    inst = {
                        match section.replace(DIFF[d], "").as_str() {
                            "single" => Instrument::Guitar,
                            "doublebass" => Instrument::Bass,
                            "doublerhythm" => Instrument::Rhythm,
                            "doubleguitar" => Instrument::GuitarCoop,
                            "ghlguitar" => Instrument::GHLGuitar,
                            "ghlbass" => Instrument::GHLBass,
                            "drums" => Instrument::Drums,
                            "keyboard" => Instrument::Keys,
                            "band" => Instrument::Band,
                            _ => Instrument::None,
                        }
                    };
                    break;
                }
            }

            continue;
        }

        // apply data when done with section
        if line == "}" {
            if inst != Instrument::None && diff >= 0 && notes_flag {
                apply_charts(song, inst, drums_flag, diff);
            }
            inst = Instrument::None;
            diff = -1i64;
            notes_flag = false;
            drums_flag = false;
        }

        // split key and value, ignore errors
        let arr_ = line.split_once('=');
        if arr_.is_none() {
            continue;
        }
        let arr = arr_.unwrap();

        // properly format key and value
        let key = arr.0.trim().to_lowercase();
        let val = arr.1.replace("\"", "").trim().to_string();

        // metadata parsing
        if section == "song" {
            if full {
                match key.as_str() {
                    "charter" => song.metadata[5] = val,
                    "artist" => song.metadata[1] = val,
                    //"offset" => song
                    "genre" => song.metadata[3] = val,
                    "album" => song.metadata[2] = val,
                    "year" => song.metadata[4] = val.replace(", ", ""),
                    "name" => {
                        if val != "TEMPO TRACK" && val != "" && val != "midi_export" {
                            song.metadata[0] = val;
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        // lyrics checking
        if section == "events" {
            if !song.lyrics && val.starts_with("E lyric") {
                song.lyrics = true;
            }
            continue;
        }

        // difficulty and instrument parsing
        if val.starts_with("N") {
            notes_flag = true;
            if inst == Instrument::Drums
                && !drums_flag
                && (val.starts_with("N 5")
                    || val.starts_with("N 32")
                    || val.starts_with("N 66")
                    || val.starts_with("N 67")
                    || val.starts_with("N 68"))
            {
                drums_flag = true;
            }
        }
    }
}

pub fn scan_folder(p: &Path, cloud_format: bool) -> Vec<SongEntry> {
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

            let mut album_art_name = String::new();
            let mut audio_files = vec![];
            let mut image_flag = false;
            let mut image_name = String::new();
            let mut video_name = String::new();

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
                    video_name = file.file_name().to_string_lossy().to_string();
                } else if IMAGE_EXTS.contains(&&extension[..]) {
                    if name == "background" {
                        image_flag = true;
                        image_name = file.file_name().to_string_lossy().to_string();
                    } else if name == "album" {
                        album_art_name = file.file_name().to_string_lossy().to_string();
                    }
                } else if AUDIO_FILES.contains(&&name[..]) && AUDIO_EXTS.contains(&&extension[..]) {
                    audio_files.push(file.file_name().to_string_lossy().to_string());
                }
            }

            // idk what this does but it works :tm:
            if !(!mid_flag && !chart_flag) || ini_flag {
                let s_path = entry.path();
                let mut song = SongEntry::default();

                if cloud_format {
                    song.folder_path = format!("/{}", s_path.strip_prefix(p).unwrap().to_string_lossy().replace('\\', "/"));
                } else {
                    song.folder_path = s_path.to_string_lossy().to_lowercase().to_string();
                }

                // skip if song.ini is invalid
                //if !read_ini(&mut song, &s_path.join("song.ini")) {
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
                    read_midi(&mut song, &notes_data);
                } else if chart_flag {
                    read_chart(&mut song, &notes_data, false);
                }

                // add some stuffs
                song.video_background = video_flag;
                song.chart_name = chart_name;
                song.date_added = 0; //DateTime.Now.Date;
                if cloud_format {
                    song.album_art_name = album_art_name;
                    song.audio_files = audio_files;
                    song.image_background = image_flag;
                    song.image_background_name = image_name;
                    song.video_background_name = video_name;
                }

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
                        }.to_lowercase();
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
