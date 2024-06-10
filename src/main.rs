use std::time::Duration;

use chrono::Utc;
use chrono_tz::Tz;
use clap::Parser;
use dirs::home_dir;
use lazy_static::lazy_static;
use types::Places;
use tzf_rs::Finder;

mod store;
mod types;

#[derive(Parser)]
#[command(name = "ruhr")]
#[command(version = "0.1.1")]
#[command(about = "A command line world clock", long_about = None)]
struct Cli {
    #[arg(index = 1)]
    place: String,
}

lazy_static! {
    static ref FINDER: Finder = Finder::new();
}

#[derive(Debug)]
pub enum RuhrError {
    NetworkError(reqwest::Error),
    DatabaseError(rusqlite::Error),
}

#[tokio::main]
async fn main() -> Result<(), RuhrError> {
    let args = Cli::parse();
    let home_dir = home_dir().unwrap();
    let store =
        store::Store::new(format!("{}/.ruhr.db3", home_dir.to_string_lossy()).as_str()).unwrap();

    let place = match store.get_place(&args.place) {
        Ok(place) => Ok(place),
        Err(_) => match fetch_places(&args.place.to_string()).await {
            Ok(result) => {
                let result = result.first().expect("No place with that name");
                let (lat, lon) = (
                    result.lat.parse().expect("Could not parse latitude"),
                    result.lon.parse().expect("Could not parse longitude"),
                );
                let zone = FINDER.get_tz_name(lon, lat);
                let tz = zone.parse::<Tz>().expect("Could not parse the time zone");
                match store.add_place(result, tz) {
                    Ok(new_place) => Ok(new_place),
                    Err(e) => Err(RuhrError::DatabaseError(e)),
                }
            }
            Err(e) => Err(RuhrError::NetworkError(e)),
        },
    }?;
    let now = Utc::now().with_timezone(&place.time_zone).format("%H:%M");
    println!("{now}");
    Ok(())
}

async fn fetch_places(search: &String) -> Result<Places, reqwest::Error> {
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
