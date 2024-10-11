use core::fmt;
use std::{fmt::Display, process, time::Duration};

use chrono::Utc;
use chrono_tz::{OffsetName, Tz};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use dirs::home_dir;
use lazy_static::lazy_static;
use types::{OpenStreetMapPlace, Places};
use tzf_rs::Finder;

mod store;
mod types;

#[derive(Parser)]
#[command(name = "ruhr")]
#[command(version = "0.3.0")]
#[command(about = "A command line world clock", long_about = None)]
struct Cli {
    /// The place name, or alias for a place, that you intend to search for.
    /// If a place with this name or alias exists locally, tht value will be returned, otherwise
    /// it will send a request to the Nominatim database
    #[arg(index = 1)]
    place: Vec<String>,
    /// This will include all information about the given place
    #[arg(short, long)]
    verbose: bool,
    /// Set an alias for a given place
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

impl Display for RuhrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuhrError::NetworkError(e) => {
                if e.is_connect() {
                    write! {f, "Connection refused"}
                } else {
                    let status_code = e.status().unwrap_or_default();
                    write!(f, "Network Error with status code {}", status_code)
                }
            }
            RuhrError::DatabaseError(_) => {
                write!(f, "DatabaseError")
            }
        }
    }
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
                // place alias is either given as an argument or defaulted to the search string
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
            Err(e) => {
                println!("{}", e);
                process::exit(1)
            }
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

async fn fetch_places(search: &String) -> Result<OpenStreetMapPlace, RuhrError> {
    println!(
        "Searching Nominatum database for places matching '{}'",
        search
    );
    let query_string = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=jsonv2",
        search
    );
    let resp = reqwest::Client::new()
        .get(query_string)
        .header("User-Agent", "ruhr/0.2.5")
        .header("Accept-Language", "en-US")
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match resp {
        Ok(res) => {
            let places: Places = res.json().await.unwrap();
            let mut options: Vec<String> =
                places.iter().map(|p| p.display_name.to_string()).collect();
            options.push(String::from("Cancel"));
            let selection_index = Select::with_theme(&ColorfulTheme::default())
                .items(&options)
                .default(0)
                .interact()
                .unwrap();
            let selection = options.get(selection_index).unwrap();
            let place = places
                .into_iter()
                .find(|place| place.display_name == *selection)
                .unwrap_or_else(|| {
                    println!("Could not find place");
                    process::exit(1);
                });
            Ok(place)
        }
        Err(e) => Err(RuhrError::NetworkError(e)),
    }
}
