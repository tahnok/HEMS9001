extern crate glob;

use glob::glob;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{env, process, thread, time};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply path to 1wire dir as argument (try /sys/bus/w1/devices/)");
        process::exit(12);
    }
    
    let ten_seconds = time::Duration::from_secs(10);

    loop {
        let file_path = find_file(&args[1]).expect("unable to find file for 1w sensor");
        let mut file = File::open(file_path).expect("unable to open 1w sensor");
        let mut contents = String::new();

        let read = file.read_to_string(&mut contents);
        read.expect("can't read file to string");

        match parse_temperature(&contents) {
            Some(temp) => println!("temp is {}", temp / 1000),
            None => println!("sensor not ready")
        }
        thread::sleep(ten_seconds);
    }

}

pub fn find_file(base_path: &str) -> Option<PathBuf> {
    //for file in glob(format!("
    for entry in glob(&format!("{}28-*", base_path)).unwrap() {
        match entry {
            Ok(path) => { return Some(path); },
            _ => ()
        }
    }
    None
}

pub fn parse_temperature(input: &str) -> Option<i32> {
    let mut lines = input.lines();
    let first = lines.next()?;
    if !first.ends_with(" YES") {
        return None;
    }
    let second = lines.next()?;

    let mut halves = second.split(" t=");
    
    let raw_temp = halves.nth(1).unwrap_or("");

    raw_temp.parse().ok()
}

#[cfg(test)]
mod tests {

    #[test]
    fn throws_error_for_failed_read() {
        assert_eq!(None, super::parse_temperature("bla"));
    }

    #[test]
    fn can_extract_temp() {
        let valid = "4b 01 4b 7f ff 05 10 e1 : crc=e1 YES\n\
                     4b 01 4b 7f ff 05 10 e1 t=20687";
        assert_eq!(Some(20687), super::parse_temperature(valid));
    }
}
