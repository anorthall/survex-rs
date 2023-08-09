//! Rust bindings to the Survex img library.
//!
//! # Usage
//! Information from a Survex .3d file is read into a [`StationManager`][`station::StationManager`],
//! which is a container for a list of [`Stations`][`station::Station`] and a graph of connections
//! between those stations.
//!
//! For more information on usage, refer to the documentation for
//! [`read::load_from_path`][`crate::read::load_from_path`] and [`Station`][`station::Station`].
pub mod point;
pub mod read;
pub mod station;
