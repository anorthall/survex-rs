//! Data structures to represent survey stations

use petgraph::graph::NodeIndex;
use std::fmt::{Display, Formatter};

/// Struct representation of a survey station
///
/// To retrieve a station, use the helper methods provided by
/// [`SurveyData`][`crate::data::SurveyData`]. To retrieve a station's connections to other
/// stations, use the graph provided by the [`SurveyData`][`crate::data::SurveyData`] instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    pub label: String,
    pub coords: Point,
    pub index: NodeIndex,
    pub lrud: LRUD,
    pub surface: bool,
    pub underground: bool,
    pub entrance: bool,
    pub exported: bool,
    pub fixed: bool,
    pub anonymous: bool,
    pub wall: bool,
}

impl Station {
    /// Create a new [`Station`] with the given label, coordinates and index. All flags will
    /// default to `false` and the [`LRUD`] measurements will default to `None`.
    pub fn new(label: String, coords: Point, index: NodeIndex) -> Self {
        Self {
            label,
            coords,
            index,
            lrud: LRUD::default(),
            surface: false,
            underground: false,
            entrance: false,
            exported: false,
            fixed: false,
            anonymous: false,
            wall: false,
        }
    }
}

impl Display for Station {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.label, self.coords)
    }
}

/// Passage dimension measurements
///
/// LRUDs (Left, Right, Up, Down) are measurements taken from a station to the walls of a cave
/// passage. The measurements are given in centimeters from the station to the wall and can be
/// used to determine the volume of a passage.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LRUD {
    pub left: Option<f64>,
    pub right: Option<f64>,
    pub up: Option<f64>,
    pub down: Option<f64>,
}

impl LRUD {
    /// Create a new [`LRUD`] instance and update it with the given values. Usually, you will want
    /// to use [`LRUD::update`] on an existing instance contained within a
    /// [`Station`][`crate::station::Station`] struct instead.
    pub fn new(left: f64, right: f64, up: f64, down: f64) -> Self {
        let mut lrud = Self::default();
        lrud.update(left, right, up, down);
        lrud
    }

    /// Update the [`LRUD`] instance with the given values.
    pub fn update(&mut self, left: f64, right: f64, up: f64, down: f64) {
        let left = if left < 0.0 { None } else { Some(left) };
        let right = if right < 0.0 { None } else { Some(right) };
        let up = if up < 0.0 { None } else { Some(up) };
        let down = if down < 0.0 { None } else { Some(down) };
        self.left = left;
        self.right = right;
        self.up = up;
        self.down = down;
    }
}

/// A point in 3D space
///
/// Coordinates are given in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    /// Create a new [`Point`] with the given coordinates.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Calculate the distance between two points. This is a simple Euclidean distance
    /// calculation. The result is given in metres.
    pub fn distance(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}, {:.2}, {:.2}", self.x, self.y, self.z)
    }
}
