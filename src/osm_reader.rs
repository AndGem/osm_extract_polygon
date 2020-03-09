use osmpbfreader::{OsmObj, OsmPbfReader, Relation, RelationId, Way, WayId, Node, NodeId};

use std::collections::{HashSet, HashMap};
use std::time::Instant;


pub fn read_osm(filename: &str) {
    let file_reference = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    read_ways_and_relation(file_reference);
}



fn read_ways_and_relation(file_reference: std::fs::File) -> (Vec<Relation>, HashMap<RelationId, Vec<Node>>) {
    let mut pbf = OsmPbfReader::new(file_reference);

    let mut relations = Vec::new(); //out
    let mut nodes: HashMap<RelationId, Vec<Node>> = HashMap::new(); //out

    let mut relation_to_way: HashMap<RelationId, WayId> = HashMap::new();
    let mut way_to_nodes: HashMap<WayId, Vec<NodeId>> = HashMap::new();

    let now = Instant::now();
    for obj in pbf.par_iter().map(Result::unwrap) {
        match obj {
            OsmObj::Relation(relation) => {
                if !relation.tags.contains("boundary", "administrative") {
                    continue;
                }

                //TODO: this can be made nicer!
                for entry in &relation.refs {
                    if !(entry.member.is_way()) {
                        continue;
                    }
                    if !(entry.role == "outer") {
                        continue;
                    }

                    let way_id = entry.member.way().unwrap();

                    relation_to_way.insert(relation.id, way_id);
                    relations.push(relation);
                    break;
                }

            }
            _ => {}
        }
    }

    // TODO: get way ids into hashset here!
    // relation_to_way.values().

    for obj in pbf.par_iter().map(Result::unwrap) {
        match obj {
            OsmObj::Way(way) => {
                println!("{:?}", way);
            }
            _ => {}
        }
    }


    println!("finished reading of osm data: {}s", now.elapsed().as_secs());
    println!(
        "data contains: {} ways, and {} nodes",
        // ways.len(),
        // nodes.len()
        1,
        1
    );

    // (nodes, ways)
    (relations, nodes)
}