use std::fmt;

// GENERAL ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum TermStyle {
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
    pub fn str(&self) -> &str {
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
pub struct Style {
    // styles: Vec<TermStyle>
}

impl Style {
    pub fn new(styles: &[TermStyle]) -> String {
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

    pub fn error() -> String {
        Style::new(&[Red, Bold])
    }
}


pub fn terminal_line(c: char) -> String {
    let mut s = String::new();
    for _ in 0..80 {
        s.push(c);
    }
    s.push('\n');
    s
}

// HELPER FUNCTIONS ------------------------------------------------------------

pub fn title(s: &str) -> String {
    format!("\n{:-^80}\n", s)
}

pub async fn parse_request_loose_json(w: Result<reqwest::Response, reqwest::Error>) -> Result<serde_json::Value, String> {    
    let r = w.map_err(|e| e.to_string())?;
    let t = r.text().await.map_err(|e| e.to_string())?;
    // dbg!(&t);
    let j = serde_json::from_str(&t).map_err(|e| e.to_string())?;
    Ok(j)
}



// CONFIG ----------------------------------------------------------------------

// TODO: Do not hard code this.
// const COORDS: (f64, f64) = DURHAM_COORDS;
// const DURHAM_COORDS: (f64, f64) = (43.13, -70.92);

pub fn coords_str(coords: (f32, f32)) -> String {
    format!("{:.2},{:.2}", coords.0, coords.1)
}

