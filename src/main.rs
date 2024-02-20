mod common;
mod astrological;
mod earthquake;
mod random;
mod wx; 

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
    /// Disables header.
    #[arg(short = 'H', long)]
    disable_header: bool,

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
}


#[tokio::main] 
async fn main() {

    // parse args
    let mut args = Args::parse();

    let cli_args: Vec<_> = std::env::args().collect();

    // dbg!(&cli_args);

    if args.default || !(args.random || args.solar_lunar || 
        args.current_conditions || args.forecast || args.teleconnections || 
        args.earthquakes) {
            
            args.random = true;
            args.solar_lunar = true;
            args.current_conditions = true;
            args.forecast = true;
            args.teleconnections = true;
            args.earthquakes = true;
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
        astrological::solar_lunar(&args), 
        wx::conditions::current_conditions(&args),
        wx::forecast::forecast(&args),

        // time_and_date();

        // forecast_analysis();
        // climatology();
        // stock_market();

        // on hold, seavey island API doesn't work
        //https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?date=latest&station=8419870&product=predictions&datum=STND&time_zone=gmt&interval=hilo&units=english&format=json
        // tides();
        
        
        wx::tele::teleconnections(&args),

        earthquake::earthquakes(&args)

    );



}



