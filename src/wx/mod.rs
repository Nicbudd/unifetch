pub mod forecast;
pub mod conditions;
pub mod tele;

use crate::common;
use common::TermStyle::*;
use common::Style;


use std::collections::BTreeMap;
use std::fmt;
use chrono::{Utc, DateTime};

use wxer_lib::*;



// WEATHER DATA ------------------------------------------------------------------------------------------------------

#[derive(Debug)]
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
    None
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





// TREND --------------------------------------------------------------------------------------------------------------

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







// FORMATTERS ------------------------------------------------------------------------------------------------------

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

fn style_500mb_height(h: f32) -> String {
    if h > 570.0 {
        Style::new(&[YellowBg, Black, Bold])
    } else if h > 585.0 {
        Style::new(&[RedBg, Bold])
    } else if h < 530.0 {
        Style::new(&[BlueBg, Black, Bold])
    } else if h < 515.0 {
        Style::new(&[PurpleBg, Bold])
    } else {
        Style::new(&[Bold])
    }
}

fn format_500mb_height(e: &WxEntry) -> WeatherData {
    if let Some(l) = e.layers.get(&Layer::MBAR(500)) {
        if let Some(h) = l.height_msl {
            let dam = h / 10.; // get height in decameters
            return WeatherData{title: "500mb Hght".into(), text: format!("{:.0}", dam), style: style_500mb_height(dam)}
        }
    }

    return WeatherData::none();
}

fn style_250mb_wind(a: f32) -> String {
    if a > 140. {
        Style::new(&[YellowBg, Black, Bold])
    } else if a > 110. {
        Style::new(&[RedBg, Black, Bold])
    } else if a > 80. {
        Style::new(&[PurpleBg, Black, Bold])
    } else if a > 60. {
        Style::new(&[BlueBg, Black, Bold])
    } else {
        Style::new(&[Bold])
    }
}

fn format_250mb_wind(e: &WxEntry) -> WeatherData {
    if let Some(l) = e.layers.get(&Layer::MBAR(250)) {
        if let Some(wind) = l.wind_speed {
            return WeatherData{title: "250mb".into(), text: format!("{:2.0}kts", wind), style: style_250mb_wind(wind)}
        }
    }

    return WeatherData::none();
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







pub fn station_line(prelude: &str, e: &WxEntry, indoor: bool,
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
      string_vec.push(format_250mb_wind(e));
      string_vec.push(format_cloud(e));
      string_vec.push(format_500mb_height(e));
  
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