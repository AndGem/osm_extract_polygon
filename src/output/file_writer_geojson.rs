use crate::converter::{Point, Polygon};
use crate::output::output_handler::FileWriter;

use geo_types::Polygon as GeoPolygon;
use geo_types::{Coordinate, LineString, MultiPolygon};

use std::fs::File;
use std::io::prelude::*;

pub struct GeoJsonWriter {}

impl FileWriter for GeoJsonWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        let vec_polygons = convert_polygon_to_geo_polygons(polygon);
        let multipolygon = MultiPolygon(vec_polygons);

        let json = geojson::Value::from(&multipolygon);

        file.write_all(json.to_string().as_bytes())?;

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

fn convert_to_linestring(points: &Vec<Point>) -> LineString<f32> {
    LineString(points.iter().map(|p| Coordinate { x: p.lon, y: p.lat }).collect())
}

//TODO: write unit tests
