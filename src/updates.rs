use std::time::Duration;

use anyhow::{Result, Context};
use reqwest::Client;
use toml::Table;

use crate::common;
use common::TermStyle::*;
use crate::config::Config;

fn str_to_version(s: &str) -> Result<(usize, usize, usize)> {
    let nums: Vec<&str> = s.split(".").collect();
    let major: usize = nums.get(0).context("No major version")?.parse()?;
    let minor: usize = nums.get(1).context("No minor version")?.parse()?;
    let patch: usize = nums.get(2).context("No patch number")?.parse()?;

    Ok((major, minor, patch))
}

fn version_greater(current: &str, origin: &str) -> Result<bool> {
    let current = str_to_version(current)?;
    let origin = str_to_version(origin)?;

    if (origin.0 > current.0) || 
       (origin.0 == current.0 && origin.1 > current.1) ||
       (origin.0 == current.0 && origin.1 == current.1 && origin.2 > current.2) {
        
        return Ok(true);
    } else {
        return Ok(false);
    }
}

async fn latest_version() -> Result<String> {
    let client = Client::builder() 
                        .timeout(Duration::from_secs(3))
                        .build()?;

    let req = client.get("https://raw.githubusercontent.com/Nicbudd/unifetch/master/Cargo.toml")
                        .build()?;
    let resp = client.execute(req).await?;
    let body = resp.text().await?;
    
    let toml = body.parse::<Table>()?;

    let version = toml
        .get("package").context("No dependencies found.")?
        .get("version").context("No version number found")?
        .as_str().context("This does not exist")?;


    Ok(version.into())
}


use crate::config::Modules;

pub async fn updates(config: &Config) {
    if !config.enabled_modules.contains(&Modules::Updates) {
        return;
    }

    let latest: Result<String> = latest_version().await;

    if let Ok(latest) = latest {
        let this_version = env!("CARGO_PKG_VERSION"); 

        if version_greater(this_version, &latest).unwrap_or(false) {
            println!("{}Unifetch {Bold}{Green}{}{Reset} is available, you are running version {Red}{}{Reset}.\n  To get the latest version do: `git pull master && cargo build --release`", 
            common::title("VERSION"), latest, this_version)
        }
    }
}
