use super::graph::{NodeId, Graph};
use rocket::State;
use rocket::request::{FormItems, FromForm};

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

pub enum DijkQueryErr {
    ParseErr,
    ItemNotPresen(&'static str),
}

impl<'f> FromForm<'f> for DijkQuery {
    type Error = DijkQueryErr;

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
            return Err(DijkQueryErr::ItemNotPresen("No parameter \"s\" present"));
        }
        if t == ::std::usize::MAX {
            return Err(DijkQueryErr::ItemNotPresen("No parameter \"t\" present"));
        }
        Ok(DijkQuery { s, t })
    }
}

impl From<::std::num::ParseIntError> for DijkQueryErr {
    fn from(_: ::std::num::ParseIntError) -> Self {
        DijkQueryErr::ParseErr
    }
}
