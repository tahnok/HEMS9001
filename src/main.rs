extern crate clap;
extern crate glob;
extern crate reqwest;

#[macro_use] extern crate hyper;
header! { (XAioKey, "X-AIO-Key") => [String] }

use glob::glob;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;


fn main() {
    let cmd_args = clap::App::new("HEMS 9001")
                          .version("0.1")
                          .author("Wesley Ellis<tahnok@gmail.com>")
                          .about("Monitor Hedgehog Environment")
                          .arg(clap::Arg::with_name("aio-key")
                               .long("aio-key")
                               .value_name("KEY")
                               .help("Adafruit.io KEY")
                               .required(true)
                               .takes_value(true))
                          .arg(clap::Arg::with_name("1wire")
                               .long("1wire-dir")
                               .value_name("DIR")
                               .help("Directory to 1wire sysfs like /sys/bus/w1/devices/")
                               .default_value("/sys/bus/w1/devices/"))
                          .arg(clap::Arg::with_name("user")
                               .long("user")
                               .value_name("ADAFRUIT.IO USERNAME")
                               .help("Adafruit.io username")
                               .default_value("tahnok42"))
                          .arg(clap::Arg::with_name("feed_name")
                               .long("feed_name")
                               .value_name("FEED NAME")
                               .help("Adafruit.io feedname")
                               .default_value("temperature"))
                          .get_matches();

    let key = cmd_args.value_of("aio-key").unwrap();
    let path = cmd_args.value_of("1wire").unwrap();
    let user = cmd_args.value_of("user").unwrap();
    let feed_name = cmd_args.value_of("feed_name").unwrap();
    
    let ten_seconds = std::time::Duration::from_secs(10);

    loop {
        match fetch_temperature(&path) {
            Some(temp) => {
                println!("temp is {}", temp);
                upload_temperature(temp, &user, &key, &feed_name);
            },
            None => println!("sensor not ready")
        }
        std::thread::sleep(ten_seconds);
    }

}

pub fn upload_temperature(celcius: f32, user: &str, key: &str, feed_name: &str) -> () {
    let mut map = HashMap::new();
    map.insert("value", format!("{}", celcius));

    let url = format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data", user, feed_name);
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

pub fn fetch_temperature(path: &str) -> Option<f32> {
        let file_path = find_file(&path).expect("unable to find file for 1w sensor");
        let mut file = File::open(file_path).expect("unable to open 1w sensor");
        let mut contents = String::new();

        let read = file.read_to_string(&mut contents);
        read.expect("can't read file to string");
        parse_temperature(&contents) 
}

pub fn find_file(base_path: &str) -> Option<PathBuf> {
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
