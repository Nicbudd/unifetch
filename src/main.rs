mod common;
mod solarlunar;
mod earthquake;
mod random;
mod wx; 
mod updates; 
mod tides;
mod config;

use std::env;

use clap::Parser;
use chrono::{Local, Utc};
use tokio;


// HEAD MATTER -----------------------------------------------------------------

fn header() {
    let utc_now = Utc::now().format("(%H:%MZ)");
    let local_now = Local::now().format("%a %Y-%b-%d @ %I:%M:%S%p");
    let r = rand::random::<u32>();

    println!("{}unifetch v{} {local_now} {utc_now} - {r:08X}", common::terminal_line('-'), env!("CARGO_PKG_VERSION"));
}



#[derive(Parser, Debug)]
pub struct Args {
    /// Reimplements all default values, equivalent to -rsweq. If no other flags are selected this is enabled by default.
    #[arg(short, long)]
    default: bool,

    /// Generates various random values (sync).
    #[arg(short, long)]
    random: bool,

    /// Sun/moon set/rise times (async).
    #[arg(short, long)]
    solar_lunar: bool,
    
    /// Current weather conditions (async).
    #[arg(short = 'w', long)]
    current_conditions: bool,

    /// Provides forecast for the home location (async).
    #[arg(short = 'F', long)]
    forecast: bool,    

    /// Grabs data about teleconnections (eg: El Nino Southern Oscillation, NAO) (async).
    #[arg(short = 'e', long)]
    teleconnections: bool,

    /// Latest earthquakes in "tallest skyscrapers" chronological order, and earthquakes that occured nearby (async).
    #[arg(short = 'q', long = "quakes")]
    earthquakes: bool,

    /// Tidal predictions from around the area (async).
    #[arg(short = 't')]
    tides: bool,

    /// Calendar date, times around the world (sync).
    #[arg(short = 'd')]
    datetime: bool,

    /// Disables header
    #[arg(short = 'H', long)]
    disable_header: bool,

    /// Disables the update notification section. Update checking is asynchronous
    #[arg(short = 'u', long)]
    disable_update_notif: bool,

    /// Add up to 2 v's to add details. Currently only for wx data.
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,
}


#[tokio::main] 
async fn main() {

    // parse args
    let mut args = Args::parse();

    // modify args
    if args.verbose > 2 {
        args.verbose = 2;
    }

    // set args to default if there are no other modules explicitly enabled.
    args.default |= !(args.random || args.solar_lunar || args.current_conditions 
        || args.forecast || args.teleconnections || args.earthquakes || 
        args.tides);

    // open config file
    let config_opt = config::read_config_file(&args);

    if let Err(e) = config_opt {
        println!("{}CONFIG FILE PARSING ERROR{}\n{e:?}", 
            common::Style::error(), common::TermStyle::Reset);
        return;
    }

    let config = config_opt.unwrap();

    // actually start doing stuff

    if !args.disable_header {
        header();
    }
    
    // sync functions
    if config.enabled_modules.contains(&config::Modules::Random) {
        random::random_section();
    }

    // async functions
    tokio::join!(
        updates::updates(&config),

        solarlunar::solar_lunar(&config), 

        wx::weather::current_conditions(&config),
        wx::forecast::forecast(&config),
        wx::tele::teleconnections(&config),

        // time_and_date();

        // obscure calendars/clocks

        // Scrape GasBuddy
        // gas_prices();

        // forecast_analysis();
        // climatology();

        // dow, nasdaq, s&p, bitcoin, eth, usdt
        // apple, microsoft, nvidia, google, amazon, meta
        // exchange rates (CAD, JPY, EUR, RUB)
        // stock_market();

        // cpu temps, hardware utilization

        // kernel/os info?

        // astrology?

        tides::tides(&config),
        
        earthquake::earthquakes(&config)

    );



}



