use std::{fmt, collections::{HashMap, BTreeMap}, time::Duration, fs};

use chrono::{Local, Utc, NaiveDate, NaiveTime, DateTime, TimeZone, NaiveDateTime};
use home::home_dir;
use rand::{self, Rng, thread_rng, rngs::ThreadRng};

use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio;

use wxer_lib::*;

// CONFIG ----------------------------------------------------------------------

const COORDS: (f64, f64) = DURHAM_COORDS;
const DURHAM_COORDS: (f64, f64) = (43.13, -70.92);

fn coords_str() -> String {
    format!("{:.2},{:.2}", COORDS.0, COORDS.1)
}


// GENERAL ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
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

    fn error() -> String {
        Style::new(&[Red, Bold])
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

    let phase_string = format!("Moon Phase: {Bold}{} ({}){Reset} | {Bold}{}{Reset} on {Bold}{}/{} ({}){Reset}", moon_phase, fracillum, closest_name, closest_month, closest_day, closest_time.format("%I:%M %p"));

    Ok(format!("For {Bold}{}{Reset}\n{}{}{}{}",
        Local::now().format("%b %d"),
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


#[derive(Debug, Clone, Serialize, Deserialize)]
struct StationEntryWithTime(DateTime<Utc>, StationEntry);


async fn wxer_query(loc: &str, time: &str) -> Result<String, String> {

    let mut path = home_dir().ok_or(String::from("Could not find the user's home directory!"))?;

    path.push(".config/unifetch/wxer_addr.txt");

    let addresses_string = fs::read_to_string(path)
                            .map_err(|e| e.to_string())?;

    let addresses = addresses_string.lines();
    
    let client = reqwest::Client::new();

    let mut err_string = String::new();

    for addr in addresses {
        let url = format!("{addr}/{loc}/{time}.json");
        // dbg!(&url);
        let q = client.get(&url)
                                .timeout(Duration::from_secs(10))
                                .send()
                                .await;

        if !q.is_err() {
            let r = q.unwrap();

            if r.status().is_success() {
                return r.text().await.map_err(|e| e.to_string());
            } else {
                err_string.push_str(&format!("{url} - {}, {}\n", r.status().as_u16(), r.status().as_str()))
            }
        } else {
            let err = q.unwrap_err();
            err_string.push_str(&format!("{url} - {:?}, {}\n", err.status(), err.to_string()));
        }
    }

    Err(format!("None of the addresses responded successfully!\n{err_string}"))
}

fn indoor_temp_style(temp: f32) -> String {

    if temp.is_nan() {
        Style::new(&[Red, Bold])
    } else {
        if temp < 65. {
            Style::new(&[BlueBg, Bold])
        } else if temp < 75. {
            Style::new(&[NoStyle, Bold])
        } else {
            Style::new(&[RedBg, Bold])
        }
    }
}

fn outdoor_temp_style(temp: f32) -> String {
    if temp.is_nan() {
        Style::new(&[Red, Bold])
    } else {
        if temp < 10. {
            Style::new(&[PurpleBg, Bold])
        } else if temp < 32. {
            Style::new(&[BlueBg, Bold])
        } else if temp < 55. {
            Style::new(&[GreenBg, Black, Bold])
        } else if temp < 70. {
            Style::new(&[YellowBg, Black, Bold])
        } else if temp < 85. {
            Style::new(&[RedBg, Bold])
        } else if temp < 95. {
            Style::new(&[WhiteBg, Red, Bold])
        } else {
            Style::new(&[PurpleBg, Red, Bold])
        }
    }
}

fn mslp_style(pres: f32) -> String {
    if pres.is_nan() {
        Style::new(&[Red, Bold])
    } else {
        if pres < 1005. {
            Style::new(&[RedBg, Black, Bold])
        } else if pres > 1025. {
            Style::new(&[BlueBg, Black, Bold])
        } else {
            Style::new(&[Bold])
        }
    }
}

enum Trend {
    Rising,
    Falling,
    UnknownChange,
    Neutral,
    RapidlyRising,
    RapidlyFalling
}

use Trend::*;

fn db_reverse_time(db: &BTreeMap<DateTime<Utc>, StationEntry>, duration: chrono::Duration) -> Result<&StationEntry, String> {
    let latest = db.last_key_value()
                .ok_or(String::from("database has nothing"))?;

    let dt_past = latest.0.clone() - duration;
    let past_entry = db.get(&dt_past)
                    .ok_or(String::from("Past entry was not found."))?;

    Ok(past_entry)
}

impl Trend {

    fn from_db_inner<'a, F: FnMut(&'a StationEntry) -> Option<f32>>
      (db: &'a BTreeMap<DateTime<Utc>, StationEntry>, 
      mut get_field: F, 
      change_criteria: (chrono::Duration, f32), 
      rapid_criteria: (chrono::Duration, f32, chrono::Duration, f32)) -> Result<Trend, ()> {
        
        let latest = db.last_key_value().ok_or(())?;

        let ref_1 = db_reverse_time(db, change_criteria.0).map_err(|_| ())?;
        let ref_2 = db_reverse_time(db, rapid_criteria.0).map_err(|_| ())?;
        let ref_3 = db_reverse_time(db, rapid_criteria.2).map_err(|_| ())?;


        let ref_1_change = get_field(latest.1).ok_or(())? - get_field(ref_1).ok_or(())?;
        let ref_2_change = get_field(latest.1).ok_or(())? - get_field(ref_2).ok_or(())?;
        let ref_3_change = get_field(latest.1).ok_or(())? - get_field(ref_3).ok_or(())?;

        // dbg!(ref_1);
        // dbg!(ref_2);
        // dbg!(ref_3);

        // dbg!(ref_1_change);
        // dbg!(ref_2_change);
        // dbg!(ref_3_change);


        if ref_2_change > rapid_criteria.1 && ref_3_change > rapid_criteria.3 {
            Ok(RapidlyRising)
        } else if ref_1_change > change_criteria.1 {
            Ok(Rising)
        } else if ref_2_change < -rapid_criteria.1 && ref_3_change < -rapid_criteria.3 {
            Ok(RapidlyFalling)
        } else if ref_1_change < -change_criteria.1 {
            Ok(Falling)
        } else {
            Ok(Neutral)
        }

    }

    fn from_db<'a, F: FnMut(&'a StationEntry) -> Option<f32>>
        (db: &'a BTreeMap<DateTime<Utc>, StationEntry>, get_field: F, 
        change_criteria: (chrono::Duration, f32), 
        rapid_criteria: (chrono::Duration, f32, chrono::Duration, f32)) -> Trend {

            match Trend::from_db_inner(db, get_field, change_criteria, rapid_criteria) {
                Ok(tr) => tr,
                Err(_) => UnknownChange,
            }
    }

    fn str(&self) -> &'static str {
        match self {
            Rising => " ↗",
            Falling => " ↘",
            RapidlyRising => " ⬆",
            RapidlyFalling => " ⬇",
            UnknownChange => " ?",
            Neutral => " ➡"
        }
    }
}

impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

#[derive(Debug, PartialEq)]
enum WxCategory {
    Snow,
    Rain,
    Severe,
    Fire,
    Fog,
    None
}

fn format_wx(o: Option<Vec<String>>) -> String {
    let mut text: String;
    let mut style: String;

    match o {
        None => {
            text = String::from("");
            style = Style::new(&[Red, Bold]);
        },
        Some(v) => {
            let mut wx_types = vec![];

            for wx in &v {

                let c = if wx.contains("FU") | wx.contains("VA") {
                    WxCategory::Fire
                } else if wx.contains("GR") | wx.contains("TS") | wx.contains("DU") | wx.contains("SA") | wx.contains("SQ") | wx.contains("DS") | wx.contains("SS") | wx.contains("FC") {
                    WxCategory::Severe
                } else if wx.contains("SN") | wx.contains("SG") | wx.contains("GS") | wx.contains("PL") | wx.contains("IC") {
                    WxCategory::Snow
                } else if wx.contains("DZ") | wx.contains("RA") | wx.contains("UP") {
                    WxCategory::Rain
                } else if wx.contains("FG") | wx.contains("BR") | wx.contains("HZ") | wx.contains("PY") {
                    WxCategory::Fog
                } else {
                    WxCategory::None
                };

                wx_types.push(c);
            }

            if wx_types.contains(&WxCategory::Fire) {
                style = Style::new(&[RedBg, White, Bold]);
            } else if wx_types.contains(&WxCategory::Severe) {
                style = Style::new(&[YellowBg, Black, Bold]);
            } else if wx_types.contains(&WxCategory::Snow) {
                style = Style::new(&[WhiteBg, Blue, Bold]);
            } else if wx_types.contains(&WxCategory::Rain) {
                style = Style::new(&[BlueBg, Black, Bold]);
            } else if wx_types.contains(&WxCategory::Fog) {
                style = Style::new(&[WhiteBg, Black, Bold]);
            } else {
                style = Style::new(&[Reset]);
            }
            
            text = v.join(" ");
        }
    }

    if text.len() == 0 {
        text.push_str("No WX");
        style = Style::new(&[Bold])
    }

    format!("{style}{text}{Reset}")


}


fn format_dewpoint(s: &StationEntry) -> String {
    let dew_text: String;
    let dew_style: String;

    if s.dewpoint_2m.is_none() {
        return String::from("");
    }

    let a = s.dewpoint_2m.unwrap();

    dew_style = if a > 70. {
        Style::new(&[PurpleBg, Black, Bold])
    } else if a > 60. {
        Style::new(&[BlueBg, Black, Bold])
    } else if a > 45. {
        Style::new(&[GreenBg, Black, Bold])
    } else if a < 30. {
        Style::new(&[YellowBg, Black, Bold])
    } else {
        Style::new(&[Bold])
    };

    dew_text = format!("{a:.0}F");

    let rh_text: String;
    let rh_style: String;
    let rh = s.relative_humidity_2m();

    match rh {
        Some(a) => {

            rh_style = if a > 95. {
                Style::new(&[PurpleBg, Black, Bold])
            } else if a > 90. {
                Style::new(&[BlueBg, Black, Bold])
            } else if a > 70. {
                Style::new(&[GreenBg, Black, Bold])
            } else if a > 40. {
                Style::new(&[Bold])
            } else {
                Style::new(&[YellowBg, Black, Bold])
            };

            rh_text = format!("{a:.0}%");

        }
        None => {
            rh_text = String::from("N/A");
            rh_style = Style::error();
        }
    }

    format!("{dew_style}{dew_text}{Reset} ({rh_style}{rh_text}{Reset})")

} 

fn format_wind(s: &StationEntry) -> String {
    let text: String;
    let style: String;

    if s.wind_10m.is_none() {
        return String::from("");
    }

    let a = s.wind_10m.unwrap();

    style = if a.speed > 45. {
        Style::new(&[YellowBg, Black, Bold])
    } else if a.speed > 32. {
        Style::new(&[RedBg, Black, Bold])
    } else if a.speed > 20. {
        Style::new(&[PurpleBg, Black, Bold])
    } else if a.speed > 12. {
        Style::new(&[BlueBg, Black, Bold])
    } else {
        Style::new(&[Bold])
    };

    format!("{style}{:03}({})@{:2}kts{Reset}", a.direction.degrees(), a.direction.cardinal(), a.speed)
}

fn format_cloud(s: &StationEntry) -> String {
    let text: String;
    let style: String;

    match &s.skycover {
        None => {text = "".into(); style = Style::new(&[Red, Bold]);},
        Some(s) => {
            match s {
                SkyCoverage::Clear => {text = "CLR".into(); style = Style::new(&[Bold]);}
                SkyCoverage::Cloudy(v) => {
                    text = v.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                    
                    let all_covers = v.iter()
                                        .map(|x| x.coverage)
                                        .collect::<Vec<_>>();

                    style = if all_covers.contains(&CloudLayerCoverage::Overcast) {
                        Style::new(&[WhiteBg, Black, Bold])
                    } else {
                        Style::new(&[Bold])
                    }
                    
                }
            }
        }
    }

    format!("{style}{text}{Reset}")
}

fn format_visibility(e: &StationEntry) -> String {
    let text: String;
    let style: String;

    match e.visibility {
        None => {return String::new()},
        Some(v) => {
            if v <= 1. {
                style = Style::new(&[WhiteBg, Black, Bold]);
                text = format!("{v:.2}mi");

            } else if v < 3. {
                style = Style::new(&[WhiteBg, Black, Bold]);
                text = format!("{v:.1}mi");

            } else {
                style = Style::new(&[Bold]);
                text = format!("{v:.0}mi");
            }
        }
    }

    format!("{style}{text}{Reset}")

}

// fn format_column_width(s: String) -> String {
//     const INDENTATION_SPACES: usize = 2;
//     const COLUMN_WIDTH: usize = 80;
//     const WIDTH: usize = COLUMN_WIDTH - INDENTATION_SPACES;

//     s.chars()
//     .enumerate()
//     .fold(String::new(),|mut a, (i, x)| {
//         if i % (WIDTH) == 0 {
//             for j in 0..INDENTATION_SPACES {
//                 a.push(' ');
//             }
//             a.push(x);
//         } else if i % (WIDTH) == (WIDTH - 1) {
//             a.push(x);
//             a.push('\n');
//         } else {
//             a.push(x);
//         }

//         a
//     })
// }

fn format_metar(e: &StationEntry) -> String {
    let s = e.raw_metar.clone();
    match s {
        Some(m) => format!("METAR: {m}"),
        None => String::new()
    }
}

fn format_temp(e: &StationEntry, indoor: bool, db: &BTreeMap<DateTime<Utc>, StationEntry>) -> String {
    
    let temp = if indoor {
        e.indoor_temperature
    } else {
        e.temperature_2m
    };

    let temp_change = if indoor {
        Trend::from_db(&db, 
            |data| {data.indoor_temperature}, 
                    (chrono::Duration::hours(2), 2.),
                    (chrono::Duration::hours(1), 2., chrono::Duration::hours(1), 2.))
    } else {
        Trend::from_db(&db, 
            |data| {data.temperature_2m}, 
            (chrono::Duration::hours(2), 4.),
            (chrono::Duration::minutes(15), 2., chrono::Duration::hours(1), 4.))
    };
    
    if let Some(temp) = temp {
        let temp_style = outdoor_temp_style(temp);
        format!("Temp: {temp_style}{temp:.0}F{temp_change}{Reset}")
    } else {
        String::new()
    }
}

fn format_pressure(e: &StationEntry, station: &Station, db: &BTreeMap<DateTime<Utc>, StationEntry>) -> String {
    if let Some(pressure) = e.slp(&station) {
        let pressure_style = mslp_style(pressure);
        let pres_change = Trend::from_db(&db, 
            |data| {data.slp(&station)}, 
            (chrono::Duration::hours(6), 3.),
            (chrono::Duration::minutes(15), 1., chrono::Duration::hours(3), 2.));
        
        format!("Pres: {pressure_style}{pressure:.1}{pres_change}{Reset}")
    } else {
        String::new()
    }
}

fn station_line(dt: &DateTime<Utc>, e: &StationEntry, station: &Station, indoor: bool,
  db: &BTreeMap<DateTime<Utc>, StationEntry>) -> Result<String, String> {

    let mut string_vec: Vec<String> = vec![];

    let mut total_string = String::new();

    let time: DateTime<Local> = DateTime::from(dt.clone());

    string_vec.push(format_temp(e, indoor, db));
    string_vec.push(format_pressure(e, station, db));
    string_vec.push(format_wx(e.present_wx.clone()));
    string_vec.push(format_dewpoint(e));
    string_vec.push(format_wind(e));
    string_vec.push(format_visibility(e));
    string_vec.push(format_cloud(e));
    string_vec.push(format_metar(e));
    
    total_string.push_str(&format!("{}: ⌛{}", station.name, time.format("%I:%M %p")));

    dbg!(&string_vec);

    let mut line_length = total_string.len();
    for s in string_vec {
        let new_len = total_string.len() + 1 + s.len();
        
        if s.len() == 0 {
            continue;
        } else if line_length <= 80 {
            total_string.push(' ');
            total_string.push_str(&s);
            line_length = new_len;
        } else {
            total_string.push_str("\n  ");
            total_string.push_str(&s);
            line_length = 2;
        };
    }

    total_string.push_str("\n\n\n\n\n");
    // s.push_str(&format!(""));
    // s.push_str(&format!(" {wx} Dew: {dew}\n    Wind: {wind} Vis: {visibility}\n{metar}\n    Clouds: {cloud}\n"));
        
    Ok(total_string)
}

async fn current_conditions_handler() -> Result<String, String> {

    let apt_station = Station {
        coords: (43.00, 0.0), // im not giving that away
        altitude: 24.,
        name: String::from("APT"),
    };

    let psm_station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };

    let apt_conditions = wxer_query("local", "hourly").await?;
    let psm_conditions = wxer_query("psm", "hourly").await?;

    // dbg!(&local_conditions);
    // dbg!(&psm_conditions);

    let apt_db: BTreeMap<DateTime<Utc>, StationEntry> = serde_json::from_str(&apt_conditions).map_err(|e| e.to_string())?;
    let psm_db: BTreeMap<DateTime<Utc>, StationEntry> = serde_json::from_str(&psm_conditions).map_err(|e| e.to_string())?;

    let mut s = title("CURRENT CONDITIONS");

    let latest_apt = apt_db.last_key_value()
        .ok_or(String::from("Local json did not have any values"))?;

    let latest_psm = psm_db.last_key_value()
        .ok_or(String::from("PSM json did not have any values"))?;

    let apt_line = station_line(latest_apt.0, latest_apt.1, &apt_station, true, &psm_db)?;
    let psm_line = station_line(latest_psm.0, latest_psm.1, &psm_station, true, &psm_db)?;

    s.push_str(&apt_line);
    s.push_str(&psm_line);

    Ok(s)   

}

async fn current_conditions() {
    match current_conditions_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", title("CURRENT CONDITIONS"), e)},
    }
}

// FORECAST --------------------------------------------------------------------

fn from_iso8601_no_seconds<'de, D>(des: D) -> Result<Vec<DateTime<Utc>>, D::Error> 
    where D: serde::Deserializer<'de> {

    let v: Vec<String> = Vec::deserialize(des)?;

    // dbg!(&v);
    
    let v_dt: Vec<DateTime<Utc>> = v.iter()
        .map(|s| {
            let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M");
            let naive = naive.map_err(serde::de::Error::custom)?;
            Ok(Utc::from_utc_datetime(&Utc, &naive))
        })
        .collect::<Result<Vec<DateTime<Utc>>, _>>()?; 
    
    // dbg!(&v_dt);

    Ok(v_dt)
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    latitude: f64,
    hourly: OpenMeteoResponseHourly
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponseHourly {
    #[serde(deserialize_with="from_iso8601_no_seconds")]
    time: Vec<DateTime<Utc>>,
    #[serde(rename="temperature_2m")]
    temperature_2m: Vec<f64>,
    #[serde(rename="dew_point_2m")]
    dewpoint_2m: Vec<f64>,
    #[serde(rename="apparent_temperature")]
    feels_like: Vec<f64>,
    #[serde(rename="precipitation_probability")]
    precip_probability: Vec<f64>,
    #[serde(rename="precipitation")]
    precip: Vec<f64>,
    #[serde(rename="rain")]
    rain: Vec<f64>,
    #[serde(rename="snowfall")]
    snowfall: Vec<f64>, // get rid of this if it is all zero
    #[serde(rename="pressure_msl")]
    sea_level_pressure: Vec<f64>,
    #[serde(rename="cloud_cover")]
    cloud_cover: Vec<f64>,
    #[serde(rename="wind_speed_10m")]
    wind_speed_10m: Vec<f64>,
    #[serde(rename="cape")]
    cape: Vec<f64>,
    #[serde(rename="windspeed_250hPa")]
    wind_speed_250mb: Vec<f64>,
    #[serde(rename="geopotential_height_500hPa")]
    height_500mb: Vec<f64>
}

async fn get_open_meteo(s: &Station) -> Result<OpenMeteoResponse, String> {

    let lat = s.coords.0;
    let long = s.coords.1;

    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat:.2}&longitude={long:.2}&hourly=temperature_2m,dew_point_2m,apparent_temperature,precipitation_probability,precipitation,rain,snowfall,pressure_msl,cloud_cover,wind_speed_10m,cape,windspeed_250hPa,geopotential_height_500hPa&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch");

    // dbg!(&url);

    let client = reqwest::Client::new();

    let q = client.get(&url)
                .timeout(Duration::from_secs(10))
                .send()
                .await;

    let r = q.map_err(|e| e.to_string())?;

    let t = r.text().await.map_err(|e| e.to_string())?;

    // dbg!(&t);
   
    serde_json::from_str(&t).map_err(|e| e.to_string())
}

async fn forecast_handler() -> Result<String, String> {
    let mut s = title("FORECAST");

    let psm_station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };

    // dbg!("hello");

    let r = get_open_meteo(&psm_station).await?;

    // dbg!(&r);

    let now = Local::now();
    let today = now.date_naive();
    
    // iterate over every day from now until 5 days from now
    for day in (1..=5).map(|d| today + chrono::Duration::days(d)) {
        let afternoon = NaiveTime::from_hms_opt(13,0,0).unwrap(); // 1PM
        let naive_dt: NaiveDateTime = day.and_time(afternoon);
        
        let local_dt: DateTime<Local> = TimeZone::from_local_datetime(&Local, &naive_dt)
                                    .single()
                                    .unwrap_or_default(); // idk how to deal with the None situation.
                                    // I don't think it's possible in this case
        let utc_dt: DateTime<Utc> = local_dt.into();

        let hourly: &OpenMeteoResponseHourly = &r.hourly;

        let idx_result: Result<usize, usize> = hourly.time.binary_search(&utc_dt);

        let idx = match idx_result {
            Ok(s) => s,
            Err(e) => e
        };

        let dewpoint = hourly.dewpoint_2m[idx];

        s.push_str(&format!("{}: Dewp: {Bold}{dewpoint:.0}°F{Reset}\n", local_dt.format("%a %l%p")));
    }


    Ok(s)
}

async fn forecast() {
    match forecast_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", title("FORECAST"), e)},
    }
}



#[tokio::main] 
async fn main() {

    
    head_matter();
    random_section();

    tokio::join!(
        solar_lunar(), 
        current_conditions(),
        forecast(),

        // time_and_date();

        // forecast_analysis();
        // climatology();
        // stock_market();

        // on hold, seavey island API doesn't work
        //https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?date=latest&station=8419870&product=predictions&datum=STND&time_zone=gmt&interval=hilo&units=english&format=json
        // tides();
        
        
        // teleconnections();

        // earthquakes();

    );
    


}



