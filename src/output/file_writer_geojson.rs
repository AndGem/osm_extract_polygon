use crate::converter::{Point, Polygon};
use crate::output::output_handler::FileWriter;

use geo_types::Polygon as GeoPolygon;
use geo_types::{Coordinate, LineString, MultiPolygon};
use geojson::{Feature, Geometry};

use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};

use serde_json::{to_value, Map};

pub struct GeoJsonWriter {}

impl FileWriter for GeoJsonWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        let vec_polygons = convert_polygon_to_geo_polygons(polygon);

        let geometry = match vec_polygons.len() {
            0 => {
                println!(
                    "Error in converting Polygon to GeoJSON: {}. Doesn't contain points",
                    polygon.name
                );
                return Err(Error::new(ErrorKind::Other, "Error in converting Polygon to GeoJSON."));
            }
            1 => Geometry::new(geojson::Value::from(vec_polygons.get(0).unwrap())),
            _ => Geometry::new(geojson::Value::from(&MultiPolygon(vec_polygons))),
        };

        //TODO: add admin level for boundaries
        let mut properties = Map::new();
        properties.insert(String::from("name"), to_value(&polygon.name).unwrap());

        let geojson = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        };

        file.write_all(geojson.to_string().as_bytes())?;

        Ok(())
    }
}

fn convert_polygon_to_geo_polygons(polygon: &Polygon) -> Vec<GeoPolygon<f32>> {
    polygon
        .points
        .iter()
        .map(|points| convert_to_linestring(points))
        .map(|linestring| GeoPolygon::new(linestring, vec![]))
        .collect()
}

fn convert_to_linestring(points: &[Point]) -> LineString<f32> {
    LineString(points.iter().map(|p| Coordinate { x: p.lon, y: p.lat }).collect())
}

//TODO: write unit tests
