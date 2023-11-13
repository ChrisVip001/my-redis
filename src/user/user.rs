use std::fs::File;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

lazy_static! {
    static ref USER_PATH: String = Config::get_config().path;
    static ref USER_MAX: u16 = Config::get_config().user_max;
}


#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    name: String,
    password: String,
    level: String,
}

struct Config {
    path: String,
    user_max: u16,
}

impl Config {
    fn default() -> Self {
        Config {
            path: "user.json".to_string(),
            user_max: 50,
        }
    }
    fn get_config() -> Self {
        let mut file = match File::open("config/user.toml") {
            Ok(f) => f,
            Err(_) => return Self::default(),
        };
        let mut c = String::new();
        file.read_to_string(&mut c)?;
        let config = toml::from_str(c.as_str())?;
        Ok(config)
    }

    pub fn login(name: String, password: String) -> Result<Self> {
        let config_path = USER
    }
}
