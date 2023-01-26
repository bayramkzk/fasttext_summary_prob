use serde::Deserialize;

#[derive(Deserialize)]
pub struct Data {
    pub config: Config,
}

#[derive(Deserialize)]
pub struct Config {
    pub langs: Vec<String>,
}

pub fn read_config() -> Config {
    let config = std::fs::read_to_string("config.toml").unwrap();
    let data: Data = toml::from_str(&config).unwrap();
    data.config
}
