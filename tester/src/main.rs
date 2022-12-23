use config_static::*;
use serde::Deserialize;
use serde::Serialize;

#[config_static_json("config.json")]
#[derive(Deserialize, Serialize)]
pub struct Config {
    field: String,
    booly: bool,
    big_int: i64,
}

pub struct KakfaConfig {}

fn main() {
    println!("Hello, world! {:?}", get_config().booly.to_string());
}
