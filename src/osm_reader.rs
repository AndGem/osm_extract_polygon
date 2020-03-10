use osmpbfreader::{OsmObj, OsmPbfReader, Relation, RelationId, WayId, Node, NodeId};

use std::collections::{HashSet, HashMap};
use std::time::Instant;


pub fn read_osm(filename: &str) -> (Vec<Relation>, HashMap<RelationId, Vec<Node>>) {
    let file_reference = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    read_ways_and_relation(file_reference)
}



fn read_ways_and_relation(file_reference: std::fs::File) -> (Vec<Relation>, HashMap<RelationId, Vec<Node>>) {
    let mut pbf = OsmPbfReader::new(file_reference);

    let mut relations = Vec::new(); //out
    let mut relation_to_nodes: HashMap<RelationId, Vec<Node>> = HashMap::new(); //out

    let mut relation_to_way: HashMap<RelationId, WayId> = HashMap::new();
    let mut way_to_nodes: HashMap<WayId, Vec<NodeId>> = HashMap::new();
    let mut nodeid_to_node: HashMap<NodeId, Node> = HashMap::new();

    let now = Instant::now();
    // let x: Vec<Relation> = pbf.par_iter()
    //     .map(Result::unwrap)
    //     .filter(|obj| obj.is_relation())
    //     .map(|obj| obj.relation().unwrap())
    //     // .filter(|obj| )
    //     .collect();

    // let osm_objects = pbf.par_iter().map(Result::unwrap);
    // println!("{:?}", type_of(osm_objects));

    // for obj in osm_objects {

    // }

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

    // println!("{:?}", relation_to_way);
    let way_ids: HashSet<WayId> = relation_to_way.iter().map( |(k, v) | v.clone()).collect();
    // println!("{:?}", way_ids);

    pbf.rewind();
    for obj in pbf.par_iter().map(Result::unwrap) {
        match obj {
            OsmObj::Way(way) => {
                if way_ids.contains(&way.id) {
                    way_to_nodes.insert(way.id, way.nodes);
                }
            }   
            _ => {}
        }
    }

    // println!("{:?}", way_to_nodes);

    let node_ids: HashSet<NodeId> = way_to_nodes.iter().flat_map(|(k, v)| v.clone()).collect();
    // println!("{:?}", node_ids);

    // 
    pbf.rewind();
    for obj in pbf.par_iter().map(Result::unwrap) {
        match obj {
            OsmObj::Node(node) => {
                if node_ids.contains(&node.id) {
                    nodeid_to_node.insert(node.id, node);
                }
            }
            _ => {}
        }
    }


    //TODO: make this nicer as well!
    for (relation_id, way_id) in relation_to_way {
        let opt_node_ids = way_to_nodes.get(&way_id);
        if opt_node_ids.is_none() {
            continue;
        }
        let node_ids: Vec<NodeId> = opt_node_ids
            .unwrap()
            .clone();


        let nodes : Option<Vec<&Node>> = node_ids.iter()
            .map(|x| nodeid_to_node.get(&x).clone())
            .collect();

        if nodes.is_some() {
            let node_data: Vec<Node> = nodes.unwrap().iter().map(|x| (*x).clone()).collect();
            relation_to_nodes.insert(relation_id, node_data);
        }
    }

    let filtered_relations: Vec<Relation> = relations.iter()
        .filter(|r| relation_to_nodes.contains_key(&r.id))
        .map(|x| x.clone())
        .collect();




    println!("finished reading of osm data: {}s", now.elapsed().as_secs());
    println!(
        "data contains: {} ways, and {} nodes",
        // ways.len(),
        // nodes.len()
        1,
        1
    );

    // (nodes, ways)
    (filtered_relations, relation_to_nodes)
}
