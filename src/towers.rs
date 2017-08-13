use graph::{Longitude, Latitude, NodeInfo};
use geom::{project, intersect, Coord, Point};

use std::path::Path;
use std::error::Error;

use csv::Reader;


#[derive(Debug, Deserialize, HeapSizeOf)]
pub struct Tower {
    radio: TowerType,
    lat: Latitude,
    lon: Longitude,
    range: usize,
}

impl Coord for Tower {
    #[inline]
    fn lat(&self) -> f64 {
        self.lat
    }
    #[inline]
    fn lon(&self) -> f64 {
        self.lon
    }
}

#[derive(Debug, Deserialize, HeapSizeOf)]
enum TowerType {
    LTE,
    UMTS,
    GSM,
}

pub fn edge_coverage(s: &NodeInfo, t: &NodeInfo, towers: &[Tower]) -> f64 {
    let mut skip_count = 0;
    let mut sections: Vec<_> = towers
        .iter()
        .filter_map(|tower| {
            let s = project(s, tower.lat);
            let t = project(t, tower.lat);
            if (t.x() - s.x()).abs() < ::std::f64::EPSILON {
                skip_count += 1;
                return None;
            }
            let tower_point = project(tower, tower.lat);
            let sec = intersect(&s, &t, &tower_point, tower.range as f64);
            if sec.is_empty() { None } else { Some(sec) }
        })
        .collect();

    sections.sort();
    sections = sections.iter().fold(Vec::new(), |mut acc, sec| {
        if acc.len() == 0 {
            acc.push(sec.clone());
        } else {
            let last_sec = acc.pop().unwrap();
            if sec.is_overlapping(&last_sec) {
                acc.push(sec.merge(&last_sec));
            } else {
                acc.push(last_sec);
                acc.push(sec.clone());
            }
        }
        acc
    });


    let res = sections.iter().fold(0.0, |acc, sec| acc + sec.length());
    assert!(res <= 1.0 && res >= 0.0);
    res
}

pub fn load_towers<P: AsRef<Path>>(path: P) -> Result<Vec<Tower>, Box<Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut result = Vec::new();
    for res in reader.deserialize() {
        result.push(res?);
    }
    Ok(result)
}
