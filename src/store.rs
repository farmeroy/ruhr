use chrono_tz::Tz;
use rusqlite::{params, Connection, Result};

use crate::types::{OpenStreetMapPlace, PlaceWithTimeZone};

#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(db_url: &str) -> Result<Self> {
        let conn = Connection::open(db_url)?;
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS place (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                time_zone_id INTEGER,
                FOREIGN KEY(time_zone_id) REFERENCES time_zone(id)
            )
            ",
            (),
        )?;
        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS time_zone (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
            )
            ",
            (),
        )?;
        Ok(Store { conn })
    }
    /// Add a new timezone
    fn add_time_zone(&self, tz: &Tz) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR IGNORE INTO time_zone (name) VALUES (?1)",
            params![tz.name()],
        )?;
        // Retrieve the ID of the inserted or existing timezone
        let time_zone_id: i64 = self.conn.query_row(
            "SELECT id FROM time_zone WHERE name = ?1",
            params![tz.name()],
            |row| row.get(0),
        )?;

        Ok(time_zone_id)
    }
    /// Add a new place
    pub fn add_place(
        &self,
        place: &OpenStreetMapPlace,
        tz: Tz,
    ) -> Result<PlaceWithTimeZone, rusqlite::Error> {
        let time_zone_id = self.add_time_zone(&tz)?;
        self.conn.execute(
            "
            INSERT INTO place (name, display_name, time_zone_id)
            VALUES (?1, ?2, ?3)
            ",
            params![place.name, place.display_name, time_zone_id],
        )?;
        Ok(PlaceWithTimeZone {
            name: place.name.to_owned(),
            display_name: place.display_name.to_owned(),
            time_zone: tz,
        })
    }
    /// Get a place
    pub fn get_place(&self, name: &String) -> Result<PlaceWithTimeZone, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, display_name, time_zone_id FROM place WHERE name = ?1")?;
        let place = stmt.query_row(params![name], |row| {
            let time_zone_id: i64 = row.get("time_zone_id")?;

            // Retrieve the timezone name using the time_zone_id
            let time_zone_name: String = self.conn.query_row(
                "SELECT name FROM time_zone WHERE id = ?1",
                params![time_zone_id],
                |row| row.get(0),
            )?;

            let tz = time_zone_name.parse::<Tz>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            print!("{:?}", row);
            Ok(PlaceWithTimeZone {
                name: row.get("name")?,
                display_name: row.get("display_name")?,
                time_zone: tz,
            })
        });
        place
    }
}
