use crate::common;
use crate::config::Config;
use crate::wx::*;

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::{DateTime, Local, Utc};
use serde::Deserialize;
use serde_json;

use wxer_lib::*;

// WEATHER ---------------------------------------------------------------------

// #[derive(Serialize, Deserialize, Clone)]
// struct CloudLayer {
//     code: String,
//     height: u32,
// }

// impl fmt::Debug for CloudLayer {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}@{:5}", self.code, self.height)
//     }
// }

async fn wxer_query(loc: &str, time: &str, config: &Config) -> Result<String, String> {
    let addresses = &config.wxer.addresses;

    let client = reqwest::Client::new();

    let mut err_string = String::new();

    for addr in addresses {
        let url = format!("{addr}/{loc}/{time}.json");
        // dbg!(&url);
        let q = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await;

        match q {
            Ok(r) => {
                if r.status().is_success() {
                    return r.text().await.map_err(|e| e.to_string());
                } else {
                    err_string.push_str(&format!(
                        "{url} - {}, {}\n",
                        r.status().as_u16(),
                        r.status().as_str()
                    ))
                }
            }

            Err(err) => {
                err_string.push_str(&format!("{url} - {:?}, {err}\n", err.status()));
            }
        }
    }

    Err(format!(
        "None of the addresses responded successfully!\n{err_string}"
    ))
}

#[derive(Debug, Clone, Deserialize)]
struct WxerResponse {
    #[allow(dead_code)]
    station: Station,
    data: BTreeMap<DateTime<Utc>, WxStructDeserialized>,
}

async fn current_conditions_handler(config: &Config) -> Result<String, String> {
    let mut s = common::title("CURRENT CONDITIONS");

    for x in config.weather.selected.sources.iter() {
        let conditions = wxer_query(x, "hourly", config).await?;

        let data: WxerResponse = serde_json::from_str(&conditions).map_err(|e| e.to_string())?;

        let db = data.data;

        let latest = db
            .last_key_value()
            .ok_or(format!("{} did not have any data.", x))?;

        let entry: WxEntryStruct = latest
            .1
            .to_struct()
            .map_err(|_| "Could not convert to struct.".to_string())?;

        let station = latest.1.station.clone();

        let indoor = station.name != "APT";

        let local_time: DateTime<Local> = (*latest.0).into();

        let name = if let Some(rename) = config.weather.rename_stations.get(x) {
            rename
        } else {
            &station.name
        };

        let prelude = format!("{}: ⌛{}", name, local_time.format("%I:%M %p"));
        let line = station_line(
            &prelude,
            &entry,
            &config.weather.selected.parameters,
            indoor,
            &db,
        )?;
        s.push_str(&line)
    }

    Ok(s)
}

use crate::config::Modules;

pub async fn current_conditions(config: &Config) {
    if !config.enabled_modules.contains(&Modules::CurrentConditions) {
        return;
    }

    match current_conditions_handler(config).await {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            println!("{}{}", common::title("CURRENT CONDITIONS"), e)
        }
    }
}
