mod common;
mod solarlunar;
mod earthquake;
mod random;
mod wx; 
mod updates; 
mod tides;

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
    let mut args = Args::parse();

    // dbg!(&cli_args);

    if args.default || !(args.random || args.solar_lunar || 
        args.current_conditions || args.forecast || args.teleconnections || 
        args.earthquakes || args.tides) {
            
            args.random = true;
            args.solar_lunar = true;
            args.current_conditions = true;
            args.teleconnections = true;
            args.earthquakes = true;
            args.tides = true;
    } 


    // actually start doing stuff

    // asyncronous functions
    if !args.disable_header {
        header();
    }

    if args.random {
        random::random_section();
    }

    tokio::join!(
        updates::updates(&args),

        solarlunar::solar_lunar(&args), 

        wx::conditions::current_conditions(&args),
        wx::forecast::forecast(&args),
        wx::tele::teleconnections(&args),

        // time_and_date();

        // Scrape GasBuddy
        // gas_prices();
        // forecast_analysis();
        // climatology();
        // stock_market();

        tides::tides(&args),
        
        earthquake::earthquakes(&args)

    );



}



