use std::fs;

use super::Args;

use anyhow::{self, Context, Result};
use home::home_dir;
use serde::Deserialize;


#[derive(Debug, Deserialize, Clone, Copy, Default)]
pub struct Modules {
    #[serde(default,
        alias = "currentconditions", 
        alias = "conditions", 
        alias = "analysis",
        alias = "weather",
        alias = "wx",
        alias = "wxer")]
    pub current_conditions: bool,

    #[serde(default,
        alias = "forecast", 
        alias = "future_weather", 
        alias = "futurecast",
        alias = "futurewx")]
    pub forecast: bool,

    #[serde(default,
        alias = "tele", 
        alias = "nao", 
        alias = "enso")]
    pub teleconnections: bool,

    #[serde(default,
        alias = "quake", 
        alias = "quakes", 
        alias = "earthquake",)]
    pub earthquakes: bool,

    #[serde(default,
        alias = "rand", 
        alias = "dice", 
        alias = "randomize",)]
    pub random: bool,

    #[serde(default,
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

    #[serde(default,
        alias = "tidal",
        alias = "tide",
        alias = "tidechart",
        alias = "tidecharts")]
    pub tides: bool,

    #[serde(skip)]
    pub updates: bool,
}

#[derive(Debug, Deserialize)]
pub struct Wxer {
    pub addresses: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub wxer: Wxer,

    default_modules: Modules,

    #[serde(skip)]
    pub enabled_modules: Modules,
}

pub fn read_config_file(args: &Args) -> Result<Config> {
    let home_dir = home_dir().context("Could not find users home directory.")?;
    let config_location = home_dir.join(".config").join("unifetch").join("config.toml");
    
    let mut config: Config = toml::from_str(&fs::read_to_string(config_location)?)?;

    // set the default modules (if we are running all default modules)
    if args.default || !(
        args.random || 
        args.solar_lunar || 
        args.current_conditions || 
        args.forecast || 
        args.teleconnections || 
        args.earthquakes || 
        args.tides
    ) {
            
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
