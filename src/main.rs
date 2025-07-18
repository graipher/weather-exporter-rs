use std::collections::HashMap;
use std::env;
use std::time::{Duration, SystemTime};

use prometheus_exporter::{self, prometheus::register_gauge, prometheus::register_gauge_vec};
use reqwest::Client;
use serde::Deserialize;
use serde_json::from_str;

static URL: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Debug, Deserialize)]
struct Weather {
    temp: f32,
    pressure: u16,
    grnd_level: u16,
    humidity: u8,
}

#[derive(Debug, Deserialize)]
struct OpenWeatherMapData {
    main: Weather,
}

async fn get_weather(
    client: &Client,
    params: &HashMap<&str, String>,
) -> Result<OpenWeatherMapData, reqwest::Error> {
    println!("{:?}", params);
    let response = client.get(URL).query(&params).send().await?;
    let json = response.json::<OpenWeatherMapData>().await?;
    Ok(json)
}

const B: f32 = 17.368;
const C: f32 = 238.88;

fn gamma(t: f32, rh: f32) -> f32 {
    (rh / 100.0).ln() + B * t / (C + t)
}

fn dew_point_calc(t: f32, rh: f32) -> f32 {
    let g = gamma(t, rh);
    C * g / (B - g)
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .or::<String>(Ok("9185".to_string()))
        .unwrap();
    let period = Duration::from_secs(
        from_str::<u64>(
            &env::var("PERIOD")
                .or::<String>(Ok("600".to_string()))
                .unwrap(),
        )
        .unwrap(),
    );
    let binding = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("Listening on {}", binding);
    println!("Updating every {:?}", period);
    let exporter = prometheus_exporter::start(binding).unwrap();

    let mut params: HashMap<&str, String> = HashMap::new();
    params.insert("lat", env::var("LAT").expect("LAT not set").to_owned());
    params.insert("lon", env::var("LON").expect("LON not set").to_owned());
    params.insert(
        "units",
        env::var("UNITS").expect("UNITS not set").to_owned(),
    );
    params.insert(
        "appid",
        env::var("APPID").expect("APPID not set").to_owned(),
    );
    let client = Client::new();

    let temperature = register_gauge_vec!(
        "weather_temperature",
        "Outside temperature in °C",
        &["city"]
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
    let last_updated =
        register_gauge_vec!("weather_last_updated", "Last update of weather", &["city"]).unwrap();
    let process_start_time =
        register_gauge!("process_start_time_seconds", "Start time of the process").unwrap();
    let mut now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    process_start_time.set(now as f64);

    let city = env::var("CITY").expect("CITY not set").to_owned();

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

    loop {
        match get_weather(&client, &params).await {
            Ok(data) => {
                now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let d = dew_point_calc(data.main.temp, data.main.humidity as f32);
                println!(
                    "time={}, temperature={}, humidity={}, pressure={}, dewpoint={}",
                    now, data.main.temp, data.main.humidity, data.main.pressure, d
                );
                temperature
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(data.main.temp as f64);
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
                last_updated
                    .get_metric_with_label_values(&[&city])
                    .unwrap()
                    .set(now as f64);
            }
            Err(err) => eprintln!("{}", err),
        }
        let _guard = exporter.wait_duration(period);
    }
}
