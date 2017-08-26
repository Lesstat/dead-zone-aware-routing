use graph::{Longitude, Latitude, NodeInfo};
use geom::{project, intersect, Coord, SegmentSection};

use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::cell::UnsafeCell;
use std::path::Path;

use csv::Reader;
use serde::{Deserializer, Deserialize, Serialize, Serializer};
use serde::de::{self, Visitor};
use heapsize::HeapSizeOf;


#[derive(Serialize, Deserialize)]
pub struct Coverage(HashMap<Provider, UnsafeVec>, usize);
impl Coverage {
    pub fn new(size: usize) -> Coverage {
        let mut map = HashMap::new();
        map.insert(
            Provider::Telekom,
            UnsafeVec(UnsafeCell::new(vec![0.0; size])),
        );
        map.insert(
            Provider::Vodafone,
            UnsafeVec(UnsafeCell::new(vec![0.0; size])),
        );
        map.insert(Provider::O2, UnsafeVec(UnsafeCell::new(vec![0.0; size])));
        Coverage(map, size)
    }


    pub fn set(&self, p: &Provider, n: usize, value: f64) {
        assert!(self.1 > n, format!("Index of {} is to high", n));
        assert!(
            0.0 <= value && 1.0 >= value,
            format!("Value {} out of range [0.0,1.0]", value)
        );
        let cell = &self.0[p];
        unsafe {
            (*cell.0.get())[n] = value;
        }
    }

    pub fn get_all(&self, p: Option<Provider>) -> Option<&Vec<f64>> {
        match p {
            Some(p) => {
                let cell = &self.0[&p];
                unsafe { Some(&*cell.0.get()) }
            }
            None => None,
        }

    }
}
unsafe impl Sync for Coverage {}

impl HeapSizeOf for Coverage {
    fn heap_size_of_children(&self) -> usize {
        let mut size = self.1.heap_size_of_children();
        for (k, v) in &self.0 {
            unsafe {
                size += (*v.0.get()).heap_size_of_children() + k.heap_size_of_children();
            }
        }
        size
    }
}

#[derive(Debug, Deserialize, Serialize, HeapSizeOf)]
pub struct Tower {
    pub radio: TowerType,
    pub net: Provider,
    pub lat: Latitude,
    pub lon: Longitude,
    pub range: f64,
}



pub fn edge_coverage<'a, I: Iterator<Item = &'a Tower>>(
    s: &NodeInfo,
    t: &NodeInfo,
    towers: Vec<I>,
) -> (f64, f64, f64) {
    let mut o2_sections = Vec::new();
    let mut telekom_sections = Vec::new();
    let mut vodafone_sections = Vec::new();
    let _: Vec<()> = towers
        .into_iter()
        .flat_map(|iter| iter)
        .filter_map(|tower| {
            let s = project(s, tower.lat);
            let t = project(t, tower.lat);
            let tower_point = project(tower, tower.lat);
            let sec = intersect(&s, &t, &tower_point, tower.range);
            if !sec.is_empty() {
                match tower.net {
                    Provider::Telekom => telekom_sections.push(sec),
                    Provider::Vodafone => vodafone_sections.push(sec),
                    Provider::O2 => o2_sections.push(sec),
                };
                return Some(());
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
        if acc.is_empty() {
            acc.push(sec.clone());
        } else {
            let last_sec = acc.pop().unwrap();
            if last_sec.is_full() {
                return vec![last_sec];
            }
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

pub fn load_towers<P: AsRef<Path>>(p: P) -> Result<Vec<Tower>, Box<Error>> {
    let mut reader = Reader::from_path(p)?;
    let mut result = Vec::new();
    for res in reader.deserialize().filter(|res| res.is_ok()) {
        let tower: Tower = res?;
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

#[derive(Debug, Deserialize, Serialize, HeapSizeOf)]
pub enum TowerType {
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

impl Serialize for Provider {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(match *self {
            Provider::Telekom => 1,
            Provider::Vodafone => 2,
            Provider::O2 => 3,
        })
    }
}

struct UnsafeVec(UnsafeCell<Vec<f64>>);

impl<'de> Deserialize<'de> for UnsafeVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<f64> = Vec::deserialize(deserializer)?;
        Ok(UnsafeVec(UnsafeCell::new(vec)))
    }
}

impl Serialize for UnsafeVec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec = unsafe { &*self.0.get() };
        vec.serialize(serializer)
    }
}
