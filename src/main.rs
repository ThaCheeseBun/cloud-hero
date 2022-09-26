mod util;
mod writer;
mod songentry;
mod reader;
mod scanner;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const VERSION: i32 = 20220812;

fn main() {
    println!("Hello, world!");

    let cloud_format = true;

    let peth = Path::new("E:\\Spel\\Annat\\Clone Hero\\Songs_");
    let songs = scanner::scan_folder(&peth, cloud_format);
    let serialized = serde_json::to_string(&songs).unwrap();

    let mut testout = File::create("stuff/debug.json").unwrap();
    write!(testout, "{}", serialized).unwrap();

    /*let mut f = File::open("stuff/songcache.bin").unwrap();
    let out = reader::read_cache(&mut f, cloud_format).unwrap();
    println!("{:?}", out.len());*/

    let mut f2 = File::create("stuff/songcache2.bin").unwrap();
    writer::write_cache(songs, &mut f2, cloud_format);
}
