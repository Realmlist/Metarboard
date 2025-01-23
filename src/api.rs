use chrono::Local;
use reqwest::blocking::get;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Cloud {
    cover: String,
}

#[derive(Deserialize, Debug)]
struct MetarData {
    #[serde(rename = "rawOb")]
    raw_metar: String,
    clouds: Option<Vec<Cloud>>,
}

#[derive(Deserialize, Debug)]
struct TafData {
    #[serde(rename = "rawTAF")]
    raw_taf: String,
    fcsts: Vec<Forecasts>,
}

#[derive(Deserialize, Debug)]
struct Forecasts {
    // We only need clouds for now
    clouds: Option<Vec<Cloud>>,
}

pub fn call_api(station: String, weather_type: String) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://aviationweather.gov/api/data/{}?ids={}&format=json",
        weather_type, station,
    );

    let response = get(&url)?.text()?;
    Ok(response)
}

fn parse_mil_color(raw_data: &str) -> String {
    if raw_data.contains("RED") {
        "{63}".to_string()
    } else if raw_data.contains("AMB") {
        "{64}".to_string()
    } else if raw_data.contains("YLO") {
        "{65}".to_string()
    } else if raw_data.contains("GRN") {
        "{66}".to_string()
    } else if raw_data.contains("WHT") {
        "{71}".to_string()
    } else if raw_data.contains("BLU") {
        "{67}".to_string()
    } else {
        " ".to_string()
    }
}

pub fn handle_data(response: String, weather_type: String) -> Result<(), Box<dyn Error>> {
    let weather_type_formatted = match weather_type.as_str() {
        "metar" => "MET",
        "taf" => "TAF",
        _ => "Unknown",
    };

    let julia_time = Local::now().format("%H%M").to_string();

    if weather_type == "metar" {
        let datas: Vec<MetarData> = serde_json::from_str(&response)?;
        if let Some(data) = datas.first() {
            // Parse MIL color
            let mil_color = parse_mil_color(&data.raw_metar);

            // Clouds
            if let Some(cloud) = data.clouds.as_ref().and_then(|c| c.first()) {
                let cloud_cover = &cloud.cover;
            }

            // First line on vesta board
            println!(
                "{} <vfr> MIL{} JT{}",
                weather_type_formatted, mil_color, julia_time
            );

            // Second line on vesta board
            println!("{}", data.raw_metar);
        } else {
            println!("No METAR data available.");
        }
    } else if weather_type == "taf" {
        let datas: Vec<TafData> = serde_json::from_str(&response)?;
        if let Some(data) = datas.first() {
            // Parse MIL color
            let mil_color = parse_mil_color(&data.raw_taf);

            // Clouds
            if let Some(forecast) = data.fcsts.first() {
                if let Some(cloud) = forecast.clouds.as_ref().and_then(|c| c.first()) {
                    let cloud_cover = &cloud.cover;
                }
            }

            // First line on vesta board
            println!(
                "{} <vfr> MIL{} JT{}",
                weather_type_formatted, mil_color, julia_time
            );

            // Second line on vesta board
            println!("{}", data.raw_taf);
        } else {
            println!("No TAF data available.");
        }
    }

    Ok(())
}
