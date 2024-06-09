use std::time::Duration;

use chrono::Utc;
use chrono_tz::Tz;
use clap::Parser;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tzf_rs::Finder;

#[derive(Parser)]
#[command(name = "ruhr")]
#[command(version = "0.1")]
#[command(about = "A command line world clock", long_about = None)]
struct Cli {
    #[arg(index = 1)]
    place: String,
}
pub type Places = Vec<Place>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// This is based on the Open Streetmap JsonV2 response
pub struct Place {
    pub place_id: i64,
    pub licence: String,
    pub osm_type: String,
    pub osm_id: i64,
    pub lat: String, // this should be parsed as f64
    pub lon: String, // this should be parsed as f64
    pub category: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub place_rank: i64,
    pub importance: f64,
    pub addresstype: String,
    pub name: String,
    pub display_name: String,
    pub boundingbox: Vec<String>,
    pub icon: Option<String>,
    pub namedetails: Option<Vec<String>>,
}

lazy_static! {
    static ref FINDER: Finder = Finder::new();
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match fetch_places(args.place.to_string()).await {
        Ok(resp) => {
            if let Some(place) = resp.first() {
                // The lat and lon should be valid f64
                let (lat, lon) = (place.lat.parse().unwrap(), place.lon.parse().unwrap());
                let zone = FINDER.get_tz_name(lon, lat);
                // if the timezone can't parse we can panic
                let tz = zone.parse::<Tz>().unwrap();
                // create a new date time with that type
                let now = Utc::now().with_timezone(&tz).format("%H:%M");
                println!("It is {} in {}", now.to_string(), place.display_name)
            } else {
                println!("Could not locate such a place")
            }
        }
        Err(e) => {
            println!("There was a problem fetching the place: {}", e);
        }
    }
}

async fn fetch_places(search: String) -> Result<Places, reqwest::Error> {
    let query_string = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2",
        search
    );
    let resp = reqwest::Client::new()
        .get(query_string)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0",
        )
        .header("Referer", "localhost")
        .header("Accept-Language", "en-US")
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match resp {
        Ok(res) => res.json().await,
        Err(e) => Err(e),
    }
}
