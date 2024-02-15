# Weather Exporter

Periodically get weather information from openweathermap.org and publish it as Prometheus metrics.

## Exposed metrics

Temperature, pressure, humidity, dew point (approximated) and last update time.

# How to run

Get an OpenWeatherMap API key: https://home.openweathermap.org/api_keys

Run with Docker:

```sh
docker run -it --rm \
    -e LAT=50.941311520039356 \
    -e LON=6.958143531830011 \
    -e CITY=Cologne \
    -e UNITS=metric \
    -e PERIOD = 600 \
    -e APPID=${OPENWEATHER_API_KEY} \
    ghcr.io/graipher/weather-exporter-rs:v0.1.1
```

## Limitations

* The units in the metric description are currently hard-coded and do not change along with the passed metrics. The environment variable only affects the units of the data returned from OpenWeatherMap.
