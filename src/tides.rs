use std::fmt::Display;

use crate::common;
use crate::config::Config;
use common::TermStyle::*;
use common::*;

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone, Utc};
use futures::future::try_join_all;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TidalStation {
    id: u64,
    #[serde(alias = "name")]
    short_name: String,
}

use crate::config::Modules;

pub async fn tides(config: &Config) {
    if !config.enabled_modules.contains(&Modules::Tides) {
        return;
    }

    match tides_handler(config).await {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            println!("{}{}", common::title("TIDES"), e)
        }
    }
}

fn tidal_time<'de, D>(des: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(des)?;
    let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M");
    let naive = naive.map_err(serde::de::Error::custom)?;
    Ok(Utc::from_utc_datetime(&Utc, &naive))
}

fn string_to_f32<'de, D>(des: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(des)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Debug)]
struct Tides {
    predictions: Vec<TideHighLow>,
}

#[derive(Deserialize, Debug)]
struct TideHighLow {
    #[serde(deserialize_with = "tidal_time")]
    t: DateTime<Utc>,
    #[serde(deserialize_with = "string_to_f32")]
    v: f32,
    #[serde(rename = "type")]
    peak: char,
}

impl Display for TideHighLow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let highlowstyle = high_low_style(self.peak);
        let time: DateTime<Local> = DateTime::from(self.t);
        let time_str = time.format("%I:%M %p %a");
        write!(
            f,
            "{highlowstyle}{} {:.1}ft{Reset} {}",
            self.peak, self.v, time_str
        )
    }
}

fn high_low_style(peak: char) -> String {
    match peak {
        'L' | 'l' => Style::string(&[Blue]),
        'H' | 'h' => Style::string(&[Red]),
        _ => String::new(),
    }
}

async fn do_tide_station(station: &TidalStation) -> Result<String, String> {
    let station_id = station.id;
    let now = Utc::now();
    let yesterday = (now - Duration::days(1)).format("%Y%m%d");
    let tomorrow = (now + Duration::days(1)).format("%Y%m%d");

    let url = format!(
        "https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?product=predictions&begin_date={yesterday}&end_date={tomorrow}&datum=MLLW&station={station_id}&time_zone=gmt&units=english&interval=hilo&format=json"
    );

    // dbg!(&url);

    let req = reqwest::get(url).await.map_err(|x| x.to_string())?;
    let text = req.text().await.map_err(|x| x.to_string())?;
    let tides: Tides = serde_json::from_str(&text).map_err(|x| x.to_string())?;

    // find the first tide after now
    let mut idxs = vec![0, 1, 2]; // default to the first few
    for (i, t) in tides.predictions.iter().enumerate() {
        // if we come across the first tide after now
        if t.t > now {
            // get the previous tide, the first tide after now, and then the next one
            idxs = vec![i - 1, i, i + 1];
            break;
        }
    }

    let mut key_tides = vec![];
    for i in idxs {
        let t = tides.predictions.get(i);

        if let Some(t) = t {
            key_tides.push(t);
        }
    }

    let s = key_tides
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    Ok(format!("{Bold}{}{Reset}: {s}\n", station.short_name))
}

async fn tides_handler(config: &Config) -> Result<String, String> {
    let s = common::title("TIDES");

    let mut futures = vec![];

    for station in &config.tides {
        futures.push(do_tide_station(station))
    }

    let string = try_join_all(futures).await?.join("");

    Ok(s + &string)
}
