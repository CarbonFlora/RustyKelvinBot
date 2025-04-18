use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{token::TokenType, RustyKelvinBot};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GeoJson {
    pub zip: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub country: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherJson {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: Main,
    pub visibility: u64,
    pub wind: Wind,
    pub clouds: Clouds,
    pub dt: i64,
    pub sys: Sys,
    pub timezone: i64,
    pub id: u64,
    pub name: String,
    pub cod: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main {
    pub temp: f64,
    #[serde(rename = "feels_like")]
    pub feels_like: f64,
    #[serde(rename = "temp_min")]
    pub temp_min: f64,
    #[serde(rename = "temp_max")]
    pub temp_max: f64,
    pub pressure: f64,
    pub humidity: f64,
    #[serde(rename = "sea_level")]
    pub sea_level: f64,
    #[serde(rename = "grnd_level")]
    pub grnd_level: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wind {
    pub speed: f64,
    pub deg: f64,
    pub gust: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clouds {
    pub all: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sys {
    #[serde(rename = "type")]
    pub type_field: u64,
    pub id: u64,
    pub country: String,
    pub sunrise: i64,
    pub sunset: i64,
}

impl Display for GeoJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {} ({}, {})",
            self.zip, self.name, self.country, self.lat, self.lon
        )
    }
}

impl Display for WeatherJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let weather = self.weather.first().expect("Weather can not be displayed.");
        write!(
            f,
            ">>> ■ {}\n{}\n{}% humidity\n{} > {} > {}°F\n{} mph winds",
            self.name,
            weather.description,
            self.main.humidity,
            self.main.temp_max.round(),
            self.main.feels_like.round(),
            self.main.temp_min.round(),
            (self.wind.speed * 10.0).round() / 10.0
        )
    }
}

impl RustyKelvinBot {
    pub async fn geo(self) {
        let response = self.clone().geo_reqwest().await;
        self.send_message(response.to_string()).await;
    }

    pub async fn weather(self) {
        let geo = self.clone().geo_reqwest().await;
        let api_key = self.tokens.get(&TokenType::OpenWeather);
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=imperial",
            geo.lat, geo.lon, api_key
        );
        let response = reqwest::get(url)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to query api for openweather. ({}, {})",
                    geo.lat, geo.lon
                )
            })
            .json::<WeatherJson>()
            .await
            .expect("Failed to parse api package as json.");
        self.send_message(response.to_string()).await;
    }

    async fn geo_reqwest(self) -> GeoJson {
        let zip_code = "91776";
        let country_code = "US";
        let api_key = self.tokens.get(&TokenType::OpenWeather);
        let url = format!(
            "http://api.openweathermap.org/geo/1.0/zip?zip={},{}&appid={}",
            zip_code, country_code, api_key
        );

        reqwest::get(url)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to query api for openweather. ({}, {})",
                    zip_code, country_code
                )
            })
            .json::<GeoJson>()
            .await
            .expect("Failed to parse api package as json.")
    }
}
