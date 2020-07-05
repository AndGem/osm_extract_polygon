extern crate osmpbfreader;

use clap::{App, AppSettings, Arg, crate_version, crate_authors};

mod converter;
mod osm_reader;
mod poly_writer;
mod utils;

fn main() {
    const INPUT_ARG: &str = "INPUT";
    const MIN_ADMIN_LEVEL_ARG: &str = "MIN_ADMIN_LEVEL";
    const MAX_ADMIN_LEVEL_ARG: &str = "MAX_ADMIN_LEVEL";

    let matches = App::new("OSM Extract Polygon")
        .version(crate_version!())
        .author(crate_authors!())
        .about(
            "Extracts administrative boundaries of OSM pbf files and produces polygon files compatible with Osmosis.",
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name(INPUT_ARG)
                .short("f")
                .long("file")
                .value_name("filename")
                .help("input file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(MIN_ADMIN_LEVEL_ARG)
                .short("m")
                .long("min")
                .value_name("min_admin_level")
                .help("minimum administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(MAX_ADMIN_LEVEL_ARG)
                .short("x")
                .long("max")
                .value_name("max_admin_level")
                .help("max administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .takes_value(true),
        )
        .get_matches();

    let in_filename = matches.value_of(INPUT_ARG).unwrap();
    println!("Using input file: {}", in_filename);

    let min_admin_level = matches.value_of(MIN_ADMIN_LEVEL_ARG).unwrap_or("8").parse::<i8>().unwrap();
    let max_admin_level = matches.value_of(MAX_ADMIN_LEVEL_ARG).unwrap_or("8").parse::<i8>().unwrap();

    if min_admin_level > max_admin_level {
        println!("error: --min={} has bigger value than --max={}", min_admin_level, max_admin_level);
        std::process::exit(-1);
    }
    
    let relations = osm_reader::read_osm(in_filename, &1, &8);
    let polygons = converter::convert(relations);
    let path = format!("{}_polygons", in_filename);
    let result = poly_writer::write(&path, &polygons);

    match result {
        Ok(size) => println!("success! wrote {} files!", size),
        Err(e) => println!("error! {:?}", e),
    }
}
