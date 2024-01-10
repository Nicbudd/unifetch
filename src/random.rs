use crate::common;
use common::TermStyle::*;
use common::Style;

use rand::{thread_rng, rngs::ThreadRng};
use rand::Rng;

use chrono::NaiveDate;


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

pub fn random_section() {

    let mut s = common::title("RANDOM");

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
