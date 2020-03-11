use osmpbfreader::{Node, NodeId};

use crate::osm_reader::RelationNodes;

pub struct Polygon {
    pub name: String,
    pub points: Vec<Vec<Point>>,
}

pub struct Point {
    pub lat: f32,
    pub lon: f32,
}

use std::fmt;
impl fmt::Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RelationNodes {{ name: {}, points: {:?} }}",
            self.name, self.points
        )
    }
}
impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ lat: {}, lon: {} }}", self.lat, self.lon)
    }
}

pub fn convert(relations: Vec<RelationNodes>) -> Vec<Polygon> {
    relations
        .iter()
        .map(|rn| merge_nodes(rn.clone()))
        .map(convert_to_poly)
        .collect()
}

fn merge_nodes(rn: RelationNodes) -> RelationNodes {
    //  -> RelationNodes
    /*
        merging of nodes is necessary because ways are split into multiple groups
        assumption:
         - ways that can be attached to each other share one node at the end or beginning
         - there are no three way intersections

         1. start with first way and iterate over the rest of nodes and try to find a match
           - if yes -> merge
           - if no -> go to next
         2. repeat process until
    */

    // take first element
    // use swap_remove to take elements out
    let mut nodes = rn.nodes;
    let mut result_nodes: Vec<Vec<Node>> = Vec::new();

    while !nodes.is_empty() {
        let mut path: Vec<Node> = nodes.swap_remove(0);

        loop {
            // let mut found_something = false;
            let (matching_first, x) = find_match(path.first().unwrap().id, nodes);
            nodes = x; //TODO: there must be a nicer way to do this!

            if let Some(mut matching_nodes) = matching_first {
                // found_something = true;
                matching_nodes.reverse();
                path = [&matching_nodes[..], &path[..]].concat();
                continue;
            }

            let (matching_last, x) = find_match(path.last().unwrap().id, nodes);
            nodes = x; //TODO: there must be a nicer way to do this!

            if let Some(matching_nodes) = matching_last {
                path = [&path[..], &matching_nodes[..]].concat();
                continue;
            }

            break;
        }
        //

        result_nodes.push(path);
    }

    RelationNodes {
        relation: rn.relation,
        nodes: result_nodes,
    }
}

fn find_match(node_id: NodeId, mut nodes: Vec<Vec<Node>>) -> (Option<Vec<Node>>, Vec<Vec<Node>>) {
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
            return (Some(result), nodes);
        } else if node.last().unwrap().id == node_id {
            let mut result = nodes.swap_remove(i);
            result.reverse();
            return (Some(result), nodes);
        }
    }
    (None, nodes)
}

fn convert_to_poly(rn: RelationNodes) -> Polygon {
    let points = rn
        .nodes
        .iter()
        .map(|x| convert_nodes_to_points(x))
        .collect();

    let name: String = rn
        .relation
        .tags
        .get("name")
        .unwrap_or(&"UNKNOWN_NAME".to_string())
        .clone();

    let name_prefix = rn
        .relation
        .tags
        .get("name:prefix")
        .unwrap_or(&"".to_string())
        .clone();

    let fullname = if rn.relation.tags.contains_key("name:prefix") {
        format!("{}_{}", name_prefix, name)
    } else {
        name
    };

    Polygon {
        name: fullname,
        points,
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
