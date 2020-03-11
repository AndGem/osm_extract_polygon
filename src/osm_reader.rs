use osmpbfreader::{OsmObj, OsmPbfReader, Relation, RelationId, WayId, Node, NodeId};

use std::collections::{HashSet, HashMap};
use std::time::Instant;

#[derive(Clone)]
pub struct RelationNodes {
    pub relation: Relation,
    pub nodes: Vec<Vec<Node>>,
}


use std::fmt;
impl fmt::Debug for RelationNodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RelationNodes {{ data: {:?}, points: {:?} }}", self.relation, 0)
    }
}


pub fn read_osm(filename: &str) -> Vec<RelationNodes> {
    let file_reference = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    read_ways_and_relation(file_reference)
}


fn read_ways_and_relation(file_reference: std::fs::File) -> Vec<RelationNodes> {
    let mut pbf = OsmPbfReader::new(file_reference);

    let mut relations: HashMap<RelationId, Relation> = HashMap::new();
    let mut relation_to_nodes: HashMap<RelationId, Vec<Vec<Node>>> = HashMap::new();

    let mut relation_to_ways: HashMap<RelationId, Vec<WayId>> = HashMap::new();
    let mut way_to_nodes: HashMap<WayId, Vec<NodeId>> = HashMap::new();
    let mut nodeid_to_node: HashMap<NodeId, Node> = HashMap::new();

    let mut now = Instant::now();

    println!("parsing relations...");
    for obj in pbf.par_iter().map(Result::unwrap) {
        match obj {
            OsmObj::Relation(relation) => {
                if !relation.tags.contains("boundary", "administrative") {
                    continue;
                }

                if !relation.tags.contains_key("admin_level") {
                    continue;
                }

                let admin_level_parse = relation.tags
                    .get("admin_level")
                    .unwrap()
                    .parse::<i8>();

                match admin_level_parse {
                    Ok(value) => {
                        if value > 8 {
                            continue;
                        }
                    }
                    Err(_) => {
                        continue;
                    }
                }

                //TODO: this can be made nicer!
                for entry in &relation.refs {
                    if !(entry.member.is_way()) {
                        continue;
                    }
                    
                    //TODO: rethink this criteria
                    // if !(entry.role == "outer") {
                    //     continue;
                    // }

                    let way_id = entry.member.way().unwrap();
                    if !relation_to_ways.contains_key(&relation.id) {
                        relation_to_ways.insert(relation.id, Vec::new());
                    }
                    relation_to_ways.get_mut(&relation.id).unwrap().push(way_id);
                }

                relations.insert(relation.id, relation);
            }
            _ => {}
        }
    }
    println!("parsing relations finished! {}s", now.elapsed().as_secs());
    now = Instant::now();
    
    // println!("{:?}", relation_to_way);
    let way_ids: HashSet<WayId> = relation_to_ways.iter().flat_map( |(k, v) | v.clone()).collect();
    // println!("{:?}", way_ids);

    println!("parsing ways...");
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
    println!("parsing ways finished! {}s", now.elapsed().as_secs());
    now = Instant::now();

    let node_ids: HashSet<NodeId> = way_to_nodes.iter().flat_map(|(k, v)| v.clone()).collect();
    // println!("{:?}", node_ids);

    // 
    println!("parsing nodes...");
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
    for (relation_id, way_ids) in relation_to_ways {
        for way_id in way_ids {
            let opt_node_ids = way_to_nodes.get(&way_id);
            if opt_node_ids.is_none() {
                continue;
            }
            let node_ids: Vec<NodeId> = opt_node_ids
                .unwrap()
                .clone();

            let nodes : Vec<Node> = node_ids.iter()
                .map(|x| nodeid_to_node.get(&x).clone())
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .map(|x| x.clone())
                .collect();

            if !relation_to_nodes.contains_key(&relation_id) {
                relation_to_nodes.insert(relation_id, Vec::new());
            }
            relation_to_nodes.get_mut(&relation_id).unwrap().push(nodes);
        }

    }
    println!("parsing nodes finished! {}s", now.elapsed().as_secs());

    //prepare output
    let output: Vec<RelationNodes> = relation_to_nodes
        .iter()
        .map(|(r_id, nodes)| RelationNodes{ relation: relations.get(&r_id).unwrap().clone(), nodes: nodes.to_vec()})
        .collect();


    // let filtered_relations: Vec<Relation> = relations.iter()
    //     .filter(|r| relation_to_nodes.contains_key(&r.id))
    //     .map(|x| x.clone())
    //     .collect();




    // println!("finished reading of osm data: {}s", now.elapsed().as_secs());
    println!(
        "data contains: {} ways, and {} nodes",
        // ways.len(),
        // nodes.len()
        1,
        1
    );

    // (nodes, ways)
    // (filtered_relations, relation_to_nodes)
    // println!("{:?}", output);
    output
}
