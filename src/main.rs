use std::{fmt, collections::{HashMap, BTreeMap}, time::Duration, fs};

use chrono::{Local, Utc, NaiveDate, NaiveTime, DateTime};
use home::home_dir;
use rand::{self, Rng, thread_rng, rngs::ThreadRng};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio;

// CONFIG ----------------------------------------------------------------------

const COORDS: (f64, f64) = DURHAM_COORDS;
const DURHAM_COORDS: (f64, f64) = (43.13, -70.92);

fn coords_str() -> String {
    format!("{:.2},{:.2}", COORDS.0, COORDS.1)
}


// GENERAL ---------------------------------------------------------------------

#[allow(dead_code)]
enum TermStyle {
    Reset,
    Bold,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    BlackBg,
    RedBg,
    GreenBg,
    YellowBg,
    BlueBg,
    PurpleBg,
    CyanBg,
    WhiteBg,
    NoStyle,
}

impl TermStyle {
    fn str(&self) -> &str {
        match self {
            Reset => "\x1b[0m",
            Bold => "\x1b[1m",
            Black => "\x1b[30m",
            Red => "\x1b[31m",
            Green => "\x1b[32m",
            Yellow => "\x1b[33m",
            Blue => "\x1b[34m",
            Purple => "\x1b[35m",
            Cyan => "\x1b[36m",
            White => "\x1b[37m",
            BlackBg => "\x1b[40m",
            RedBg => "\x1b[41m",
            GreenBg => "\x1b[42m",
            YellowBg => "\x1b[43m",
            BlueBg => "\x1b[44m",
            PurpleBg => "\x1b[45m",
            CyanBg => "\x1b[46m",
            WhiteBg => "\x1b[47m",
            NoStyle => "",
        }
    }
}

impl fmt::Display for TermStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

use TermStyle::*;

// TODO: Maybe not make this a struct? 
struct Style {
    // styles: Vec<TermStyle>
}

impl Style {
    fn new(styles: &[TermStyle]) -> String {
        let mut nums = vec![];

        for s in styles {
            nums.push(match s {
                Reset => "0",
                Bold => "1",
                Black => "30",
                Red => "31",
                Green => "32",
                Yellow => "33",
                Blue => "34",
                Purple => "35",
                Cyan => "36",
                White => "37",
                BlackBg => "40",
                RedBg => "41",
                GreenBg => "42",
                YellowBg => "43",
                BlueBg => "44",
                PurpleBg => "45",
                CyanBg => "46",
                WhiteBg => "47",
                NoStyle => "",
            });
        }

        format!("\x1b[{}m", nums.join(";"))
    }
}


fn terminal_line(c: char) -> String {
    let mut s = String::new();
    for _ in 0..80 {
        s.push(c);
    }
    s.push('\n');
    s
}

// HELPER FUNCTIONS ------------------------------------------------------------

fn title(s: &str) -> String {
    format!("\n{:-^80}\n", s)
}

async fn parse_request_loose_json(w: Result<reqwest::Response, reqwest::Error>) -> Result<serde_json::Value, String> {    
    let r = w.map_err(|e| e.to_string())?;
    let t = r.text().await.map_err(|e| e.to_string())?;
    // dbg!(&t);
    let j = serde_json::from_str(&t).map_err(|e| e.to_string())?;
    Ok(j)
}



// HEAD MATTER -----------------------------------------------------------------

fn head_matter() {
    // let s: String = "DS2 ".into()
    let utc_now = Utc::now().format("(%H:%MZ)");
    let local_now = Local::now().format("%a %Y-%b-%d @ %I:%M:%S%p");
    let r = rand::random::<u32>();

    println!("{}unifetch v{} {local_now} {utc_now} - {r:08X}", terminal_line('-'), env!("CARGO_PKG_VERSION"));
}



// RANDOM -----------------------------------------------------------------

fn d6(rng: &mut ThreadRng) -> char {
    rng.gen_range('1'..='6')
}

fn d20(rng: &mut ThreadRng) -> String {

    let d20: i32 = rng.gen_range(1..=20);

    let d20_style = if d20 == 1 {
        Style::new(&[RedBg, Black])
    } else if d20 == 20 {
        Style::new(&[GreenBg, Black])
    } else {
        "".into()
    };

    format!("{d20_style}{}{Reset}", d20)
}

fn rand_date(rng: &mut ThreadRng) -> NaiveDate {
    let year = rng.gen_range(1800..=2199);

    let mut day;
    let mut opt_date: Option<NaiveDate> = None;

    while opt_date.is_none() {
        day = rng.gen_range(0..=366);
        opt_date = NaiveDate::from_yo_opt(year, day)
    }

    opt_date.unwrap()
}

fn random_section() {

    let mut s = title("RANDOM");

    let mut rng = thread_rng();
    
    let coin = if rng.gen_bool(0.5) {
        "Heads"
    } else {
        "Tails"
    };

    let d3 = rng.gen_range(1..=3);
    let d4 = rng.gen_range(1..=4);
    let d8 = rng.gen_range(1..=8);
    let d10 = rng.gen_range(1..=10);
    let d12 = rng.gen_range(1..=12);    
    let d100 = rng.gen_range(1..=100);

    let bits: u64 = rng.gen();

    let first_8_bits = bits & 0xFF;
    let next_8_bits = (bits & 0xFF00) >> 8;
    let next_16_bits = (bits & 0xFFFF0000) >> 16;
    let next_32_bits = (bits & 0xFFFFFFFF00000000) >> 32;

    let hex: (u128, u128)= (rng.gen(), rng.gen());
    let prob = rng.gen_range(0.0..1.0);
    let ten_digit: u64 = rng.gen_range(0..10_000_000_000);

    let six_letters: [char; 6] = [ // yandere dev moment
            rng.gen_range('A'..='Z'), 
            rng.gen_range('A'..='Z'), 
            rng.gen_range('A'..='Z'), 
            rng.gen_range('A'..='Z'), 
            rng.gen_range('A'..='Z'),
            rng.gen_range('A'..='Z'),
        ];

    let date = rand_date(&mut rng);

    s.push_str(&format!("Coin: {Bold}{}{Reset} | Dice (D6): {Bold}{} {}{Reset} {} {} {} {}\n", 
            coin, d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng)));
    s.push_str(&format!("D20: {} {} {} {}\n", d20(&mut rng), d20(&mut rng), d20(&mut rng), d20(&mut rng)));
    s.push_str(&format!("D3: {} | D4: {} | D8: {} | D10: {} | D12: {} | D100: {}\n", d3, d4, d8, d10, d12, d100));
    s.push_str(&format!("Bits: {Bold}{:032b}{Reset}{:016b}{Bold}{:08b}{Reset}{:08b}\n", next_32_bits, next_16_bits, next_8_bits, first_8_bits));
    s.push_str(&format!("Hex:  {:032x}{:032x}\n", hex.0, hex.1));
    s.push_str(&format!("Prob: {:.08} | 10 Digits: {:010} | 6 Letters: {}\n", prob, ten_digit, six_letters.iter().collect::<String>()));
    s.push_str(&format!("Date: {} \x1b[0;47;37m{}{Reset}", date.format("%Y-%m-%d"), date.format("%a")));

    println!("{}", s);

}





// SOLAR/LUNAR --------------------------------------------------

fn parse_navy_times(v: &Value) -> Result<NaiveTime, String> {
    let text = v.as_str().ok_or("Could not parse JSON")?;
    let date = NaiveTime::parse_from_str(text, "%H:%M").map_err(|e| e.to_string())?;

    Ok(date)
}

fn string_from_rise_set_times(name: &str, start_name: &str, end_name: &str, start: NaiveTime, end: NaiveTime) -> String {
    let s = start.format("%I:%M %p");
    let e = end.format("%I:%M %p");
    let d = end - start;
    
    format!("{name:<8} {start_name:<5} {Bold}{s}{Reset} | {end_name} {Bold}{e}{Reset} | Duration {Bold}{}h{}m{Reset}\n", d.num_hours(), d.num_minutes() % 60)
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

    let moonrise = parse_navy_times(&moondata[0]["time"])?;
    let moonset = parse_navy_times(&moondata[2]["time"])?;   

    let moon_phase = &data["curphase"].as_str().ok_or("Could not parse JSON properly")?;
    let fracillum = &data["fracillum"].as_str().ok_or("Could not parse JSON properly")?;

    let closest_phase = &data["closestphase"];

    let closest_name = &closest_phase["phase"].as_str().ok_or("Could not parse JSON properly")?;
    let closest_month = &closest_phase["month"].as_i64().ok_or("Could not parse JSON properly")?;
    let closest_day = &closest_phase["day"].as_i64().ok_or("Could not parse JSON properly")?;
    let closest_time = &closest_phase["time"].as_str().ok_or("Could not parse JSON properly")?;

    let closest_time: NaiveTime = NaiveTime::parse_from_str(closest_time, "%H:%M").map_err(|e| e.to_string())?;

    let phase_string = format!("Moon Phase: {} ({}) | {} on {}/{} ({})", moon_phase, fracillum, closest_name, closest_month, closest_day, closest_time.format("%I:%M %p"));

    Ok(format!("{}{}{}{}",
        string_from_rise_set_times("Sun", "Rise", "Set", sunrise, sunset),
        string_from_rise_set_times("Twilight", "Begin", "End", twilight_start, twilight_end),
        string_from_rise_set_times("Moon", "Rise", "Set", moonrise, moonset),   
        phase_string
    ))
}    // dbg!(&r);


async fn solar_lunar() {
    let mut s: String = title("SOLAR & LUNAR");

    let now = Local::now();

    let tz_offset = now.offset().local_minus_utc() / 60 / 60;

    let mut map = HashMap::new();

    map.insert("date", now.format("%Y-%m-%d").to_string());
    map.insert("coords", coords_str());
    map.insert("tz", tz_offset.to_string());


    let client = reqwest::Client::new();
    
    let r = client.get("https://aa.usno.navy.mil/api/rstt/oneday")
                .query(&map)
                .timeout(Duration::from_secs(5));

    // dbg!(&r);
    
    let form = r.send()
                .await;


    match parse_request_loose_json(form).await {
        Ok(json) => {
            match generate_solar_lunar_string(json) {
                Ok(res) => {s.push_str(&res)}
                Err(res) => {s.push_str(&res)}
            }
        }
        Err(e) => {s.push_str(&e)}
    }


    println!("{}", s);
}





// WEATHER ---------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
struct CloudLayer {
    code: String,
    height: u32,
}

impl fmt::Debug for CloudLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{:5}", self.code, self.height)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StationEntry {
    indoor_temperature: Option<f32>, // in Fahrenheit

    temperature_2m: Option<f32>, // in Fahrenheit
    dewpoint_2m: Option<f32>, // in Fahrenheit
    sea_level_pressure: Option<f32>, // in hPa
    wind_10m: Option<(f32, u16)>, // in Knots, Degrees
    skycover: Vec<CloudLayer>, // in Feet
    visibility: Option<f32>, // in mile
    precip_today: Option<f32>,

    present_wx: Option<Vec<String>>,
    raw_metar: Option<String>, 
    raw_pressure: Option<f32>, // in hPa

}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StationEntryWithTime(DateTime<Utc>, StationEntry);


async fn wxer_query(loc: &str, time: &str) -> Result<String, String> {

    let mut path = home_dir().ok_or(String::from("Could not find the user's home directory!"))?;

    path.push(".config/unifetch/wxer_addr.txt");

    let addresses_string = fs::read_to_string(path)
                            .map_err(|e| e.to_string())?;

    let addresses = addresses_string.lines();
    
    let client = reqwest::Client::new();

    for addr in addresses {
        let q = client.get(format!("{addr}/{loc}/{time}.json"))
                                .timeout(Duration::from_secs(2))
                                .send()
                                .await;

        if !q.is_err() {
            let r = q.unwrap();

            if r.status().is_success() {
                return r.text().await.map_err(|e| e.to_string());
            }
        }
    }

    Err("None of the addresses responded successfully!".into())
}


async fn current_conditions_wrapper() {
    match current_conditions().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", title("CURRENT CONDITIONS"), e)},
    }
}

fn indoor_temp_style(temp: f32) -> String {
    if temp.is_nan() {
        Style::new(&[Red])
    } else {
        if temp < 65. {
            Style::new(&[BlueBg])
        } else if temp < 75. {
            Style::new(&[NoStyle])
        } else {
            Style::new(&[RedBg])
        }
    }
}

fn outdoor_temp_style(temp: f32) -> String {
    if temp.is_nan() {
        Style::new(&[Red])
    } else {
        if temp < 10. {
            Style::new(&[PurpleBg])
        } else if temp < 32. {
            Style::new(&[BlueBg])
        } else if temp < 55. {
            Style::new(&[GreenBg, Black])
        } else if temp < 70. {
            Style::new(&[YellowBg])
        } else if temp < 85. {
            Style::new(&[RedBg])
        } else if temp < 95. {
            Style::new(&[WhiteBg, Red])
        } else {
            Style::new(&[PurpleBg, Red])
        }
    }
}

enum TempChange {
    Rising,
    Falling,
    UnknownChange,
    Neutral,
    RapidlyRising,
    RapidlyFalling
}

use TempChange::*;

fn db_reverse_time(db: &BTreeMap<DateTime<Utc>, StationEntry>, duration: chrono::Duration) -> Result<&StationEntry, String> {
    let latest = db.last_key_value()
                .ok_or(String::from("database has nothing"))?;

    let dt_past = latest.0.clone() - duration;
    let past_entry = db.get(&dt_past)
                    .ok_or(String::from("Past entry was not found."))?;

    Ok(past_entry)
}

impl TempChange {

    fn from_db_inner(db: &BTreeMap<DateTime<Utc>, StationEntry>, indoor: bool) -> Result<TempChange, ()> {
        let latest = db.last_key_value().ok_or(())?;

        let _15_minutes_ago = db_reverse_time(db, chrono::Duration::minutes(15)).map_err(|_| ())?;
        let _1_hour_ago = db_reverse_time(db, chrono::Duration::hours(1)).map_err(|_| ())?;

        if indoor {
            let hourly_change = _1_hour_ago.indoor_temperature.ok_or(())? - latest.1.indoor_temperature.ok_or(())?;
            let _15_minute_change = _15_minutes_ago.indoor_temperature.ok_or(())? - latest.1.indoor_temperature.ok_or(())?;

            if hourly_change > 5. && _15_minute_change > 3. {
                Ok(RapidlyRising)
            } else if hourly_change > 2. {
                Ok(Rising)
            } else if hourly_change < -5. && _15_minute_change < -3. {
                Ok(RapidlyFalling)
            } else if hourly_change < -2. {
                Ok(Falling)
            } else {
                Ok(Neutral)
            }

        } else {
            let hourly_change = _1_hour_ago.temperature_2m.ok_or(())? - latest.1.temperature_2m.ok_or(())?;
            let _15_minute_change = _15_minutes_ago.temperature_2m.ok_or(())? - latest.1.temperature_2m.ok_or(())?;

            if hourly_change > 10. && _15_minute_change > 5. {
                Ok(RapidlyRising)
            } else if hourly_change > 3. {
                Ok(Rising)
            } else if hourly_change < -10. && _15_minute_change < -5. {
                Ok(RapidlyFalling)
            } else if hourly_change < -3. {
                Ok(Falling)
            } else {
                Ok(Neutral)
            }
        }
    }

    fn from_db(db: &BTreeMap<DateTime<Utc>, StationEntry>, indoor: bool) -> TempChange {
        match TempChange::from_db_inner(db, indoor) {
            Ok(t) => t,
            Err(_) => UnknownChange,
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Rising => "↗",
            Falling => "↘",
            RapidlyRising => "⬆",
            RapidlyFalling => "⬇",
            UnknownChange => "?",
            Neutral => "➡"
        }
    }
}

impl fmt::Display for TempChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

async fn current_conditions() -> Result<String, String> {

    let local_conditions = wxer_query("local", "all").await?;
    let psm_conditions = wxer_query("psm", "all").await?;

    // dbg!(&local_conditions);
    // dbg!(&psm_conditions);

    let local_conditions: BTreeMap<DateTime<Utc>, StationEntry> = serde_json::from_str(&local_conditions).map_err(|e| e.to_string())?;
    let psm_conditions: BTreeMap<DateTime<Utc>, StationEntry> = serde_json::from_str(&psm_conditions).map_err(|e| e.to_string())?;

    let mut s = title("CURRENT CONDITIONS");

    let latest_local = local_conditions.last_key_value()
                            .ok_or(String::from("Local json did not have any values"))?;
    
    let latest_psm = psm_conditions.last_key_value()
                            .ok_or(String::from("PSM json did not have any values"))?;

    let apt_temp = latest_local.1.indoor_temperature.unwrap_or(f32::NAN);
    let apt_temp_style = indoor_temp_style(apt_temp);
    let apt_temp_change = TempChange::from_db(&local_conditions, true);

    let psm_temp = latest_psm.1.temperature_2m.unwrap_or(f32::NAN);
    let psm_temp_style = outdoor_temp_style(apt_temp);
    let psm_temp_change = TempChange::from_db(&psm_conditions, true);



    s.push_str(&format!("Apt: {apt_temp_style}{apt_temp:.1}°F{apt_temp_change}{Reset}\n"));
    s.push_str(&format!("KPSM: {psm_temp_style}{psm_temp:.1}°F{psm_temp_change}{Reset}\n"));

    
    Ok(s)

}


#[tokio::main] 
async fn main() {

    
    head_matter();
    random_section();

    futures::join!(
        solar_lunar(), 
        current_conditions_wrapper(),
        // current_conditions_wrapper()
    );
        

    // time_and_date();

    // forecast();
    // forecast_analysis();
    // climatology();
    // stock_market();

    //https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?date=latest&station=8419870&product=predictions&datum=STND&time_zone=gmt&interval=hilo&units=english&format=json
    // tides();
    
    
    // teleconnections();

    // earthquakes();

}



