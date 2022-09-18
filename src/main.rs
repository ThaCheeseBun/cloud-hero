use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use md5;

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
            write_string(lists[i][j].clone(), f);
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

fn main() {
    println!("Hello, world!");

    let mut f = File::open("stuff/songcache.bin").unwrap();
    let out = parse_cache(&mut f);

    //println!("{:?}", out.len());
    //println!("{:#?}", out[0]);

    let mut f2 = File::create("stuff/songcache2.bin").unwrap();
    write_cache(out, &mut f2);
}
