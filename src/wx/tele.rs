use chrono::NaiveDate;
use csv;
use serde::Deserialize;

use crate::wx::*;
use crate::config::{Config, Modules};

pub async fn teleconnections(config: &Config){
    if !config.enabled_modules.contains(&Modules::Teleconnections) {
        return;
    }


    match teleconnections_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", common::title("TELECONNECTIONS"), e)},
    }
}

async fn get_enso() -> Result<(Vec<f32>, String), String> {
    let url = "https://psl.noaa.gov/enso/mei/data/meiv2.data";

    let data = reqwest::get(url)
        .await.map_err(|x| x.to_string())?
        .text().await.map_err(|x| x.to_string())?;


    // let mut reader = csv::ReaderBuilder::new()
    //                                     .delimiter(b'\t')
    //                                     .has_headers(false)
    //                                     .from_reader(data.as_bytes());

    let mut all_months = vec![];

    let mut month_name = "None";

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
            .flatten()
            .filter(|x| *x < 20.0 && *x > -20.0)
            .collect::<Vec<f32>>();

        all_months.extend_from_slice(&months);

        let month = months.len();

        match month {
            0 => {},
            1 => month_name = "Dec-Jan",
            2 => month_name = "Jan-Feb",
            3 => month_name = "Feb-Mar",
            4 => month_name = "Mar-Apr",
            5 => month_name = "Apr-May",
            6 => month_name = "May-Jun",
            7 => month_name = "Jun-Jul",
            8 => month_name = "Jul-Aug",
            9 => month_name = "Aug-Sep",
            10 => month_name = "Sep-Oct",
            11 => month_name = "Oct-Nov",
            12 => month_name = "Nov-Dec",
            _ => Err("Unexpected length for ENSO count".to_string())?,
        };

    }

    Ok((all_months, month_name.to_string()))

}

fn format_enso_num(m: f32) -> String {
    let color = if m >= 1.0 {Style::new(&[RedBg, Black])}
        else if m >= 0.5 {Style::new(&[Red, Bold])}
        else if m <= -1.0 {Style::new(&[BlueBg, Black, Bold])}
        else if m <= -0.5 {Style::new(&[Blue, Bold])}
        else {Style::new(&[Bold])};

    format!("{color}{m:.1}{Reset}")
}

fn format_enso(enso: (Vec<f32>, String)) -> Result<String, String> {

    let mut months = enso.0;
    let month_name = enso.1;

    months.reverse();

    let m = months.get(0).ok_or("Not enough items found in ENSO database")?;
    let m3 = months.get(3).ok_or("Not enough items found in ENSO database")?;
    let m6 = months.get(6).ok_or("Not enough items found in ENSO database")?;
    // let m12 = months.get(12).ok_or("Not enough items found in ENSO database")?;

    let three_month_change = m - m3;

    let trend = if three_month_change >= 0.3 {" ↗"}
        else if three_month_change <= -0.3 {" ↘"}
        else {""};

    Ok(format!("ENSO ({}): {}{trend} (6 months ago: {})\n", month_name, format_enso_num(*m), format_enso_num(*m6)))

}

// NAO
#[derive(Deserialize)]
struct NaoRecord {
    lead: u32,
    #[allow(dead_code)]
    time: NaiveDate,
    nao_index: f32,
    valid_time: NaiveDate,
} 

async fn get_nao() -> Result<BTreeMap<NaiveDate, f32>, String>{    
    let url = "https://ftp.cpc.ncep.noaa.gov/cwlinks/norm.daily.nao.gfs.z500.120days.csv";
    // this thing is overkill
    let data = reqwest::get(url).await.map_err(|e| e.to_string())?;
    // let bytes = data.bytes().await.map_err(|e| e.to_string())?;

    let text = data.text().await.map_err(|e| e.to_string())?;
    
    let mut map = BTreeMap::new();

    let mut reader = csv::Reader::from_reader(text.as_bytes());

    for result in reader.records() {
        let r: NaoRecord = result
            .map_err(|x| x.to_string())?
            .deserialize(None)
            .map_err(|x| x.to_string())?;
        
        if r.lead == 0 {
            map.insert(r.valid_time, r.nao_index);
        } else { // higher lead values occur after this so we wait
            break;
        }

    }

    Ok(map)
}

fn style_nao(nao: f32) -> String {
    if nao > 2.0 {
        Style::new(&[RedBg, Black, Bold])
    } else if nao > 1.0 {
        Style::new(&[Red, Bold])
    } else if nao < -2.0 {
        Style::new(&[BlueBg, Black, Bold])
    } else if nao < -1.0 {
        Style::new(&[Blue, Bold])
    } else {
        Style::new(&[Bold])
    }
}

fn format_nao(nao: BTreeMap<NaiveDate, f32>) -> Result<String, String>{    
    let current = nao.iter().nth_back(0).ok_or("No values in NAO data")?;
    let three_days_ago = nao.iter().nth_back(3).ok_or("No values in NAO data")?;
    let seven_days_ago = nao.iter().nth_back(7).ok_or("No values in NAO data")?;

    let three_days_change = current.1 - three_days_ago.1;

    let trend = if three_days_change >= 0.6 {" ↗"}
        else if three_days_change <= -0.6 {" ↘"}
        else {""};

    Ok(format!("NAO: {}{:.2}{Reset}{trend} (7 days ago: {}{:.2}{Reset})\n", style_nao(*current.1), *current.1, style_nao(*seven_days_ago.1), *seven_days_ago.1))
}

async fn teleconnections_handler() -> Result<String, String>  {
    let mut s = common::title("TELECONNECTIONS");
    
    let enso = get_enso().await?;
    let nao: BTreeMap<NaiveDate, f32> = get_nao().await?;

    // dbg!(&nao);

    s.push_str(&format_enso(enso)?);
    s.push_str(&format_nao(nao)?);

    Ok(s)

}