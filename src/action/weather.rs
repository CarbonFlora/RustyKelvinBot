use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{err::RKBServiceRequestErr, token::TokenType, RKBServiceRequest};

#[derive(Debug, Error)]
pub enum Error {
    #[error("placeholder")]
    Placeholder,
    #[error("failed to query for openweather")]
    OpenWeatherQueryError,
    #[error("failed to parse openweather")]
    OpenWeatherParseError,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GeoJson {
    pub zip: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub country: String,
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

/// From openweathermap-0.2.4
/// Location coordinates

#[derive(Deserialize, Debug)]

pub struct Coord {
    /// geo location, longitude
    pub _lon: f64,

    /// geo location, latitude
    pub _lat: f64,
}

/// Weather condition description

#[derive(Deserialize, Debug)]

pub struct Weather {
    /// Weather condition id
    pub _id: u64,

    /// Group of weather parameters (Rain, Snow, Extreme etc.)
    pub _main: String,

    /// Weather condition
    pub description: String,

    /// Weather icon id
    pub _icon: String,
}

/// Detailed weather report

#[derive(Deserialize, Debug)]

pub struct Main {
    /// Temperature. Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit.
    pub _temp: f64,

    /// Temperature. This temperature parameter accounts for the human perception of weather.
    /// Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit.
    pub feels_like: f64,

    /// Atmospheric pressure (on the sea level, if there is no sea_level or grnd_level data), hPa
    pub _pressure: f64,

    /// Humidity, %
    pub humidity: f64,

    /// Minimum temperature at the moment.
    /// This is minimal currently observed temperature (within large megalopolises and urban areas).
    /// Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit.
    pub temp_min: f64,

    /// Maximum temperature at the moment.
    /// This is maximal currently observed temperature (within large megalopolises and urban areas).
    /// Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit.
    pub temp_max: f64,

    /// Atmospheric pressure on the sea level, hPa
    pub _sea_level: Option<f64>,

    /// Atmospheric pressure on the ground level, hPa
    pub _grnd_level: Option<f64>,
}

/// Detailed wind report

#[derive(Deserialize, Debug)]

pub struct Wind {
    /// Wind speed. Unit Default: meter/sec, Metric: meter/sec, Imperial: miles/hour.
    pub speed: f64,

    /// Wind direction, degrees (meteorological)
    pub _deg: f64,

    /// Wind gust. Unit Default: meter/sec, Metric: meter/sec, Imperial: miles/hour
    pub _gust: Option<f64>,
}

/// Detailed clouds report

#[derive(Deserialize, Debug)]

pub struct Clouds {
    /// Cloudiness, %
    pub _all: f64,
}

/// Rain or snow volume report

#[derive(Deserialize, Debug)]

pub struct Volume {
    /// Volume for the last 1 hour, mm
    #[serde(rename = "1h")]
    pub _h1: Option<f64>,

    /// Volume for the last 3 hours, mm
    #[serde(rename = "3h")]
    pub _h3: Option<f64>,
}

/// Additional information

#[derive(Deserialize, Debug)]

pub struct Sys {
    /// Internal parameter
    #[serde(rename = "type")]
    pub _type_: Option<u64>,

    /// Internal parameter
    pub _id: Option<u64>,

    /// Internal parameter
    pub _message: Option<f64>,

    /// Country code (GB, JP etc.)
    pub _country: String,

    /// Sunrise time, unix, UTC
    pub _sunrise: i64,

    /// Sunset time, unix, UTC
    pub _sunset: i64,
}

#[derive(Deserialize, Debug)]

/// current weather report in a nested struct
pub struct WeatherJson {
    /// report origin coordinates
    pub _coord: Coord,

    /// vector with one item of weather condition descriptions
    pub weather: Vec<Weather>,

    /// Internal parameter
    pub _base: String,

    /// detailed weather report
    pub main: Main,

    /// Visibility, meter
    pub _visibility: u64,

    /// detailed wind report
    pub wind: Wind,

    /// detailed clouds report
    pub _clouds: Clouds,

    /// detailed rain report
    pub _rain: Option<Volume>,

    /// detailed snow report
    pub _snow: Option<Volume>,

    /// Time of data calculation, unix, UTC
    pub _dt: i64,

    /// additional information
    pub _sys: Sys,

    /// Shift in seconds from UTC
    pub _timezone: i64,

    /// City ID
    pub _id: u64,

    /// City name
    pub name: String,

    /// Internal parameter
    pub _cod: u64,
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

impl RKBServiceRequest {
    pub async fn geo(self) -> Result<(), RKBServiceRequestErr> {
        let response = self.clone().geo_reqwest().await?;
        self.try_send_message(response.to_string()).await?;
        Ok(())
    }

    pub async fn weather(self) -> Result<(), RKBServiceRequestErr> {
        let geo = self.clone().geo_reqwest().await?;
        let api_key = self.tkn.get(&TokenType::OpenWeather)?;
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
        self.try_send_message(response.to_string()).await?;
        Ok(())
    }

    async fn geo_reqwest(self) -> Result<GeoJson, RKBServiceRequestErr> {
        let zip_code = "91776";
        let country_code = "US";
        let api_key = self.tkn.get(&TokenType::OpenWeather)?;
        let url = format!(
            "http://api.openweathermap.org/geo/1.0/zip?zip={},{}&appid={}",
            zip_code, country_code, api_key
        );

        let query_response = reqwest::get(url)
            .await
            .map_err(|_| Error::OpenWeatherQueryError)?;
        let geojson = query_response
            .json::<GeoJson>()
            .await
            .map_err(|_| Error::OpenWeatherParseError)?;
        Ok(geojson)
    }
}
