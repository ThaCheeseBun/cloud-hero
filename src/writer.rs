use std::io::prelude::*;
use std::{io::Cursor, fs::File};
use byteorder::{LittleEndian, WriteBytesExt};
use crate::songentry::SongEntry;
use crate::VERSION;

// .NET 7 bit integer writer
fn write_7_bit_int(v: u8, f: &mut File) {
    let mut val = v;
    while val >= 0x80 {
        f.write_u8((val | 0x80) & 0xff).unwrap();
        val >>= 7;
    }
    f.write_u8(val & 0x7f).unwrap();
}

// .NET length prefixed string writer
fn write_string(v: String, f: &mut File) {
    let bytes = v.into_bytes();
    write_7_bit_int(bytes.len() as u8, f);
    f.write(&bytes).unwrap();
}

// .NET bool writer
fn write_boolean(b: bool, f: &mut File) {
    if b {
        f.write_u8(1).unwrap();
    } else {
        f.write_u8(0).unwrap();
    }
}

pub fn write_cache(list: Vec<SongEntry>, f: &mut File) {
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
