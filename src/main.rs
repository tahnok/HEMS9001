extern crate clap;
extern crate glob;
extern crate reqwest;

#[macro_use] extern crate hyper;
header! { (XAioKey, "X-AIO-Key") => [String] }

use clap::{Arg, App};

use glob::glob;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{thread, time};


const USERNAME: &'static str = "tahnok42";
const FEED_NAME: &'static str = "temperature";


fn main() {
    let matches = App::new("HEMS 9001")
                          .version("0.1")
                          .author("Wesley Ellis<tahnok@gmail.com>")
                          .about("Monitor Hedgehog Environment")
                          .arg(Arg::with_name("aio-key")
                               .long("aio-key")
                               .value_name("KEY")
                               .help("Adafruit.io KEY")
                               .required(true)
                               .takes_value(true))
                          .arg(Arg::with_name("1wire")
                               .long("1wire-dir")
                               .value_name("DIR")
                               .help("Directory to 1wire sysfs like /sys/bus/w1/devices/")
                               .default_value("/sys/bus/w1/devices/"))
                          .get_matches();

    let key = matches.value_of("aio-key").unwrap();
    let path = matches.value_of("1wire").unwrap();

    
    let ten_seconds = time::Duration::from_secs(10);

    loop {
        let file_path = find_file(&path).expect("unable to find file for 1w sensor");
        let mut file = File::open(file_path).expect("unable to open 1w sensor");
        let mut contents = String::new();

        let read = file.read_to_string(&mut contents);
        read.expect("can't read file to string");

        match parse_temperature(&contents) {
            Some(temp) => {
                println!("temp is {}", temp / 1000);
                upload_temperature(temp, &key);
            },
            None => println!("sensor not ready")
        }
        thread::sleep(ten_seconds);
    }

}

pub fn upload_temperature(milli_celcius: i32, key: &str) -> () {
    let mut map = HashMap::new();
    map.insert("value", format!("{}", milli_celcius / 1000));

    let url = format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data", USERNAME, FEED_NAME);
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(XAioKey(key.to_owned()))
        .json(&map)
        .send()
        .expect("should be able to send");

    if !response.status().is_success() {
        println!("failed to send:");
        println!("\t{:?}", response);
    }
}

pub fn find_file(base_path: &str) -> Option<PathBuf> {
    //for file in glob(format!("
    for entry in glob(&format!("{}*/w1_slave", base_path)).unwrap() {
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
