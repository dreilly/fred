use std::io::{self, BufRead};
use std::{fs::File, path::Path};

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename);
    match file {
        Err(e) => panic!("{}", e),
        Ok(f) => Ok(io::BufReader::new(f).lines()),
    }
}
