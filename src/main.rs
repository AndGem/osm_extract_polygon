extern crate osmpbfreader;

use clap::{App, Arg};

mod converter;
mod osm_reader;
mod poly_writer;
mod utils;

fn main() {
    let matches = App::new("OSM Extract Polygon")
        .version("0.1")
        .author("Andreas <andreas.gemsa@googlemail.com>")
        .about(
            "Extracts administrative boundaries of OSM pbf files and produces polygon files compatible with Osmosis.",
        )
        .arg(
            Arg::with_name("INPUT")
                .short("f")
                .long("file")
                .value_name("file")
                .help("input file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let in_filename = matches.value_of("INPUT").unwrap();
    println!("Using input file: {}", in_filename);

    let relations = osm_reader::read_osm(in_filename);
    let polygons = converter::convert(relations);
    let path = format!("{}_polygons", in_filename);
    let result = poly_writer::write(&path, &polygons);

    match result {
        Ok(size) => println!("success! wrote {} files!", size),
        Err(e) => println!("error! {:?}", e),
    }
}
