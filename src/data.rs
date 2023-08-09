//! Data structures to represent processed Survex data

use crate::station::{Point, Station};
use petgraph::graph::{NodeIndex, UnGraph};
use std::cell::RefCell;
use std::rc::Rc;

pub type Stations = Vec<RefStation>;
pub type RefStation = Rc<RefCell<Station>>;
pub type StationGraph = UnGraph<String, f64>;

/// Handles the creation and management of stations, as well as holding the
/// [`graph`][`petgraph::graph::Graph`] of stations.
pub struct SurveyData {
    pub stations: Stations,
    pub graph: StationGraph,
}

impl Default for SurveyData {
    /// Returns an empty [`SurveyData`] instance with no stations.
    fn default() -> Self {
        Self::new()
    }
}

impl SurveyData {
    /// Create an empty [`SurveyData`] instance with no stations or connections. This method should
    /// not be used directly. Instead, create a [`SurveyData`] instance from a Survex file using the
    /// [`load_from_path`][`crate::read::load_from_path`] helper function.
    pub fn new() -> Self {
        Self {
            stations: Vec::new(),
            graph: StationGraph::new_undirected(),
        }
    }

    /// Retrieve a reference to a [`Station`] by its label.
    pub fn get_by_label(&self, label: &str) -> Option<RefStation> {
        for station in &self.stations {
            if station.borrow().label == label {
                return Some(Rc::clone(station));
            }
        }
        None
    }

    /// Retrieve a reference to a [`Station`] by its label, allowing for partial matches. If
    /// multiple stations match the given label, [`None`] is returned, unless one of the matches is
    /// an exact match, in which case that station is returned.
    pub fn get_by_label_part(&self, label: &str) -> Option<RefStation> {
        let matches = self
            .stations
            .iter()
            .filter(|&node| node.borrow().label.contains(label))
            .collect::<Vec<_>>();

        if matches.len() == 1 {
            return Some(Rc::clone(matches[0]));
        } else {
            for station in matches.iter() {
                if station.borrow().label == label {
                    return Some(Rc::clone(station));
                }
            }
        }

        // We have ruled out an exact match, so there is either no match or multiple matches, so
        // just return None and hope the user can be more specific.
        None
    }

    /// Retrieve a reference to a [`Station`] by its coordinates. If multiple stations exist at the
    /// given coordinates, the first station found is returned.
    pub fn get_by_coords(&self, coords: &Point) -> Option<RefStation> {
        for station in &self.stations {
            if station.borrow().coords == *coords {
                return Some(Rc::clone(station));
            }
        }
        None
    }

    /// Retrieve a reference to a [`Station`] by its index in the graph.
    pub fn get_by_index(&self, index: NodeIndex) -> Option<RefStation> {
        for station in &self.stations {
            if station.borrow().index == index {
                return Some(Rc::clone(station));
            }
        }
        None
    }

    /// This helper method is used to add or update a [`Station`] to both the stations vector and
    /// the graph.
    ///
    /// If a [`Station`] with the given label already exists, the existing station is updated with
    /// the new coordinates. Otherwise, a new [`Station`] is created and added to the stations
    /// vector and the graph. In either case, a reference to the station is returned in a tuple
    /// along with the index of the station in the graph.
    pub fn add_or_update(&mut self, coords: Point, label: &str) -> (RefStation, NodeIndex) {
        if let Some(station) = self.get_by_label(label) {
            let index = station.borrow().index;
            let station_clone = Rc::clone(&station);
            let mut station_mut = station.borrow_mut();
            station_mut.coords = coords;
            return (station_clone, index);
        }

        let index = self.graph.add_node(String::from(label));
        let station = Station::new(String::from(label), coords, index);
        let ref_station = Rc::new(RefCell::new(station));
        let station_clone = Rc::clone(&ref_station);
        self.stations.push(ref_station);
        (station_clone, index)
    }
}
