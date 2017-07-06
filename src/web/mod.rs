use super::graph::{NodeId, Graph, RoutingGoal, Movement};
use super::grid::NodeInfoWithIndex;

use rocket::State;
use rocket::request::{FormItems, FromForm};
use rocket::response::{self, Response, Responder, NamedFile};
use rocket::response::content::JSON;
use geojson::{Value, Geometry, Feature, GeoJson};

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;


#[get("/route?<q>")]
pub fn route(q: DijkQuery, graph: State<Graph>) -> JSON<String> {
    let mut d = graph.dijkstra();
    let route = d.distance(q.s, q.t, q.goal, q.movement);
    let route = match route {
        Some(r) => r,
        None => {
            return JSON(
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

    JSON(
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
}

pub enum ParseQueryErr {
    ParseErr,
    ItemNotPresen(&'static str),
}

impl<'f> FromForm<'f> for DijkQuery {
    type Error = ParseQueryErr;

    /// Parses an instance of `Self` from the form items or returns an `Error`
    /// if one cannot be parsed.
    fn from_form_items(form_items: &mut FormItems<'f>) -> Result<Self, Self::Error> {
        let mut s: NodeId = ::std::usize::MAX;
        let mut t: NodeId = ::std::usize::MAX;
        let mut goal = RoutingGoal::Length;
        let mut movement = Movement::Car;
        for item in form_items {

            match item.0 {
                "s" => s = item.1.parse()?,
                "t" => t = item.1.parse()?,
                "goal" => goal = item.1.parse()?,
                "move" => movement = item.1.parse()?,
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
    fn from_form_items(form_items: &mut FormItems<'f>) -> Result<Self, Self::Error> {
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
    fn respond(self) -> response::Result<'a> {
        Response::build()
            .sized_body(Cursor::new(format!("{}", self.0)))
            .ok()

    }
}

#[get("/files/<path..>")]
pub fn serve_files(path: PathBuf) -> Option<NamedFile> {
    let p = Path::new("static/").join(path);
    NamedFile::open(p).ok()
}
