use crate::config::{Config, Modules};

fn datetime_handler(config: &Config) -> Result<String> {
    
}


pub fn datetime(config: &Config) {
    if !config.enabled_modules.contains(&Modules::DateTime) {
        return;
    }

    match datetime_handler(config) {
        Ok(s) => {println!("{}", s)}
        Err(e) => {println!("{}{}", common::title("DATETIME"), e)},
    }
}