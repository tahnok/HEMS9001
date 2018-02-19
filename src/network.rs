use std::collections::HashMap;

use reqwest;

header! { (XAioKey, "X-AIO-Key") => [String] }

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
