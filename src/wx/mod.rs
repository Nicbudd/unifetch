pub mod forecast;
pub mod tele;
pub mod weather;

use crate::common;
use common::Style;
use common::TermStyle::*;

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::fmt;
use wxer_lib::WxEntryLayer;

use wxer_lib::*;

// WEATHER DATA ------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct WeatherData {
    title: String,
    text: String,
    style: String,
}

#[derive(Debug, PartialEq)]
enum WxCategory {
    Snow,
    Rain,
    Severe,
    Fire,
    Fog,
    None,
}

impl fmt::Display for WeatherData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            write!(f, "")
        } else if self.title.is_empty() {
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
        WeatherData {
            title: String::new(),
            text: String::new(),
            style: String::new(),
        }
    }

    fn is_none(&self) -> bool {
        self.text.is_empty()
    }
}

fn format_wx(o: Option<Vec<String>>) -> WeatherData {
    let mut text: String;
    let mut style: String;

    match o {
        None => {
            return WeatherData::none();
        }
        Some(v) => {
            let mut wx_types = vec![];

            for wx in &v {
                let c = if wx.contains("FU") | wx.contains("VA") {
                    WxCategory::Fire
                } else if wx.contains("GR")
                    | wx.contains("TS")
                    | wx.contains("DU")
                    | wx.contains("SA")
                    | wx.contains("SQ")
                    | wx.contains("DS")
                    | wx.contains("SS")
                    | wx.contains("FC")
                {
                    WxCategory::Severe
                } else if wx.contains("SN")
                    | wx.contains("SG")
                    | wx.contains("GS")
                    | wx.contains("PL")
                    | wx.contains("IC")
                {
                    WxCategory::Snow
                } else if wx.contains("DZ") | wx.contains("RA") | wx.contains("UP") {
                    WxCategory::Rain
                } else if wx.contains("FG")
                    | wx.contains("BR")
                    | wx.contains("HZ")
                    | wx.contains("PY")
                {
                    WxCategory::Fog
                } else {
                    WxCategory::None
                };

                wx_types.push(c);
            }

            if wx_types.contains(&WxCategory::Fire) {
                style = Style::string(&[RedBg, White, Bold]);
            } else if wx_types.contains(&WxCategory::Severe) {
                style = Style::string(&[YellowBg, Black, Bold]);
            } else if wx_types.contains(&WxCategory::Snow) {
                style = Style::string(&[WhiteBg, Blue, Bold]);
            } else if wx_types.contains(&WxCategory::Rain) {
                style = Style::string(&[BlueBg, Black, Bold]);
            } else if wx_types.contains(&WxCategory::Fog) {
                style = Style::string(&[WhiteBg, Black, Bold]);
            } else {
                style = Style::string(&[Reset]);
            }

            text = v.join(" ");
        }
    }

    if text.is_empty() {
        text.push_str("No WX");
        style = Style::string(&[Bold])
    }

    WeatherData {
        title: "".into(),
        text,
        style,
    }
}

// TREND --------------------------------------------------------------------------------------------------------------

enum Trend {
    Rising,
    Falling,
    UnknownChange,
    Neutral,
    RapidlyRising,
    RapidlyFalling,
}

use Trend::*;

fn db_reverse_time(
    db: &BTreeMap<DateTime<Utc>, WxStructDeserialized>,
    duration: chrono::Duration,
) -> Result<&WxStructDeserialized, String> {
    let latest = db
        .last_key_value()
        .ok_or(String::from("database has nothing"))?;

    let dt_past = *latest.0 - duration;
    let past_entry = db
        .get(&dt_past)
        .ok_or(String::from("Past entry was not found."))?;

    Ok(past_entry)
}

impl Trend {
    fn from_db_inner<'a, F: FnMut(&'a WxStructDeserialized) -> Option<f32>>(
        db: &'a BTreeMap<DateTime<Utc>, WxStructDeserialized>,
        mut get_field: F,
        change_criteria: (chrono::Duration, f32),
        rapid_criteria: (chrono::Duration, f32, chrono::Duration, f32),
    ) -> Result<Trend, ()> {
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

    fn from_db<'a, F: FnMut(&'a WxStructDeserialized) -> Option<f32>>(
        db: &'a BTreeMap<DateTime<Utc>, WxStructDeserialized>,
        get_field: F,
        change_criteria: (chrono::Duration, f32),
        rapid_criteria: (chrono::Duration, f32, chrono::Duration, f32),
    ) -> Trend {
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
            Neutral => " ➡",
        }
    }
}

impl fmt::Display for Trend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str())
    }
}

// FORMATTERS ------------------------------------------------------------------------------------------------------

fn format_dewpoint(e: &WxEntryStruct) -> (WeatherData, WeatherData) {
    let near_surface = e.layer(Layer::NearSurface);
    let dewpoint = near_surface.and_then(|x| x.dewpoint());

    if dewpoint.is_none() {
        return (WeatherData::none(), WeatherData::none());
    }

    let a = dewpoint.unwrap().value_in(Fahrenheit);

    let dew_style: String = if a > 70. {
        Style::string(&[PurpleBg, Black, Bold])
    } else if a > 60. {
        Style::string(&[BlueBg, Black, Bold])
    } else if a > 45. {
        Style::string(&[GreenBg, Black, Bold])
    } else if a < 30. {
        Style::string(&[YellowBg, Black, Bold])
    } else {
        Style::string(&[Bold])
    };

    let dew_text: String = format!("{a:.0}F");

    let rh_text: String;
    let rh_style: String;
    let rh = near_surface.and_then(|x| x.relative_humidity());

    match rh {
        Some(a) => {
            let a = a.value_in(Percent);
            rh_style = if a > 95. {
                Style::string(&[PurpleBg, Black, Bold])
            } else if a > 90. {
                Style::string(&[BlueBg, Black, Bold])
            } else if a > 70. {
                Style::string(&[GreenBg, Black, Bold])
            } else if a > 40. {
                Style::string(&[Bold])
            } else {
                Style::string(&[YellowBg, Black, Bold])
            };

            rh_text = format!("{a:.0}%");
        }
        None => {
            rh_text = String::from("N/A");
            rh_style = Style::error();
        }
    }

    (
        WeatherData {
            title: "Dew".into(),
            text: dew_text,
            style: dew_style,
        },
        WeatherData {
            title: "RH".into(),
            text: rh_text,
            style: rh_style,
        },
    )
}

fn format_wind(e: &WxEntryStruct) -> WeatherData {
    let near_surface = e.layers.get(&Layer::NearSurface);
    let wind = near_surface.and_then(|x| x.wind());

    if wind.is_none() {
        return WeatherData::none();
    }

    let a = wind.unwrap();
    let speed = a.speed.value_in(Knots);

    let style: String = if speed > 45. {
        Style::string(&[YellowBg, Black, Bold])
    } else if speed > 32. {
        Style::string(&[RedBg, Black, Bold])
    } else if speed > 20. {
        Style::string(&[PurpleBg, Black, Bold])
    } else if speed > 12. {
        Style::string(&[BlueBg, Black, Bold])
    } else {
        Style::string(&[Bold])
    };

    let text: String = if speed > 0. {
        if let Some(dir) = a.direction {
            format!("{:03}({})@{:2.0}kts", dir.degrees(), dir.cardinal(), speed)
        } else {
            format!("{speed:2.0}kts")
        }
    } else {
        String::from("Calm")
    };

    WeatherData {
        title: "Wind".into(),
        text,
        style,
    }
}

fn format_cloud(s: &WxEntryStruct) -> WeatherData {
    let text: String;
    let style: String;

    match &s.skycover {
        None => {
            text = "".into();
            style = Style::string(&[Red, Bold]);
        }
        Some(s) => match s {
            SkyCoverage::Clear => {
                text = "CLR".into();
                style = Style::string(&[Bold]);
            }
            SkyCoverage::Cloudy(v) => {
                text = v
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                let all_covers = v.iter().map(|x| x.coverage).collect::<Vec<_>>();

                style = if all_covers.contains(&CloudLayerCoverage::Overcast) {
                    Style::string(&[WhiteBg, Black, Bold])
                } else {
                    Style::string(&[Bold])
                }
            }
        },
    }

    WeatherData {
        title: "Clouds".into(),
        text,
        style,
    }
}

fn format_visibility(e: &WxEntryStruct) -> WeatherData {
    let text: String;
    let style: String;

    let near_surface = e.layers.get(&Layer::NearSurface);
    let visibility = near_surface.and_then(|x| x.visibility);

    match visibility {
        None => return WeatherData::none(),
        Some(v) => {
            let v = v.value_in(Mile);
            if v <= 1. {
                style = Style::string(&[WhiteBg, Black, Bold]);
                text = format!("{v:.2}mi");
            } else if v < 3. {
                style = Style::string(&[WhiteBg, Black, Bold]);
                text = format!("{v:.1}mi");
            } else {
                style = Style::string(&[Bold]);
                text = format!("{v:.0}mi");
            }
        }
    }

    WeatherData {
        title: "Vis".into(),
        text,
        style,
    }
}

fn format_metar(e: &WxEntryStruct) -> String {
    let s = e.raw_metar.clone();
    match s {
        Some(text) => {
            let mut metar_string = String::from("\n  METAR:");
            let mut metar_length = metar_string.len();

            for s in text.split_ascii_whitespace() {
                let new_len = metar_length + 1 + s.len();

                if new_len <= COLUMN_WIDTH {
                    metar_string.push(' ');
                    metar_string.push_str(s);
                    metar_length = new_len;
                } else {
                    metar_string.push_str("\n    ");
                    metar_string.push_str(s);
                    metar_length = 4 + s.len();
                }
            }

            metar_string
        }
        None => String::new(),
    }
}

fn indoor_temp_style(temp: Temperature) -> String {
    let temp = temp.value_in(Fahrenheit);

    if temp.is_nan() {
        Style::string(&[Red, Bold])
    } else if temp < 65. {
        Style::string(&[BlueBg, Bold])
    } else if temp < 75. {
        Style::string(&[NoStyle, Bold])
    } else {
        Style::string(&[RedBg, Bold])
    }
}

fn outdoor_temp_style(temp: Temperature) -> String {
    let temp = temp.value_in(Fahrenheit);

    if temp.is_nan() {
        Style::string(&[Red, Bold])
    } else if temp < 10. {
        Style::string(&[PurpleBg, Bold])
    } else if temp < 32. {
        Style::string(&[BlueBg, Bold])
    } else if temp < 55. {
        Style::string(&[GreenBg, Black, Bold])
    } else if temp < 70. {
        Style::string(&[YellowBg, Black, Bold])
    } else if temp < 85. {
        Style::string(&[RedBg, Bold])
    } else if temp < 95. {
        Style::string(&[WhiteBg, Red, Bold])
    } else {
        Style::string(&[PurpleBg, Red, Bold])
    }
}

fn format_temp(
    e: &WxEntryStruct,
    indoor: bool,
    db: &BTreeMap<DateTime<Utc>, WxStructDeserialized>,
) -> WeatherData {
    let temp = if indoor {
        e.layers.get(&Layer::Indoor).map(|x| x.temperature)
    } else {
        e.layers.get(&Layer::NearSurface).map(|x| x.temperature)
    };

    let temp_change = if indoor {
        Trend::from_db(
            db,
            |data| {
                data.layers
                    .get(&Layer::Indoor)
                    .and_then(|x| x.temperature)
                    .map(|x| x.value_in(Fahrenheit))
            },
            (chrono::Duration::hours(2), 2.),
            (
                chrono::Duration::hours(1),
                2.,
                chrono::Duration::hours(1),
                2.,
            ),
        )
    } else {
        Trend::from_db(
            db,
            |data| {
                data.layers
                    .get(&Layer::NearSurface)
                    .and_then(|x| x.temperature)
                    .map(|x| x.value_in(Fahrenheit))
            },
            (chrono::Duration::hours(2), 4.),
            (
                chrono::Duration::minutes(15),
                2.,
                chrono::Duration::hours(1),
                4.,
            ),
        )
    };

    if let Some(temp) = temp.flatten() {
        let style = if indoor {
            indoor_temp_style(temp)
        } else {
            outdoor_temp_style(temp)
        };

        WeatherData {
            title: "Temp".into(),
            style,
            text: format!("{:.0}F{temp_change}", temp.value_in(Fahrenheit)),
        }
    } else {
        WeatherData::none()
    }
}

fn format_apparent_temp(e: &WxEntryStruct) -> WeatherData {
    let apparent_temp = e
        .layers
        .get(&Layer::NearSurface)
        .and_then(|x| x.apparent_temp());

    if let Some(a) = apparent_temp {
        let style = outdoor_temp_style(a);
        WeatherData {
            title: "Feels".into(),
            text: format!("{:.0}F", a.value_in(Fahrenheit)),
            style,
        }
    } else {
        WeatherData::none()
    }
}

fn mslp_style(pres: Pressure) -> String {
    let pres = pres.value_in(Mbar);
    if pres.is_nan() {
        Style::string(&[Red, Bold])
    } else if pres < 1005. {
        Style::string(&[RedBg, Black, Bold])
    } else if pres > 1025. {
        Style::string(&[BlueBg, Black, Bold])
    } else {
        Style::string(&[Bold])
    }
}

fn format_pressure(
    e: &WxEntryStruct,
    db: &BTreeMap<DateTime<Utc>, WxStructDeserialized>,
) -> WeatherData {
    // dbg!(&e);

    let slp = e.best_slp();
    // dbg!(slp);

    if let Some(pressure) = slp {
        let style = mslp_style(pressure);
        let pres_change = Trend::from_db(
            db,
            |data| data.best_slp().map(|x| x.value_in(Mbar)),
            (chrono::Duration::hours(6), 3.),
            (
                chrono::Duration::minutes(15),
                1.,
                chrono::Duration::hours(3),
                2.,
            ),
        );

        WeatherData {
            title: "Pres".into(),
            text: format!("{pressure:.1}{pres_change}"),
            style,
        }
    } else {
        WeatherData::none()
    }
}

fn style_500mb_height(h: f32) -> String {
    if h > 570.0 {
        Style::string(&[YellowBg, Black, Bold])
    } else if h > 585.0 {
        Style::string(&[RedBg, Bold])
    } else if h < 530.0 {
        Style::string(&[BlueBg, Black, Bold])
    } else if h < 515.0 {
        Style::string(&[PurpleBg, Bold])
    } else {
        Style::string(&[Bold])
    }
}

fn format_500mb_height(e: &WxEntryStruct) -> WeatherData {
    if let Some(l) = e.layers.get(&Layer::MBAR(500))
        && let Some(h) = l.height_msl()
    {
        let dam = h.value_in(Meter) / 10.; // get height in decameters
        return WeatherData {
            title: "500mb Hght".into(),
            text: format!("{:.0}0m", dam),
            style: style_500mb_height(dam),
        };
    }

    WeatherData::none()
}

fn style_250mb_wind(a: Wind) -> String {
    let a = a.speed.value_in(Knots);

    if a > 140. {
        Style::string(&[YellowBg, Black, Bold])
    } else if a > 110. {
        Style::string(&[RedBg, Black, Bold])
    } else if a > 80. {
        Style::string(&[PurpleBg, Black, Bold])
    } else if a > 60. {
        Style::string(&[BlueBg, Black, Bold])
    } else {
        Style::string(&[Bold])
    }
}

fn format_250mb_wind(e: &WxEntryStruct) -> WeatherData {
    if let Some(l) = e.layers.get(&Layer::MBAR(250))
        && let Some(wind) = l.wind()
    {
        return WeatherData {
            title: "250mb".into(),
            text: format!("{:2.0}kts", wind.speed.value_in(Knots)),
            style: style_250mb_wind(wind),
        };
    }

    WeatherData::none()
}

fn format_cape(e: &WxEntryStruct) -> WeatherData {
    if let Some(cape) = e.cape {
        let cape = cape.value_in(Jkg);
        let style = if cape < 200. {
            Style::string(&[])
        } else if cape < 1000. {
            Style::string(&[Bold])
        } else if cape < 2000. {
            Style::string(&[Blue, Bold])
        } else if cape < 3000. {
            Style::string(&[YellowBg, Black, Bold])
        } else if cape < 4000. {
            Style::string(&[RedBg, Black, Bold])
        } else if cape < 5000. {
            Style::string(&[PurpleBg, Black, Bold])
        } else if cape < 6000. {
            Style::string(&[PurpleBg, White, Bold])
        } else {
            Style::string(&[RedBg, White, Bold])
        };

        WeatherData {
            title: "CAPE".into(),
            text: format!("{cape:.0} J/kg"),
            style,
        }
    } else {
        WeatherData::none()
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
enum FlightRules {
    VFR,
    MVFR,
    IFR,
    LIFR,
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
            Self::VFR => Style::string(&[GreenBg, Bold]),
            Self::MVFR => Style::string(&[BlueBg, Bold]),
            Self::IFR => Style::string(&[RedBg, Bold]),
            Self::LIFR => Style::string(&[PurpleBg, Bold]),
        }
    }
}

fn format_flight_rules(e: &WxEntryStruct) -> WeatherData {
    use CloudLayerCoverage::*;
    use FlightRules::*;

    let mut ceiling_height = u32::MAX;

    let near_surface = e.layers.get(&Layer::NearSurface);
    let visibility = near_surface.and_then(|x| x.visibility);

    if let (Some(cover), Some(vis)) = (&e.skycover, visibility) {
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

        let vis = vis.value_in(Mile);

        let fr: FlightRules = match ceiling_height {
            0..=499 => LIFR,
            _ if vis < 1.0 => LIFR,
            500..=999 => IFR,
            _ if vis < 3.0 => IFR,
            1000..=2999 => MVFR,
            _ if vis < 5.0 => MVFR,
            _ => VFR,
        };

        WeatherData {
            title: "".into(),
            text: fr.to_string(),
            style: fr.style(),
        }
    } else {
        WeatherData::none()
    }
}

const COLUMN_WIDTH: usize = 80;

use crate::config::WxParams;

pub fn station_line(
    prelude: &str,
    e: &WxEntryStruct,
    parameters: &Vec<WxParams>,
    indoor: bool,
    db: &BTreeMap<DateTime<Utc>, WxStructDeserialized>,
) -> Result<String, String> {
    let mut data_vec: Vec<WeatherData> = vec![];

    let mut total_string = String::new();

    let (dewpoint, rh) = format_dewpoint(e);

    for p in parameters {
        match p {
            WxParams::ApparentTemp => data_vec.push(format_apparent_temp(e)),
            WxParams::Cape => data_vec.push(format_cape(e)),
            WxParams::Cloud => data_vec.push(format_cloud(e)),
            WxParams::Dewpoint => data_vec.push(dewpoint.clone()),
            WxParams::FlightRules => data_vec.push(format_flight_rules(e)),
            WxParams::Height500mb => data_vec.push(format_500mb_height(e)),
            WxParams::Metar => {} // METARs are dealt with separately at the end
            WxParams::Pressure => data_vec.push(format_pressure(e, db)),
            WxParams::RelativeHumidity => data_vec.push(rh.clone()),
            WxParams::Temperature => data_vec.push(format_temp(e, indoor, db)),
            WxParams::Visibility => data_vec.push(format_visibility(e)),
            WxParams::Wind => data_vec.push(format_wind(e)),
            WxParams::Wind250mb => data_vec.push(format_250mb_wind(e)),
            WxParams::WxCode => data_vec.push(format_wx(e.wx_codes.clone())),
        }
    }

    total_string.push_str(prelude);

    let mut line_length = total_string.len();

    for data in data_vec {
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

    if parameters.contains(&WxParams::Metar) {
        total_string.push_str(&format_metar(e));
    }

    total_string.push('\n');

    Ok(total_string)
}
