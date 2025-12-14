use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

static WEATHER_URL: &str = "https://api.openweathermap.org/data/2.5/weather";
static AIR_POLLUTION_URL: &str = "https://api.openweathermap.org/data/2.5/air_pollution";

#[derive(Debug, Deserialize)]
pub struct Weather {
    pub temp: f32,
    pub pressure: u16,
    pub grnd_level: u16,
    pub humidity: u8,
}

#[derive(Debug, Deserialize)]
pub struct OpenWeatherMapData {
    pub main: Weather,
    pub dt: u64,
}

#[derive(Debug, Deserialize)]
pub struct Components {
    // pub co: f32,
    // pub no: f32,
    // pub no2: f32,
    // pub o3: f32,
    // pub so2: f32,
    pub pm2_5: f32,
    pub pm10: f32,
    // pub nh3: f32,
}

#[derive(Debug, Deserialize)]
pub struct AirPollutionData {
    pub dt: u64,
    pub components: Components,
}

#[derive(Debug, Deserialize)]
pub struct OpenWeatherMapAirPollutionData {
    pub list: Vec<AirPollutionData>,
}

pub(crate) async fn get_weather(
    client: &Client,
    params: &HashMap<&str, String>,
) -> Result<OpenWeatherMapData, reqwest::Error> {
    #[cfg(debug_assertions)]
    println!("{:?}", params);
    let response = client.get(WEATHER_URL).query(&params).send().await?;
    let json = response.json::<OpenWeatherMapData>().await?;
    Ok(json)
}

pub(crate) async fn get_air_pollution(
    client: &Client,
    params: &HashMap<&str, String>,
) -> Result<OpenWeatherMapAirPollutionData, reqwest::Error> {
    #[cfg(debug_assertions)]
    println!("{:?}", params);
    let response = client.get(AIR_POLLUTION_URL).query(&params).send().await?;
    let json = response.json::<OpenWeatherMapAirPollutionData>().await?;
    Ok(json)
}
