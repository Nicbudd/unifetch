use crate::common;
use crate::config::Config;
use crate::config::DistanceUnits;
use crate::config::Service;
use common::Style;
use common::TermStyle::*;
use futures::future::try_join_all;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::fmt;
use std::hash::Hash;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// EARTHQUAKES ----------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
struct USGSResponse {
    features: Vec<USGSEarthquake>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct USGSEarthquake {
    #[serde(rename = "type")]
    shaketype: String,
    properties: EarthquakeProperties,
    geometry: EarthquakeGeometry,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EarthquakeProperties {
    mag: f32,
    place: Option<String>,
    time: i64,
    updated: i64,
    url: String,
    detail: String,
    felt: Option<u32>, // I really hope an earthquake isn't felt by more than 4 billion people
    cdi: Option<f32>,
    mmi: Option<f32>,
    alert: Option<String>,
    status: Option<String>,
    tsunami: i64,
    sig: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EarthquakeGeometry {
    #[serde(rename = "type")]
    geotype: String,
    coordinates: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Earthquake {
    // put the USGS formatted earthquakes in a form that's easy for us to use.
    shaketype: String,
    mag: f32,
    place: String,
    time: DateTime<Utc>,
    felt: Option<u32>,
    mmi: Option<f32>,
    alert: Option<String>,
    latitude: f32,
    longitude: f32,
    depth: f32,
    id: String,
    dist: Option<f32>,
}

impl Hash for Earthquake {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Earthquake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Earthquake {}

impl Earthquake {
    #[allow(non_snake_case)]
    fn from_USGS(usgs: USGSEarthquake, home_coords: Option<(f32, f32)>) -> Result<Self, String> {
        // dbg!(&usgs);

        let dist = if let Some(coords) = home_coords {
            Some(distance_between_coords_miles(
                usgs.geometry.coordinates.1,
                usgs.geometry.coordinates.0,
                coords.0,
                coords.1,
            ))
        } else {
            None
        };

        Ok(Earthquake {
            shaketype: usgs.shaketype,
            mag: usgs.properties.mag,
            place: usgs
                .properties
                .place
                .unwrap_or("No Nearby Locations".into()),
            time: DateTime::<Utc>::from_timestamp(usgs.properties.time / 1000, 0)
                .ok_or(String::from("Timestamp invalid"))?,
            felt: usgs.properties.felt,
            mmi: usgs.properties.mmi,
            alert: usgs.properties.alert,
            latitude: usgs.geometry.coordinates.1,
            longitude: usgs.geometry.coordinates.0,
            depth: usgs.geometry.coordinates.2,
            dist,
            id: usgs.id,
        })
    }

    fn mag_style(&self) -> String {
        if self.mag > 8. {
            Style::string(&[RedBg, Black, Bold])
        } else if self.mag > 7. {
            Style::string(&[Red, Bold])
        } else if self.mag > 6. {
            Style::string(&[YellowBg, Black, Bold])
        } else if self.mag > 5. {
            Style::string(&[Yellow, Bold])
        } else if self.mag > 3. {
            Style::string(&[Blue, Bold])
        } else {
            Style::string(&[Bold])
        }
    }

    fn mmi_format(&self) -> String {
        if let Some(mmi) = self.mmi {
            if mmi > 11.5 {
                format!(
                    "MMI: {}XII{Reset}, ",
                    Style::string(&[RedBg, Bold, Blinking])
                )
            } else if mmi > 10.5 {
                format!(
                    "MMI: {}XI{Reset}, ",
                    Style::string(&[RedBg, Bold, Blinking])
                )
            } else if mmi > 9.5 {
                format!("MMI: {}X{Reset}, ", Style::string(&[RedBg, Bold]))
            } else if mmi > 8.5 {
                format!("MMI: {}IX{Reset}, ", Style::string(&[Red, Bold]))
            } else if mmi > 7.5 {
                format!("MMI: {}VIII{Reset}, ", Style::string(&[YellowBg, Bold]))
            } else if mmi > 6.5 {
                format!("MMI: {}VII{Reset}, ", Style::string(&[Yellow, Bold]))
            } else if mmi > 5.5 {
                format!("MMI: {}VI{Reset}, ", Style::string(&[Yellow, Bold]))
            } else if mmi > 4.5 {
                format!("MMI: {}V{Reset}, ", Style::string(&[Green, Bold]))
            } else if mmi > 3.5 {
                format!("MMI: {}IV{Reset}, ", Style::string(&[Blue, Bold]))
            } else if mmi > 2.5 {
                format!("MMI: {}III{Reset}, ", Style::string(&[Cyan, Bold]))
            } else if mmi > 1.5 {
                format!("MMI: {}II{Reset}, ", Style::string(&[Cyan, Bold]))
            } else {
                format!("MMI: I{Reset}, ")
            }
        } else {
            String::new()
        }
    }
}

impl fmt::Display for Earthquake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alert_color = if let Some(a) = &self.alert {
            if a == "green" {
                Style::string(&[Green])
            } else if a == "yellow" {
                Style::string(&[Yellow, Bold])
            } else if a == "orange" {
                Style::string(&[YellowBg, Black, Bold])
            } else if a == "red" {
                Style::string(&[RedBg, Black, Blinking, Bold])
            } else {
                Style::string(&[PurpleBg, Black, Bold])
            }
        } else {
            "".into()
        };

        let dist = if self.dist.is_some_and(|dist| dist < 1000.) {
            format!(" ({:.0} mi)", self.dist.unwrap())
        } else {
            String::new()
        };

        writeln!(
            f,
            "{}M{:.1}{Reset}, {}{:.0}km dp, {}, {}{}{Reset}{}",
            self.mag_style(),
            self.mag,
            self.mmi_format(),
            self.depth,
            self.time.format("%Y-%m-%d %H:%MZ"),
            alert_color,
            self.place,
            dist
        )
    }
}

async fn get_earthquakes(
    url: &str,
    client: &reqwest::Client,
    home_coords: Option<(f32, f32)>,
) -> Result<Vec<Earthquake>, String> {
    // dbg!(&url);

    let q = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    let r = q.map_err(|e| e.to_string())?;

    let t = r.text().await.map_err(|e| e.to_string())?;

    // dbg!(&t);

    let usgs: USGSResponse = serde_json::from_str(&t).map_err(|e| e.to_string())?;

    let quakes = usgs
        .features
        .into_iter()
        .map(|x| Earthquake::from_USGS(x, home_coords))
        .collect::<Result<Vec<_>, String>>()?;

    Ok(quakes)
}

fn tallest_skyscrapers(v: &Vec<Earthquake>) -> Vec<&Earthquake> {
    let mut result_vec = vec![];
    let mut max_seen = 0.0;

    for quake in v {
        if quake.mag > max_seen {
            max_seen = quake.mag;
            result_vec.push(quake)
        }
    }

    result_vec
}

fn distance_between_coords_miles(lat1: f32, long1: f32, lat2: f32, long2: f32) -> f32 {
    // Haversine formula
    let earth_radius = 3956.5; // miles, approx
    let phi_1 = lat1 * PI / 180.;
    let phi_2 = lat2 * PI / 180.;
    let delta_phi = (lat2 - lat1) * PI / 180.;
    let delta_lmbda = (long2 - long1) * PI / 180.;

    let a = (delta_phi / 2.).sin() * (delta_phi / 2.).sin()
        + phi_1.cos() * phi_2.cos() * (delta_lmbda / 2.).sin() * (delta_lmbda / 2.).sin();

    let c = 2. * (a.sqrt()).atan2((1. - a).sqrt());

    earth_radius * c
}

fn convert_to_km(units: &DistanceUnits, value: f32) -> f32 {
    match units {
        DistanceUnits::Kilometers => value,
        DistanceUnits::NauticalMiles => value * 1.852,
        DistanceUnits::Miles => value * 1.60934,
    }
}

async fn earthquake_handler(config: &Config) -> Result<String, String> {
    // "tallest skyscrapers" (>5 mag) for last 3 months of earthquakes
    // "local" earthquakes - earthquakes >2 mag within 150 km of PSM or >3 mag within 300km or >4 mag within 800km

    let mut s = common::title("EARTHQUAKES");

    let coords_opt = config.localization.get_coordinates(&Service::Usgs);

    // this boolean can be adjusted later
    let get_local_quakes: bool = coords_opt.is_some() && config.earthquakes.enable_local;

    let now = Utc::now();
    let three_months_ago = now - chrono::Duration::days(180);
    let starttime = three_months_ago.format("%Y-%m-%d");

    let client = reqwest::Client::new();

    // >5 mag anywhere for last 3 months of earthquakes
    let url1 = format!(
        "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime={starttime}&minmagnitude=5&orderby=time"
    );

    if get_local_quakes {
        let lat_f = coords_opt.unwrap().0;
        let long_f = coords_opt.unwrap().1;

        let lat = lat_f.round() as i32;
        let long = long_f.round() as i32;

        let mut urls = vec![];

        for rad in &config.earthquakes.local_search {
            let mag = rad.min_magnitude;
            let rad_km = convert_to_km(&config.earthquakes.units, rad.radius);
            urls.push(format!("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minmagnitude={mag}&latitude={lat}&longitude={long}&maxradiuskm={rad_km}&orderby=time"));
        }

        let mut futures = vec![];

        for url in &urls {
            futures.push(get_earthquakes(url, &client, coords_opt));
        }

        let local_quakes: Vec<Vec<Earthquake>> = try_join_all(futures).await?;
        let local_quakes: HashSet<&Earthquake> = local_quakes.iter().flatten().collect();
        let mut local_quakes: Vec<&&Earthquake> = local_quakes.iter().collect();
        local_quakes.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(Ordering::Greater));

        if !local_quakes.is_empty() {
            s.push_str("Local Earthquakes:\n");

            for q in local_quakes {
                s.push_str(&format!("{}", q));
                // dbg!(&q);
            }
            s.push('\n');
        } else {
            s.push_str("Local Earthquakes: None\n");
        }
    } else {
        s.push_str("Local Earthquakes Disabled\n");
    }

    // global quakes
    let v1 = get_earthquakes(&url1, &client, coords_opt).await?;

    let global_quakes = tallest_skyscrapers(&v1);

    s.push_str("Global Earthquakes:\n");

    for q in global_quakes {
        s.push_str(&format!("{}", q));
    }

    Ok(s)
}

use crate::config::Modules;

pub async fn earthquakes(config: &Config) {
    if !config.enabled_modules.contains(&Modules::Earthquakes)
        || (!config.earthquakes.enable_global && !config.earthquakes.enable_local)
    {
        return;
    }

    match earthquake_handler(config).await {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            println!("{}{}", common::title("EARTHQUAKE"), e)
        }
    }
}
