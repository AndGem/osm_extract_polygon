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

    convert_to_geometry(polygons).map(|geometry| Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    })
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
    use geojson::Value;
    use std::matches;

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

    #[test]
    fn test_convert_multiple_polygon_to_geo_polygons() {
        let p11 = Point { lat: 1.0, lon: 1.0 };
        let p12 = Point { lat: 2.0, lon: 10.0 };
        let p13 = Point { lat: 3.0, lon: 100.0 };

        let p21 = Point { lat: 4.0, lon: 1.0 };
        let p22 = Point { lat: 5.0, lon: 10.0 };
        let p23 = Point { lat: 6.0, lon: 100.0 };

        let p31 = Point { lat: 7.0, lon: 1.0 };
        let p32 = Point { lat: 8.0, lon: 10.0 };
        let p33 = Point { lat: 9.0, lon: 100.0 };

        let poly = Polygon {
            name: "barfoo".to_string(),
            points: vec![
                vec![p11.clone(), p12.clone(), p13.clone()],
                vec![p21.clone(), p22.clone(), p23.clone()],
                vec![p31.clone(), p32.clone(), p33.clone()],
            ],
        };

        let result = convert_polygon_to_geo_polygons(&poly);
        let expected_line_str1 = LineString(vec![
            Coordinate { x: p11.lon, y: p11.lat },
            Coordinate { x: p12.lon, y: p12.lat },
            Coordinate { x: p13.lon, y: p13.lat },
        ]);
        let expected_line_str2 = LineString(vec![
            Coordinate { x: p21.lon, y: p21.lat },
            Coordinate { x: p22.lon, y: p22.lat },
            Coordinate { x: p23.lon, y: p23.lat },
        ]);
        let expected_line_str3 = LineString(vec![
            Coordinate { x: p31.lon, y: p31.lat },
            Coordinate { x: p32.lon, y: p32.lat },
            Coordinate { x: p33.lon, y: p33.lat },
        ]);
        let expected = vec![
            GeoPolygon::new(expected_line_str1, vec![]),
            GeoPolygon::new(expected_line_str2, vec![]),
            GeoPolygon::new(expected_line_str3, vec![]),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_to_geometry_for_multi_polygon_should_return_multipolygon() {
        let p11 = Point { lat: 1.0, lon: 1.0 };
        let p12 = Point { lat: 2.0, lon: 10.0 };
        let p13 = Point { lat: 3.0, lon: 100.0 };

        let p21 = Point { lat: 4.0, lon: 1.0 };
        let p22 = Point { lat: 5.0, lon: 10.0 };
        let p23 = Point { lat: 6.0, lon: 100.0 };

        let expected_line_str1 = LineString(vec![
            Coordinate { x: p11.lon, y: p11.lat },
            Coordinate { x: p12.lon, y: p12.lat },
            Coordinate { x: p13.lon, y: p13.lat },
        ]);
        let expected_line_str2 = LineString(vec![
            Coordinate { x: p21.lon, y: p21.lat },
            Coordinate { x: p22.lon, y: p22.lat },
            Coordinate { x: p23.lon, y: p23.lat },
        ]);

        let geo_polys = vec![
            GeoPolygon::new(expected_line_str1, vec![]),
            GeoPolygon::new(expected_line_str2, vec![]),
        ];
        let result = convert_to_geometry(geo_polys);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap().value, Value::MultiPolygon(_)));
    }

    #[test]
    fn test_convert_to_geometry_for_single_polygon_should_return_polygon() {
        let p1 = Point { lat: 1.0, lon: 1.0 };
        let p2 = Point { lat: 2.0, lon: 10.0 };
        let p3 = Point { lat: 3.0, lon: 100.0 };
        let expected_line_str = LineString(vec![
            Coordinate { x: p1.lon, y: p1.lat },
            Coordinate { x: p1.lon, y: p2.lat },
            Coordinate { x: p1.lon, y: p3.lat },
        ]);

        let geo_poly = GeoPolygon::new(expected_line_str, vec![]);

        let result = convert_to_geometry(vec![geo_poly]);

        assert!(result.is_ok());
        assert!(matches!(result.unwrap().value, Value::Polygon(_)));
    }

    #[test]
    fn test_convert_to_geometry_for_empty_input_should_return_err() {
        let result = convert_to_geometry(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_properties_contains_name_of_polygon() {
        let poly_name = "barfoo";
        let single_polygon = Polygon {
            name: poly_name.to_string(),
            points: vec![vec![]],
        };
        let result = create_properties(&single_polygon);

        assert!(result.contains_key("name"));
        assert_eq!(result.get("name").unwrap(), poly_name);
    }
}
