mod api;
mod config;

use api::{call_api, handle_data};
use config::read_config_file;
use std::thread;
use std::time::Duration;

fn main() {
    // Get config
    let conf = read_config_file();

    loop {
        // Call API
        match call_api(conf.station.clone(), conf.weather_type.clone()) {
            Ok(response) => {
                if let Err(e) = handle_data(response, conf.weather_type.clone()) {
                    eprintln!("Error handling data: {}", e);
                }
            }
            Err(e) => eprintln!("Error calling API: {}", e),
        }

        // Sleep for x minutes (stated in config file)
        thread::sleep(Duration::from_secs(conf.interval * 60));
    }
}
