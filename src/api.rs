use reqwest::blocking::get;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Cloud {
    cover: String,
    base: i32,
}

#[derive(Deserialize, Debug)]
struct Metar {
    #[serde(rename = "metar_id")]
    metar_id: i64,
    #[serde(rename = "icaoId")]
    icao_id: String,
    #[serde(rename = "receiptTime")]
    receipt_time: String,
    #[serde(rename = "obsTime")]
    obs_time: i64,
    #[serde(rename = "reportTime")]
    report_time: String,
    temp: Option<f32>,
    dewp: Option<f32>,
    wdir: Option<i32>,
    wspd: Option<i32>,
    wgst: Option<i32>,
    visib: Option<String>,
    altim: Option<i32>,
    slp: Option<f32>,
    #[serde(rename = "qcField")]
    qc_field: Option<i32>,
    #[serde(rename = "wxString")]
    wx_string: Option<String>,
    #[serde(rename = "presTend")]
    pres_tend: Option<String>,
    #[serde(rename = "maxT")]
    max_t: Option<f32>,
    #[serde(rename = "minT")]
    min_t: Option<f32>,
    #[serde(rename = "maxT24")]
    max_t24: Option<f32>,
    #[serde(rename = "minT24")]
    min_t24: Option<f32>,
    precip: Option<f32>,
    pcp3hr: Option<f32>,
    pcp6hr: Option<f32>,
    pcp24hr: Option<f32>,
    snow: Option<f32>,
    #[serde(rename = "vertVis")]
    vert_vis: Option<i32>,
    #[serde(rename = "metarType")]
    metar_type: String,
    #[serde(rename = "rawOb")]
    raw_ob: String,
    #[serde(rename = "mostRecent")]
    most_recent: Option<i32>,
    lat: Option<f32>,
    lon: Option<f32>,
    elev: Option<i32>,
    prior: Option<i32>,
    name: String,
    clouds: Option<Vec<Cloud>>,
}
pub fn call_api(station: String) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "https://aviationweather.gov/api/data/metar?ids={}&format=json",
        station
    );

    let response = get(&url)?.text()?;

    let metars: Vec<Metar> = serde_json::from_str(&response)?;

    // Examples:
    if let Some(metar) = metars.first() {
        // Regular values
        println!("Station Name: {}", metar.name);
        println!("Temperature: {}°C", metar.temp.unwrap_or(0.0)); // On Option<> I need unwrap_or() incase value is null.
        println!("Dewpoint: {}", metar.dewp.unwrap_or(0.0));
        println!("Pressure (altimeter): {} mb", metar.altim.unwrap_or(0));
        if let Some(wdir) = metar.wdir {
            println!(
                "Winds: {}° ({}) at {} knots",
                wdir,
                degrees_to_direction(wdir),
                metar.wspd.unwrap_or(0)
            );
        }
        println!("Visibility: {} sm", metar.visib.as_deref().unwrap_or("N/A"));
        println!("Ceiling: {}000 feet", metar.elev.unwrap_or(0));

        // Clouds
        if let Some(clouds) = &metar.clouds {
            for cloud in clouds {
                println!("{} clouds at {} feet", cloud.cover, cloud.base);
            }
        } else {
            println!("No cloud information available.");
        }
    } else {
        println!("No METAR data available.");
    }

    Ok(())
}

fn degrees_to_direction(degrees: i32) -> &'static str {
    match degrees {
        0..=11 | 349..=360 => "N",
        12..=33 => "NNE",
        34..=56 => "NE",
        57..=78 => "ENE",
        79..=101 => "E",
        102..=123 => "ESE",
        124..=146 => "SE",
        147..=168 => "SSE",
        169..=191 => "S",
        192..=213 => "SSW",
        214..=236 => "SW",
        237..=258 => "WSW",
        259..=281 => "W",
        282..=303 => "WNW",
        304..=326 => "NW",
        327..=348 => "NNW",
        _ => "Unknown",
    }
}
