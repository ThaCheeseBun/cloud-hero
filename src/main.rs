mod util;
mod writer;
mod songentry;
mod reader;
mod scanner;

use std::fs::File;

const VERSION: i32 = 20220812;

fn main() {
    println!("Hello, world!");

    scanner::scan_folder();

    /*let mut f = File::open("stuff/songcache.bin").unwrap();
    let out = reader::read_cache(&mut f).unwrap();
    println!("{:?}", out.len());

    let mut f2 = File::create("stuff/songcache2.bin").unwrap();
    writer::write_cache(out, &mut f2);*/
}
