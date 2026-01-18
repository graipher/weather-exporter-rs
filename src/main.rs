use std::collections::HashMap;
use std::env;
use std::time::{Duration, SystemTime};

use prometheus_exporter::{self, prometheus::register_gauge, prometheus::register_gauge_vec};
use reqwest::Client;
use serde_json::from_str;

mod openweathermap;
mod utils;
use crate::openweathermap::{get_air_pollution, get_weather};
use crate::utils::dew_point_calc;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").unwrap_or("9185".to_string());
    let period = Duration::from_secs(
        from_str::<u64>(&env::var("PERIOD").unwrap_or("600".to_string())).unwrap(),
    );
    let binding = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Listening on {}", binding);
    println!("Updating every {:?}", period);
    let exporter = prometheus_exporter::start(binding).unwrap();

    let params: HashMap<&str, String> = vec![
        ("lat", env::var("LAT").expect("LAT not set")),
        ("lon", env::var("LON").expect("LON not set")),
        ("units", "metric".to_string()),
        ("appid", env::var("APPID").expect("APPID not set")),
    ]
    .into_iter()
    .collect();
    let client = Client::new();

    let temperature = register_gauge_vec!(
        "weather_temperature",
        "Outside temperature in °C",
        &["city"]
    )
    .unwrap();
    let temperature_feels_like = register_gauge_vec!(
        "weather_temperature_feels_like",
        "Outside temperature feels-like in °C",
        &["city"]
    )
    .unwrap();
    let temperature_min = register_gauge_vec!(
        "weather_temperature_min",
        "Outside temperature min in °C",
        &["city"]
    )
    .unwrap();
    let temperature_max = register_gauge_vec!(
        "weather_temperature_max",
        "Outside temperature max in °C",
        &["city"]
    )
    .unwrap();
    let weather_description = register_gauge_vec!(
        "weather_description",
        "Weather description",
        &["city", "main", "description"]
    )
    .unwrap();
    let dew_point =
        register_gauge_vec!("weather_dew_point", "Outside dew point in °C", &["city"]).unwrap();
    let humidity =
        register_gauge_vec!("weather_humidity", "Outside humidity in %", &["city"]).unwrap();
    let pressure =
        register_gauge_vec!("weather_pressure", "Outside pressure in hPa", &["city"]).unwrap();
    let pressure_grnd = register_gauge_vec!(
        "weather_pressure_grnd",
        "Outside pressure at ground level in hPa",
        &["city"]
    )
    .unwrap();
    let air_pollution_pm_2_5 = register_gauge_vec!(
        "air_pollution_pm_2_5",
        "PM2.5 concentration in µg/m³",
        &["city"]
    )
    .unwrap();
    let air_pollution_pm_10 = register_gauge_vec!(
        "air_pollution_pm_10",
        "PM10 concentration in µg/m³",
        &["city"]
    )
    .unwrap();
    let weather_last_updated =
        register_gauge_vec!("weather_last_updated", "Last update of weather", &["city"]).unwrap();
    let air_pollution_last_updated = register_gauge_vec!(
        "air_pollution_last_updated",
        "Last update of air pollution",
        &["city"]
    )
    .unwrap();
    let process_start_time =
        register_gauge!("process_start_time_seconds", "Start time of the process").unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    process_start_time.set(now as f64);

    let compile_datetime = compile_time::datetime_str!();
    let rustc_version = compile_time::rustc_version_str!();
    let rust_info = register_gauge_vec!(
        "rust_info",
        "Info about the Rust version",
        &["rustc_version", "compile_time", "version"]
    )
    .unwrap();
    rust_info
        .get_metric_with_label_values(&[rustc_version, compile_datetime, env!("CARGO_PKG_VERSION")])
        .unwrap()
        .set(1.);
    let mut city = "Unknown".to_string();
    loop {
        match get_weather(&client, &params).await {
            Ok(data) => {
                city = data.name;
                let d = dew_point_calc(data.main.temp, data.main.humidity as f32);
                println!(
                    "time={}, temperature={}, humidity={}, pressure={}, dewpoint={}",
                    data.dt, data.main.temp, data.main.humidity, data.main.pressure, d
                );
                temperature
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.temp as f64);
                temperature_feels_like
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.feels_like as f64);
                temperature_min
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.temp_min as f64);
                temperature_max
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.temp_max as f64);
                humidity
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.humidity as f64);
                pressure
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.pressure as f64);
                pressure_grnd
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.grnd_level as f64);
                dew_point
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(d as f64);
                weather_description
                    .get_metric_with_label_values(&[
                        &city,
                        &data.weather[0].main,
                        &data.weather[0].description,
                    ])
                    .unwrap()
                    .set(1.0_f64);
                weather_last_updated
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.dt as f64);
            }
            Err(err) => eprintln!("{}", err),
        }
        match get_air_pollution(&client, &params).await {
            Ok(data) => {
                println!(
                    "time={}, pm2_5={}, pm10={}",
                    data.list[0].dt, data.list[0].components.pm2_5, data.list[0].components.pm10
                );
                air_pollution_pm_2_5
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.list[0].components.pm2_5 as f64);
                air_pollution_pm_10
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.list[0].components.pm10 as f64);
                air_pollution_last_updated
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.list[0].dt as f64);
            }
            Err(err) => eprintln!("{}", err),
        }
        let _guard = exporter.wait_duration(period);
    }
}
