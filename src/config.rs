use crate::breakpoints::HandlerFunction;
use libafl_qemu::Emulator;
use serde::Deserialize;
use serde::Deserializer;
use std::fs;
use std::path::Path;

pub const CONFIG_PATH: &str = "firmware_config.json";

#[derive(Deserialize, Debug, Clone)]
pub struct FirmwareFunction {
    pub name: String,
    #[serde(deserialize_with = "hex_address_string_to_u32")]
    pub address: u32,
    pub handler: HandlerFunction,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub fuzz: bool,
    #[serde(deserialize_with = "hex_address_string_to_u32")]
    pub fuzz_target_address: u32,
    #[serde(deserialize_with = "hex_address_string_to_u32")]
    pub fuzz_target_return_address: u32,
    pub firmware: String,
    pub qemu_args: Vec<String>,
    #[serde(deserialize_with = "hex_string_to_u32")]
    pub broker_port: u32,
    #[serde(deserialize_with = "hex_string_to_u32")]
    pub timeout_seconds: u32,
    #[serde(deserialize_with = "hex_string_to_u32")]
    pub cores: u32,
    pub breakpoints: Vec<FirmwareFunction>,
}

// A custom deserializer function to convert the hex address string to a u32
fn hex_address_string_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(&s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
}

fn hex_string_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
}

pub fn parse_config(config_file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(config_file_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}
