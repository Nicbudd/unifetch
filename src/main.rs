mod common;
mod astrological;
mod earthquake;
mod random;
mod wx; 

use clap::Parser;
use chrono::{Local, Utc};
use tokio;


// HEAD MATTER -----------------------------------------------------------------

fn head_matter() {
    let utc_now = Utc::now().format("(%H:%MZ)");
    let local_now = Local::now().format("%a %Y-%b-%d @ %I:%M:%S%p");
    let r = rand::random::<u32>();

    println!("{}unifetch v{} {local_now} {utc_now} - {r:08X}", common::terminal_line('-'), env!("CARGO_PKG_VERSION"));
}



#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    single_line: bool

}


#[tokio::main] 
async fn main() {

    let args = Args::parse();

    if args.single_line == false {
        
        head_matter();
        random::random_section();
    
        tokio::join!(
            astrological::solar_lunar(), 
            wx::conditions::current_conditions(),
            wx::forecast::forecast(),
    
            // time_and_date();
    
            // forecast_analysis();
            // climatology();
            // stock_market();
    
            // on hold, seavey island API doesn't work
            //https://api.tidesandcurrents.noaa.gov/api/prod/datagetter?date=latest&station=8419870&product=predictions&datum=STND&time_zone=gmt&interval=hilo&units=english&format=json
            // tides();
            
            
            // teleconnections();
    
            earthquake::earthquakes()
    
        );
    }


}



