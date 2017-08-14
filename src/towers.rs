use graph::{Longitude, Latitude, NodeInfo};
use geom::{project, intersect, Coord, Point, SegmentSection};

use std::path::Path;
use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::cell::UnsafeCell;

use csv::Reader;
use serde::{Deserializer, Deserialize};
use serde::de::{self, Visitor};
use heapsize::HeapSizeOf;


pub struct Coverage(HashMap<Provider, UnsafeCell<Vec<f64>>>, usize);
impl Coverage {
    pub fn new(size: usize) -> Coverage {
        let mut map = HashMap::new();
        map.insert(Provider::Telekom, UnsafeCell::new(vec![0.0; size]));
        map.insert(Provider::Vodafone, UnsafeCell::new(vec![0.0; size]));
        map.insert(Provider::O2, UnsafeCell::new(vec![0.0; size]));
        Coverage(map, size)
    }

    pub fn get(&self, p: &Provider, n: usize) -> f64 {
        assert!(self.1 > n, format!("Index of {} is to high", n));
        let cell = &self.0[p];
        unsafe { (*cell.get())[n] }
    }

    pub fn set(&self, p: &Provider, n: usize, value: f64) {
        assert!(self.1 > n, format!("Index of {} is to high", n));
        assert!(
            0.0 <= value && 1.0 >= value,
            format!("Value {} out of range [0.0,1.0]", value)
        );
        let cell = &self.0[p];
        unsafe {
            (*cell.get())[n] = value;
        }
    }
}
unsafe impl Sync for Coverage {}

impl HeapSizeOf for Coverage {
    fn heap_size_of_children(&self) -> usize {
        let mut size = 0;
        for (k, v) in &self.0 {
            unsafe {
                size += (*v.get()).heap_size_of_children() + k.heap_size_of_children();
            }
        }
        size
    }
}

#[derive(Debug, Deserialize, HeapSizeOf)]
pub struct Tower {
    radio: TowerType,
    net: Provider,
    lat: Latitude,
    lon: Longitude,
    range: f64,
}



pub fn edge_coverage<'a, I: Iterator<Item = &'a Tower>>(
    s: &NodeInfo,
    t: &NodeInfo,
    towers: Vec<I>,
) -> (f64, f64, f64) {
    let mut skip_count = 0;
    let mut tower_count = 0;
    let mut o2_sections = Vec::new();
    let mut telekom_sections = Vec::new();
    let mut vodafone_sections = Vec::new();
    let _: Vec<()> = towers
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
            if !sec.is_empty() {
                match tower.net {
                    Provider::Telekom => telekom_sections.push(sec),
                    Provider::Vodafone => vodafone_sections.push(sec),
                    Provider::O2 => o2_sections.push(sec),
                };
            }

            None
        })
        .collect();
    (
        accumulate_sections(telekom_sections),
        accumulate_sections(vodafone_sections),
        accumulate_sections(o2_sections),
    )

}

fn accumulate_sections(mut sections: Vec<SegmentSection>) -> f64 {
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
    for res in reader.deserialize().filter(|res| res.is_ok()) {
        let mut tower: Tower = res?;
        tower.range /= 1000.0; //convert range into km
        result.push(tower);
    }
    Ok(result)
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

#[derive(Debug, HeapSizeOf, PartialEq, Eq, Hash)]
pub enum Provider {
    Telekom,
    Vodafone,
    O2,
}

impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(ProviderVisitor)
    }
}

struct ProviderVisitor;

impl<'de> Visitor<'de> for ProviderVisitor {
    type Value = Provider;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 1 and 3")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Provider, E>
    where
        E: de::Error,
    {
        match value {
            1 => Ok(Provider::Telekom),
            2 => Ok(Provider::Vodafone),
            3 => Ok(Provider::O2),
            _ => Err(E::custom(format!("value out of range 1-3: {}", value))),
        }
    }
}
