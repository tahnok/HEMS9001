use glob::glob;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;


pub fn fetch(path: &str) -> Option<f32> {
    let file_path = find_file(&path).expect("unable to find file for 1w sensor");
    let mut file = File::open(file_path).expect("unable to open 1w sensor");
    let mut contents = String::new();

    let read = file.read_to_string(&mut contents);
    read.expect("can't read file to string");
    parse_temperature(&contents) 
}

fn find_file(base_path: &str) -> Option<PathBuf> {
    for entry in glob(&format!("{}*/w1_slave", base_path)).unwrap() {
        match entry {
            Ok(path) => { return Some(path); },
            _ => ()
        }
    }
    None
}

pub fn parse_temperature(input: &str) -> Option<f32> {
    let mut lines = input.lines();
    let first = lines.next()?;
    if !first.ends_with(" YES") {
        return None;
    }
    let second = lines.next()?;

    let mut halves = second.split(" t=");

    match halves.nth(1).unwrap_or("").parse::<f32>() {
        Ok(raw_temp) => Some(raw_temp / 1000.0),
        _ => None
    }
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
        assert_eq!(Some(20.687), super::parse_temperature(valid));
    }
}
