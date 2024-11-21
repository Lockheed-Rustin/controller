use std::fs;
use wg_2024::config::Config;

fn parse_config(file: &str) -> Config {
    let file_str = fs::read_to_string(file).unwrap();
    toml::from_str(&file_str).unwrap()
}

fn main() {
    let config = parse_config("config.toml");
    println!("{:?}", config);
    // TODO: add UI
    // yeah I know really useful comment
}
