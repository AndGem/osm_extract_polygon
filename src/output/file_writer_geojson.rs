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
        let feature = convert_polygon_to_geojson_feature(polygon);
        if feature.is_ok() {
            Ok(file.write_all(feature.unwrap().to_string().as_bytes())?)
        } else {
            Err(Error::new(ErrorKind::Other, "Error in converting Polygon to GeoJSON."))
        }
    }
}

fn convert_polygon_to_geojson_feature(polygon: &Polygon) -> Result<Feature, ()> {
    let properties = create_properties(polygon);
    let polygons = convert_polygon_to_geo_polygons(polygon);

    convert_to_geometry(polygons).and_then(|p| convert_to_feature(p, properties))
}

fn convert_to_feature(geometry: Geometry, properties: Map<String, serde_json::Value>) -> Result<Feature, ()> {
    let feature = Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    };

    Ok(feature)
}

fn create_properties(polygon: &Polygon) -> Map<String, serde_json::Value> {
    //TODO: add admin level for boundaries
    let mut properties = Map::new();
    properties.insert(String::from("name"), to_value(&polygon.name).unwrap());

    properties
}

fn convert_to_geometry(polygons: Vec<GeoPolygon<f32>>) -> Result<Geometry, ()> {
    match polygons.len() {
        0 => Err(()),
        1 => Ok(Geometry::new(geojson::Value::from(polygons.get(0).unwrap()))),
        _ => Ok(Geometry::new(geojson::Value::from(&MultiPolygon(polygons)))),
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

// ////////////////////////////////////
// ////////////////////////////////////
// UNIT TESTS
// ////////////////////////////////////
// ////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_single_polygon_to_geo_polygons() {
        let p1 = Point { lat: 1.0, lon: 1.0 };
        let p2 = Point { lat: 2.0, lon: 10.0 };
        let p3 = Point { lat: 3.0, lon: 100.0 };

        let single_polygon = Polygon {
            name: "barfoo".to_string(),
            points: vec![vec![p1.clone(), p2.clone(), p3.clone()]],
        };

        let result = convert_polygon_to_geo_polygons(&single_polygon);
        let expected_line_str = LineString(vec![
            Coordinate { x: p1.lon, y: p1.lat },
            Coordinate { x: p2.lon, y: p2.lat },
            Coordinate { x: p3.lon, y: p3.lat },
        ]);
        let expected = vec![GeoPolygon::new(expected_line_str, vec![])];

        assert_eq!(result, expected);
    }
}
