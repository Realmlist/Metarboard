mod api;
mod config;

use api::call_api;
use config::read_config_file;
use std::thread;
use std::time::Duration;

fn main() {
    // Get config
    let conf = read_config_file();

    loop {
        // Call API
        if let Err(e) = call_api(conf.station.clone()) {
            eprintln!("Error calling API: {}", e);
        }

        // Sleep for x minutes (stated in config file)
        thread::sleep(Duration::from_secs(conf.interval * 60));
    }
}
