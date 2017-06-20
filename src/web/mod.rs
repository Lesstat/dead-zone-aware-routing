use super::graph::{NodeId, NodeInfo, Graph};
use rocket::State;
use rocket::request::{FormItems, FromForm};
use rocket::response::{self, Response, Responder};

use std::io::Cursor;
#[get("/route?<q>")]
pub fn route(q: DijkQuery, graph: State<Graph>) -> String {
    let mut d = graph.dijkstra();
    let dist = d.distance(q.s, q.t);
    match dist {
        Some(d) => d.to_string(),
        None => "No route found".to_string(),
    }
}

pub struct DijkQuery {
    s: NodeId,
    t: NodeId,
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
        for item in form_items {
            if item.0 == "s" {
                s = item.1.parse()?;
            }
            if item.0 == "t" {
                t = item.1.parse()?;
            }
        }
        if s == ::std::usize::MAX {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"s\" present"));
        }
        if t == ::std::usize::MAX {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"t\" present"));
        }
        Ok(DijkQuery { s, t })
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
pub fn next_node_to(q: NNQuery, graph: State<Graph>) -> Option<NodeInfo> {
    graph.next_node_to(q.lat, q.long).map(|node| node.clone())
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
        let mut lat: f64 = ::std::f64::MAX;
        let mut long: f64 = ::std::f64::MAX;
        for item in form_items {
            if item.0 == "lat" {
                lat = item.1.parse()?;
            }
            if item.0 == "long" {
                long = item.1.parse()?;
            }
        }
        if lat == ::std::f64::MAX {
            return Err(ParseQueryErr::ItemNotPresen("No parameter \"lat\" present"));
        }
        if long == ::std::f64::MAX {
            return Err(ParseQueryErr::ItemNotPresen(
                "No parameter \"long\" present",
            ));
        }
        Ok(NNQuery { lat, long })
    }
}

impl<'a> Responder<'a> for NodeInfo {
    fn respond(self) -> response::Result<'a> {
        Response::build()
            .sized_body(Cursor::new(format!("{{ \"id\": {}  }}", self.osm_id)))
            .ok()

    }
}
