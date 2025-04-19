use std::{collections::HashMap, fs};

use toml::Table;

const TOKEN_FILE_PATH_STR: &str = "./Secrets.toml";
const OPEN_WEATHER_TOKEN: &str = "OPEN_WEATHER_TOKEN";
const DEEPSEEK_TOKEN: &str = "DEEPSEEK_TOKEN";

#[derive(Debug, Clone)]
pub struct RKBTokens {
    map: HashMap<TokenType, String>,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum TokenType {
    OpenWeather,
    DeepSeek,
}

impl TryFrom<String> for TokenType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            OPEN_WEATHER_TOKEN => Ok(TokenType::OpenWeather),
            DEEPSEEK_TOKEN => Ok(TokenType::DeepSeek),
            _ => Err(format!("Failed to parse key ({}) into token.", value)),
        }
    }
}

impl RKBTokens {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        let token_file_content =
            fs::read_to_string(TOKEN_FILE_PATH_STR).expect("Token file path is not a path.");
        token_file_content
            .parse::<Table>()
            .expect("Token file path can not be parsed into toml.")
            .into_iter()
            .filter_map(|(key, value)| {
                let (Ok(a), Some(b)) = (TokenType::try_from(key), value.as_str()) else {
                    return None;
                };
                Some((a, b.to_string()))
            })
            .for_each(|(key, value)| {
                map.insert(key, value);
            });
        Self { map }
    }

    pub fn get(&self, key: &TokenType) -> &str {
        self.map.get(key).expect("Key does not exist.")
    }
}

impl Default for RKBTokens {
    fn default() -> Self {
        Self::new()
    }
}
