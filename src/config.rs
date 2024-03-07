use std::fs;

use super::Args;

use anyhow::{self, Context, Result};
use home::home_dir;
use serde::Deserialize;
use serde;

use crate::tides;

// one of the stupidest functions I've ever written
fn t() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
pub struct Modules {
    #[serde(default = "t",
        alias = "currentconditions", 
        alias = "conditions", 
        alias = "analysis",
        alias = "weather",
        alias = "wx",
        alias = "wxer")]
    pub current_conditions: bool,

    #[serde(default = "t",
        alias = "forecast", 
        alias = "future_weather", 
        alias = "futurecast",
        alias = "futurewx")]
    pub forecast: bool,

    #[serde(default = "t",
        alias = "tele", 
        alias = "nao", 
        alias = "enso")]
    pub teleconnections: bool,

    #[serde(default = "t",
        alias = "quake", 
        alias = "quakes", 
        alias = "earthquake",)]
    pub earthquakes: bool,

    #[serde(default = "t",
        alias = "rand", 
        alias = "dice", 
        alias = "randomize",)]
    pub random: bool,

    #[serde(default = "t",
        alias = "solar", 
        alias = "lunar", 
        alias = "sunrise",
        alias = "sunset",
        alias = "moonrise",
        alias = "moonset",
        alias = "moonphase",
        alias = "daylight",
        alias = "sunandmoon",)]
    pub solarlunar: bool,

    #[serde(default = "t",
        alias = "tidal",
        alias = "tide",
        alias = "tidechart",
        alias = "tidecharts")]
    pub tides: bool,

    #[serde(skip)]
    pub updates: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Wxer,
    Usno,
    Usgs,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Localization {
    latitude: Option<f32>,
    longitude: Option<f32>,
    altitude: Option<f32>,

    #[serde(default)]
    allowed_services: Vec<Service>,
}

impl Localization {
    // this function allows services to access the coordinates
    // it's not secure and is easy to "fool", but all of the modules are 
    // isolated and trusted for now.
    pub fn get_coordinates(&self, service: &Service) -> Option<(f32, f32)> {
        if self.allowed_services.contains(service) &&
            self.latitude.is_some() && self.longitude.is_some() {
            return Some((self.latitude.unwrap(), self.longitude.unwrap()));

        } else {
            return None;
        }
    }
    pub fn get_altitude(&self, service: &Service) -> Option<f32> {
        if self.allowed_services.contains(service) {
            return self.altitude;
        } else {
            return None;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Wxer {
    pub addresses: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {

}

#[derive(Debug, Deserialize, Default)]
pub enum DistanceUnits {
    #[serde(alias = "mi", alias = "english", alias = "imperial")]
    Miles,

    #[default]
    #[serde(alias = "km", alias = "metric")]
    Kilometers,

    #[serde(alias = "nmi", alias = "nauts")]
    NauticalMiles,
}

#[derive(Debug, Deserialize)]
pub struct EarthquakeRadii {
    pub min_magnitude: f32,
    pub radius: f32,
}

#[derive(Debug, Deserialize)]
pub struct Earthquakes {
    #[serde(default)]
    pub units: DistanceUnits,

    #[serde(default = "t")]
    pub enable_local: bool, // must allow Usgs to access coordinates
    
    #[serde(default = "t")]
    pub enable_global: bool,
    
    #[serde(default)]
    pub local_search: Vec<EarthquakeRadii>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(skip_serializing)]
    pub localization: Localization,

    pub general: GeneralConfig,
    pub wxer: Wxer,
    pub tides: Vec<tides::TidalStation>,

    default_modules: Modules,

    pub earthquakes: Earthquakes,

    #[serde(skip)]
    pub enabled_modules: Modules,
}

pub fn read_config_file(args: &Args) -> Result<Config> {
    let home_dir = home_dir().context("Could not find users home directory.")?;
    let config_location = home_dir
        .join(".config").join("unifetch").join("config.toml");
    
    let mut config: Config = toml::from_str(&fs::read_to_string(config_location)?)?;

    // set the default modules (if we are running all default modules)
    if args.default || 
        !(args.random || args.solar_lunar || args.current_conditions || 
        args.forecast || args.teleconnections || args.earthquakes || args.tides)
    {
            
        config.enabled_modules = config.default_modules.clone();
    } 

    // set config enabled modules if arg explicitly enables it
    config.enabled_modules.random |= args.random;
    config.enabled_modules.solarlunar |= args.solar_lunar;
    config.enabled_modules.current_conditions |= args.current_conditions;
    config.enabled_modules.forecast |= args.forecast;
    config.enabled_modules.teleconnections |= args.teleconnections;
    config.enabled_modules.earthquakes |= args.earthquakes;
    config.enabled_modules.tides |= args.tides;

    config.enabled_modules.updates = !args.disable_update_notif;

    Ok(config)
}
