use graph::{Longitude, Latitude};

use csv;

#[derive(Debug, Deserialize)]
pub struct Tower {
    radio: TowerType,
    lat: Latitude,
    lon: Longitude,
    range: usize,
}

#[derive(Debug, Deserialize)]
enum TowerType {
    LTE,
    UMTS,
    GSM,
}
