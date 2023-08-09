use crate::point::Point;
use petgraph::graph::{NodeIndex, UnGraph};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

type Stations = Vec<RefStation>;
type RefStation = Rc<RefCell<Station>>;
type StationGraph = UnGraph<String, f64>;

/// Represents a survey station in a Survex file. To retrieve a station, use the helper methods
/// provided by the StationManager. To retrieve a station's connections to other stations, use
/// the graph provided by the StationManager.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    pub label: String,
    pub coords: Point,
    pub index: NodeIndex,
    pub lruds: Option<LRUD>,
}

impl Station {
    pub fn new(label: String, coords: Point, index: NodeIndex) -> Self {
        Self {
            label,
            coords,
            index,
            lruds: None,
        }
    }
}

impl Display for Station {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.label, self.coords)
    }
}

impl Default for StationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles the creation and management of stations, as well as the graph of stations.
pub struct StationManager {
    pub stations: Stations,
    pub graph: StationGraph,
}

impl StationManager {
    /// Create an empty [`StationManager`] with no stations or connections. This method should not
    /// be used directly. Instead, create a [`StationManager`] from a Survex file using the
    /// [`read::load_from_path`][`crate::read::load_from_path`] helper function.
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

    /// This helper method is used to add or update a [`Station`] to both the stations vector and
    /// the graph.
    ///
    /// If a [`Station`] with the given label already exists, the existing station is updated with
    /// the new coordinates and the existing index is returned. Otherwise, a new [`Station`] is
    /// created and added to the stations vector and the graph, and the new index is returned.
    pub fn add_or_update(&mut self, coords: Point, label: &str) -> NodeIndex {
        let index = self.graph.add_node(String::from(label));
        let station = Station::new(String::from(label), coords, index);
        self.stations.push(Rc::new(RefCell::new(station)));
        index
    }
}

/// LRUD: Left, Right, Up, Down.
/// These are the measurements taken from a station to the walls of a cave passage.
/// The measurements are given in centimeters from the station to the wall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LRUD {
    pub left: i32,
    pub right: i32,
    pub up: i32,
    pub down: i32,
}
