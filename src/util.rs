use std::io::prelude::*;
use std::io::SeekFrom;
use std::{fs::File, io::Read, path::PathBuf};
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

/*
   helper function to decode various text formats
   currently supports:
   UTF-16 LE
   UTF-16 BE
   UTF-8 BOM
   UTF-8
*/
pub fn string_from_file(p: &PathBuf) -> String {
    let mut f = File::open(p).unwrap();
    let mut buf = [0u8; 3];
    f.read_exact(&mut buf).unwrap();

    // UTF-16 BE
    if buf[0] == 0xFE && buf[1] == 0xFF {
        f.seek(SeekFrom::Start(2)).unwrap();

        let mut buffer = vec![];
        loop {
            let num = f.read_u16::<BigEndian>();
            if num.is_err() {
                break;
            }
            buffer.push(num.unwrap());
        }

        String::from_utf16_lossy(&buffer)

    // UTF-16 LE
    } else if buf[0] == 0xFF && buf[1] == 0xFE {
        //let len = f.seek(SeekFrom::End(0)).unwrap() - 2;
        f.seek(SeekFrom::Start(2)).unwrap();

        let mut buffer = vec![];
        loop {
            let num = f.read_u16::<LittleEndian>();
            if num.is_err() {
                break;
            }
            buffer.push(num.unwrap());
        }

        String::from_utf16_lossy(&buffer)

    // UTF-8 BOM
    } else if buf[0] == 0xEF && buf[1] == 0xBB && buf[2] == 0xBF {
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer).to_string()

    // assume UTF-8
    } else {
        f.seek(SeekFrom::Start(0)).unwrap();
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();
        String::from_utf8_lossy(&buffer).to_string()
    }
}
