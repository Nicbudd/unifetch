use crate::{
    common,
    config::{Config, Modules},
};
use chrono::{self, DateTime, Local, NaiveDate, Offset, Timelike, Utc};
use chrono_tz::Etc::GMTPlus1;

fn datetime_handler(config: &Config) -> Result<String, String> {
    // let current_timezone = chrono::Local::;
    let local = Local::now();
    let utc = local.to_utc();
    let mut s = common::title("DATETIME");

    s.push_str(&format!(
        "Local: {}\n\n",
        local.format("%a %v %I:%M:%S%.9f %p %z")
    ));

    for tz in &config.datetime.timezones {
        let adjusted = utc.with_timezone(&tz.tz);
        let fmt_time = adjusted.format("%a %I:%M %p %z");
        let name = tz.name.clone().unwrap_or(tz.tz.name().to_string());
        let diff = (local.offset().local_minus_utc() - adjusted.offset().fix().local_minus_utc())
            as f32
            / (60. * 60.);
        let later_earlier = if diff > 0. { "-" } else { "+" };
        let display_diff = ((diff * 100.).round_ties_even().abs()) / 100.;

        s.push_str(&format!(
            "{name}: {fmt_time} (Local{later_earlier}{display_diff}h)\n",
        ));
        // s.push_str(&format!(
        //     "\t{name} is {display_diff} hours {later_earlier} in the day\n",
        // ));
    }
    if config.datetime.timestamps {
        s.push('\n');
        s.push_str(&format!("Unix: {}\n", utc.timestamp()));
        s.push_str(&format!("ISO 8601: {}\n", utc.to_rfc3339()));
    }
    if config.datetime.beat_time {
        s.push('\n');
        s.push_str(&format!(".beat time: {}\n", beat_time(utc)));
    }
    if config.datetime.mayan {
        s.push('\n');
        s.push_str(&mayan_calendar(local).to_string());
    }

    Ok(s)
}

fn beat_time(now: DateTime<Utc>) -> String {
    let cet = now.with_timezone(&GMTPlus1);
    let secs = cet.num_seconds_from_midnight(); // cant figure out how to account for leapseconds. Grrr.
    let beats = secs as f32 / 86.4;
    format!("@{beats:03}")
}

fn mayan_calendar(now: DateTime<Local>) -> String {
    let date = now.date_naive();
    // use Dec 21 2012 as the epoch because funny\
    let epoch = NaiveDate::from_ymd_opt(2012, 12, 21).unwrap();
    let days_since_epoch = (date - epoch).num_days();

    // Tzolk'in
    // Dec 21 2012 is 4 Ajaw
    const EPOCH_DAY_NUM: i64 = 4;
    const EPOCH_NAME_IDX: usize = 19; // Ajaw
    let tzolkin_day_num = ((EPOCH_DAY_NUM + days_since_epoch - 1) % 13) + 1; // 4 (Ajaw), -1 and +1 to number the days 1-13 instead of 0-12
    let name_idx = (EPOCH_NAME_IDX + days_since_epoch as usize) % 20;
    let tzolkin_name = TZOLKIN_NAMES[name_idx];

    // Haab'
    // Dec 21 2012 is 3 K'ank'in
    const EPOCH_HAAB_DAY: i64 = 3;
    const EPOCH_HAAB_MONTH_IDX: usize = 13;
    let day_in_mayan_year =
        ((EPOCH_HAAB_MONTH_IDX as i64 * 20 + EPOCH_HAAB_DAY) + days_since_epoch) % 365;
    let haab_day = ((day_in_mayan_year - 1) % 20) + 1;
    let haab_month_idx = day_in_mayan_year as usize / 20;
    let haab_month = HAAB_MONTHS[haab_month_idx];

    // Long Count
    // Dec 21 2012 is famously 13.0.0.0.0
    let baktun = 13 + (days_since_epoch / 144000);
    let katun = days_since_epoch % 144000 / 7200;
    let tun = days_since_epoch % 7200 / 360;
    let uinal = days_since_epoch % 360 / 20;
    let kin = days_since_epoch % 20;

    format!(
        "Tzolkin: {tzolkin_day_num} {tzolkin_name}\n\
         Haab: {haab_day} {haab_month}\n\
         Long Count: {baktun}.{katun}.{tun}.{uinal}.{kin}"
    )
}

pub const TZOLKIN_NAMES: [&str; 20] = [
    "Imix", "Ik'", "Ak'b'al", "K'an", "Chikchan", "Kimi", "Manik'", "Lamat", "Muluk", "Ok",
    "Chuwen", "Eb'", "B'en", "Ix", "Men", "Kib'", "Kab'an", "Etz'nab", "Kawak", "Ajaw",
];

pub const HAAB_MONTHS: [&str; 19] = [
    "Pop",
    "Wo'",
    "Sip",
    "Sotz'",
    "Sek",
    "Xul",
    "Yaxk'in",
    "Mol",
    "Ch'en",
    "Yax",
    "Sak'",
    "Keh",
    "Mak",
    "K'ank'in",
    "Muwan",
    "Pax",
    "K'ayab'",
    "Kumk'u",
    "Wayeb' (unlucky days)",
];

pub fn datetime(config: &Config) {
    if !config.enabled_modules.contains(&Modules::DateTime) {
        return;
    }

    match datetime_handler(config) {
        Ok(s) => {
            println!("{}", s)
        }
        Err(e) => {
            println!("{}{}", common::title("DATETIME"), e)
        }
    }
}
