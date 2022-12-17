use config_static::*;
use serde::Deserialize;
use serde::Serialize;

#[config_static_check("config.json")]
#[derive(Deserialize, Serialize)]
pub struct Config {
    field: String,
}

fn main() {
    println!("Hello, world! {}", get_config().field);
}
