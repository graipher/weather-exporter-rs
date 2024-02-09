# Weather Exporter

Periodically get weather information from openweathermap.org and publish it as Prometheus metrics.

## Exposed metrics

Temperature, pressure, humidity and last update time.

# How to run

Get an OpenWeatherMap API key: https://home.openweathermap.org/api_keys

Build and run with Docker:

```sh
docker build -t weather-exporter .
docker run --it -rm \
    -e LAT=50.941311520039356 \
    -e LON=6.958143531830011 \
    -e UNITS=metric \
    -e CITY=Cologne \
    -e APPID=${OPENWEATHER_API_KEY} \
    weather-exporter
```

## Limitations

Currently not configurable are

* The time between calls to the API is not configurable. It is currently set to 10 minutes (in order not to exceed the 1000 free API calls per day).
* The units in the metric description are currently hard-coded and do not change along with the passed metrics. The environment variable only affects the units of the data returned from OpenWeatherMap.
