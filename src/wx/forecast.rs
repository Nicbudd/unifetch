use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;

use crate::common;
use common::TermStyle::*;
use common::Style;

use crate::wx::*;

use chrono::{Local, Utc, DateTime, TimeZone, NaiveDateTime, Datelike};
use chrono::Weekday::*;

use serde::Deserialize;

use wxer_lib::*;

// FORECAST --------------------------------------------------------------------

fn from_iso8601_no_seconds<'de, D>(des: D) -> Result<Vec<DateTime<Utc>>, D::Error> 
    where D: serde::Deserializer<'de> {

    let v: Vec<String> = Vec::deserialize(des)?;

    // dbg!(&v);
    
    let v_dt: Vec<DateTime<Utc>> = v.iter()
        .map(|s| {
            let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M");
            let naive = naive.map_err(serde::de::Error::custom)?;
            Ok(Utc::from_utc_datetime(&Utc, &naive))
        })
        .collect::<Result<Vec<DateTime<Utc>>, _>>()?; 
    
    // dbg!(&v_dt);

    Ok(v_dt)
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    hourly: OpenMeteoResponseHourly
}

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenMeteoResponseHourly {
    #[serde(deserialize_with="from_iso8601_no_seconds")]
    time: Vec<DateTime<Utc>>,
    #[serde(rename="temperature_2m")]
    temperature_2m: Vec<f32>,
    #[serde(rename="dew_point_2m")]
    dewpoint_2m: Vec<f32>,
    #[serde(rename="apparent_temperature")]
    feels_like: Vec<f32>,
    #[serde(rename="precipitation_probability")]
    precip_probability: Vec<f32>,
    #[serde(rename="precipitation")]
    precip: Vec<f32>,
    #[serde(rename="rain")]
    rain: Vec<f32>,
    #[serde(rename="snowfall")]
    snowfall: Vec<f32>, // get rid of this if it is all zero
    #[serde(rename="pressure_msl")]
    sea_level_pressure: Vec<f32>,
    #[serde(rename="cloud_cover")]
    cloud_cover: Vec<f32>,
    #[serde(rename="wind_speed_10m")]
    wind_speed_10m: Vec<f32>,
    #[serde(rename="wind_direction_10m")]
    wind_dir_10m: Vec<u16>,
    #[serde(rename="cape")]
    cape: Vec<f32>,
    #[serde(rename="windspeed_250hPa")]
    wind_speed_250mb: Vec<f32>,
    #[serde(rename="geopotential_height_500hPa")]
    height_500mb: Vec<f32>,
    #[serde(rename="visibility")]
    visibility: Vec<f32>
}

async fn get_open_meteo(s: &Station) -> Result<OpenMeteoResponse, String> {

    let lat = s.coords.0;
    let long = s.coords.1;

    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat:.2}&longitude={long:.2}&hourly=temperature_2m,dew_point_2m,visibility,apparent_temperature,precipitation_probability,precipitation,rain,snowfall,pressure_msl,cloud_cover,wind_speed_10m,wind_direction_10m,cape,windspeed_250hPa,geopotential_height_500hPa&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch");

    // dbg!(&url);

    let client = reqwest::Client::new();

    let q = client.get(&url)
                .timeout(Duration::from_secs(10))
                .send()
                .await;

    let r = q.map_err(|e| e.to_string())?;

    let t = r.text().await.map_err(|e| e.to_string())?;

    // dbg!(&t);
   
    serde_json::from_str(&t).map_err(|e| e.to_string())
}

fn open_meteo_to_entries(open_meteo: OpenMeteoResponse) -> Vec<WxEntry> {

    let psm_station: Station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };  

    let mut entries: Vec<WxEntry> = vec![];

    let hourly = open_meteo.hourly;

    for (idx, date_time) in hourly.time.iter().enumerate() {
        let temperature_2m = Some(hourly.temperature_2m[idx]);
        let dewpoint_2m = Some(hourly.dewpoint_2m[idx]);
        let sea_level_pressure = Some(hourly.sea_level_pressure[idx]);
        let visibility = Some(hourly.visibility[idx] / 5280.);

        let wind_direction = Direction::from_degrees(hourly.wind_dir_10m[idx]).ok();
        let wind_speed = Some(hourly.wind_speed_10m[idx]);

        let rain = hourly.rain[idx];
        let snow = hourly.snowfall[idx];
        let unknown = hourly.precip[idx] - rain - snow;

        //todo: precip in station entry structs
        #[allow(unused_variables)]
        let precip = Some(Precip {rain, snow, unknown}); 


        let mut weather = vec![];
        if rain > 0. {weather.push("RA".into())};
        if snow > 0. {weather.push("SN".into())};
        let present_wx = Some(weather);

        let mut layers = HashMap::new();

        let near_surface = WxEntryLayer {
            layer: Layer::NearSurface,
            height_agl: Some(2.0),
            height_msl: Some(psm_station.altitude),
            temperature: temperature_2m,
            dewpoint: dewpoint_2m,
            pressure: None,
            wind_direction,
            wind_speed,
            visibility,
        };

        let mut sea_level = WxEntryLayer::empty(Layer::SeaLevel);

        sea_level.pressure = sea_level_pressure;

        layers.insert(Layer::NearSurface, near_surface);
        layers.insert(Layer::SeaLevel, sea_level);


        let e = WxEntry {
            date_time: date_time.clone(),
            station: psm_station.clone(),

            layers,

            cape: Some(hourly.cape[idx]),
            skycover: None,
            present_wx,
            raw_metar: None,
            precip,
            precip_probability: Some(hourly.precip_probability[idx]),
            precip_today: None,
        };

        entries.push(e);
    }

    entries

}

fn day_of_week_style<T: TimeZone>(dt: &DateTime<T>) -> String {

    match dt.weekday() {
        Mon => Style::new(&[Red, Bold]),
        Tue => Style::new(&[Yellow, Bold]),
        Wed => Style::new(&[Green, Bold]),
        Thu => Style::new(&[Purple, Bold]),
        Fri => Style::new(&[Blue, Bold]),
        Sat => Style::new(&[Cyan, Bold]),
        Sun => Style::new(&[Bold]),
    }

}

async fn forecast_handler() -> Result<String, String> {
    let mut s = common::title("FORECAST");

    let psm_station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };

    let now = Utc::now();

    let r = get_open_meteo(&psm_station).await?;
    let entries = open_meteo_to_entries(r);



    let mut included = BTreeMap::new();

    for hours_from_now in [0, 1, 2, 3, 4, 5, 6, 9, 12, 18, 
                            (24*1 + 0), (24*1 + 6), (24*1 + 12), (24*1 + 18),
                            (24*2 + 0), (24*2 + 6), (24*2 + 12), (24*2 + 18),
                            (24*3 + 0), (24*3 + 12),
                            (24*4 + 0), (24*4 + 12),
                            (24*5 + 0), (24*5 + 12),
                            (24*6 + 0), (24*6 + 12),
                            (24*7 + 0), (24*7 + 12)] {
        let dt = now + chrono::Duration::hours(hours_from_now);
        
        let entry_idx = entries.binary_search_by(|x| x.date_time.cmp(&dt));
        let entry_idx = match entry_idx {
            Ok(s) => s,
            Err(e) => {
                if e == entries.len() {
                    e - 1
                } else {
                    e
                }
            },
        };

        let entry = entries.get(entry_idx).unwrap();

        included.insert(entry.date_time, entry);    
    }

    for (dt, entry) in included {
        let local_dt: DateTime<Local> = dt.into();
        let day_of_week_style = day_of_week_style(&local_dt);

        let prelude = format!("{day_of_week_style}{}{Reset} {}:", local_dt.format("%a"), local_dt.format("%d %l%p"));
        s.push_str(&station_line(&prelude, entry,  false, &BTreeMap::new())?);
    }


    Ok(s)
}

pub async fn forecast() {
    match forecast_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", common::title("FORECAST"), e)},
    }
}
