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

    /// Disables header
    #[arg(short = 'H', long)]
    disable_header: bool,

    /// Disables the update notification section. Update checking is asynchronous
    #[arg(short = 'u', long)]
    disable_update_notif: bool,
}


#[tokio::main] 
async fn main() {

    // parse args
    let args = Args::parse();

    // open config file
    let config = config::read_config_file(&args).unwrap();

    // actually start doing stuff

    // sync functions
    if !args.disable_header {
        header();
    }

    if args.random {
        random::random_section();
    }

    // async functions
    tokio::join!(
        updates::updates(&config),

        solarlunar::solar_lunar(&config), 

        wx::conditions::current_conditions(&config),
        wx::forecast::forecast(&config),
        wx::tele::teleconnections(&config),

        // time_and_date();

        // obscure calendars/clocks

        // Scrape GasBuddy
        // gas_prices();

        // forecast_analysis();
        // climatology();
        // stock_market();

        // cpu temps, hardware utilization

        // kernel info?

        // astrology?

        tides::tides(&config),
        
        earthquake::earthquakes(&config)

    );



}



