use std::fs::File;
use std::io::{prelude::*, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::songentry::SongEntry;
use crate::VERSION;

// .NET 7 bit integer reader
fn read_7_bit_int(f: &mut File) -> i32 {
    let mut count = 0;
    let mut shift = 0;
    let mut b: u8;
    loop {
        b = f.read_u8().unwrap();
        count |= ((b & 0x7f) << shift) as i32;
        shift += 7;
        if !((b & 0x80) != 0) {
            break;
        }
    }
    println!("{}", count);
    return count;
}

// .NET length prefixed string reader
fn read_string(f: &mut File) -> String {
    let len = read_7_bit_int(f);
    let mut buf: Vec<u8> = vec![0; len as usize];
    f.read(&mut buf).unwrap();
    return String::from_utf8_lossy(&buf).to_string();
}

// .NET bool reader
fn read_boolean(f: &mut File) -> bool {
    if f.read_u8().unwrap() == 0 {
        false
    } else {
        true
    }
}

pub fn read_cache(f: &mut File) -> Option<Vec<SongEntry>> {

    // verify version
    let version = f.read_i32::<LittleEndian>().unwrap();
    if version != VERSION {
        println!("Expected version \"{}\", got \"{}\"", VERSION, version);
        return None;
    }
    println!("Version: \"{}\"", version);

    // get file checksum
    let mut checksum = [0u8; 16];
    f.read_exact(&mut checksum).unwrap();
    println!("Checksum: {:x?}", checksum);

    // get all key value data
    let mut lists = [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
    for _ in 0..lists.len() {
        let list_index = f.read_u8().unwrap() as usize;
        let num = f.read_i32::<LittleEndian>().unwrap();
        for _ in 0..num {
            lists[list_index].push(read_string(f));
        }
    }

    // loop through all entries
    let mut out = vec![];
    let num = f.read_i32::<LittleEndian>().unwrap();

    for _ in 0..num {
        let text = read_string(f);
        let _ = f.read_i64::<LittleEndian>().unwrap();
        let _ = f.read_i64::<LittleEndian>().unwrap();

        let testtemp = read_string(f);
        println!("{:?}{:?}", testtemp, f.seek(SeekFrom::Current(0)).unwrap());

        let mut song_entry = SongEntry {
            folder_path: text,

            chart_name: testtemp,
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
        };
        println!("{:?} {:?}", out.len(), song_entry.folder_path);
        out.push(song_entry);
    }

    Some(out)

}