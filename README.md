# Weather Exporter

Periodically get weather and air pollutin information from openweathermap.org and publish it as Prometheus metrics.
Uses [https://openweathermap.org/current](https://openweathermap.org/current) for weather and [https://openweathermap.org/api/air-pollution](https://openweathermap.org/api/air-pollution) for air pollution data.

## Exposed metrics

Temperature, pressure, humidity, dew point (approximated) and last update time for weather.

PM2.5, PM10 and last update time for air pollutionl.

# How to run

Get an OpenWeatherMap API key: https://home.openweathermap.org/api_keys

Run with Docker:

```sh
docker run -it --rm \
    -e LAT=50.941311520039356 \
    -e LON=6.958143531830011 \
    -e CITY=Cologne \
    -e UNITS=metric \
    -e PERIOD=600 \
    -e APPID=${OPENWEATHER_API_KEY} \
    ghcr.io/graipher/weather-exporter-rs:latest
```

## Limitations

* The units in the metric description are currently hard-coded and do not change along with the passed metrics. The environment variable only affects the units of the data returned from OpenWeatherMap.
