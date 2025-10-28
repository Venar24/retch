use std::fs;
use serde::Deserialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string("config.toml")?;
    let cfg: Config = toml::from_str(&data)?;
    println!("{:?}", cfg);
    Ok(())
}
