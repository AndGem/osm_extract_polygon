use crate::osm_reader::RelationNodes;
use osmpbfreader::Tags;
use osmpbfreader::{Node, NodeId};
use std::fmt;

pub struct Polygon {
    pub name: String,
    pub points: Vec<Vec<Point>>,
    pub relation_id: i64,
    pub admin_level: i64,
}

#[derive(Clone)]
pub struct Point {
    pub lat: f32,
    pub lon: f32,
}

impl fmt::Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RelationNodes {{ name: {}, points: {:?} }}", self.name, self.points)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ lat: {}, lon: {} }}", self.lat, self.lon)
    }
}

pub fn convert(relations: Vec<RelationNodes>) -> Vec<Polygon> {
    relations.iter().map(merge_nodes).map(convert_to_poly).collect()
}

fn merge_nodes(rn: &RelationNodes) -> RelationNodes {
    /*
        merging of nodes is necessary because ways are split into multiple groups
        assumption:
         - ways that can be attached to each other share one node at the end or beginning
         - there are no three way intersections

         1. start with first way and iterate over the rest of nodes and try to find a match
           - if yes -> merge
           - if no -> go to next
         2. repeat process until nothing to merge
    */

    let mut nodes = rn.nodes.clone();
    let mut result_nodes: Vec<Vec<Node>> = Vec::new();

    while !nodes.is_empty() {
        let mut path: Vec<Node> = nodes.swap_remove(0);

        loop {
            let matching_first = find_match(path.first().unwrap().id, &mut nodes);

            if let Some(mut matching_nodes) = matching_first {
                matching_nodes.reverse();
                matching_nodes.append(&mut path);
                path = matching_nodes;
                continue;
            }

            let matching_last = find_match(path.last().unwrap().id, &mut nodes);

            if let Some(mut matching_nodes) = matching_last {
                path.append(&mut matching_nodes);
                continue;
            }

            break;
        }

        result_nodes.push(path);
    }

    RelationNodes {
        relation: rn.relation.clone(),
        nodes: result_nodes,
    }
}

fn find_match(node_id: NodeId, nodes: &mut Vec<Vec<Node>>) -> Option<Vec<Node>> {
    /*
        n_id, [------, n_id-----, -----]
        => Some(n_id-----), [------, -----]

        n_id, [------, -----n_id, -----]
        => Some(n_id-----), [------, -----]
    */
    for (i, node) in nodes.iter().enumerate() {
        if node.is_empty() {
            continue;
        }
        if node.first().unwrap().id == node_id {
            let result = nodes.swap_remove(i);
            return Some(result);
        } else if node.last().unwrap().id == node_id {
            let mut result = nodes.swap_remove(i);
            result.reverse();
            return Some(result);
        }
    }
    None
}

fn get_full_name(tags: &Tags) -> String {
    let name = tags
        .get("name")
        .map(|x| x.to_string())
        .unwrap_or(String::from("UNKNOWN_NAME"));

    let name_prefix = tags
        .get("name:prefix")
        .map(|x| x.to_string())
        .unwrap_or(String::from(""));

    if !name_prefix.is_empty() {
        format!("{}_{}", name_prefix, name)
    } else {
        name.to_string()
    }
}

fn convert_to_poly(rn: RelationNodes) -> Polygon {
    let points = rn.nodes.iter().map(|x| convert_nodes_to_points(x)).collect();
    let relation_id: i64 = rn.relation.id.0;
    let tags = &rn.relation.tags;

    let fullname = get_full_name(tags);
    let admin_level = tags.get("admin_level").and_then(|x| x.parse::<i64>().ok()).unwrap_or(0);

    Polygon {
        name: fullname,
        points,
        relation_id,
        admin_level,
    }
}

fn convert_nodes_to_points(nodes: &[Node]) -> Vec<Point> {
    nodes
        .iter()
        .map(|node| Point {
            lat: ((node.decimicro_lat as f64) / 10_000_000.0) as f32,
            lon: ((node.decimicro_lon as f64) / 10_000_000.0) as f32,
        })
        .collect()
}
