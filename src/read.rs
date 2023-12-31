//! Helper functions for reading Survex files
//!
//! At present, this module only contains a single function:
//! [`load_from_path`][`crate::read::load_from_path`]. Refer to the documentation for that function,
//! or the [examples in the documentation index][`crate`] for more information.

use crate::data::SurveyData;
use crate::station::Point;
use crate::survex;
use log::trace;
use std::error::Error;
use std::ffi::{c_char, CStr};
use std::path::PathBuf;
use std::ptr;
use uuid::Uuid;

/// Create a [`SurveyData`] instance from a Survex file.
///
/// The path to the Survex file will be passed to the binding to the Survex C library, which will
/// open and read the file. The data within the file will be iterated over to build a list of
/// [Stations][`crate::station::Station`] and a graph of connections between them. The resulting
/// [`SurveyData`] instance will be returned.
pub fn load_from_path(path: PathBuf) -> Result<SurveyData, Box<dyn Error>> {
    // Convert the path to the format required by img.c
    let filename = path
        .to_str()
        .expect("Could not convert path to string")
        .as_ptr() as *const c_char;

    // Create an SurveyData instance to store and update data as it is read.
    let mut data = SurveyData::new();

    // The way Survex 3D file reading works is that it will first spit out a bunch of coordinates
    // and centrelines (determined by MOVE and LINE) commands, and it will then later give names
    // to those coordinates by means of a LABEL command. As such, we will store the connections
    // between two coordinates in a vector and then later - once we have read the full .3d file and
    // have labels for all sets of coordinates - add the connections to the graph.
    let mut connections = Vec::new();

    // These variables are used to store the data which is returned by each call to img_read_item.
    // After a call to img_read_item, pimg will be updated with information from the current item,
    // and p will be updated with the latest set of coordinates.

    // (x, y, z) and label are used to store the previous label and set of coordinates after a
    // call to img_read_item, as the next call may require them (such as in the case of a LINE
    // command to create a leg between two points).
    let pimg;
    let (mut x, mut y, mut z) = (-1.0, -1.0, -1.0);
    let mut label = "";
    let mut p = survex::img_point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    // Open the Survex file and check that it was successful.
    trace!(
        "Opening Survex file '{:?}' in load_from_path function via Survex img library.",
        path
    );
    unsafe {
        pimg = survex::img_open_survey(filename, ptr::null_mut());
    }
    if pimg.is_null() {
        trace!("Survex library returned a null pointer. Read failed.");
        return Err("Could not open Survex file".into());
    }

    // Read the data from the Survex file - loop through calls to img_read_item until it returns
    // a value below zero which indicates that the end of the data has been reached (-1) or that
    // there is an error (-2).
    trace!("Reading Survex file in load_from_path function.");
    loop {
        let result = unsafe { survex::img_read_item(pimg, &mut p) };

        #[allow(clippy::if_same_then_else)]
        if result == -2 {
            // Bad data in Survex file
            panic!("Bad data in Survex file.");
        } else if result == -1 {
            trace!("STOP: End of Survex file reached.");
            // STOP command
            break;
        } else if result == 0 {
            // MOVE command
            (x, y, z) = (p.x, p.y, p.z);
            trace!("MOVE: {}, {}, {}.", x, y, z);
        } else if result == 1 {
            // LINE command
            // At this point (x, y, z) will have been set by a previous MOVE command. We can use
            // the previous coordinates to create a connection between the previous station and
            // the current station. After the 3d file has been read, we can use the connections
            // vector to add the connections to the graph.
            let from_coords = Point::new(x, y, z);
            let to_coords = Point::new(p.x, p.y, p.z);
            connections.push((from_coords, to_coords));
            trace!("LINE: {} -> {}.", from_coords, to_coords);
            (x, y, z) = (p.x, p.y, p.z);
        } else if result == 2 {
            // CROSS command
            trace!("CROSS command received. Ignoring.");
        } else if result == 3 {
            // LABEL command
            let flags;
            unsafe {
                label = CStr::from_ptr((*pimg).label).to_str().unwrap();
                flags = (*pimg).flags & 0x7f;
            }
            let coords = Point::new(p.x, p.y, p.z);
            let (station, _) = data.add_or_update(coords, label);
            trace!("LABEL: {} -> {}.", coords, label);

            // Set the flags for the station
            if flags & 0x01 != 0 {
                station.borrow_mut().surface = true;
                trace!("LABEL: surface flag set for station '{}'.", label);
            }
            if flags & 0x02 != 0 {
                station.borrow_mut().underground = true;
                trace!("LABEL: underground flag set for station '{}'.", label);
            }
            if flags & 0x04 != 0 {
                station.borrow_mut().entrance = true;
                trace!("LABEL: entrance flag set for station '{}'.", label);
            }
            if flags & 0x08 != 0 {
                station.borrow_mut().exported = true;
                trace!("LABEL: exported flag set for station '{}'.", label);
            }
            if flags & 0x10 != 0 {
                station.borrow_mut().fixed = true;
                trace!("LABEL: fixed flag set for station '{}'.", label);
            }
            if flags & 0x20 != 0 {
                // Anonymous stations are given a UUID as their label
                station.borrow_mut().anonymous = true;
                trace!("LABEL: anonymous flag set for station '{}'.", label);
                station.borrow_mut().label = Uuid::new_v4().to_string();
                trace!(
                    "LABEL: UUID '{}' set for anonymous station.",
                    station.borrow().label,
                );
            }
            if flags & 0x40 != 0 {
                station.borrow_mut().wall = true;
                trace!("LABEL: wall flag set for station '{}'.", label);
            }
        } else if result == 4 {
            // XSECT command
            let (l, r, u, d, flags);
            unsafe {
                l = (*pimg).l;
                r = (*pimg).r;
                u = (*pimg).u;
                d = (*pimg).d;
                flags = (*pimg).flags & 0x7f;

                // If 0x20 flag is set, do *not* update the label buffer, and instead use the
                // previous label.
                if flags & 0x20 == 0 {
                    label = CStr::from_ptr((*pimg).label).to_str().unwrap();
                    trace!("XSECT: label set to '{}'.", label);
                } else {
                    trace!("XSECT: label not from {}.", label);
                }
            }
            trace!("XSECT: l={}, r={}, u={}, d={} for {}.", l, r, u, d, label);
            data.get_by_label(label)
                .unwrap_or_else(|| panic!("Could not find station with label {:?}", label))
                .borrow_mut()
                .lrud
                .update(l, r, u, d);
        } else if result == 5 {
            // XSECT_END command
            trace!("XSECT_END command received. Ignoring.");
        } else if result == 6 {
            // ERROR_INFO command
            trace!("ERROR_INFO command received. Ignoring.");
        } else {
            panic!("Unknown item type in Survex file");
        }
    }

    trace!(
        "Survex file reading complete. Processed {} stations and {} connections.",
        data.stations.len(),
        connections.len()
    );

    // Survex file reading is complete. We now need to iterate over the connections vector and
    // add the connections to the graph by looking up the node index for each station and adding
    // an edge between them with the distance between the two stations as the weight.
    for (p1, p2) in connections.iter() {
        let from_station_node_index = data
            .get_by_coords(p1)
            .unwrap_or_else(|| panic!("Could not find station with coordinates {:?}", p1))
            .borrow()
            .index;
        let to_station_node_index = data
            .get_by_coords(p2)
            .unwrap_or_else(|| panic!("Could not find station with coordinates {:?}", p2))
            .borrow()
            .index;
        data.graph.add_edge(
            from_station_node_index,
            to_station_node_index,
            p1.distance(p2),
        );
    }

    trace!(
        "Graph now has {} nodes and {} edges.",
        data.graph.node_count(),
        data.graph.edge_count()
    );

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_file() {
        let path = PathBuf::from("tests/data/0733.3d");
        assert!(load_from_path(path).is_ok());
    }

    #[test]
    fn load_invalid_file() {
        let path = PathBuf::from("tests/data/this-file-does-not-exist.3d");
        assert!(load_from_path(path).is_err());
    }

    /// Check that the correct number of stations are generated from the 3d file. The verification
    /// values were created by checking how many NODE lines were generated when running the same 3d
    /// file through Survex `dump3d`.
    #[test]
    fn check_correct_number_nodes_generated() {
        let path = PathBuf::from("tests/data/0733.3d");
        let manager = load_from_path(path).unwrap();
        assert_eq!(manager.stations.len(), 6104);

        let path = PathBuf::from("tests/data/nottsii.3d");
        let manager = load_from_path(path).unwrap();
        assert_eq!(manager.stations.len(), 1904);
    }

    #[test]
    /// As above, the verification values were calculated by checking how many LEG lines were
    /// generated when running the 3d file through Survex `dump3d` with the `-l` option.
    fn check_correct_number_legs_generated() {
        let path = PathBuf::from("tests/data/0733.3d");
        let manager = load_from_path(path).unwrap();
        assert_eq!(manager.graph.edge_count(), 5929);

        let path = PathBuf::from("tests/data/nottsii.3d");
        let manager = load_from_path(path).unwrap();
        assert_eq!(manager.graph.edge_count(), 1782);
    }

    #[test]
    fn test_absent_lrud_measurements_are_represented_correctly() {
        let path = PathBuf::from("tests/data/nottsii.3d");
        let manager = load_from_path(path).unwrap();
        let station = manager
            .get_by_label("nottsii.inlet5.inlet5-resurvey-4.22")
            .unwrap();
        let station = station.borrow();
        assert_eq!(station.lrud.left, None);
        assert_eq!(station.lrud.right, None);
        assert_eq!(station.lrud.up, None);
        assert_eq!(station.lrud.down, Some(9.0));
    }

    #[test]
    fn test_lrud_measurements_are_represented_correctly() {
        let path = PathBuf::from("tests/data/nottsii.3d");
        let manager = load_from_path(path).unwrap();
        let station = manager
            .get_by_label("nottsii.inlet5.inlet5-resurvey-4.26")
            .unwrap();
        let station = station.borrow();
        assert_eq!(station.lrud.left, Some(1.0));
        assert_eq!(station.lrud.right, Some(0.0));
        assert_eq!(station.lrud.up, Some(0.3));
        assert_eq!(station.lrud.down, Some(0.6));
    }

    #[test]
    fn test_flags_are_set_correctly() {
        let path = PathBuf::from("tests/data/nottsii.3d");
        let manager = load_from_path(path).unwrap();
        let station = manager.get_by_label("nottsii.entrance").unwrap();
        let station = station.borrow();
        assert_eq!(station.surface, false);
        assert_eq!(station.underground, false);
        assert_eq!(station.entrance, true);
        assert_eq!(station.exported, true);
        assert_eq!(station.fixed, true);
        assert_eq!(station.anonymous, false);
        assert_eq!(station.wall, false);

        let station = manager
            .get_by_label("nottsii.inlet5.inlet5-resurvey-2.3.17")
            .unwrap();
        let station = station.borrow();
        assert_eq!(station.surface, false);
        assert_eq!(station.underground, true);
        assert_eq!(station.entrance, false);
        assert_eq!(station.exported, true);
        assert_eq!(station.fixed, false);
        assert_eq!(station.anonymous, false);
        assert_eq!(station.wall, false);

        let station = manager
            .get_by_label("nottsii.mainstreamway.mainstreamway3.27")
            .unwrap();
        let station = station.borrow();
        assert_eq!(station.surface, false);
        assert_eq!(station.underground, true);
        assert_eq!(station.entrance, false);
        assert_eq!(station.exported, false);
        assert_eq!(station.fixed, false);
        assert_eq!(station.anonymous, false);
        assert_eq!(station.wall, false);

        let station = manager
            .get_by_label("nottsii.countlazloall.thecupcake.009")
            .unwrap();
        let station = station.borrow();
        assert_eq!(station.surface, true);
        assert_eq!(station.underground, false);
        assert_eq!(station.entrance, false);
        assert_eq!(station.exported, false);
        assert_eq!(station.fixed, false);
        assert_eq!(station.anonymous, false);
        assert_eq!(station.wall, false);
    }
}
