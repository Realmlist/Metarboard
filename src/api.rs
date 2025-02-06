use chrono::Local;
use regex::Regex;
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
    #[serde(rename = "icaoId")]
    station_id: String,
    #[serde(rename = "rawTAF")]
    raw_taf: String,
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

// Function to calculate the VFR color code from a METAR string
fn calculate_vfr_color_code(metar: &str) -> String {
    // Regex for extracting visibility and cloud cover
    let visibility_re = Regex::new(r"(\d{4})(SM|KM)").unwrap();
    let cloud_cover_re = Regex::new(r"([A-Z]{3})(\d{3})").unwrap(); // for clouds (BKN, OVC, etc.)

    // Extract visibility (in statute miles or kilometers)
    let visibility = if let Some(captures) = visibility_re.captures(metar) {
        let value: i32 = captures[1].parse().unwrap();
        let unit = &captures[2];

        match unit {
            "SM" => value,                                    // statute miles
            "KM" => (value as f64 * 0.621371).round() as i32, // convert km to statute miles
            _ => 0,
        }
    } else {
        7 // If no visibility shown, set visib above 6.
    };

    // Extract cloud cover (lowest cloud layer height)
    let cloud_cover_height = if let Some(captures) = cloud_cover_re.captures(metar) {
        captures[2].parse::<i32>().unwrap() * 100
    } else {
        9999 // No cloud cover (clear)
    };

    // Calculate the VFR color code
    if visibility >= 6 && cloud_cover_height > 3000 {
        "{66}".to_string() // VFR (Green)
    } else if visibility >= 3
        && visibility <= 6
        && cloud_cover_height >= 1000
        && cloud_cover_height <= 3000
    {
        "{67}".to_string() // MVFR (Blue)
    } else if visibility >= 1
        && visibility < 3
        && cloud_cover_height >= 500
        && cloud_cover_height < 1000
    {
        "{63}".to_string() // IFR (Red)
    } else if visibility < 1 || cloud_cover_height < 500 {
        "{68}".to_string() // LIFR (Purple)
    } else {
        "{66}".to_string() // Default to VFR
    }
}

fn vfr_string(cloud_cover: &str, vfr_color: String) -> String {
    match cloud_cover {
        "FEW" => vfr_color,
        "SCT" => format!("{}{}", vfr_color, vfr_color),
        "BKN" => format!("{}{}{}", vfr_color, vfr_color, vfr_color),
        "OVC" | "OVX" | "VV" => format!("{}{}{}{}", vfr_color, vfr_color, vfr_color, vfr_color),
        _ => vfr_color,
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

            let vfr_color = calculate_vfr_color_code(&data.raw_metar);

            // Clouds
            let vfr = if let Some(cloud) = data.clouds.as_ref().and_then(|c| c.first()) {
                let cloud_cover = &cloud.cover;
                vfr_string(cloud_cover, vfr_color)
            } else {
                "-".to_string()
            };

            // First line on vesta board
            println!(
                "{} VFR{} MIL{}JT{}\\n{}",
                weather_type_formatted, vfr, mil_color, julia_time, data.raw_metar
            );
        } else {
            println!("No METAR data available.");
        }
    } else if weather_type == "taf" {
        let datas: Vec<TafData> = serde_json::from_str(&response)?;
        if let Some(data) = datas.first() {
            let raw_taf_display = if !data.raw_taf.is_empty() {
                data.raw_taf.clone()
            } else {
                format!("{} {}", weather_type_formatted, &data.station_id)
            };

            // First line on vesta board
            println!("{} {}", julia_time, raw_taf_display);
        } else {
            println!(
                "{} {}\nInvalid response from API.",
                julia_time, weather_type_formatted
            );
        }
    }

    Ok(())
}
