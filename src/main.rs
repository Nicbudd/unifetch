use std::{fmt, collections::HashMap, time::Duration};

use chrono::{Local, Utc, NaiveDate, NaiveTime};
use rand::{self, Rng, thread_rng, rngs::ThreadRng};

use serde_json::Value;
use tokio;

const COORDS: (f64, f64) = DURHAM_COORDS;
const DURHAM_COORDS: (f64, f64) = (43.13, -70.92);

fn coords_str() -> String {
    format!("{:.2},{:.2}", COORDS.0, COORDS.1)
}

// GENERAL -----------------------------------------------------------

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

fn title(s: &str) -> String {
    format!("\n{:-^80}\n", s)
}



// HEAD MATTER -----------------------------------------------------------

fn head_matter() {
    // let s: String = "DS2 ".into()
    let utc_now = Utc::now().format("(%H:%MZ)");
    let local_now = Local::now().format("%a %Y-%b-%d @ %I:%M:%S%p");
    let r = rand::random::<u32>();

    println!("{}DS2 {} {} - {:08X}", terminal_line('-'), local_now, utc_now, r);
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
    
    let d2 = rng.gen_range(0..=1);

    let coin = if d2 == 0 {
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

    s.push_str(&format!("Coin: {Bold}{}{Reset} | Dice (D6): {Bold}{} {}{Reset} {} {} {} {}\n", 
            coin, d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng), d6(&mut rng)));

    s.push_str(&format!("D20: {} {} {} {}\n", d20(&mut rng), d20(&mut rng), d20(&mut rng), d20(&mut rng)));

    s.push_str(&format!("D3: {} | D4: {} | D8: {} | D10: {} | D12: {} | D100: {}\n", d3, d4, d8, d10, d12, d100));

    // s.push_str(&format!("Bits: {:064b}\n", bits));
    s.push_str(&format!("Bits: {Bold}{:032b}{Reset}{:016b}{Bold}{:08b}{Reset}{:08b}\n", next_32_bits, next_16_bits, next_8_bits, first_8_bits));

    s.push_str(&format!("Hex:  {:032x}{:032x}\n", hex.0, hex.1));

    s.push_str(&format!("Prob: {:.08} | 10 Digits: {:010} | 6 Letters: {}\n", prob, ten_digit, six_letters.iter().collect::<String>()));

    let date = rand_date(&mut rng);

    s.push_str(&format!("Date: {} \x1b[0;47;37m{}{Reset}", date.format("%Y-%m-%d"), date.format("%a")));

    println!("{}", s);

}

// SOLAR/LUNAR --------------------------------------------------

async fn parse_request_json(w: Result<reqwest::Response, reqwest::Error>) -> Result<serde_json::Value, String> {
    
    let r = w.map_err(|e| e.to_string())?;
    // dbg!(&r);
    let j = r.json().await.map_err(|e| e.to_string())?;

    Ok(j)
}

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
    let data = &json["properties"]["data"];
    let sundata = &data["sundata"];
    let moondata = &data["moondata"];

    // dbg!(&sundata);

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

    Ok(format!("{}{}{}{}\n",
        string_from_rise_set_times("Sun", "Rise", "Set", sunrise, sunset),
        string_from_rise_set_times("Twilight", "Begin", "End", twilight_start, twilight_end),
        string_from_rise_set_times("Moon", "Rise", "Set", moonrise, moonset),   
        phase_string
    ))
}

async fn solar_lunar() {
    let mut s: String = title("SOLAR & LUNAR");

    let now = Local::now();

    let mut map = HashMap::new();

    map.insert("date", now.format("%Y-%m-%d").to_string());
    map.insert("coords", coords_str());


    let client = reqwest::Client::new();
    
    let r = client.get("https://aa.usno.navy.mil/api/rstt/oneday")
                .query(&map)
                .timeout(Duration::from_secs(5));

    // dbg!(&r);
    
    let form = r.send()
                .await;


    match parse_request_json(form).await {
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

fn current_conditions() {

}


#[tokio::main] 
async fn main() {

    head_matter();
    random_section();
    solar_lunar().await;
    // time_and_date();

    current_conditions();
    // forecast();
    // forecast_analysis();
    // climatology();
    // stock_market();

    //https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?date=latest&station=8419870&product=predictions&datum=STND&time_zone=gmt&interval=hilo&units=english&format=json
    // tides();
    
    
    // teleconnections();

    // earthquakes();

}



