use graph::{NodeId, Graph, RoutingGoal, Movement, NodeInfo};
use grid::{BoundingBox, NodeInfoWithIndex};
use towers::{Provider, Tower};

use rocket::State;
use rocket::request::{FormItems, FromForm, Request, FromFormValue};
use rocket::response::{self, Response, Responder, NamedFile};
use rocket::response::content::Json;
use rocket::http::RawStr;
use geojson::{Value, Geometry, Feature, GeoJson, FeatureCollection};
use serde_json;
use bincode;
use rayon::prelude::*;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::error::Error;


#[allow(needless_pass_by_value)]
#[get("/low_coverage?<query>")]
pub fn low_coverage(query: TowerQuery, graph: State<Graph>) -> Json<String> {
    let mut bbox = BoundingBox::new();
    bbox.add_coord(&(query.lat_min, query.lon_min));
    bbox.add_coord(&(query.lat_max, query.lon_max));
    let coverage = graph.coverage.get_all(Some(query.provider)).expect(
        "No coverage data for given Provider",
    );

    let mut edges: Vec<(&NodeInfo, &NodeInfo)> = Vec::new();
    for (n_i, n) in graph.node_info.iter().enumerate() {
        if bbox.contains_point(n.lat, n.long) {
            for (e_i, e) in graph.outgoing_edges_for(n_i) {
                let cov = coverage[e_i];
                if cov <= 0.5 {
                    edges.push((n, &graph.node_info[e.endpoint]));
                }
            }

        }
    }

    let mut features = Vec::new();
    for edge in edges {
        let geometry = Geometry::new(Value::LineString(vec![
            vec![edge.0.long, edge.0.lat],
            vec![edge.1.long, edge.1.lat],
        ]));
        features.push(Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        });
    }

    let collection = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features: features,
        foreign_members: None,
    });

    Json(collection.to_string())
}

#[allow(needless_pass_by_value)]
#[get("/map_coords")]
pub fn map_boundary(graph: State<Graph>) -> Json<String> {
    let bbox = &graph.grid.b_box;

    Json(format!(
        "[ [{}, {}], [{}, {}]]",
        bbox.lat_min,
        bbox.long_min,
        bbox.lat_max,
        bbox.long_max
    ))

}

#[allow(needless_pass_by_value)]
#[get("/towers?<query>")]
pub fn towers(query: TowerQuery, towers: State<Vec<Tower>>) -> Result<Json<String>, Box<Error>> {

    let mut bbox = BoundingBox::new();
    bbox.add_coord(&(query.lat_max, query.lon_max));
    bbox.add_coord(&(query.lat_min, query.lon_min));
    let towers: Vec<&Tower> = towers
        .par_iter()
        .filter(|t| {
            t.net == query.provider && bbox.contains_point(t.lat, t.lon)
        })
        .collect();
    Ok(Json(serde_json::to_string(&towers)?))
}

#[derive(Debug, FromForm)]
pub struct TowerQuery {
    lat_max: f64,
    lat_min: f64,
    lon_max: f64,
    lon_min: f64,
    provider: Provider,
}


#[allow(needless_pass_by_value)]
#[get("/route?<q>")]
pub fn route(q: DijkQuery, graph: State<Graph>) -> Json<String> {
    let mut d = graph.dijkstra();
    let route = d.distance(q.s, q.t, q.goal, q.movement, q.provider);
    let route = match route {
        Some(r) => r,
        None => {
            return Json(
                "{\"distance\": 0, \"travel_time\": 0, \"route\": []}".to_string(),
            )
        }
    };
    let geometry = Geometry::new(Value::LineString(
        route
            .node_seq
            .iter()
            .map(|&n| {
                let node = &graph.node_info[n];
                vec![node.long, node.lat]
            })
            .collect(),
    ));

    let geo: GeoJson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: None,
        foreign_members: None,
    });

    Json(
        format!(
            "{{ \"distance\": {:.*}, \"travel_time\": {:.*},   \"route\": {} }}",
            2,
            route.distance / 1000.0,
            2,
            route.travel_time / 1000.0,
            geo.to_string()
        ).to_string(),
    )
}


pub struct DijkQuery {
    s: NodeId,
    t: NodeId,
    goal: RoutingGoal,
    movement: Movement,
    provider: Option<Provider>,
}

#[derive(Debug)]
pub enum ParseQueryErr {
    ParseErr,
    ItemNotPresen(&'static str),
}

impl<'f> FromForm<'f> for DijkQuery {
    type Error = ParseQueryErr;

    /// Parses an instance of `Self` from the form items or returns an `Error`
    /// if one cannot be parsed.
    fn from_form(form_items: &mut FormItems<'f>, _: bool) -> Result<Self, Self::Error> {
        let mut s: NodeId = ::std::usize::MAX;
        let mut t: NodeId = ::std::usize::MAX;
        let mut goal = RoutingGoal::Length;
        let mut movement = Movement::Car;
        let mut provider = None;
        for item in form_items {

            match item.0.as_str() {
                "s" => s = item.1.parse()?,
                "t" => t = item.1.parse()?,
                "goal" => goal = item.1.parse()?,
                "move" => movement = item.1.parse()?,
                "provider" => provider = Some(item.1.parse()?), 
                _ => (),
            };
        }
        if s == ::std::usize::MAX {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"s\" present"));
        }
        if t == ::std::usize::MAX {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"t\" present"));
        }
        Ok(DijkQuery {
            s,
            t,
            goal,
            movement,
            provider,
        })
    }
}

impl FromStr for RoutingGoal {
    type Err = ParseQueryErr;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "length" => Ok(RoutingGoal::Length),
            "speed" => Ok(RoutingGoal::Speed),
            _ => Err(ParseQueryErr::ParseErr),
        }
    }
}

impl FromStr for Movement {
    type Err = ParseQueryErr;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "car" => Ok(Movement::Car),
            "foot" => Ok(Movement::Foot),
            _ => Err(ParseQueryErr::ParseErr),
        }
    }
}


impl FromStr for Provider {
    type Err = ParseQueryErr;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "telekom" => Ok(Provider::Telekom),
            "vodafone" => Ok(Provider::Vodafone),
            "o2" => Ok(Provider::O2),
            _ => Err(ParseQueryErr::ParseErr),
        }
    }
}

impl<'v> FromFormValue<'v> for Provider {
    type Error = ParseQueryErr;

    fn from_form_value(form_value: &RawStr) -> Result<Self, Self::Error> {
        form_value.parse()
    }
}

impl From<::std::num::ParseIntError> for ParseQueryErr {
    fn from(_: ::std::num::ParseIntError) -> Self {
        ParseQueryErr::ParseErr
    }
}
impl From<::std::num::ParseFloatError> for ParseQueryErr {
    fn from(_: ::std::num::ParseFloatError) -> Self {
        ParseQueryErr::ParseErr
    }
}


#[allow(needless_pass_by_value)]
#[get("/node_at?<q>")]
pub fn next_node_to(q: NNQuery, graph: State<Graph>) -> Option<NodeInfoWithIndex> {
    graph.next_node_to(q.lat, q.long)
}

pub struct NNQuery {
    lat: f64,
    long: f64,
}

impl<'f> FromForm<'f> for NNQuery {
    type Error = ParseQueryErr;

    /// Parses an instance of `Self` from the form items or returns an `Error`
    /// if one cannot be parsed.
    fn from_form(form_items: &mut FormItems<'f>, _: bool) -> Result<Self, Self::Error> {
        use std::f64;
        let mut lat: f64 = f64::MAX;
        let mut long: f64 = f64::MAX;
        for item in form_items {
            if item.0 == "lat" {
                lat = item.1.parse()?;
            }
            if item.0 == "long" {
                long = item.1.parse()?;
            }
        }
        if f64::MAX - lat < f64::EPSILON {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"lat\" present"));
        }
        if f64::MAX - long < f64::EPSILON {
            return Err(ParseQueryErr::ItemNotPresen(
                "No parameter \"long\" present",
            ));
        }
        Ok(NNQuery { lat, long })
    }
}


impl<'a> Responder<'a> for NodeInfoWithIndex {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        Response::build()
            .sized_body(Cursor::new(format!("{}", self.0)))
            .ok()

    }
}

pub struct GraphDownload(Vec<u8>);

impl<'a> Responder<'a> for GraphDownload {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        Response::build().sized_body(Cursor::new(self.0)).ok()
    }
}


#[derive(Serialize)]
pub struct ApplicationStateRef<'a> {
    graph: &'a Graph,
    towers: &'a Vec<Tower>,
}

#[allow(needless_pass_by_value)]
#[get("/download_graph")]
pub fn download(
    g: State<Graph>,
    towers: State<Vec<Tower>>,
) -> Result<GraphDownload, Box<bincode::ErrorKind>> {
    let state = ApplicationStateRef {
        graph: g.inner(),
        towers: towers.inner(),
    };

    Ok(GraphDownload(
        bincode::serialize(&state, bincode::Infinite)?,
    ))
}

#[get("/files/<path..>")]
pub fn serve_files(path: PathBuf) -> Option<NamedFile> {
    let p = Path::new("static/").join(path);
    NamedFile::open(p).ok()
}
