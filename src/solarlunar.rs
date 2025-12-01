use crate::common;
use crate::config::{Config, Service};
use common::TermStyle::*;

use std::collections::HashMap;
use std::time::Duration;

use chrono::{Local, NaiveTime};
use serde_json::Value;

fn parse_navy_times(v: &Value) -> Result<NaiveTime, String> {
    let text = v.as_str().ok_or("Could not parse JSON")?;
    let date = NaiveTime::parse_from_str(text, "%H:%M").map_err(|e| e.to_string())?;

    Ok(date)
}

fn string_from_rise_set_times(
    name: &str,
    start_name: &str,
    end_name: &str,
    start: NaiveTime,
    end: NaiveTime,
) -> String {
    let s = start.format("%I:%M %p");
    let e = end.format("%I:%M %p");

    if end < start {
        format!("{name:<8} {start_name:<5} {Bold}{s}{Reset} | {end_name} {Bold}{e}{Reset}")
    } else {
        let d = end - start;
        format!(
            "{name:<8} {start_name:<5} {Bold}{s}{Reset} | {end_name} {Bold}{e}{Reset} | Duration {Bold}{}h{}m{Reset}\n",
            d.num_hours(),
            d.num_minutes() % 60
        )
    }
}

fn generate_solar_lunar_string(json: serde_json::Value) -> Result<String, String> {
    // this entire function could be written better tbh
    let data = &json["properties"]["data"];
    let sundata = &data["sundata"];
    let moondata = &data["moondata"];

    let twilight_start = parse_navy_times(&sundata[0]["time"])?;
    let sunrise = parse_navy_times(&sundata[1]["time"])?;
    let sunset = parse_navy_times(&sundata[3]["time"])?;
    let twilight_end = parse_navy_times(&sundata[4]["time"])?;

    let mut moonset = None;
    let mut moonrise = None;

    // dbg!(&moondata);

    for i in 0..5 {
        let entry = moondata.get(i);

        if let Some(entry) = entry {
            if entry["phen"] == "Set" {
                moonset = Some(parse_navy_times(&entry["time"])?);
            } else if entry["phen"] == "Rise" {
                moonrise = Some(parse_navy_times(&entry["time"])?);
            }
        }
    }

    if moonset.is_none() || moonrise.is_none() {
        Err(String::from("Unexpected values for moon data"))?;
    }

    let moonset = moonset.unwrap();
    let moonrise = moonrise.unwrap();

    let moon_phase = &data["curphase"]
        .as_str()
        .ok_or("Could not parse JSON properly")?;
    let fracillum = &data["fracillum"]
        .as_str()
        .ok_or("Could not parse JSON properly")?;

    let closest_phase = &data["closestphase"];

    let closest_name = &closest_phase["phase"]
        .as_str()
        .ok_or("Could not parse JSON properly")?;
    let closest_month = &closest_phase["month"]
        .as_i64()
        .ok_or("Could not parse JSON properly")?;
    let closest_day = &closest_phase["day"]
        .as_i64()
        .ok_or("Could not parse JSON properly")?;
    let closest_time = &closest_phase["time"]
        .as_str()
        .ok_or("Could not parse JSON properly")?;

    let closest_time: NaiveTime =
        NaiveTime::parse_from_str(closest_time, "%H:%M").map_err(|e| e.to_string())?;

    let phase_string = format!(
        "\nMoon Phase: {Bold}{} ({}){Reset} | {Bold}{}{Reset} on {Bold}{}/{} ({}){Reset}",
        moon_phase,
        fracillum,
        closest_name,
        closest_month,
        closest_day,
        closest_time.format("%I:%M %p")
    );

    Ok(format!(
        "For {Bold}{}{Reset}\n{}{}{}{}",
        Local::now().format("%b %d"),
        string_from_rise_set_times("Sun", "Rise", "Set", sunrise, sunset),
        string_from_rise_set_times("Twilight", "Begin", "End", twilight_start, twilight_end),
        string_from_rise_set_times("Moon", "Rise", "Set", moonrise, moonset),
        phase_string
    ))
} // dbg!(&r);

use crate::config::Modules;

pub async fn solar_lunar(config: &Config) {
    if !config.enabled_modules.contains(&Modules::SolarLunar) {
        return;
    }

    let mut s: String = common::title("SOLAR & LUNAR");

    let coordinates_opt = config.localization.get_coordinates(&Service::Usno);

    if coordinates_opt.is_none() {
        println!(
            "{s}Coordinates not provided, cannot get solar/lunar times from unknown location\n"
        );
        return;
    }

    let coords_str = common::coords_str(coordinates_opt.unwrap());

    let now = Local::now();

    let tz_offset = now.offset().local_minus_utc() / 60 / 60;

    let mut map = HashMap::new();

    map.insert("date", now.format("%Y-%m-%d").to_string());
    map.insert("coords", coords_str);
    map.insert("tz", tz_offset.to_string());

    let client = reqwest::Client::new();

    let r = client
        .get("https://aa.usno.navy.mil/api/rstt/oneday")
        .query(&map)
        .timeout(Duration::from_secs(5));

    // dbg!(&r);

    let form = r.send().await;

    // dbg!(&form);

    match common::parse_request_loose_json(form).await {
        Ok(json) => match generate_solar_lunar_string(json) {
            Ok(res) => s.push_str(&res),
            Err(res) => s.push_str(&res),
        },
        Err(e) => s.push_str(&e),
    }

    println!("{}\n", s);
}
