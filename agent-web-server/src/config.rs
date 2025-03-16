use crate::states::DATA_DIR;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub db_pool_size: u32,
    pub jwt_expire_days: u64,
    pub chat_expire_days: u64,
    pub api_base: String,
    pub api_key: String,
    pub model: String,
    pub sys_prompt: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            db_pool_size: 10,
            chat_expire_days: 30,
            jwt_expire_days: 30,
            api_base: "https://api.openai.com/v1".into(),
            api_key: "<API Key>".into(),
            model: "chatgpt-4o-latest".into(),
            sys_prompt: "You are a helpful assistant.".into(),
        }
    }
}

/// Initialize config, either read or create.  
///
/// ([ServerConfig], true) if read from existing config,  
/// ([ServerConfig], false) if created a new config.
pub fn init_config() -> Result<(ServerConfig, bool)> {
    let data_dir = DATA_DIR.get().unwrap();
    let config_path = data_dir.join("config.toml");

    match OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&config_path)
    {
        Ok(mut config_file) => {
            let template = ServerConfig::default();
            let toml_str =
                toml::to_string_pretty(&template).with_context(|| "serialize server config")?;
            config_file
                .write_all(toml_str.as_bytes())
                .with_context(|| "write server config")?;
            Ok((ServerConfig::default(), false))
        }
        Err(_) => {
            let mut config_file =
                File::open(&config_path).with_context(|| "open existing server config")?;
            let mut toml_str = String::new();
            config_file
                .read_to_string(&mut toml_str)
                .with_context(|| "read server config")?;
            let config = toml::from_str(&toml_str).with_context(|| "deserialize config")?;
            Ok((config, true))
        }
    }
}
