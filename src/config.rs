use std::collections::HashSet;
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

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Modules {
    #[serde(alias = "currentconditions", alias = "current_conditions",
        alias = "conditions", 
        alias = "analysis",
        alias = "weather",
        alias = "wx",
        alias = "wxer",
        alias = "current_weather", alias = "currentweather")]
    CurrentConditions,

    #[serde(alias = "forecast", 
        alias = "future_weather", 
        alias = "futurecast",
        alias = "futurewx")]
    Forecast,

    #[serde(alias = "tele", 
        alias = "nao", 
        alias = "enso")]
    Teleconnections,

    #[serde(alias = "quake", 
        alias = "quakes", 
        alias = "earthquake",)]
    Earthquakes,

    #[serde(alias = "rand", 
        alias = "dice", 
        alias = "randomize",)]
    Random,

    #[serde(alias = "solar", 
        alias = "lunar", 
        alias = "sunrise",
        alias = "sunset",
        alias = "moonrise",
        alias = "moonset",
        alias = "moonphase",
        alias = "riseset",
        alias = "daylight",
        alias = "sunandmoon",)]
    SolarLunar,

    #[serde(alias = "tidal",
        alias = "tide",
        alias = "tidechart",
        alias = "tidecharts")]
    Tides,

    #[serde(skip)]
    Updates,
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
    // intentionally left blank for now
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Teleconnections {
    Enso,
    Nao,
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
    pub enable_local: bool, // must allow USGS to access coordinates
    
    #[serde(default = "t")]
    pub enable_global: bool,
    
    #[serde(default)]
    pub local_search: Vec<EarthquakeRadii>,
}

// modules run when no specific CLI args are given.
#[derive(Debug, Deserialize)]
pub struct DefaultModules {
    standard: HashSet<Modules>,
    verbose: HashSet<Modules>,
    extra_verbose: HashSet<Modules>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum WxParams {
    #[serde(alias = "IFR", alias = "VFR", alias = "flight_rules")]
    FlightRules,

    #[serde(alias = "temp", alias = "2m_temperature", alias = "temperature_2m", 
    alias = "outdoor_temp", alias = "outdoortemp")]
    Temperature,

    #[serde(alias = "feels_like", alias = "feelslike", alias = "feels",
    alias = "apparent_temp",
    alias = "heatindex", alias = "heat_index", 
    alias = "windchill", alias = "wind_chill")]
    ApparentTemp,

    #[serde(alias = "pres", alias = "mslp", alias = "slp")]
    Pressure,

    #[serde(alias = "dew")]
    Dewpoint,

    #[serde(alias = "rh", alias = "humidity", alias = "humid")]
    RelativeHumidity,

    #[serde(alias = "vis")]
    Visibility,

    #[serde(alias = "wx_codes", alias = "wx_code", alias = "wxcodes",
    alias = "weather", 
    alias = "weather_code", alias = "code",)]
    WxCode,

    #[serde(alias = "250mb_wind", alias = "250mbwind", alias = "250mb_winds", alias = "250mbwinds",
    alias = "250hpa_wind", alias = "250hpawind", alias = "250hpa_winds", alias = "250hpawinds",
    alias = "jetstream", alias = "jetstreak",
    alias = "upper_level_wind", alias = "upper_level_winds", 
    alias = "upper_level_windspeed", alias = "upper_level_wind_speed")]
    Wind250mb,

    #[serde(alias = "500mb_height", alias = "500mbheight", alias = "500mbhght",
    alias = "500mb_geopotential_height", alias = "500mb_geopot_height",
    alias = "500hpa_height", alias = "500hpaheight", alias = "500hpahght",
    alias = "500hpa_geopotential_height", alias = "500hpa_geopot_height",
    alias = "troughs", alias = "trough", alias = "ridges", alias = "ridge")]
    Height500mb,

    #[serde(alias = "wind", alias = "surface_wind", alias = "wind_speed",
    alias = "10m_wind", alias = "windspeed")]
    Wind,

    #[serde(alias = "clouds", alias = "cloud_layers")]
    Cloud,

    #[serde(alias = "sbcape", alias = "CAPE")]
    Cape,

    #[serde(alias = "raw_metar", alias = "METAR")]
    Metar,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ForecastVerboseParams {
    pub parameters: Vec<WxParams>,
    pub hours: Vec<u32>
}

#[derive(Debug, Deserialize)]
pub struct ForecastConfig {
    standard: ForecastVerboseParams,
    verbose: ForecastVerboseParams,
    extra_verbose: ForecastVerboseParams,

    #[serde(skip)]
    pub selected: ForecastVerboseParams,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ConditionsVerboseParams {
    pub parameters: Vec<WxParams>,
    // pub hours: Vec<u32>

    // sources: Vec<WxSourceOptions>,

    // #[serde(skip)]
    // pub stations: Vec<WxConditionStation>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionsConfig {
    standard: ConditionsVerboseParams,
    verbose: ConditionsVerboseParams,
    extra_verbose: ConditionsVerboseParams,

    #[serde(skip)]
    pub selected: ConditionsVerboseParams,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(skip_serializing)]
    pub localization: Localization,

    pub general: GeneralConfig,
    
    pub wxer: Wxer,
    
    pub current_weather: ConditionsConfig,
    pub teleconnections: Vec<Teleconnections>,
    pub tides: Vec<tides::TidalStation>,
    pub earthquakes: Earthquakes,
    pub forecast: ForecastConfig,

    default_modules: DefaultModules,
    #[serde(skip)]
    pub enabled_modules: HashSet<Modules>,
}

pub fn read_config_file(args: &Args) -> Result<Config> {
    let home_dir = home_dir().context("Could not find users home directory.")?;
    let config_location = home_dir
        .join(".config").join("unifetch").join("config.toml");
    
    let mut config: Config = toml::from_str(&fs::read_to_string(config_location)?)?;

    // enable all default modules if we are running default.
    if args.default {
        // clone the default modules
        match args.verbose {
            0 => {config.enabled_modules = config.default_modules.standard.clone()}
            1 => {config.enabled_modules = config.default_modules.verbose.clone()}
            2 => {config.enabled_modules = config.default_modules.extra_verbose.clone()}
            _ => unreachable!()
        }
    } 

    match args.verbose {
        0 => {
            config.forecast.selected = config.forecast.standard.clone();
            config.current_weather.selected = config.current_weather.standard.clone();
        }
        1 => {
            config.forecast.selected = config.forecast.verbose.clone();
            config.current_weather.selected = config.current_weather.verbose.clone();
        }
        2 => {
            config.forecast.selected = config.forecast.extra_verbose.clone();
            config.current_weather.selected = config.current_weather.extra_verbose.clone();
        }
        _ => unreachable!()
    }

    // set config enabled modules if arg explicitly enables it
    if args.random {config.enabled_modules.insert(Modules::Random);}
    if args.solar_lunar {config.enabled_modules.insert(Modules::SolarLunar);}
    if args.current_conditions {config.enabled_modules.insert(Modules::CurrentConditions);}
    if args.forecast {config.enabled_modules.insert(Modules::Forecast);}
    if args.teleconnections {config.enabled_modules.insert(Modules::Teleconnections);}
    if args.earthquakes {config.enabled_modules.insert(Modules::Earthquakes);}
    if args.tides {config.enabled_modules.insert(Modules::Tides);}

    if !args.disable_update_notif {config.enabled_modules.insert(Modules::Updates);}

    Ok(config)
}
