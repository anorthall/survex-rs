use crate::point::Point;
use crate::station::StationManager;
use std::error::Error;
use std::ffi::{c_char, CStr};
use std::path::PathBuf;
use std::ptr;

/// Create a [`StationManager`] instance from a Survex file.
///
/// The path to the Survex file will be passed to the binding to the Survex C library, which will
/// open and read the file. The data within the file will be iterated over to build a list of
/// [Stations][`crate::station::Station`] and a graph of connections between them. The resulting
/// [`StationManager`] will be returned.
pub fn load_from_path(path: PathBuf) -> Result<StationManager, Box<dyn Error>> {
    // Convert the path to the format required by img.c
    let filename = path
        .to_str()
        .expect("Could not convert path to string")
        .as_ptr() as *const c_char;

    // Create a StationManager to store and update data as it is read.
    let mut manager = StationManager::new();

    // The way Survex 3D file reading works is that it will first spit out a bunch of coordinates
    // and centrelines (determined by MOVE and LINE) commands, and it will then later give names
    // to those coordinates by means of a LABEL command. As such, we will store the connections
    // between two coordinates in a vector and then later - once we have read the full .3d file and
    // have labels for all sets of coordinates - add the connections to the graph.
    let mut connections = Vec::new();

    // These variables are used to store the data which is returned by each call to img_read_item.
    // After a call to img_read_item, pimg will be updated with information from the current item,
    // and p will be updated with the latest set of coordinates.

    // x, y, z and label are used to store the previous label and set of coordinates after a
    // call to img_read_item, as the next call may require them (such as in the case of a LINE
    // command to create a leg between two points).
    let pimg;
    let mut label: &str;
    let mut x = -1.0;
    let mut y = -1.0;
    let mut z = -1.0;
    let mut p = survex::img_point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    // Open the Survex file and check that it was successful.
    unsafe {
        pimg = survex::img_open_survey(filename, ptr::null_mut());
    }
    if pimg.is_null() {
        return Err("Could not open Survex file".into());
    }

    // Read the data from the Survex file - loop through calls to img_read_item until it returns
    // a value below zero which indicates that the end of the data has been reached (-1) or that
    // there is an error (-2).
    loop {
        let result = unsafe { survex::img_read_item(pimg, &mut p) };
        #[allow(clippy::if_same_then_else)]
        if result == -2 {
            // Bad data in Survex file
            panic!("Bad data in Survex file");
        } else if result == -1 {
            // STOP command
            break;
        } else if result == 0 {
            // MOVE command
            (x, y, z) = (p.x, p.y, p.z);
        } else if result == 1 {
            // LINE command
            // At this point (x, y, z) will have been set by a previous MOVE command. We can use
            // the previous coordinates to create a connection between the previous station and
            // the current station. After the 3d file has been read, we can use the connections
            // vector to add the connections to the graph.
            let from_coords = Point::new(x, y, z);
            let to_coords = Point::new(p.x, p.y, p.z);
            connections.push((from_coords, to_coords));

            (x, y, z) = (p.x, p.y, p.z);
        } else if result == 2 {
            // CROSS command
            // Ignore: CROSS is not implemented in Survex
        } else if result == 3 {
            // LABEL command
            unsafe {
                label = CStr::from_ptr((*pimg).label).to_str().unwrap();
            }
            let coords = Point::new(p.x, p.y, p.z);
            manager.add_or_update(coords, label);
        } else if result == 4 {
            // XSECT command
            // TODO: Handle XSECTs
        } else if result == 5 {
            // XSECT_END command
            // TODO: Handle XSECTs
        } else if result == 6 {
            // ERROR_INFO command
            // TODO: Handle error info
        } else {
            panic!("Unknown item type in Survex file");
        }
    }

    // Survex file reading is complete. We now need to compile our list of connections between
    // two points into a list of connections between two nodes in the graph. To do this, we will
    // loop through the connections vector and find the index of the node in the graph which
    // corresponds to each set of coordinates. We will then add the connection to the graph.
    let mut node_connections = Vec::new();
    for (p1, p2) in connections.iter() {
        let from_station_node_index = manager
            .get_by_coords(p1)
            .unwrap_or_else(|| panic!("Could not find station with coordinates {:?}", p1))
            .borrow()
            .index;
        let to_station_node_index = manager
            .get_by_coords(p2)
            .unwrap_or_else(|| panic!("Could not find station with coordinates {:?}", p2))
            .borrow()
            .index;
        node_connections.push((from_station_node_index, to_station_node_index));
    }

    manager.graph.extend_with_edges(&node_connections);

    Ok(manager)
}

/// A container module for the rust bindings to the Survex img library.
mod survex {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::upper_case_acronyms)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
