use std::{collections::{HashMap, HashSet}, error::Error, fs::File, hash::RandomState, io::Read, num::ParseIntError};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct JunctionBox {
    x: u64,
    y: u64,
    z: u64,
    unique_id: Uuid 
}

fn modulus_subtract(op_a: u64, op_b: u64)->u64 {
    if op_a>op_b {
        op_a - op_b
    } else {
        op_b - op_a
    }
}

impl JunctionBox {
    /**
     * Creates a new JunctionBox from a string of x,y,z co-ordinates
     */
    pub fn from_string(input: &str) -> Result<JunctionBox, Box<dyn Error>> {
        let coords_res:Result<Vec<u64>, ParseIntError> = input.split(",").map(|num| num.parse::<u64>()).collect();
        
        coords_res.map_err(|e| e.into()).map(|coords| if coords.len()==3 {
            Ok(JunctionBox { x: coords[0], y: coords[1], z: coords[2], unique_id: Uuid::new_v4() })
        } else {
            Err("there were the wrong number of co-ordinates".into())
        }).flatten()
    }

    /**
     * Calculates Euclidean distance between two junction boxes
     */
    pub fn distance(&self, other:&JunctionBox) -> f64 {
        let total: f64 = vec![
            modulus_subtract(self.x, other.x),
            modulus_subtract(self.y, other.y),
            modulus_subtract(self.z, other.z)
        ].iter().map(|n| n.pow(2) as f64).sum::<f64>();
        total.sqrt()
    } 

    pub fn coord(&self) -> String {
        format!("{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(PartialEq, Debug, Eq)]
pub struct JunctionBoxPair<'a> {
    box_one: &'a JunctionBox,
    box_two: &'a JunctionBox
}

impl JunctionBoxPair<'_> {
    pub fn distance(&self) ->f64 {
        self.box_one.distance(&self.box_two)
    }
}

impl PartialOrd for JunctionBoxPair<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.distance() < other.distance() {
            Some(std::cmp::Ordering::Less)
        } else if self.distance() > other.distance() {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

impl Ord for JunctionBoxPair<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.distance() < other.distance() {
            std::cmp::Ordering::Less
        } else if self.distance() > other.distance() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

/**
 * This object tracks the circuits, in the form of an association table and a set of the valid circuit IDs
 */
#[derive(Debug)]
pub struct Circuits {
    //a HashMap, linking box ID on the left to circuit ID on the right. (many-to-one)
    memberships: HashMap<Uuid, Uuid>,
    //the inverse of `memberships`, linking circuit ID on the left to box IDs on the right (one-to-many)
    circuit_memberships: HashMap<Uuid, HashSet<Uuid>>,
    valid_circits: HashSet<Uuid>,
    all_boxes: HashMap<Uuid, JunctionBox>
}

impl Circuits {
    pub fn new(boxes:&Vec<JunctionBox>)->Circuits {
        Circuits { 
            memberships: HashMap::new(), 
            circuit_memberships: HashMap::new(),
            valid_circits: HashSet::new(),
            all_boxes: HashMap::from_iter(boxes.iter().map(|b| (b.unique_id, b.to_owned())))
        }
    }

    pub fn circuit_for(&self, bx:&JunctionBox) -> Option<Uuid> {
        self.memberships.get(&bx.unique_id).cloned()
    }

    pub fn all_circuits(&self) -> impl Iterator<Item = (&Uuid, &HashSet<Uuid>)> {
        self.circuit_memberships.iter()
    }

    pub fn all_connected_boxes(&self) -> impl Iterator<Item = &JunctionBox> {
        self.memberships.keys().filter_map(|k| self.all_boxes.get(k))
    }

    pub fn disconnected_boxes(&self) -> Vec<&JunctionBox> {
        let connected_ids:HashSet<&Uuid, RandomState> = HashSet::from_iter(self.memberships.keys());
        let all_ids = HashSet::from_iter(self.all_boxes.keys());
        all_ids.difference(&connected_ids).filter_map(|id| self.all_boxes.get(id)).collect()
    }

    pub fn sorted_circuits(&self) -> Vec<HashSet<Uuid>> {
        let mut temp:Vec<HashSet<Uuid>> = self.circuit_memberships.values().map(|v| v.to_owned()).collect();
        temp.sort_by(|a, b| {
            a.len().cmp(&b.len())
        });
        temp
    }

    pub fn count(&self) -> usize {
        self.all_circuits().count()
    }

    pub fn connect_pair(&mut self, pair:&JunctionBoxPair) -> Option<Uuid> {
        self.connect(pair.box_one, pair.box_two)
    }

    /**
     * Associates the two boxes given to a circuit and returns the ID of that circuit.
     * Returns the id of the circuit, or None if they were already joined
     */
    pub fn connect(&mut self, box_a:&JunctionBox, box_b:&JunctionBox) -> Option<Uuid> {
        // {} -> {}", box_a.coord(), box_b.coord());
        match (self.circuit_for(box_a), self.circuit_for(box_b)) {
            (None, None)=>{
                //Neither box is in a circuit; create a new circuit ID and associate them
                let c_id = Uuid::new_v4();
                self.valid_circits.insert(c_id);
                self.memberships.insert(box_a.unique_id, c_id);
                self.memberships.insert(box_b.unique_id, c_id);
                match self.circuit_memberships.get_mut(&c_id) {
                    Some(existing_content)=>{
                        existing_content.insert(box_a.unique_id);
                        existing_content.insert(box_b.unique_id);
                    },
                    None=>{
                        let mut m:HashSet<Uuid> = HashSet::with_capacity(2);
                        m.insert(box_a.unique_id);
                        m.insert(box_b.unique_id);
                        self.circuit_memberships.insert(c_id, m);
                    }
                };
                Some(c_id)
            },
            (Some(c_id), None)=>{
                //We are joining a new box onto an existing circuit
                self.memberships.insert(box_b.unique_id, c_id);
                match self.circuit_memberships.get_mut(&c_id) {
                    Some(existing_content)=>{
                        existing_content.insert(box_b.unique_id);
                    },
                    None=>{
                        let mut m:HashSet<Uuid> = HashSet::with_capacity(2);
                        m.insert(box_b.unique_id);
                        self.circuit_memberships.insert(c_id, m);
                    }
                };
                Some(c_id)
            },
            (None, Some(c_id))=>{
                //We are joining a new box onto an existing circuit
                self.memberships.insert(box_a.unique_id, c_id);
                match self.circuit_memberships.get_mut(&c_id) {
                    Some(existing_content)=>{
                        existing_content.insert(box_a.unique_id);
                    },
                    None=>{
                        let mut m:HashSet<Uuid> = HashSet::with_capacity(2);
                        m.insert(box_a.unique_id);
                        self.circuit_memberships.insert(c_id, m);
                    }
                };
                Some(c_id)
            },
            (Some(circuit_a), Some(circuit_b))=>{
                if circuit_a==circuit_b {   //if they are already part of the same circuit, then we don't need to join again
                    return None;
                }
                //Both boxes are members of different circuits; we must merge the circuits by removing one and moving all
                //its contents to the other
                self.valid_circits.remove(&circuit_b);
                let members_to_move = self.circuit_memberships.remove(&circuit_b);

                match members_to_move {
                    Some(set)=>{
                        set.iter().for_each(|box_id| {
                            self.memberships.insert(*box_id, circuit_a);
                        });
                        match self.circuit_memberships.get_mut(&circuit_a) {
                            Some(existing_circuit_a)=>existing_circuit_a.extend(set),
                            None=>panic!("when merging there should already be memberships in circuit a!")
                        }
                    },
                    None=>panic!("when merging there should already be memberships in circuit b!")
                }
                Some(circuit_a)
            }
        }
    }
}

pub fn parse_input(input:&str) -> Result<Vec<JunctionBox>, Box<dyn Error>> {
    input.lines().into_iter().map(|l| JunctionBox::from_string(l)).collect()
}

/**
 * Generates every permutation of box pairs from the incoming list.
 * We only permute going forwards; otherwise there would be two pairs for every box (one the mirror-image of the other)
 */
pub fn pair_up<'a> (boxes:&'a Vec<JunctionBox>) -> Vec<JunctionBoxPair<'a>> {
    let mut result:Vec<JunctionBoxPair> = vec![];

    let top = boxes.len();
    for i in 0..boxes.len() {
        match boxes[i..top].split_first() {
            Some((first, others))=>{
                for box_two in others {
                    result.push(JunctionBoxPair { box_one: first, box_two})
                }
            },
            None=>{ }
        }
    }

    result
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let boxes = parse_input(&content)?;
    println!("Parsed in {} boxes", boxes.len());
    let mut pairs = pair_up(&boxes);
    pairs.sort();
    // pairs.iter().take(10).enumerate().for_each(|(i, p)| println!("{}: {}", i, p.distance()));
    let mut circuits = Circuits::new(&boxes);

    println!("There are a total of {} pairs", pairs.len());
    if pairs.len() < 1000 {
        return Err("there were insufficient boxes to complete the task".into());
    }

    //Connect closest 1,000 pairs
    for i in 0..1000 {
        circuits.connect_pair(&pairs[i]);
    }

    println!("There were {} connected circuits and {} loose boxes", circuits.all_circuits().count(), circuits.disconnected_boxes().len());
    
    //Debug - show the contents of the 10 largest
    // circuits.sorted_circuits().iter().rev().take(10).enumerate().for_each(|(i, c)| {
    //     let coords:Vec<String> = c.iter().filter_map(|id| circuits.all_boxes.get(id).map(|bx| bx.coord())).collect();
    //     let desc_str = coords.join(";");
    //     println!("{} (size {}): {}", i, c.len(), desc_str);
    // });
    //Calculate the size of the three largest circuits multiplied together
    let three_largest = circuits.sorted_circuits().iter()
        .rev()
        .take(3)
        .map(|c| c.len())
        .reduce(|total, size| total*size);
    
    match three_largest {
        Some(size)=>println!("The product of the size of the three largest circuits was {}", size),
        None=>println!("There were not enough circuits to take the product")
    }
    
    Ok( () )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_pairup() {
        let input = "162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689";
        let boxes = parse_input(&input).unwrap();
        let mut pairs = pair_up(&boxes);
        println!("From {} boxes we got {} pairs", boxes.len(), pairs.len());
        pairs.sort();

        // pairs.iter().for_each(|p| {
        //     println!("{} -> {} is {}", p.box_one.coord(), p.box_two.coord(), p.distance())
        // });

        //Based on the example data, these are the first few pairs we should have
        assert_eq!(pairs[0].box_one.coord(), "162,817,812");
        assert_eq!(pairs[0].box_two.coord(), "425,690,689");
        assert_eq!(pairs[1].box_one.coord(), "162,817,812");
        assert_eq!(pairs[1].box_two.coord(), "431,825,988");
        assert_eq!(pairs[2].box_one.coord(), "906,360,560");
        assert_eq!(pairs[2].box_two.coord(), "805,96,715");
    }

    #[test]
    fn test_example() {
        let input = "162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689";
        let boxes = parse_input(&input).unwrap();
        let mut pairs = pair_up(&boxes);
        pairs.sort();

        let mut circuits = Circuits::new(&boxes);

        // circuits.connect_pair(&pairs[0]);
        // circuits.connect_pair(&pairs[1]);
        // circuits.connect_pair(&pairs[2]);
        for i in 0..10 {
            circuits.connect_pair(&pairs[i]);

            for (i, c) in circuits.sorted_circuits().iter().enumerate() {
                let boxes:Vec<String> = c.iter()
                    .filter_map(|boxid| {
                        boxes.iter().find(|b| b.unique_id==*boxid)
                    })
                    .map(|b| b.coord())
                    .collect();

                println!("{}: circuit of {}", i, boxes.join(";"))
            }
        }
        assert_eq!(circuits.count(), 4);
        assert_eq!(circuits.disconnected_boxes().len(), 7);
        let circuit_sizes_sorted:Vec<usize> = circuits.sorted_circuits().iter().map(|c| c.iter().count()).collect();
        assert_eq!(circuit_sizes_sorted, vec![2,2,4,5]);

        let final_product = circuits.sorted_circuits().iter().rev()
            .take(3)
            .map(|c| c.len())
            .reduce(|total, size| total*size);

        assert_eq!(final_product, Some(40));
    }

    #[test]
    fn test_connect_new() {
        let box_a = JunctionBox::from_string("1,2,3").unwrap();
        let box_b = JunctionBox::from_string("4,5,6").unwrap();
        let mut c = Circuits::new(&vec![box_a,box_b]);
        c.connect(&box_a, &box_b);

        assert_eq!(c.count(), 1);
        let c_id = c.circuit_for(&box_a).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_a.unique_id));
        assert!(content.contains(&box_b.unique_id));
        assert_eq!(content.len(), 2)
    }

    #[test]
    fn test_connect_existing_l() {
        let box_a = JunctionBox::from_string("1,2,3").unwrap();
        let box_b = JunctionBox::from_string("4,5,6").unwrap();
        let box_c = JunctionBox::from_string("7,8,9").unwrap();
        let mut c = Circuits::new(&vec![box_a, box_b, box_c]);
        c.connect(&box_a, &box_b);
        c.connect(&box_b, &box_c);

        assert_eq!(c.count(), 1);
        let c_id = c.circuit_for(&box_a).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_a.unique_id));
        assert!(content.contains(&box_b.unique_id));
        assert!(content.contains(&box_c.unique_id));
        assert_eq!(content.len(), 3)
    }

    #[test]
    fn test_connect_existing_r() {
        let box_a = JunctionBox::from_string("1,2,3").unwrap();
        let box_b = JunctionBox::from_string("4,5,6").unwrap();
        let box_c = JunctionBox::from_string("7,8,9").unwrap();
        let mut c = Circuits::new(&vec![box_a, box_b, box_c]);
        c.connect(&box_a, &box_b);
        c.connect(&box_c, &box_b);

        assert_eq!(c.count(), 1);
        let c_id = c.circuit_for(&box_a).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_a.unique_id));
        assert!(content.contains(&box_b.unique_id));
        assert!(content.contains(&box_c.unique_id));
        assert_eq!(content.len(), 3)
    }

    #[test]
    fn test_connect_existing_both() {
        let box_a = JunctionBox::from_string("1,2,3").unwrap();
        let box_b = JunctionBox::from_string("4,5,6").unwrap();
        let box_c = JunctionBox::from_string("7,8,9").unwrap();
        let box_d = JunctionBox::from_string("0,1,2").unwrap();
        
        let mut c = Circuits::new(&vec![box_a, box_b, box_c, box_d]);
        c.connect(&box_a, &box_b);
        c.connect(&box_c, &box_d);
        assert_eq!(c.count(), 2);

        c.connect(&box_a, &box_c);
        assert_eq!(c.count(), 1);
        let c_id = c.circuit_for(&box_a).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_a.unique_id));
        assert!(content.contains(&box_b.unique_id));
        assert!(content.contains(&box_c.unique_id));
        assert!(content.contains(&box_d.unique_id));
    }

    #[test]
    fn test_connect_existing_both_same() {
        let box_a = JunctionBox::from_string("1,2,3").unwrap();
        let box_b = JunctionBox::from_string("4,5,6").unwrap();
        let box_c = JunctionBox::from_string("7,8,9").unwrap();
        let box_d = JunctionBox::from_string("0,1,2").unwrap();
        
        let mut c = Circuits::new(&vec![box_a, box_b, box_c, box_d]);
        c.connect(&box_a, &box_b);
        c.connect(&box_c, &box_d);
        assert_eq!(c.count(), 2);

        c.connect(&box_d, &box_c);
        assert_eq!(c.count(), 2);
        let c_id = c.circuit_for(&box_a).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_a.unique_id));
        assert!(content.contains(&box_b.unique_id));
        assert_eq!(content.len(), 2);

        let c_id = c.circuit_for(&box_c).unwrap();
        let content = c.circuit_memberships.get(&c_id).unwrap();
        assert!(content.contains(&box_c.unique_id));
        assert!(content.contains(&box_d.unique_id));
        assert_eq!(content.len(), 2);
    }

    #[test]
    fn test_distance() {
        let box_a = JunctionBox::from_string("0,0,0").unwrap();
        let box_b = JunctionBox::from_string("90000,0,0").unwrap();
        let dist = box_b.distance(&box_a);
        assert_eq!(dist, 90000.0);
    }
}