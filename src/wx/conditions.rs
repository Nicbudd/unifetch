use crate::common;
use crate::wx::*;
use crate::config::Config;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json;
use chrono::{Local, Utc, DateTime, TimeZone, NaiveDateTime, Datelike, LocalResult};

use wxer_lib::*;


// WEATHER ---------------------------------------------------------------------

fn deserialize_unh_dt<'de, D>(des: D) -> Result<DateTime<Utc>, D::Error> 
    where D: serde::Deserializer<'de> {

    let s = String::deserialize(des)?;

    let dt_naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)?;

    let local_result: LocalResult<DateTime<Local>> = Local.from_local_datetime( &dt_naive); 

    let dt_local = match local_result {
        LocalResult::None => {DateTime::default()}
        LocalResult::Single(a) => a,
        LocalResult::Ambiguous(a, _) => a, // idc 
    };

    let dt_utc = dt_local.naive_utc().and_utc();

    Ok(dt_utc)
}
#[derive(Debug, Serialize, Deserialize)]
struct UNHWxEntry {
    #[serde(rename="Datetime")]
    #[serde(deserialize_with="deserialize_unh_dt")]
    dt: DateTime<Utc>,

    #[serde(rename="RecNbr")]
    record_num: usize,

    #[serde(rename="WS_mph_Avg")]
    wind_speed: f32,

    #[serde(rename="PAR_Den_Avg")]
    photo_rad: f32,

    #[serde(rename="WS_mph_S_WVT")]
    wind_speed_dev: f32,

    #[serde(rename="WindDir_SD1_WVT")]
    wind_dir_dev: f32,

    #[serde(rename="AirTF_Avg")]
    temperature_2m: f32,

    #[serde(rename="Rain_in_Tot")]
    rain: f32,

    #[serde(rename="RH")]
    relative_humidity: f32,

    #[serde(rename="WindDir_D1_WVT")]
    wind_dir: f32,
}

fn rh_to_dewpoint(temp: f32, rh: f32) -> f32 {
    let t_c = f_to_c(temp);
    
    let beta = 17.62; // constant
    let lambda = 243.12; // degrees C
    
    let ln_rh = (rh/100.).ln();
    let temp_term = (beta*t_c)/(lambda+t_c);
    let combined_term = ln_rh + temp_term;

    let dp_c = (lambda*combined_term)/(beta-combined_term);

    c_to_f(dp_c)
}

impl UNHWxEntry {
    fn to_wx_entry(self) -> WxEntry {
        let unh_station = Station {
            name: "UNH".into(),
            altitude: 28.0, //meters
            coords: (43.1348, -70.9358)
        };

        let mut layers = HashMap::new();

        layers.insert(Layer::NearSurface, WxEntryLayer { 
            layer: Layer::NearSurface, 
            height_agl: Some(6.0), 
            height_msl: Some(28.0), 
            temperature: Some(self.temperature_2m), 
            dewpoint: Some(rh_to_dewpoint(self.temperature_2m, self.relative_humidity)), 
            pressure: None, 
            wind_direction: Direction::from_degrees(self.wind_dir as u16).ok(), 
            wind_speed: Some(self.wind_speed * 0.868976), 
            visibility: None 
        });
    
        WxEntry { 
            date_time: self.dt, 
            station: unh_station, 
            layers, 
            cape: None, 
            skycover: None, 
            present_wx: None, 
            raw_metar: None, 
            precip_today: None, 
            precip: Some(Precip {
                rain: self.rain,
                snow: 0.,
                unknown: 0.,
            }), 
            precip_probability: None 
        }
    }
}

async fn get_unh_wx() -> Result<BTreeMap<DateTime<Utc>, WxEntry>, Box<dyn Error>> {
    // let unh_station = Station {
    //     name: "UNH".into(),
    //     altitude: 28.0, //meters
    //     coords: (43.1348, -70.9358)
    // };

    let day = Local::now().ordinal();
    let year = Local::now().year();

    let url = format!("https://www.weather.unh.edu/data/{year}/{day}.txt");
 
    let unh_text = reqwest::get(&url).await?.text().await?;

    let mut rdr = csv::Reader::from_reader(unh_text.as_bytes());

    let mut result_vec = BTreeMap::new();

    for entry_result in rdr.deserialize() {
        let entry: UNHWxEntry = entry_result?;
        let wx_entry: WxEntry = entry.to_wx_entry();

        result_vec.insert(wx_entry.date_time, wx_entry);
    }

    Ok(result_vec)
}

#[derive(Serialize, Deserialize, Clone)]
struct CloudLayer {
    code: String,
    height: u32,
}

impl fmt::Debug for CloudLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:5}", self.code, self.height)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct StationEntryWithTime(DateTime<Utc>, WxEntry);


async fn wxer_query(loc: &str, time: &str, config: &Config) -> Result<String, String> {

    let addresses = &config.wxer.addresses;
    
    let client = reqwest::Client::new();

    let mut err_string = String::new();

    for addr in addresses {
        let url = format!("{addr}/{loc}/{time}.json");
        // dbg!(&url);
        let q = client.get(&url)
                                .timeout(Duration::from_secs(10))
                                .send()
                                .await;

        if !q.is_err() {
            let r = q.unwrap();

            if r.status().is_success() {
                return r.text().await.map_err(|e| e.to_string());
            } else {
                err_string.push_str(&format!("{url} - {}, {}\n", r.status().as_u16(), r.status().as_str()))
            }
        } else {
            let err = q.unwrap_err();
            err_string.push_str(&format!("{url} - {:?}, {}\n", err.status(), err.to_string()));
        }
    }

    Err(format!("None of the addresses responded successfully!\n{err_string}"))
}




async fn current_conditions_handler(config: &Config) -> Result<String, String> {

    let psm_station: Station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };  

    let apt_station: Station = Station {
        coords: (43.00, 0.0), // im not giving that away
        altitude: 24.,
        name: String::from("APT"),
    }; 

    let unh_station = Station {
        name: "UNH".into(),
        altitude: 28.0, //meters
        coords: (43.1348, -70.9358)
    };


    let apt_conditions = wxer_query("local", "hourly", config).await?;
    let psm_conditions = wxer_query("psm", "hourly", config).await?;

    // dbg!(&local_conditions);
    // dbg!(&psm_conditions);

    let apt_db: BTreeMap<DateTime<Utc>, WxEntry> = serde_json::from_str(&apt_conditions).map_err(|e| e.to_string())?;
    let psm_db: BTreeMap<DateTime<Utc>, WxEntry> = serde_json::from_str(&psm_conditions).map_err(|e| e.to_string())?;
    let unh_db: BTreeMap<DateTime<Utc>, WxEntry> = get_unh_wx().await.map_err(|e| e.to_string())?;


    let mut s = common::title("CURRENT CONDITIONS");

    let latest_apt = apt_db.last_key_value()
        .ok_or(String::from("Local json did not have any values"))?;

    let latest_psm = psm_db.last_key_value()
        .ok_or(String::from("PSM json did not have any values"))?;

    let latest_unh = unh_db.last_key_value()
        .ok_or(String::from("UNH json did not have any values"))?;

    let local_time_apt: DateTime<Local> = latest_apt.0.clone().into();
    let local_time_psm: DateTime<Local> = latest_psm.0.clone().into();
    let local_time_unh: DateTime<Local> = latest_unh.0.clone().into();

    let apt_prelude = format!("{}: ⌛{}", apt_station.name, local_time_apt.format("%I:%M %p"));
    let psm_prelude = format!("{}: ⌛{}", psm_station.name, local_time_psm.format("%I:%M %p"));
    let unh_prelude = format!("{}: ⌛{}", unh_station.name, local_time_unh.format("%I:%M %p"));

    let apt_line = station_line(&apt_prelude, latest_apt.1, true, &apt_db)?;
    let psm_line = station_line(&psm_prelude, latest_psm.1, false, &psm_db)?;
    let unh_line = station_line(&unh_prelude, latest_unh.1, false, &unh_db)?;

    s.push_str(&apt_line);
    s.push_str(&unh_line);
    s.push_str(&psm_line);

    Ok(s)   
}

pub async fn current_conditions(config: &Config) {

    if !config.enabled_modules.current_conditions {
        return;
    }

    match current_conditions_handler(config).await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", common::title("CURRENT CONDITIONS"), e)},
    }
}


