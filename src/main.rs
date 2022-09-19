use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::PathBuf;
use std::str::from_utf8;
use md5;
use walkdir::WalkDir;
use std::fs;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use configparser::ini::Ini;

#[derive(Debug)]
#[allow(dead_code)]
struct SongEntry {
    album_track: i16,           // short
    chart_name: String,         // string
    charts: i64,                // GStruct6, change later
    checksum: [u8; 16],         // SongHash
    containers: String,         // dict<string, GClass9> PRIVATE, change later
    date_added: i64,            // DateTime, change later
    filtered: bool,             // bool
    folder_path: String,        // string
    force_five_lane: bool,      // bool
    force_pro_drums: bool,      // bool
    icon_name: String,          // string
    intensities: [i8; 10],      // sbyte[] PRIVATE
    is_available_online: bool,  // bool
    is_enc: bool,               // bool
    is_midi_chart_cache: bool,  // bool PRIVATE
    is_type_cached: bool,       // bool PRIVATE
    lyrics: bool,               // bool
    metadata: [String; 7],      // GClass47[] PRIVATE
    metadata_cache: String,     // string[] PRIVATE
    metadata_loaded: bool,      // bool
    modchart: bool,             // bool
    playlist_track: i16,        // short
    preview_start: i32,         // int
    scores: String,             // GClass55, Change later
    song_enc: String,           // GClass9, change later
    song_length: i32,           // int
    sub_playlist: String,       // string
    top_level_playlist: String, // string
    video_background: bool,     // bool
}

const VERSION: i32 = 20220812;
const VIDEO_EXTS: [&str; 6] = [".mp4", ".avi", ".webm", ".vp8", ".ogv", ".mpeg"];

// .net 7 bit integer
fn read_7_bit_int(f: &mut File) -> u8 {
    let mut byte: u8;
    let mut shift = 0;
    let mut num = 0;
    loop {
        byte = f.read_u8().unwrap();
        num |= (byte & 0x7f) << shift;
        shift += 7;
        if !((byte & 0x80) != 0) {
            break;
        }
    }
    return num;
}
fn write_7_bit_int(v: u8, f: &mut File) {
    let mut val = v;
    while val >= 0x80 {
        f.write_u8((val | 0x80) & 0xff).unwrap(); // set 8th to 1, keep only the first 8 bits
        val >>= 7;
    }
    f.write_u8(val & 0x7f).unwrap();
}

// .net strings
fn read_string(f: &mut File) -> String {
    let len = read_7_bit_int(f);
    let mut buf: Vec<u8> = vec![0; len as usize];
    f.read(&mut buf).unwrap();
    return String::from_utf8_lossy(&buf).to_string();
}
fn write_string(v: String, f: &mut File) {
    let bytes = v.into_bytes();
    write_7_bit_int(bytes.len() as u8, f);
    f.write(&bytes).unwrap();
}

// .net booleans
fn read_boolean(f: &mut File) -> bool {
    if f.read_u8().unwrap() == 0 {
        false
    } else {
        true
    }
}
fn write_boolean(b: bool, f: &mut File) {
    if b {
        f.write_u8(1).unwrap();
    } else {
        f.write_u8(0).unwrap();
    }
}

// parse cache from open file
fn parse_cache(f: &mut File) -> Vec<SongEntry> {
    // verify version
    let version = f.read_i32::<LittleEndian>().unwrap();
    if version != VERSION {
        panic!("wrong version")
    }
    println!("Version: {:?}", version);

    // get file checksum
    let mut checksum: [u8; 16] = [0; 16];
    f.read_exact(&mut checksum).unwrap();
    println!("Checksum: {:x?}", checksum);

    // get all key value data
    let mut lists = [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
    for _ in 0..7 {
        let list_index = f.read_u8().unwrap() as usize;
        let num = f.read_i32::<LittleEndian>().unwrap();
        for _ in 0..num {
            //list.push(GClass47.smethod_0(genum, buf.readString(), j));
            let data = read_string(f);
            lists[list_index].push(data);
        }
    }

    // loop through all entries
    let mut out = vec![];
    let num = f.read_i32::<LittleEndian>().unwrap();
    for _ in 0..num {
        let text = read_string(f);
        let _ = f.read_i64::<LittleEndian>().unwrap();
        let _ = f.read_i64::<LittleEndian>().unwrap();
        let song_entry = SongEntry {
            folder_path: text,

            chart_name: read_string(f),
            is_enc: read_boolean(f),
            metadata: [
                lists[0][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[1][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[2][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[3][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[4][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[5][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
                lists[6][f.read_i32::<LittleEndian>().unwrap() as usize].clone(),
            ],
            charts: f.read_i64::<LittleEndian>().unwrap(),
            lyrics: read_boolean(f),
            intensities: {
                let mut a: [i8; 10] = [0; 10];
                a[8] = f.read_i8().unwrap();
                a[0] = f.read_i8().unwrap();
                a[2] = f.read_i8().unwrap();
                a[1] = f.read_i8().unwrap();
                a[6] = f.read_i8().unwrap();
                a[9] = f.read_i8().unwrap();
                a[7] = f.read_i8().unwrap();
                a[4] = f.read_i8().unwrap();
                a[5] = f.read_i8().unwrap();
                a
            },
            preview_start: f.read_i32::<LittleEndian>().unwrap(),
            icon_name: read_string(f),
            album_track: f.read_i16::<LittleEndian>().unwrap(),
            playlist_track: f.read_i16::<LittleEndian>().unwrap(),
            modchart: read_boolean(f),
            video_background: read_boolean(f),
            force_pro_drums: read_boolean(f),
            force_five_lane: read_boolean(f),
            song_length: f.read_i32::<LittleEndian>().unwrap(),
            date_added: f.read_i64::<LittleEndian>().unwrap(), // TODO
            top_level_playlist: read_string(f),
            sub_playlist: read_string(f),
            checksum: {
                let mut a: [u8; 16] = [0; 16];
                f.read_exact(&mut a).unwrap();
                a
            },

            containers: String::new(),
            filtered: false,
            is_available_online: false,
            is_midi_chart_cache: false,
            is_type_cached: false,
            metadata_cache: String::new(),
            metadata_loaded: false,
            scores: String::new(),
            song_enc: String::new(),
        };
        out.push(song_entry);
    }
    out
}

fn write_cache(list: Vec<SongEntry>, f: &mut File) {
    f.write_i32::<LittleEndian>(VERSION).unwrap();

    let mut checksum = Cursor::new(vec![0u8; list.len() * 16]);
    let mut lists = [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];

    for i in 0..list.len() {
        checksum.write(&list[i].checksum).unwrap();
        for j in 0..7 {
            if !lists[j].contains(&list[i].metadata[j]) {
                lists[j].push(list[i].metadata[j].clone());
            }
        }
    }

    let check = md5::compute(checksum.into_inner());
    println!("{:x?}", check);
    f.write(&check.0).unwrap();

    for i in 0usize..7 {
        f.write_u8(i as u8).unwrap();
        let len = lists[i].len();
        f.write_i32::<LittleEndian>(len as i32).unwrap();
        for j in 0..len {
            write_string(format!("{} hej", lists[i][j].clone()), f);
        }
    }

    f.write_i32::<LittleEndian>(list.len() as i32).unwrap();
    for i in 0..list.len() {
        write_string(list[i].folder_path.clone(), f);
        f.write_i64::<LittleEndian>(0).unwrap();
        f.write_i64::<LittleEndian>(0).unwrap();
        write_string(list[i].chart_name.clone(), f);
        write_boolean(list[i].is_enc, f);

        for j in 0..list[i].metadata.len() {
            let idx = lists[j].iter().position(|x| x == &list[i].metadata[j]);
            f.write_i32::<LittleEndian>(idx.unwrap() as i32).unwrap();
        }

        f.write_i64::<LittleEndian>(list[i].charts).unwrap();
        write_boolean(list[i].lyrics, f);
        
        f.write_i8(list[i].intensities[8]).unwrap();
        f.write_i8(list[i].intensities[0]).unwrap();
        f.write_i8(list[i].intensities[2]).unwrap();
        f.write_i8(list[i].intensities[1]).unwrap();
        f.write_i8(list[i].intensities[6]).unwrap();
        f.write_i8(list[i].intensities[9]).unwrap();
        f.write_i8(list[i].intensities[7]).unwrap();
        f.write_i8(list[i].intensities[4]).unwrap();
        f.write_i8(list[i].intensities[5]).unwrap();

        f.write_i32::<LittleEndian>(list[i].preview_start).unwrap();

        write_string(list[i].icon_name.clone(), f);
        f.write_i16::<LittleEndian>(list[i].album_track).unwrap();
        f.write_i16::<LittleEndian>(list[i].playlist_track).unwrap();
        write_boolean(list[i].modchart, f);
        write_boolean(list[i].video_background, f);
        write_boolean(list[i].force_pro_drums, f);
        write_boolean(list[i].force_five_lane, f);
        f.write_i32::<LittleEndian>(list[i].song_length).unwrap();
        f.write_i64::<LittleEndian>(list[i].date_added).unwrap();
        write_string(list[i].top_level_playlist.clone(), f);
        write_string(list[i].sub_playlist.clone(), f);
        f.write(&list[i].checksum).unwrap();
    }
}

/*
TODO
fix this shit
*/
fn read_ini(p: PathBuf) -> bool {
    let mut flag = false;

    let mut f = File::open(p).unwrap();
    let mut f_buf = vec![];
    f.read_to_end(&mut f_buf).unwrap();
    let f_str = String::from(from_utf8(&f_buf).unwrap_or(""));

    let mut config = Ini::new();
    let yesp = config.read(f_str);
    if yesp.is_err() {
        return false;
    }

    if config.sections().contains(&String::from("song")) {
        /*SongEntry.metadataCache[0] = iniparser.ReadValue("song", "name", "");
        SongEntry.metadataCache[1] = iniparser.ReadValue("song", "artist", "");
        SongEntry.metadataCache[2] = iniparser.ReadValue("song", "album", "");
        SongEntry.metadataCache[3] = iniparser.ReadValue("song", "genre", "");
        SongEntry.metadataCache[4] = iniparser.ReadValue("song", "year", "");*/
        let mut intensities = [-1i8; 10];
        intensities[8] = config.getint("song", "diff_band").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[0] = config.getint("song", "diff_guitar").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[2] = config.getint("song", "diff_rhythm").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[1] = config.getint("song", "diff_bass").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[6] = config.getint("song", "diff_drums").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[9] = config.getint("song", "diff_drums_real").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[7] = config.getint("song", "diff_keys").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[4] = config.getint("song", "diff_guitarghl").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;
        intensities[5] = config.getint("song", "diff_bassghl").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i8;

        // 10 / 10 error handling right here
        // i really need to make this better
        let preview_start = config.getint("song", "preview_start_time").or::<Option<i64>>(Ok(Some(-1))).unwrap().or(Some(-1)).unwrap() as i32;
        let icon_name = config.get("song", "icon").or(Some("".to_string())).unwrap().to_lowercase();
        let playlist_track = config.getint("song", "playlist_track").or::<Option<i64>>(Ok(Some(16000))).unwrap().or(Some(16000)).unwrap() as i16;
        let modchart = config.getboolcoerce("song", "modchart").or::<Option<bool>>(Ok(Some(false))).unwrap().or(Some(false)).unwrap();
        let song_length = config.getint("song", "song_length").or::<Option<i64>>(Ok(Some(0))).unwrap().or(Some(0)).unwrap() as i32;
        let force_pro_drums = config.getboolcoerce("song", "pro_drums").or::<Option<bool>>(Ok(Some(false))).unwrap().or(Some(false)).unwrap();
        let force_five_lane = config.getboolcoerce("song", "five_lane_drums").or::<Option<bool>>(Ok(Some(false))).unwrap().or(Some(false)).unwrap();
        let top_level_playlist = config.get("song", "playlist").or(Some("".to_string())).unwrap().to_lowercase();
        let sub_playlist = config.get("song", "sub_playlist").or(Some("".to_string())).unwrap().to_lowercase();
        if config.get("song", "album_track") != None {
            let album_track = config.getint("song", "album_track").or::<Option<i64>>(Ok(Some(16000))).unwrap().or(Some(16000)).unwrap();
        } else {
            let album_track = config.getint("song", "track").or::<Option<i64>>(Ok(Some(16000))).unwrap().or(Some(16000)).unwrap();
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
    let folderPath = p;
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
        println!("yes: {:?}", p);
        bool = true;
        //this.metadataLoaded = true;
    }
    bool
}

// recursively scan and create list from folders
fn scan_folder() {
    let mut aaaa = 0;

    for entry in WalkDir::new("E:\\Spel\\Annat\\Clone Hero\\Songs") {
        let entry = entry.unwrap();
        if entry.path().extension() != None && entry.path().extension().unwrap() == OsStr::new("sng") {
            // todo
        } else if entry.file_type().is_dir() {
            let mut flag = false;
            let mut flag2 = false;
            let mut flag3 = false;
            let mut flag4 = false;
            let mut text2: Option<OsString> = None;

            for file in fs::read_dir(entry.path()).unwrap() {
                let file_entry = file.unwrap();
                let file_name = file_entry.path().file_stem().unwrap().to_string_lossy().to_lowercase();
                let extension = file_entry.path().extension().or(Some(&OsStr::new(""))).unwrap().to_string_lossy().to_lowercase();
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

            if !(!flag && !flag2) || flag3
            {
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

fn main() {
    println!("Hello, world!");

    scan_folder();

    /*let mut f = File::open("stuff/songcache.bin").unwrap();
    let out = parse_cache(&mut f);

    //println!("{:?}", out.len());
    //println!("{:#?}", out[0]);

    let mut f2 = File::create("stuff/songcache2.bin").unwrap();
    write_cache(out, &mut f2);*/
}
