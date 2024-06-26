extern crate osmpbfreader;

use crate::output::output_handler::OutputHandlerConfiguration;
use crate::output::OverwriteConfiguration;
use clap::{crate_authors, crate_version, Arg,command};

mod converter;
mod osm_reader;
mod output;
mod utils;

fn main() {
    const INPUT_ARG: &str = "INPUT";
    const OUTPUT_FOLDER: &str = "OUTPUT";
    const MIN_ADMIN_LEVEL_ARG: &str = "MIN_ADMIN_LEVEL";
    const MAX_ADMIN_LEVEL_ARG: &str = "MAX_ADMIN_LEVEL";
    const OVERWRITE_ARG: &str = "OVERWRITE";
    const SKIP_ARG: &str = "SKIP";
    const GEOJSON_ARG: &str = "GEOJSON";

    let matches = command!("OSM Extract Polygon")
        .version(crate_version!())
        .author(crate_authors!())
        .about(
            "Extracts administrative boundaries of OSM pbf files and produces polygon files compatible with Osmosis.",
        )
        .arg(
            Arg::new(INPUT_ARG)
                .short('f')
                .long("file")
                .value_name("filename")
                .help("input file")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new(MIN_ADMIN_LEVEL_ARG)
                .short('m')
                .long("min")
                .value_name("min_admin_level")
                .help("minimum administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new(MAX_ADMIN_LEVEL_ARG)
                .short('x')
                .long("max")
                .value_name("max_admin_level")
                .help("max administrative level (can take value from 1-11) [default: 8]")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new(OUTPUT_FOLDER)
                .short('p')
                .long("path")
                .value_name("path")
                .help("path to which the output will be saved to [default: '<input_filename>_polygons/']")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new(OVERWRITE_ARG)
            .short('o')
            .long("overwrite")
            .num_args(0)
            .help("set this flag to overwrite files without asking; if neither this nor --skip is set the user is being prompted should a file be overwritten.")
        )
        .arg(
            Arg::new(SKIP_ARG)
            .short('s')
            .long("skip")
            .num_args(0)
            .help("set this flag to skip overwriting files; if neither this nor --overwrite is set the user is being prompted should a file be overwritten.")
        )
        .arg(
            Arg::new(GEOJSON_ARG)
            .short('g')
            .long("geojson")
            .num_args(0)
            .help("set this flag to generate geojson output")
        )
        .get_matches();

    let min_admin_level = matches
        .get_one::<String>(MIN_ADMIN_LEVEL_ARG)
        .unwrap_or(&("8".to_string()))
        .parse::<i8>()
        .unwrap();

    let max_admin_level = matches
        .get_one::<String>(MAX_ADMIN_LEVEL_ARG)
        .unwrap_or(&("8".to_string()))
        .parse::<i8>()
        .unwrap();

    if min_admin_level > max_admin_level {
        println!(
            "error: --min={} has bigger value than --max={}",
            min_admin_level, max_admin_level
        );
        std::process::exit(-1);
    }

    let overwrite_all = matches.get_flag(OVERWRITE_ARG);
    let skip_all = matches.get_flag(SKIP_ARG);

    if overwrite_all && skip_all {
        println!("error: cannot set both -o (--overwrite) and -s (--skip)!");
        std::process::exit(-1);
    }

    let overwrite_configuration = if overwrite_all {
        OverwriteConfiguration::OverwriteAll
    } else if skip_all {
        OverwriteConfiguration::SkipAll
    } else {
        OverwriteConfiguration::Ask
    };

    let geojson_output = matches.get_flag(GEOJSON_ARG);

    let output_handler_config = OutputHandlerConfiguration {
        overwrite_configuration,
        geojson_output,
    };

    let in_filename = matches.get_one::<String>(INPUT_ARG).unwrap();
    println!("Using input file: {}", in_filename);
    let default_path = format!("{}_polygons/", in_filename);
    let path = matches.get_one::<String>(OUTPUT_FOLDER).unwrap_or(&default_path);
    println!("Output path: {}", path);

    let relations = osm_reader::read_osm(in_filename, &min_admin_level, &max_admin_level);
    let polygons = converter::convert(relations.unwrap());
    let result = output::output_handler::write(path, &polygons, output_handler_config);

    match result {
        Ok(size) => println!("success! wrote {} files!", size),
        Err(e) => println!("error! {:?}", e),
    }
}
