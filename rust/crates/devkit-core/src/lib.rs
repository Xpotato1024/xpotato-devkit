use serde_derive::Deserialize;
use std::path::Path;
use std::fs;

#[derive(Debug, Default, Deserialize)]
pub struct DevkitConfig {
    #[serde(default)]
    pub encoding: EncodingConfig,
    #[serde(default)]
    pub git: GitConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct GitConfig {
    #[serde(default)]
    pub lang: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct EncodingConfig {
    #[serde(default)]
    pub ignore: Vec<String>,
}

pub fn load_config(cwd: &Path) -> Result<DevkitConfig, std::io::Error> {
    let config_path = cwd.join("devkit.toml");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        match toml::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    } else {
        Ok(DevkitConfig::default())
    }
}
