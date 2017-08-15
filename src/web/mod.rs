use graph::{NodeId, Graph, RoutingGoal, Movement};
use grid::{BoundingBox, NodeInfoWithIndex};
use towers::{Provider, Tower};

use rocket::State;
use rocket::request::{FormItems, FromForm, Request};
use rocket::response::{self, Response, Responder, NamedFile};
use rocket::response::content::Json;
use geojson::{Value, Geometry, Feature, GeoJson};
use serde_json;
use bincode;

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::error::Error;



#[get("/towers?<bbox>")]
pub fn towers(bbox: BoundingBox, towers: State<Vec<Tower>>) -> Result<Json<String>, Box<Error>> {
    let towers: Vec<&Tower> = towers
        .iter()
        .filter(|t| {
            t.net == Provider::O2 && bbox.contains_point(t.lat, t.lon)
        })
        .collect();

    Ok(Json(serde_json::to_string(&towers)?))

}


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
            route.distance,
            2,
            route.travel_time,
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
#[get("/download_graph")]
pub fn download(
    g: State<Graph>,
    towers: State<Vec<Tower>>,
) -> Result<GraphDownload, Box<bincode::ErrorKind>> {
    let state = ApplicationStateRef {
        graph: &g.inner(),
        towers: &towers.inner(),
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
