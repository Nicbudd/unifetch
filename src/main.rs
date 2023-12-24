use std::{fmt, collections::{HashMap, BTreeMap, HashSet}, time::Duration, fs, f32::consts::PI, hash::Hash, cmp::Ordering, path::Display};

use chrono::{Local, Utc, NaiveDate, NaiveTime, DateTime, TimeZone, NaiveDateTime, Datelike};
use chrono::Weekday::*;

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
    Underline,
    Blinking,
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
            Underline => "\x1b[4m",
            Blinking => "\x1b[5m",
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
                Underline => "4",
                Blinking => "5",
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
struct StationEntryWithTime(DateTime<Utc>, WxEntry);


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

enum Trend {
    Rising,
    Falling,
    UnknownChange,
    Neutral,
    RapidlyRising,
    RapidlyFalling
}

use Trend::*;

fn db_reverse_time(db: &BTreeMap<DateTime<Utc>, WxEntry>, duration: chrono::Duration) -> Result<&WxEntry, String> {
    let latest = db.last_key_value()
                .ok_or(String::from("database has nothing"))?;

    let dt_past = latest.0.clone() - duration;
    let past_entry = db.get(&dt_past)
                    .ok_or(String::from("Past entry was not found."))?;

    Ok(past_entry)
}

impl Trend {

    fn from_db_inner<'a, F: FnMut(&'a WxEntry) -> Option<f32>>
      (db: &'a BTreeMap<DateTime<Utc>, WxEntry>, 
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

    fn from_db<'a, F: FnMut(&'a WxEntry) -> Option<f32>>
        (db: &'a BTreeMap<DateTime<Utc>, WxEntry>, get_field: F, 
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
            UnknownChange => "",
            Neutral => " ➡"
        }
    }
}

impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}



#[derive(Debug)]
struct WeatherData {
    title: String,
    text: String,
    style: String,
}

impl fmt::Display for WeatherData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            write!(f, "")
        } else if self.title.len() == 0 {
            write!(f, "{}{}{Reset}", self.style, self.text)
        } else {
            write!(f, "{}: {}{}{Reset}", self.title, self.style, self.text)
        }
    }
}

impl WeatherData {
    fn len(&self) -> usize {
        if self.is_none() {
            0
        } else {
            self.title.len() + 2 + self.text.len()
            // Temp: 38F
        }
    }

    fn none() -> Self {
        WeatherData {title: String::new(), text: String::new(), style: String::new()}
    }

    fn is_none(&self) -> bool {
        return self.text.len() == 0;
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

fn format_wx(o: Option<Vec<String>>) -> WeatherData {
    let mut text: String;
    let mut style: String;

    match o {
        None => {
            return WeatherData::none();
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

    WeatherData {title: "".into(), text, style}


}


fn format_dewpoint(e: &WxEntry) -> (WeatherData, WeatherData) {
    let dew_text: String;
    let dew_style: String;

    let near_surface = e.layers.get(&Layer::NearSurface);
    let dewpoint = near_surface.map(|x| x.dewpoint).flatten();

    if dewpoint.is_none() {
        return (WeatherData::none(), WeatherData::none());
    }

    let a = dewpoint.unwrap();

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
    let rh = near_surface.map(|x| x.relative_humidity_2m()).flatten();

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

    (WeatherData{title: "Dew".into(), text: dew_text, style: dew_style}, WeatherData{title: "RH".into(), text: rh_text, style: rh_style})

} 

fn format_wind(e: &WxEntry) -> WeatherData {
    let text: String;
    let style: String;

    let near_surface = e.layers.get(&Layer::NearSurface);
    let wind_speed = near_surface.map(|x| x.wind()).flatten();

    if wind_speed.is_none() {
        return WeatherData::none();
    }

    let a = wind_speed.unwrap();

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

    text = if a.speed > 0. {
        format!("{:03}({})@{:2.0}kts", a.direction.degrees(), a.direction.cardinal(), a.speed)
    } else {
        String::from("Calm")
    };

    WeatherData {title: "Wind".into(), text, style}
}

fn format_cloud(s: &WxEntry) -> WeatherData {
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

    WeatherData{title: "Clouds".into(), text, style}
}

fn format_visibility(e: &WxEntry) -> WeatherData {
    let text: String;
    let style: String;

    let near_surface = e.layers.get(&Layer::NearSurface);
    let visibility = near_surface.map(|x| x.visibility).flatten();

    match visibility {
        None => return WeatherData::none(),
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

    WeatherData {title: "Vis".into(), text, style}

}

fn format_metar(e: &WxEntry) -> String {
    let s = e.raw_metar.clone();
    match s {
        Some(text) => { 
            let mut metar_string = String::from("\n  METAR:");
            let mut metar_length = metar_string.len();
            
            for s in text.split_ascii_whitespace() {
                let new_len = metar_length + 1 + s.len();

                if new_len <= COLUMN_WIDTH { 
                    metar_string.push(' ');
                    metar_string.push_str(&s.to_string());
                    metar_length = new_len;
                } else {
                    metar_string.push_str("\n    ");
                    metar_string.push_str(&s.to_string());
                    metar_length = 4 + s.len();
                }
            }

            metar_string

        }
        None => {String::new()}
    }
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

fn format_temp(e: &WxEntry, indoor: bool, db: &BTreeMap<DateTime<Utc>, WxEntry>) -> WeatherData {
    
    let temp = if indoor {
        e.layers.get(&Layer::Indoor).map(|x| x.temperature)
    } else {
        e.layers.get(&Layer::NearSurface).map(|x| x.temperature)
    };


    let temp_change = if indoor {
        Trend::from_db(&db, 
            |data| {data.layers.get(&Layer::Indoor).map(|x| x.temperature).flatten()}, 
                    (chrono::Duration::hours(2), 2.),
                    (chrono::Duration::hours(1), 2., chrono::Duration::hours(1), 2.))
    } else {
        Trend::from_db(&db, 
            |data| {data.layers.get(&Layer::NearSurface).map(|x| x.temperature).flatten()}, 
            (chrono::Duration::hours(2), 4.),
            (chrono::Duration::minutes(15), 2., chrono::Duration::hours(1), 4.))
    };
    
    if let Some(temp) = temp.flatten() {

        let style = if indoor {
            indoor_temp_style(temp)
        } else {
            outdoor_temp_style(temp)
        };

        WeatherData{title: "Temp".into(), style, text: format!("{temp:.0}F{temp_change}")}
    } else {
        WeatherData::none()
    }
}

fn format_apparent_temp(e: &WxEntry) -> WeatherData {
    let apparent_temp = e.layers.get(&Layer::NearSurface)
                                        .map(|x| x.apparent_temp())
                                        .flatten();

    if let Some(a) = apparent_temp {
        let style = outdoor_temp_style(a);
        WeatherData{title:"Feels".into(), text: format!("{a:.0}F"), style}
    } else {
        WeatherData::none()
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

fn format_pressure(e: &WxEntry, db: &BTreeMap<DateTime<Utc>, WxEntry>) -> WeatherData {
    
    // dbg!(&e);

    let slp = e.best_slp();
    // dbg!(slp);
    
    if let Some(pressure) = slp {
        let style = mslp_style(pressure);
        let pres_change = Trend::from_db(&db, 
            |data| {data.best_slp()}, 
            (chrono::Duration::hours(6), 3.),
            (chrono::Duration::minutes(15), 1., chrono::Duration::hours(3), 2.));
        
        WeatherData{title: "Pres".into(), text: format!("{pressure:.1}{pres_change}"), style}

    } else {
        WeatherData::none()
    }
}

#[derive(Copy, Clone, Debug)]
enum FlightRules {
    VFR,
    MVFR,
    IFR,
    LIFR
}

impl fmt::Display for FlightRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match *self {
            Self::VFR => "VFR",
            Self::MVFR => "MVFR",
            Self::IFR => "IFR",
            Self::LIFR => "LIFR",
        };
        write!(f, "{}", string)
    }
}

impl FlightRules {
    fn style(&self) -> String {
        match *self {
            Self::VFR => {Style::new(&[GreenBg, Bold])},
            Self::MVFR => {Style::new(&[BlueBg, Bold])},
            Self::IFR => {Style::new(&[RedBg, Bold])},
            Self::LIFR => {Style::new(&[PurpleBg, Bold])}
        }
    }
}

fn format_flight_rules(e: &WxEntry) -> WeatherData {
    use CloudLayerCoverage::*;
    use FlightRules::*;
    
    let mut ceiling_height = u32::MAX;

    let near_surface = e.layers.get(&Layer::Indoor);
    let visibility = near_surface.map(|x| x.visibility).flatten();

    if let (Some(cover), Some(vis)) = (&e.skycover, visibility) {
        let fr: FlightRules;
        
        if let SkyCoverage::Cloudy(v) = cover {
            for layer in v {
                match layer.coverage {
                    Scattered | Broken | Overcast => {
                        if layer.height < ceiling_height {
                            ceiling_height = layer.height;
                        }
                    }
                    Few => {}
                }
            }
        } // otherwise we have clear skys

        fr = match ceiling_height {
            0..=499 => LIFR,
            _ if vis < 1.0 => LIFR,
            500..=999 => IFR,
            _ if vis < 3.0 => IFR,
            1000..=2999 => MVFR,
            _ if vis < 5.0 => MVFR,
            _ => VFR,
        };

        return WeatherData{title: "".into(), text: fr.to_string(), style: fr.style()};
    
    } else {
        return WeatherData::none();
    }

} 


const COLUMN_WIDTH: usize = 80;


fn station_line(prelude: &str, e: &WxEntry, indoor: bool,
  db: &BTreeMap<DateTime<Utc>, WxEntry>) -> Result<String, String> {

    let mut string_vec: Vec<WeatherData> = vec![];

    let mut total_string = String::new();

    let (dewpoint, rh) = format_dewpoint(e);

    string_vec.push(format_flight_rules(e));
    string_vec.push(format_temp(e, indoor, db));
    string_vec.push(format_apparent_temp(e));
    string_vec.push(format_pressure(e, db));
    string_vec.push(dewpoint);
    string_vec.push(rh);
    string_vec.push(format_visibility(e));
    string_vec.push(format_wx(e.present_wx.clone()));
    string_vec.push(format_wind(e));
    string_vec.push(format_cloud(e));

    total_string.push_str(prelude);

    let mut line_length = total_string.len();

    for data in string_vec {
        let new_len = line_length + 1 + data.len();
        
        // dbg!(&data, data.len(), new_len);

        if data.is_none() {
            continue;
        } else if new_len <= COLUMN_WIDTH {
            total_string.push(' ');
            total_string.push_str(&data.to_string());
            line_length = new_len;
        } else {
            total_string.push_str("\n  ");
            total_string.push_str(&data.to_string());
            line_length = 2 + data.len();
        };
    }

    total_string.push_str(&format_metar(e));
    
    total_string.push('\n');

    Ok(total_string)
}

async fn current_conditions_handler() -> Result<String, String> {

    let psm_station: Station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };  

    let apt_station: Station = Station {
        coords: (43.00, 0.0), // im not giving that away
        altitude: 24.,
        name: String::from("APT"),
    }; 


    let apt_conditions = wxer_query("local", "hourly").await?;
    let psm_conditions = wxer_query("psm", "hourly").await?;

    // dbg!(&local_conditions);
    // dbg!(&psm_conditions);

    let apt_db: BTreeMap<DateTime<Utc>, WxEntry> = serde_json::from_str(&apt_conditions).map_err(|e| e.to_string())?;
    let psm_db: BTreeMap<DateTime<Utc>, WxEntry> = serde_json::from_str(&psm_conditions).map_err(|e| e.to_string())?;

    let mut s = title("CURRENT CONDITIONS");

    let latest_apt = apt_db.last_key_value()
        .ok_or(String::from("Local json did not have any values"))?;

    let latest_psm = psm_db.last_key_value()
        .ok_or(String::from("PSM json did not have any values"))?;

    let local_time_apt: DateTime<Local> = latest_apt.0.clone().into();
    let local_time_psm: DateTime<Local> = latest_psm.0.clone().into();

    let apt_prelude = format!("{}: ⌛{}", apt_station.name, local_time_apt.format("%I:%M %p"));
    let psm_prelude = format!("{}: ⌛{}", psm_station.name, local_time_psm.format("%I:%M %p"));

    let apt_line = station_line(&apt_prelude, latest_apt.1, true, &apt_db)?;
    let psm_line = station_line(&psm_prelude, latest_psm.1, false, &psm_db)?;

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
    hourly: OpenMeteoResponseHourly
}

// #[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenMeteoResponseHourly {
    #[serde(deserialize_with="from_iso8601_no_seconds")]
    time: Vec<DateTime<Utc>>,
    #[serde(rename="temperature_2m")]
    temperature_2m: Vec<f32>,
    #[serde(rename="dew_point_2m")]
    dewpoint_2m: Vec<f32>,
    #[serde(rename="apparent_temperature")]
    feels_like: Vec<f32>,
    #[serde(rename="precipitation_probability")]
    precip_probability: Vec<f32>,
    #[serde(rename="precipitation")]
    precip: Vec<f32>,
    #[serde(rename="rain")]
    rain: Vec<f32>,
    #[serde(rename="snowfall")]
    snowfall: Vec<f32>, // get rid of this if it is all zero
    #[serde(rename="pressure_msl")]
    sea_level_pressure: Vec<f32>,
    #[serde(rename="cloud_cover")]
    cloud_cover: Vec<f32>,
    #[serde(rename="wind_speed_10m")]
    wind_speed_10m: Vec<f32>,
    #[serde(rename="wind_direction_10m")]
    wind_dir_10m: Vec<u16>,
    #[serde(rename="cape")]
    cape: Vec<f32>,
    #[serde(rename="windspeed_250hPa")]
    wind_speed_250mb: Vec<f32>,
    #[serde(rename="geopotential_height_500hPa")]
    height_500mb: Vec<f32>,
    #[serde(rename="visibility")]
    visibility: Vec<f32>
}

async fn get_open_meteo(s: &Station) -> Result<OpenMeteoResponse, String> {

    let lat = s.coords.0;
    let long = s.coords.1;

    let url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat:.2}&longitude={long:.2}&hourly=temperature_2m,dew_point_2m,visibility,apparent_temperature,precipitation_probability,precipitation,rain,snowfall,pressure_msl,cloud_cover,wind_speed_10m,wind_direction_10m,cape,windspeed_250hPa,geopotential_height_500hPa&daily=temperature_2m_max,temperature_2m_min,precipitation_probability_max&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch");

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

fn open_meteo_to_entries(open_meteo: OpenMeteoResponse) -> Vec<WxEntry> {

    let psm_station: Station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };  

    let mut entries: Vec<WxEntry> = vec![];

    let hourly = open_meteo.hourly;

    for (idx, date_time) in hourly.time.iter().enumerate() {
        let temperature_2m = Some(hourly.temperature_2m[idx]);
        let dewpoint_2m = Some(hourly.dewpoint_2m[idx]);
        let sea_level_pressure = Some(hourly.sea_level_pressure[idx]);
        let visibility = Some(hourly.visibility[idx] / 5280.);

        let wind_direction = Direction::from_degrees(hourly.wind_dir_10m[idx]).ok();
        let wind_speed = Some(hourly.wind_speed_10m[idx]);

        let rain = hourly.rain[idx];
        let snow = hourly.snowfall[idx];
        let unknown = hourly.precip[idx] - rain - snow;

        //todo: precip in station entry structs
        #[allow(unused_variables)]
        let precip = Some(Precip {rain, snow, unknown}); 


        let mut weather = vec![];
        if rain > 0. {weather.push("RA".into())};
        if snow > 0. {weather.push("SN".into())};
        let present_wx = Some(weather);

        let mut layers = HashMap::new();

        let near_surface = WxEntryLayer {
            layer: Layer::NearSurface,
            height_agl: Some(2.0),
            height_msl: Some(psm_station.altitude),
            temperature: temperature_2m,
            dewpoint: dewpoint_2m,
            pressure: None,
            wind_direction,
            wind_speed,
            visibility,
        };

        let mut sea_level = WxEntryLayer::empty(Layer::SeaLevel);

        sea_level.pressure = sea_level_pressure;

        layers.insert(Layer::NearSurface, near_surface);
        layers.insert(Layer::SeaLevel, sea_level);


        let e = WxEntry {
            date_time: date_time.clone(),
            station: psm_station.clone(),

            layers,

            cape: Some(hourly.cape[idx]),
            skycover: None,
            present_wx,
            raw_metar: None,
            precip,
            precip_probability: Some(hourly.precip_probability[idx]),
            precip_today: None,
        };

        entries.push(e);
    }

    entries

}

fn day_of_week_style<T: TimeZone>(dt: &DateTime<T>) -> String {

    match dt.weekday() {
        Mon => Style::new(&[Red, Bold]),
        Tue => Style::new(&[Yellow, Bold]),
        Wed => Style::new(&[Green, Bold]),
        Thu => Style::new(&[Purple, Bold]),
        Fri => Style::new(&[Blue, Bold]),
        Sat => Style::new(&[Cyan, Bold]),
        Sun => Style::new(&[Bold]),
    }

}

async fn forecast_handler() -> Result<String, String> {
    let mut s = title("FORECAST");

    let psm_station = Station {
        coords: (43.08, -70.82),
        altitude: 30.,
        name: String::from("KPSM"),
    };

    let now = Utc::now();

    let r = get_open_meteo(&psm_station).await?;
    let entries = open_meteo_to_entries(r);



    let mut included = BTreeMap::new();

    for hours_from_now in [0, 1, 2, 3, 4, 5, 6, 9, 12, 18, 
                            (24*1 + 0), (24*1 + 6), (24*1 + 12), (24*1 + 18),
                            (24*2 + 0), (24*2 + 6), (24*2 + 12), (24*2 + 18),
                            (24*3 + 0), (24*3 + 12),
                            (24*4 + 0), (24*4 + 12),
                            (24*5 + 0), (24*5 + 12),
                            (24*6 + 0), (24*6 + 12),
                            (24*7 + 0), (24*7 + 12)] {
        let dt = now + chrono::Duration::hours(hours_from_now);
        
        let entry_idx = entries.binary_search_by(|x| x.date_time.cmp(&dt));
        let entry_idx = match entry_idx {
            Ok(s) => s,
            Err(e) => {
                if e == entries.len() {
                    e - 1
                } else {
                    e
                }
            },
        };

        let entry = entries.get(entry_idx).unwrap();

        included.insert(entry.date_time, entry);    
    }

    for (dt, entry) in included {
        let local_dt: DateTime<Local> = dt.into();
        let day_of_week_style = day_of_week_style(&local_dt);

        let prelude = format!("{day_of_week_style}{}{Reset} {}:", local_dt.format("%a"), local_dt.format("%d %l%p"));
        s.push_str(&station_line(&prelude, entry,  false, &BTreeMap::new())?);
    }


    Ok(s)
}

async fn forecast() {
    match forecast_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", title("FORECAST"), e)},
    }
}

// EARTHQUAKES ----------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
struct USGSResponse {
    features: Vec<USGSEarthquake>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct USGSEarthquake {
    #[serde(rename="type")]
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
    #[serde(rename="type")]
    geotype: String,
    coordinates: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Earthquake { // put the USGS formatted earthquakes in a form that's easy for us to use.
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
    dist: f32,
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
    fn from_USGS(usgs: USGSEarthquake, lat: f32, long: f32) -> Result<Self, String> {
        // dbg!(&usgs);

        Ok(Earthquake { 
            shaketype: usgs.shaketype,
            mag: usgs.properties.mag,
            place: usgs.properties.place.unwrap_or("No Nearby Locations".into()),
            time: DateTime::<Utc>::from_timestamp(usgs.properties.time / 1000, 0).ok_or(String::from("Timestamp invalid"))?,
            felt: usgs.properties.felt,
            mmi: usgs.properties.mmi,
            alert: usgs.properties.alert,
            latitude: usgs.geometry.coordinates.1,
            longitude: usgs.geometry.coordinates.0,
            depth: usgs.geometry.coordinates.2,
            dist: distance_between_coords(usgs.geometry.coordinates.1, usgs.geometry.coordinates.0, lat, long),
            id: usgs.id,
        })
    }

    fn mag_style(&self) -> String {
        if self.mag > 8. {
            Style::new(&[RedBg, Black, Bold])
        } else if self.mag > 7. {
            Style::new(&[Red, Bold])
        } else if self.mag > 6. {
            Style::new(&[YellowBg, Black, Bold])
        } else if self.mag > 5. {
            Style::new(&[Yellow, Bold])
        } else if self.mag > 3. {
            Style::new(&[Blue, Bold])
        } else {
            Style::new(&[Bold])
        }
    }

    fn mmi_format(&self) -> String {

        if let Some(mmi) = self.mmi {
            if mmi > 11.5 {
                format!("MMI: {}XII{Reset}, ", Style::new(&[RedBg, Bold, Blinking]))
            } else if mmi > 10.5 {
                format!("MMI: {}XI{Reset}, ", Style::new(&[RedBg, Bold, Blinking]))
            } else if mmi > 9.5 {
                format!("MMI: {}X{Reset}, ", Style::new(&[RedBg, Bold]))
            } else if mmi > 8.5 {
                format!("MMI: {}IX{Reset}, ", Style::new(&[Red, Bold]))
            } else if mmi > 7.5 {
                format!("MMI: {}VIII{Reset}, ", Style::new(&[YellowBg, Bold]))
            } else if mmi > 6.5 {
                format!("MMI: {}VII{Reset}, ", Style::new(&[Yellow, Bold]))
            } else if mmi > 5.5 {
                format!("MMI: {}VI{Reset}, ", Style::new(&[Yellow, Bold]))
            } else if mmi > 4.5 {
                format!("MMI: {}V{Reset}, ", Style::new(&[Green, Bold]))
            } else if mmi > 3.5 {
                format!("MMI: {}IV{Reset}, ", Style::new(&[Blue, Bold]))
            } else if mmi > 2.5 {
                format!("MMI: {}III{Reset}, ", Style::new(&[Cyan, Bold]))
            } else if mmi > 1.5 {
                format!("MMI: {}II{Reset}, ", Style::new(&[Cyan, Bold]))
            } else {
                format!("MMI: I, ")
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
                Style::new(&[Green])
            } else if a == "yellow" {
                Style::new(&[Yellow, Bold])
            } else if a == "orange" {
                Style::new(&[YellowBg, Black, Bold])
            } else if a == "red" {
                Style::new(&[RedBg, Black, Blinking, Bold])
            } else {
                Style::new(&[PurpleBg, Black, Bold])
            }


        } else {
            "".into()
        };

        let dist = if self.dist < 1000. {
            format!(" ({:.0} mi)", self.dist)
        } else {
            String::new()
        };

        write!(f, "{}M{:.1}{Reset}, {}{:.0}km dp, {}, {}{}{Reset}{}\n", self.mag_style(), self.mag, self.mmi_format(), self.depth, self.time.format("%Y-%m-%d %H:%MZ"), alert_color, self.place, dist)
    }
}

async fn get_earthquakes(url: &str, client: &reqwest::Client, lat: f32, long: f32) -> Result<Vec<Earthquake>, String> {

    // dbg!(&url);

    let q = client.get(url)
            .timeout(Duration::from_secs(10))
            .send()
            .await;

    let r = q.map_err(|e| e.to_string())?;

    let t = r.text().await.map_err(|e| e.to_string())?;

    // dbg!(&t);
    
    let usgs: USGSResponse = serde_json::from_str(&t).map_err(|e| e.to_string())?;

    let quakes = usgs.features
                .into_iter()
                .map(|x| Earthquake::from_USGS(x, lat, long))
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

fn distance_between_coords(lat1: f32, long1: f32, lat2: f32, long2: f32) -> f32 {

    // dbg!(lat1, lat2, long1, long2);

    // Haversine formula
    let earth_radius = 3956.5; // miles, approx
    let phi_1 = lat1 * PI / 180.;
    let phi_2 = lat2 * PI / 180.;
    let delta_phi = (lat2-lat1) * PI / 180.;
    let delta_lmbda = (long2-long1) * PI / 180.;

    let a = (delta_phi/2.).sin() * (delta_phi/2.).sin() + 
    phi_1.cos() * phi_2.cos() * 
    (delta_lmbda / 2.).sin() * (delta_lmbda / 2.).sin();

    let c = 2. * (a.sqrt()).atan2((1.-a).sqrt());

    let d = earth_radius  * c;

    d
} 

async fn earthquake_handler() -> Result<String, String> {
    let mut s = title("EARTHQUAKES");

    //TODO: Make this more generic
    let lat = 43;
    let long = -71;

    let lat_f = 43.08;
    let long_f = -70.86;

    let now = Utc::now();
    let three_months_ago = now - chrono::Duration::days(180);
    let starttime = three_months_ago.format("%Y-%m-%d");
    
    // >5 mag anywhere for last 3 months of earthquakes
    let url1 = format!("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime={starttime}&minmagnitude=5&orderby=time");
    // earthquakes >2 mag within 150 km of PSM
    let url2 = format!("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minmagnitude=2&latitude={lat}&longitude={long}&maxradiuskm=150&orderby=time");
    // earthquakes >3 mag within 300 km of PSM
    let url3 = format!("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minmagnitude=3&latitude={lat}&longitude={long}&maxradiuskm=300&orderby=time");
    // earthquakes >4 mag within 800 km of PSM
    let url4 = format!("https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&minmagnitude=4&latitude={lat}&longitude={long}&maxradiuskm=800&orderby=time");

    let client = reqwest::Client::new();

    // global quakes
    let v1 = get_earthquakes(&url1, &client, lat_f, long_f).await?;

    // local quakes
    let v2 = get_earthquakes(&url2, &client, lat_f, long_f).await?;
    let v3 = get_earthquakes(&url3, &client, lat_f, long_f).await?;
    let v4 = get_earthquakes(&url4, &client, lat_f, long_f).await?;

    // get a set containing local quakes to remove duplicates
    let mut local_quakes = HashSet::new();
    v2.iter().chain(v3.iter()).chain(v4.iter()).for_each(|x| {local_quakes.insert(x);});

    // collect them back into a vector so we can sort them by distance.
    let mut local_quakes = local_quakes.iter().collect::<Vec<_>>();
    local_quakes.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap_or(Ordering::Greater));


    if local_quakes.len() >= 1 {
        s.push_str(&format!("Local Earthquakes:\n"));

        for q in local_quakes {
            s.push_str(&format!("{}", q));
            // dbg!(&q);
        }
        s.push_str("\n");
    }

    let global_quakes = tallest_skyscrapers(&v1);

    s.push_str(&format!("Global Earthquakes:\n"));

    for q in global_quakes {
        s.push_str(&format!("{}", q));
    }

    Ok(s)


    // "tallest skyscrapers" (>5 mag) for last 3 months of earthquakes
    // "local" earthquakes - earthquakes >2 mag within 150 km of PSM or >3 mag within 300km or >4 mag within 800km

}


async fn earthquakes() {
    match earthquake_handler().await {
        Ok(s) => {println!("{}", s)},
        Err(e) => {println!("{}{}", title("EARTHQUAKE"), e)},
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

        earthquakes()

    );
    


}



