use std::time::Duration;

use chrono::Utc;
use chrono_tz::{OffsetName, Tz};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use dirs::home_dir;
use lazy_static::lazy_static;
use reqwest::Request;
use types::{OpenStreetMapPlace, Places};
use tzf_rs::Finder;

mod store;
mod types;

#[derive(Parser)]
#[command(name = "ruhr")]
#[command(version = "0.1.1")]
#[command(about = "A command line world clock", long_about = None)]
struct Cli {
    #[arg(index = 1)]
    place: Vec<String>,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    alias: Option<String>,
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

    let place = match store.get_place(&args.place.join(" ")) {
        Ok(place) => {
            if args.alias.is_some() {
                store
                    .add_alias(&args.alias.unwrap(), place.id)
                    .expect("Could not create alias");
            }
            Ok(place)
        }
        Err(_) => match fetch_places(&args.place.join("+")).await {
            Ok(result) => {
                let alias = match args.alias {
                    Some(alias) => alias,
                    None => args.place.join(" ").to_owned(),
                };
                let result = result;
                let (lat, lon) = (
                    result.lat.parse().expect("Could not parse latitude"),
                    result.lon.parse().expect("Could not parse longitude"),
                );
                let zone = FINDER.get_tz_name(lon, lat);
                let tz = zone.parse::<Tz>().expect("Could not parse the time zone");
                match store.add_place(&result, tz, alias) {
                    Ok(new_place) => Ok(new_place),
                    Err(e) => Err(RuhrError::DatabaseError(e)),
                }
            }
            Err(e) => Err(RuhrError::NetworkError(e)),
        },
    }?;
    let now = Utc::now().with_timezone(&place.time_zone);
    let (display_name, timezone, format) = match args.verbose {
        true => (
            place.display_name,
            place.time_zone.name(),
            "%Y-%m-%d %H:%M:%S",
        ),
        false => ("".to_string(), now.offset().abbreviation(), "%H:%M"),
    };
    println!("{} {} {}", display_name, now.format(format), timezone);
    Ok(())
}

async fn fetch_places(search: &String) -> Result<OpenStreetMapPlace, reqwest::Error> {
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
        Ok(res) => {
            let places: Places = res.json().await.unwrap();
            let options: Vec<String> = places.iter().map(|p| p.display_name.to_string()).collect();
            let selection_index = Select::with_theme(&ColorfulTheme::default())
                .items(&options)
                .default(0)
                .interact()
                .unwrap();
            let selection = options.get(selection_index).unwrap();
            let place = places
                .into_iter()
                .find(|place| place.display_name == *selection)
                .expect("No place found");
            Ok(place)
        }
        Err(e) => Err(e),
    }
}
