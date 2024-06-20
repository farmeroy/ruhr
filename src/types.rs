pub type Places = Vec<OpenStreetMapPlace>;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
/// This is based on the Open Streetmap JsonV2 response
pub struct OpenStreetMapPlace {
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

pub struct PlaceWithTimeZone {
    pub id: i64,
    pub _name: String,
    pub display_name: String,
    pub time_zone: Tz,
}
