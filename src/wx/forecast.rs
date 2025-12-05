use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;

use crate::common;
use crate::config::Config;
use chrono_tz::America;
use common::Style;
use common::TermStyle::*;

use crate::wx::*;

use chrono::Weekday::*;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};

use serde::Deserialize;

use wxer_lib::*;

// FORECAST --------------------------------------------------------------------

fn from_iso8601_no_seconds<'de, D>(des: D) -> Result<Vec<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<String> = Vec::deserialize(des)?;

    // dbg!(&v);

    let v_dt: Vec<DateTime<Utc>> = v
        .iter()
        .map(|s| {
            let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M");
            let naive = naive.map_err(serde::de::Error::custom)?;
            Ok(Utc::from_utc_datetime(&Utc, &naive))
        })
        .collect::<Result<Vec<DateTime<Utc>>, _>>()?;

    // dbg!(&v_dt);

    Ok(v_dt)
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    hourly: OpenMeteoResponseHourly,
}

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenMeteoResponseHourly {
    #[serde(deserialize_with = "from_iso8601_no_seconds")]
    time: Vec<DateTime<Utc>>,
    #[serde(rename = "temperature_2m")]
    temperature_2m: Vec<f32>,
    #[serde(rename = "dew_point_2m")]
    dewpoint_2m: Vec<f32>,
    // #[serde(rename="apparent_temperature")]
    // feels_like: Vec<f32>,
    #[serde(rename = "precipitation_probability")]
    precip_probability: Vec<f32>,
    #[serde(rename = "precipitation")]
    precip: Vec<f32>,
    #[serde(rename = "rain")]
    rain: Vec<f32>,
    #[serde(rename = "snowfall")]
    snowfall: Vec<f32>, // get rid of this if it is all zero
    #[serde(rename = "pressure_msl")]
    sea_level_pressure: Vec<f32>,
    // #[serde(rename = "cloud_cover")]
    // cloud_cover: Vec<f32>,
    #[serde(rename = "wind_speed_10m")]
    wind_speed_10m: Vec<f32>,
    #[serde(rename = "wind_direction_10m")]
    wind_dir_10m: Vec<u16>,
    #[serde(rename = "cape")]
    cape: Vec<f32>,
    #[serde(rename = "windspeed_250hPa")]
    wind_speed_250mb: Vec<f32>,
    #[serde(rename = "geopotential_height_500hPa")]
    height_500mb: Vec<f32>,
    #[serde(rename = "visibility")]
    visibility: Vec<f32>,
}

async fn get_open_meteo(s: &Station) -> Result<OpenMeteoResponse, String> {
    let lat = s.coords.latitude;
    let long = s.coords.longitude;

    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={lat:.2}&longitude={long:.2}&hourly=temperature_2m,dew_point_2m,visibility,apparent_temperature,precipitation_probability,precipitation,rain,snowfall,pressure_msl,cloud_cover,wind_speed_10m,wind_direction_10m,cape,windspeed_250hPa,geopotential_height_500hPa&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch"
    );

    // dbg!(&url);

    let client = reqwest::Client::new();

    let q = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    let r = q.map_err(|e| e.to_string())?;

    let t = r.text().await.map_err(|e| e.to_string())?;

    serde_json::from_str(&t).map_err(|e| e.to_string())
}

fn open_meteo_to_entries(open_meteo: OpenMeteoResponse) -> Vec<WxEntryStruct> {
    let psm_station: &'static Station = Box::leak(Box::new(Station {
        altitude: Altitude::new_const(30., Meter),
        coords: Coordinates {
            latitude: 43.08,
            longitude: -70.82,
        },
        name: "KPSM".to_string(),
        time_zone: America::New_York,
    }));

    let mut entries: Vec<WxEntryStruct> = vec![];

    let hourly = open_meteo.hourly;

    for (idx, date_time) in hourly.time.iter().enumerate() {
        let temperature_2m = Temperature::new(hourly.temperature_2m[idx], Fahrenheit);
        let dewpoint_2m = Temperature::new(hourly.dewpoint_2m[idx], Fahrenheit);
        let sea_level_pressure = Pressure::new(hourly.sea_level_pressure[idx], HPa);
        let visibility = Distance::new(hourly.visibility[idx], Feet);

        let wind_direction = Direction::from_degrees(hourly.wind_dir_10m[idx]).ok();
        let wind_speed = Speed::new(hourly.wind_speed_10m[idx], Mph);

        let wind = Wind {
            direction: wind_direction,
            speed: wind_speed,
        };

        let rain = hourly.rain[idx];
        let snow = hourly.snowfall[idx];
        let unknown = hourly.precip[idx] - rain - snow;

        let mut weather = vec![];
        if rain > 0. {
            weather.push("RA".into())
        };
        if snow > 0. {
            weather.push("SN".into())
        };
        let wx_codes = Some(weather);

        let rain = ProportionalUnit::new(rain, Inch);
        let snow = ProportionalUnit::new(snow, Inch);
        let unknown = ProportionalUnit::new(unknown, Inch);

        let precip = Some(Precip {
            rain,
            snow,
            unknown,
        });

        let mut layers = HashMap::new();

        let near_surface = WxEntryLayerStruct {
            layer: Layer::NearSurface,
            station: psm_station,
            wind: Some(wind),
            temperature: Some(temperature_2m),
            dewpoint: Some(dewpoint_2m),
            pressure: None,
            visibility: Some(visibility),
            height_msl: Some(Altitude::new(2.0, Meter)),
        };

        let mut sea_level = WxEntryLayerStruct::new(Layer::SeaLevel, psm_station);
        sea_level.pressure = Some(sea_level_pressure);

        let mut layer_250mb = WxEntryLayerStruct::new(Layer::MBAR(250), psm_station);
        let wind_speed_250mb = Speed::new(hourly.wind_speed_250mb[idx], Mph);
        layer_250mb.wind = Some(Wind {
            direction: None,
            speed: wind_speed_250mb,
        });

        // let hght = Altitude::new(hourly.height_500mb[idx], Feet);
        let layer = Layer::MBAR(500);
        let mut layer_500mb = WxEntryLayerStruct::new(layer, psm_station);
        layer_500mb.height_msl = Some(Altitude::new(hourly.height_500mb[idx], Feet)); // convert from feet to meters

        layers.insert(Layer::NearSurface, near_surface);
        layers.insert(Layer::SeaLevel, sea_level);
        layers.insert(layer_500mb.layer, layer_500mb);
        layers.insert(layer_250mb.layer, layer_250mb);

        let cape = Some(SpecEnergy::new(hourly.cape[idx], Jkg));

        let precip_probability = Some(Fraction::new(hourly.precip_probability[idx], Percent));

        let e = WxEntryStruct {
            date_time: *date_time,
            station: psm_station,
            layers,
            cape,
            skycover: None,
            wx_codes,
            raw_metar: None,
            precip,
            precip_probability,
            precip_today: None,
            altimeter: None,
        };

        entries.push(e);
    }

    entries
}

fn day_of_week_style<T: TimeZone>(dt: &DateTime<T>) -> String {
    match dt.weekday() {
        Mon => Style::string(&[Red, Bold]),
        Tue => Style::string(&[Yellow, Bold]),
        Wed => Style::string(&[Green, Bold]),
        Thu => Style::string(&[Purple, Bold]),
        Fri => Style::string(&[Blue, Bold]),
        Sat => Style::string(&[Cyan, Bold]),
        Sun => Style::string(&[Bold]),
    }
}

async fn forecast_handler(config: &Config) -> Result<String, String> {
    let mut s = common::title("FORECAST");

    s.push_str("Weather data by Open-Meteo.com (https://open-meteo.com/)\n\n");

    let psm_station: &'static Station = Box::leak(Box::new(Station {
        altitude: Altitude::new_const(30., Meter),
        coords: Coordinates {
            latitude: 43.08,
            longitude: -70.82,
        },
        name: "KPSM".to_string(),
        time_zone: America::New_York,
    }));

    let now = Utc::now();

    let r = get_open_meteo(psm_station).await?;
    let entries = open_meteo_to_entries(r);

    let mut included = BTreeMap::new();

    for hours_from_now in &config.forecast.selected.hours {
        let dt = now + chrono::Duration::hours(*hours_from_now as i64);

        let entry_idx = entries.binary_search_by(|x| x.date_time.cmp(&dt));
        let entry_idx = match entry_idx {
            Ok(s) => s,
            Err(e) => {
                if e == entries.len() {
                    e - 1
                } else {
                    e
                }
            }
        };

        let entry = entries.get(entry_idx).unwrap();

        included.insert(entry.date_time, entry);
    }

    for (dt, entry) in included {
        let local_dt: DateTime<Local> = dt.into();
        let day_of_week_style = day_of_week_style(&local_dt);

        let prelude = format!(
            "{day_of_week_style}{}{Reset} {}:",
            local_dt.format("%a"),
            local_dt.format("%d %l%p")
        );

        s.push_str(&station_line(
            &prelude,
            entry,
            &config.forecast.selected.parameters,
            false,
            &BTreeMap::new(),
        )?);
    }

    Ok(s)
}

use crate::config::Modules;

pub async fn forecast(config: &Config) {
    if !config.enabled_modules.contains(&Modules::Forecast) {
        return;
    }

    match forecast_handler(config).await {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            println!("{}{}", common::title("FORECAST"), e)
        }
    }
}
