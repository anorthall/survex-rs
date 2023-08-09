# survex-rs
Rust bindings to the Survex `img.c` library which can be used via a safe API utilising
[`SurveyData`][`data::SurveyData`] and [`Station`][`station::Station`] structs, or directly via
unsafe Rust. For more information, view the [documentation](https://docs.rs/survex-rs).

## Safe API
The safe API is able to read data from a Survex .3d file and store it in a
[`SurveyData`][`data::SurveyData`] instance. [`SurveyData`][`data::SurveyData`] instances
contain a vector of references to [`Station`][`station::Station`] structs and a graph, built
using [`petgraph`][`petgraph::graph::Graph`], of connections between those stations.

A helper function, [`load_from_path`][`crate::read::load_from_path`], is provided to read a
given Survex .3d file and return a [`SurveyData`][`data::SurveyData`] instance.

### Example
```rust
use std::path::PathBuf;
use survex_rs::read::load_from_path;
use survex_rs::station::Point;

let path = PathBuf::from("tests/data/nottsii.3d");
let data = load_from_path(path).unwrap();

println!("Loaded {} stations", data.stations.len());
// Loaded 1904 stations

println!("Loaded {} survey legs", data.graph.edge_count());
// Loaded 1782 survey legs

let station = data.get_by_label("nottsii.entrance").unwrap();
let station = station.borrow();
println!("Station '{}' is at {}", station.label, station.coords);
// Station 'nottsii.entrance' is at 66668.00, 78303.00, 319.00

let coords = Point::new(66668.00, 78303.00, 319.00);
let station = data.get_by_coords(&coords).unwrap();
let station = station.borrow();
println!("{:#?}", station);
// Station {
//     label: "nottsii.entrance",
//     coords: Point {
//         x: 66668.0,
//         y: 78303.0,
//         z: 319.0,
//     },
//     index: NodeIndex(1901),
//     lrud: LRUD {
//         left: None,
//         right: None,
//         up: None,
//         down: None,
//     },
//     surface: false,
//     underground: false,
//     entrance: true,
//     exported: true,
//     fixed: true,
//     anonymous: false,
//     wall: false,
// }
```

## Unsafe API
If you wish to simply access the Survex `img.c` library directly using unsafe Rust, you can do so
via the bindings in the [`survex`][`crate::survex`] module.

For an example of how to use the unsafe API, take a look at the source for
[`load_from_path`][`crate::read::load_from_path`] in `src/read.rs`.  You can browse the functions in the
[`survex`][`crate::survex`] module and reference them to the Survex `img.c` and `img.h` files found in the `src/`
directory of the [Survex source code](https://github.com/ojwb/survex).

## Project status
This project is currently in early development and is not ready for production use. The API is subject to change at
any time and semantic versioning is not yet being used.

## Contributing
Pull requests are [welcome on GitHub](https://github.com/anorthall/survex-rs).

## License
This project is licensed under the GNU General Public License v3.0 - see the LICENCE file for details.
