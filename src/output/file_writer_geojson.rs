use crate::converter::{Point, Polygon};
use crate::output::output_handler::FileWriter;

use geo_types::Polygon as GeoPolygon;
use geo_types::{Coordinate, LineString, MultiPolygon};
use geojson::Geometry;

use std::fs::File;
use std::io::prelude::*;

use serde_json::json;
use serde_json::map::Map;

pub struct GeoJsonWriter {}

impl FileWriter for GeoJsonWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        let vec_polygons = convert_polygon_to_geo_polygons(polygon);
        let multipolygon = MultiPolygon(vec_polygons);

        //TODO: add admin level for boundaries
        let mut foreign_members = Map::new();
        foreign_members.insert("name".to_string(), json!(polygon.name));

        let geometry = Geometry {
            bbox: None,
            value: geojson::Value::from(&multipolygon),
            foreign_members: Some(foreign_members),
        };

        file.write_all(geometry.to_string().as_bytes())?;

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
