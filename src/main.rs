extern crate clap;
extern crate glob;
extern crate reqwest;

#[macro_use] extern crate hyper;


mod network;
mod temperature;

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
        match temperature::fetch(&path) {
            Some(temp) => {
                println!("temp is {}", temp);
                network::upload_temperature(temp, &user, &key, &feed_name);
            },
            None => println!("sensor not ready")
        }
        std::thread::sleep(ten_seconds);
    }

}
