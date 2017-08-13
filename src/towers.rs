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
    range: f64,
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

pub fn edge_coverage<'a, I: Iterator<Item = &'a Tower>>(
    s: &NodeInfo,
    t: &NodeInfo,
    towers: Vec<I>,
) -> f64 {
    let mut skip_count = 0;
    let mut tower_count = 0;
    let mut sections: Vec<_> = towers
        .into_iter()
        .flat_map(|iter| iter)
        .filter_map(|tower| {
            tower_count += 1;
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
        let mut tower: Tower = res?;
        tower.range /= 1000.0; //convert range into km
        result.push(tower);
    }
    Ok(result)
}
