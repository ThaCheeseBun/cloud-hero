use std::{path::{PathBuf, Path}, ffi::{OsStr, OsString}, fs};
use configparser::ini::Ini;
use walkdir::WalkDir;
use crate::util;

const VIDEO_EXTS: [&str; 6] = [".mp4", ".avi", ".webm", ".vp8", ".ogv", ".mpeg"];

fn ini_get_int(c: &Ini, s: &str, k: &str, d: Option<i64>) -> Option<i64> {
    let raw = c.get(s, k);
    if raw.is_none() {
        return d;
    }
    let parsed = raw.unwrap().parse::<i64>();
    if parsed.is_err() {
        return d;
    }
    Some(parsed.unwrap())
}

fn ini_get_bool(c: &Ini, s: &str, k: &str, d: Option<bool>) -> Option<bool> {
    let raw = c.get(s, k);
    if raw.is_none() {
        return d;
    }
    match raw.unwrap().to_lowercase().as_str() {
        "true" | "yes" | "t" | "y" | "1" | "on" => Some(true),
        "false" | "no" | "f" | "n" | "0" | "off" => Some(false),
        _ => d,
    }
}

fn read_ini(p: PathBuf) -> bool {
    let mut flag = false;

    let raw_text = util::string_from_file(&p);
    let mut config = Ini::new();
    let i_maybe = config.read(raw_text);

    if i_maybe.is_err() {
        println!("{}", i_maybe.err().unwrap());
        println!("VERYBAD: {:?}", p);
        return false;
    }

    if config.sections().contains(&String::from("song")) {
        /*SongEntry.metadataCache[0] = iniparser.ReadValue("song", "name", "");
        SongEntry.metadataCache[1] = iniparser.ReadValue("song", "artist", "");
        SongEntry.metadataCache[2] = iniparser.ReadValue("song", "album", "");
        SongEntry.metadataCache[3] = iniparser.ReadValue("song", "genre", "");
        SongEntry.metadataCache[4] = iniparser.ReadValue("song", "year", "");*/
        let mut intensities = [0i8; 10];
        intensities[8] = ini_get_int(&config, "song", "diff_band", Some(-1)).unwrap() as i8;
        intensities[0] = ini_get_int(&config, "song", "diff_guitar", Some(-1)).unwrap() as i8;
        intensities[2] = ini_get_int(&config, "song", "diff_rhythm", Some(-1)).unwrap() as i8;
        intensities[1] = ini_get_int(&config, "song", "diff_bass", Some(-1)).unwrap() as i8;
        intensities[6] = ini_get_int(&config, "song", "diff_drums", Some(-1)).unwrap() as i8;
        intensities[9] = ini_get_int(&config, "song", "diff_drums_real", Some(-1)).unwrap() as i8;
        intensities[7] = ini_get_int(&config, "song", "diff_keys", Some(-1)).unwrap() as i8;
        intensities[4] = ini_get_int(&config, "song", "diff_guitarghl", Some(-1)).unwrap() as i8;
        intensities[5] = ini_get_int(&config, "song", "diff_bassghl", Some(-1)).unwrap() as i8;

        // 10 / 10 error handling right here
        // i really need to make this better

        //ini_get_int(sec, "track", Some(16000)).unwrap() as i16;
        //ini_get_bool(sec, "five_lane_drums", Some(false)).unwrap();

        let preview_start = ini_get_int(&config, "song", "preview_start_time", Some(-1)).unwrap() as i32;
        let icon_name = config.get("song", "icon").unwrap_or(String::from("")).to_lowercase();
        let playlist_track = ini_get_int(&config, "song", "playlist_track", Some(16000)).unwrap() as i16;
        let modchart = ini_get_bool(&config, "song", "modchart", Some(false)).unwrap();
        let song_length = ini_get_int(&config, "song", "song_length", Some(0)).unwrap() as i32;
        let force_pro_drums = ini_get_bool(&config, "song", "pro_drums", Some(false)).unwrap();
        let force_five_lane = ini_get_bool(&config, "song", "five_lane_drums", Some(false)).unwrap();
        let top_level_playlist = config.get("song", "playlist").unwrap_or(String::from("")).to_lowercase();
        let sub_playlist = config.get("song", "sub_playlist").unwrap_or(String::from("")).to_lowercase();
        if config.get("song", "album_track").is_some() {
            let album_track = ini_get_int(&config, "song", "album_track", Some(16000)).unwrap() as i16;
        } else {
            let album_track = ini_get_int(&config, "song", "track", Some(16000)).unwrap() as i16;
        }
        //SongEntry.metadataCache[5] = iniparser.ReadValue("song", iniparser.IsKeyExists("song", "charter") ? "charter" : "frets", "");
        flag = true;
    } else {
        flag = false;
    }

    flag
}

fn create_song_entry(p: &Path) -> bool {
    let is_enc = !p.is_dir() && p.extension().unwrap().to_string_lossy().to_lowercase() == "sng";
    let intensities: [i8; 10] = [0; 10]; // sbyte[10]
                                         //Array.Clear(SongEntry.metadataCache, 0, SongEntry.metadataCache.Length);
    let text = p.join("song.ini");
    let text2 = p.join("notes.chart");
    let folder_path = p;
    /*if is_enc {
        this.method_5();
        return;
    }*/
    let mut bool = false;
    if !read_ini(text) {
        println!("no: {:?}", p);
        /*if this.method_19(text2) {
            this.metadataLoaded = true;
            return;
        }*/
    } else {
        //println!("yes: {:?}", p);
        bool = true;
        //this.metadataLoaded = true;
    }
    bool
}

pub fn scan_folder() {
    let mut aaaa = 0;

    for entry in WalkDir::new("E:\\Spel\\Annat\\Clone Hero\\Songs") {
        let entry = entry.unwrap();
        if entry.path().extension() != None
            && entry.path().extension().unwrap() == OsStr::new("sng")
        {
            // todo
        } else if entry.file_type().is_dir() {
            let mut flag = false;
            let mut flag2 = false;
            let mut flag3 = false;
            let mut flag4 = false;
            let mut text2: Option<OsString> = None;

            for file in fs::read_dir(entry.path()).unwrap() {
                let file_entry = file.unwrap();
                let file_name = file_entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_lowercase();
                let extension = file_entry
                    .path()
                    .extension()
                    .or(Some(&OsStr::new("")))
                    .unwrap()
                    .to_string_lossy()
                    .to_lowercase();
                if file_name == "notes" {
                    if extension == "mid" {
                        flag = true;
                        text2 = Some(file_entry.file_name());
                    } else if extension == "chart" {
                        flag2 = true;
                        text2 = Some(file_entry.file_name());
                    }
                } else if file_name == "song" && extension == "ini" {
                    flag3 = true;
                } else if file_name == "video" && VIDEO_EXTS.contains(&&extension[..]) {
                    if extension != ".webm" && extension != ".vp8" && extension != ".ogv" {
                        //this.list_3.Add(text);
                    }
                    flag4 = true;
                }
            }

            if !(!flag && !flag2) || flag3 {
                if create_song_entry(entry.path()) {
                    aaaa += 1;
                }

                /*let songEntry2 = SongEntry {

                };
                if (!songEntry2.HasValidName)
                {
                    this.list_10.Add(text);
                }
                else
                {
                    int num = (flag ? 2 : (flag2 ? 1 : 0));
                    songEntry2.videoBackground = flag4;
                    songEntry2.chartName = text2;
                    if (songEntry2.metadataLoaded)
                    {
                        this.bool_0 = false;
                        if (this.method_6(songEntry2))
                        {
                            if (num > 0 && this.method_13(songEntry2))
                            {
                                songEntry2.method_4();
                                songEntry2.dateAdded = DateTime.Now.Date;
                                GClass54.list_0.Add(songEntry2); // THE REAL LIST
                                if (this.bool_0 || (num == 1 && !flag3))
                                {
                                    songEntry2.method_10(num == 1 && !flag3);
                                }
                            }
                        }
                        else
                        {
                            this.list_6.Add(text);
                        }
                    }
                    else
                    {
                        this.list_10.Add(text);
                    }
                }*/
            }
        }
    }

    println!("{:?}", aaaa);
}
