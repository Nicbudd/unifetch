use crate::common;

use serde::Deserialize;
use reqwest::get;

use super::Args;
pub async fn tides(args: &Args){
    if !args.tides {
        return;
    }

    match tides_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", common::title("TIDES"), e)},
    }
}


async fn tides_handler() -> Result<String, String>  {
    let mut s = common::title("TIDES");

    Ok(s)
}