use config_static::*;
use serde::Deserialize;
use serde::Serialize;

// need to support Strings, Arrays, Other Deserializable structs { objects in json terms } (hard to do), usize
// isize or size whatever is the rust type for signed usize.
#[config_static_check("config.json")]
#[derive(Deserialize, Serialize)]
pub struct Config {
    field: String,
    vals: Vec<String>,
    fails: Vec<String>,
}

fn main() {
    println!("Hello, world! {:?}", get_config().vals);
}
