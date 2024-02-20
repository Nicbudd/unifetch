use std::num::ParseFloatError;

use crate::wx::*;

pub async fn teleconnections(){
    match teleconnections_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", common::title("TELECONNECTIONS"), e)},
    }
}

async fn get_enso() -> Result<Vec<f32>, String> {
    let url = "https://psl.noaa.gov/enso/mei/data/meiv2.data";

    let data = reqwest::get(url)
        .await.map_err(|x| x.to_string())?
        .text().await.map_err(|x| x.to_string())?;


    // let mut reader = csv::ReaderBuilder::new()
    //                                     .delimiter(b'\t')
    //                                     .has_headers(false)
    //                                     .from_reader(data.as_bytes());

    let mut all_months = vec![];

    for line in data.lines() {
        let mut split = line.split_ascii_whitespace();

        // make sure the first bit of text is a year.
        let year = split.next().map(|x| x.parse::<i32>());

        let _ = match year {
            None => continue,
            Some(Err(_)) => continue,
            Some(Ok(r)) => r
        };

        let months = split
            .map(|x| x.parse::<f32>())
            .collect::<Result<Vec<f32>, ParseFloatError>>()
            .map_err(|x| x.to_string())?;

        all_months.extend_from_slice(&months);

    }

    Ok(all_months)

}

fn format_enso_num(m: f32) -> String {
    let color = if m >= 1.0 {Style::new(&[RedBg, Black])}
        else if m >= 0.5 {Style::new(&[Red])}
        else if m <= -1.0 {Style::new(&[BlueBg, Black])}
        else if m <= -0.5 {Style::new(&[Blue])}
        else {Style::new(&[White])};

    format!("{color}{m:.1}{Reset}")
}

fn format_enso(mut months: Vec<f32>) -> Result<String, String> {
    months.reverse();

    let m = months.get(0).ok_or("Not enough items found in ENSO database")?;
    let m3 = months.get(3).ok_or("Not enough items found in ENSO database")?;
    let m6 = months.get(6).ok_or("Not enough items found in ENSO database")?;
    // let m12 = months.get(12).ok_or("Not enough items found in ENSO database")?;

    let three_month_change = m - m3;

    let trend = if three_month_change >= 0.3 {" ↗"}
        else if three_month_change <= 0.3 {" ↘"}
        else {""};

    Ok(format!("ENSO: {}{trend} (6 months ago: {})", format_enso_num(*m), format_enso_num(*m6)))

}

async fn teleconnections_handler() -> Result<String, String>  {
    let mut s = common::title("TELECONNECTIONS");
    
    let enso: Vec<f32> = get_enso().await?;

    let formatted = format_enso(enso)?;

    s.push_str(&formatted);
    s.push('\n');

    Ok(s)

}