use std::collections::HashMap;
use std::env;
use std::time::{Duration, SystemTime};

use prometheus_exporter::{
    self,
    prometheus::register_gauge_vec,
};
use prometheus_exporter::prometheus::register_gauge;
use reqwest::Client;
use serde::Deserialize;

static URL: &str = "https://api.openweathermap.org/data/2.5/weather";


#[derive(Debug, Deserialize)]
struct Weather {
    temp: f32,
    pressure: u16,
    humidity: u8,
}

#[derive(Debug, Deserialize)]
struct OpenWeatherMapData {
    main: Weather,
}

async fn get_weather(client: &Client, params: &HashMap<&str, String>) -> Result<OpenWeatherMapData, reqwest::Error> {
    let response = client.get(URL).query(&params).send().await?;
    let json = response.json::<OpenWeatherMapData>().await?;
    Ok(json)
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").or::<String>(Ok("9185".to_string())).unwrap();
    println!("Listening on port :{}", port);
    let binding = format!("0.0.0.0:{}", port).parse().unwrap();
    let exporter = prometheus_exporter::start(binding).unwrap();

    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("lat", env::var("LAT").unwrap().to_owned());
    params.insert("lon", env::var("LON").unwrap().to_owned());
    params.insert("units", env::var("UNITS").unwrap().to_owned());
    params.insert("appid", env::var("APPID").unwrap().to_owned());
    let client = Client::new();

    let temperature = register_gauge_vec!("weather_temperature", "Outside temperature in °C", &["city"]).unwrap();
    let humidity = register_gauge_vec!("weather_humidity", "Outside humidity in %", &["city"]).unwrap();
    let pressure = register_gauge_vec!("weather_pressure", "Outside pressure in hPa", &["city"]).unwrap();
    let last_updated = register_gauge_vec!("weather_last_updated", "Last update of weather", &["city"]).unwrap();
    let process_start_time = register_gauge!("process_start_time_seconds", "Start time of the process").unwrap();

    let mut now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    process_start_time.set(now as f64);

    let city = env::var("CITY").unwrap().to_owned();

    loop {
        match get_weather(&client, &params).await {
            Ok(data) => {
                now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                println!("time={}, temperature={}, humidity={}, pressure={}", now, data.main.temp, data.main.humidity, data.main.pressure);
                temperature.get_metric_with_label_values(&[&city]).unwrap().set(data.main.temp as f64);
                humidity.get_metric_with_label_values(&[&city]).unwrap().set(data.main.humidity as f64);
                pressure.get_metric_with_label_values(&[&city]).unwrap().set(data.main.pressure as f64);
                last_updated.get_metric_with_label_values(&[&city]).unwrap().set(now as f64);
            }
            Err(err) => eprintln!("{}", err)
        }
        let _guard = exporter.wait_duration(Duration::from_secs(10 * 60));
    }
}
