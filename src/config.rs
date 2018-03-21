extern crate toml;
extern crate failure;

use std::fs::File;
use std::io::prelude::*;

use self::failure::Error;

#[derive(Debug, Deserialize)]
/// Manages the config of the application.
pub struct Config {
    pub num_processes: i64,
    pub max_bits: u32,
}

/// Simple function parse the config of the application.
/// 
/// The default config location is `~/.config/rust-evc/config.toml`
pub fn parse_config(name: &str) -> Result<Config, Error> {
    let mut f = File::open(name)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents.as_str())?;

    Ok(config)
}


#[test]
fn test_parse_config() {
    let config = match parse_config("/Users/bIgB/.config/rust-evc/config.toml") {
        Ok(c) => assert_eq!(c.num_processes, 5),
        Err(e) => {
            println!("Error: {}", e);
            assert!(false);
        }
    };

    assert!(true);
}
