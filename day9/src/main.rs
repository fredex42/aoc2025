use std::{collections::HashSet, error::Error, fs::File, hash::RandomState, io::Read};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Tile {
    x: i64,
    y: i64
}

impl Tile {
    pub fn from_string(input:&str) -> Result<Tile, Box<dyn Error>> {
        let matcher = Regex::new("^\\s*(\\d+)\\s*,\\s*(\\d+)\\s*$").unwrap();
        match matcher.captures(input).map(|c| c.extract()) {
            Some((_, [xstr, ystr]))=>{
                match (xstr.parse::<i64>(), ystr.parse::<i64>()) {
                    (Ok(x), Ok(y))=>Ok(Tile { x, y}),
                    (_, _)=>Err(format!("one or more invalid co-ordinates on {}", input).into())
                }
            },
            None=>Err(format!("invalid line in {}", input).into())
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Rectangle<'a> {
    tile_a: &'a Tile,
    tile_b: &'a Tile,
}

impl Rectangle<'_> {
    pub fn new<'a>(tile_a: &'a Tile, tile_b: &'a Tile) -> Rectangle<'a> {
        Rectangle { tile_a, tile_b }
    }

    pub fn area_of_rectangle(&self) -> u64 {
        (
            //(x1-x2)*(y1-y2) does not include the last row and column because the co-ordinates are exclusive.
            //We want the area _bounded by_ the co-ordinates inclusively, i.e. include 1 extra column on the end, 
            //and 1 extra row on the end
            //
            (self.tile_a.x - self.tile_b.x + 1).abs() * (self.tile_a.y - self.tile_b.y + 1).abs()
        ).try_into().expect("there was an integer overflow calculating area")
    }

    pub fn corners(&self, nudge: i64) -> Vec<Tile> {
        let min_x = self.tile_a.x.min(self.tile_b.x);
        let max_x = self.tile_a.x.max(self.tile_b.x);
        let min_y = self.tile_a.y.min(self.tile_b.y);
        let max_y = self.tile_a.y.max(self.tile_b.y);

        vec![
            Tile { x: min_x + nudge, y: min_y + nudge },
            Tile { x: min_x + nudge, y: max_y - nudge },
            Tile { x: max_x - nudge, y: min_y + nudge },
            Tile { x: max_x - nudge, y: max_y - nudge }
        ]
    }

    /**
     * Returns a list of four edges that define the rectangle
     */
    pub fn edges(&self) -> Vec<Edge> {
        let tl_x = self.tile_a.x.min(self.tile_b.x);
        let tl_y = self.tile_a.y.min(self.tile_b.y);
        let tr_x = self.tile_a.x.max(self.tile_b.x);
        let tr_y = tl_y;    //tops are at the same Y
        let bl_x = tl_x;    //lefts are at the same X
        let bl_y = self.tile_a.y.max(self.tile_b.y);
        let br_x = tr_x;
        let br_y = bl_y;

        let v = vec![
            Edge { start: Tile { x: tl_x, y: tl_y }, end: Tile {x: tr_x, y: tr_y}, direction: Direction::LR},
            Edge { start: Tile { x: tr_x, y: tr_y }, end: Tile {x:br_x, y: br_y}, direction: Direction::TB},
            Edge { start: Tile { x: br_x, y: br_y }, end: Tile {x: bl_x, y: bl_y }, direction: Direction::RL},
            Edge { start: Tile { x: bl_x, y: bl_y }, end: Tile {x: tl_x, y: tl_y}, direction: Direction::BT}
        ];
        println!("edges: {:?}", v);
        v
    }
}

impl PartialOrd for Rectangle<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.area_of_rectangle() < other.area_of_rectangle() {
            Some(std::cmp::Ordering::Less)
        } else if self.area_of_rectangle() > other.area_of_rectangle() {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}

impl Ord for Rectangle<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.area_of_rectangle() < other.area_of_rectangle() {
            std::cmp::Ordering::Less
        } else if self.area_of_rectangle() > other.area_of_rectangle() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LR, //Left->Right
    RL, //Right-Left
    TB, //Top-Bottom
    BT  //Bottom-top
}

impl Direction {
    /**
     * "Turns" the direction 90 degrees clockwise and returns the new direction
     */
    pub fn turn(&self) -> Self {
        match self {
            Self::LR=>Self::TB,
            Self::TB=>Self::RL,
            Self::RL=>Self::BT,
            Self::BT=>Self::LR
        }
    }

    /**
     * Returns true if this direction is the exact inverse of the one given
     */
    pub fn is_inverse(&self, other:&Direction) -> bool {
        match self {
            Self::LR=>*other==Self::RL,
            Self::TB=>*other==Self::BT,
            Self::RL=>*other==Self::LR,
            Self::BT=>*other==Self::TB
        }
    }
}

/**
 * Edge defines a vector that makes up a permimeter - including a direction, so we can 
 * determine "inside" and "outside"
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    start: Tile,
    end: Tile,
    direction: Direction
}

impl Edge {
    /**
     * Checks if a given point lies within the polygon defined by this vector.
     * This assumes that the polygon was constructed in a clockwise-following manner;
     * if it was anti-clockwise then the test must be reversed
     */
    pub fn is_inside(&self, point: &Tile) -> bool {
        match self.direction {
            Direction::LR=>point.y >= self.start.y,
            Direction::RL=>point.y <= self.start.y,
            Direction::TB=>point.x <= self.start.x,
            Direction::BT=>point.x >= self.start.x
        }
    }

    pub fn length(&self) -> i64 {
        let x = (self.end.x - self.start.x).pow(2);
        let y = (self.end.y - self.start.y).pow(2);
        (x + y).isqrt()
    }

    /**
     * Checks if this edge is perpendicular to the other one.
     * Note, this assumes that the edges are axis-aligned
     */
    pub fn is_perpendicular(&self, other: &Edge) -> bool {
        (self.start.x==self.end.x && other.start.y==other.end.y) ||
        (self.start.y==self.end.y && other.start.x==other.end.x)
    }

    /**
     * Checks if this edge intersects the other edge.  Touching does _not_ count as an intersection.
     */
    pub fn intersects(&self, other: &Edge) -> bool {
       // println!("Checking intersection of {:?} with {:?}", self, other );
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        if !self.is_perpendicular(other) {  //2d grid aligned vectors - if not perpendicular we are parallel so never cross
            false
        } else {
            if min_x==max_x && 
                other.start.x.min(other.end.x) < min_x && 
                other.end.x.max(other.start.x) > max_x &&
                min_y < other.start.y.min(other.end.y) &&
                max_y > other.end.y.max(other.start.y)
                { //horizontal case so other is vertical
                //println!("horizontal intersection between {}->{} and {}->{}", self.start.y, self.end.y, other.start.y, other.end.y);
                true
            } else if min_y==max_y && 
                other.start.y.min(other.end.y) < min_y && 
                other.end.y.max(other.start.y) > max_y &&
                min_x < other.start.x.min(other.end.x) &&
                max_x > other.end.x.max(other.start.x) { //horizontal case so other is vertical
                //println!("vertical intersection {} {} {} {}", self.start.x, other.start.x, self.end.x, other.end.x);
                true
            } else {
                false
            }
        }
    }

    pub fn new(start: &Tile, end: &Tile) -> Edge {
        let direction = if start.x==end.x {   //vertical case
            if start.y > end.y {    //start is lower than end
                Direction::BT 
            } else if start.y==end.y {
                panic!("the provided start and edge points describe a zero-length edge");
            } else {
                Direction::TB
            }
        } else if start.y==end.y { //horizontal case
            if start.x > end.x { //start is to the right of end
                Direction::RL
            } else if start.x==end.x {
                panic!("the provided start and edge points describe a zero-length edge");
            } else {
                Direction::LR
            }
        } else {
            panic!("the provided start and end points do not describe a correctly aligned edge");
        };

        Edge { start: *start, end: *end, direction }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Perimeter {
    edges: Vec<Edge>
}

impl Perimeter {
    pub fn is_inside(&self, rect:&Rectangle) -> bool {
        //A rectangle is inside the perimeter if: (a) no edges of the rectangle intersect the perimeter and (b) a ray cast to the outer axis intersects an odd number of perimeter edges

        //Test a: no edges of the rectangle intersect the perimeter
        if rect.edges().iter().any(|re| {
            let v = self.edges.par_iter().any(|pe| pe.intersects(re));
            //println!("{}",v);
            v
        }) {
            //println!("at least 1 edge intersected");
            return false
        }

        //Test b: a ray cast to the outer axis intersects an odd number of perimeter edges
        //We're going to need to cast from a point _just inside_ each corner of the rectangle
        if rect.edges().iter().any(|e| e.length()<3) {
            //If the edges are not long enough to nudge in, use a center-point test
            let test_x = (rect.tile_a.x + rect.tile_b.x) / 2;
            let test_y = (rect.tile_a.y + rect.tile_b.y) / 2; 

            let edge_crossings = self.edges.par_iter()
                .filter(|pe| pe.direction==Direction::TB || pe.direction==Direction::BT)
                .filter(|pe| {
                    // The edge must be to the right of our point
                    let edge_x = pe.start.x; // Vertical edges have constant X
                    if edge_x <= test_x {
                        return false;
                    }

                    // Get the Y-range of this vertical edge
                    let edge_min_y = pe.start.y.min(pe.end.y);
                    let edge_max_y = pe.start.y.max(pe.end.y);
                    
                    // Check if our ray's Y-coordinate intersects the edge's Y-range
                    // Use < and >= to handle edge cases consistently
                    test_y >= edge_min_y && test_y < edge_max_y
                })
                .count();
            edge_crossings % 2 == 1
        } else {
            rect.corners(1).par_iter().all(|corner| {
                let edge_crossings = self.edges.par_iter()
                    .filter(|pe| pe.direction==Direction::TB || pe.direction==Direction::BT)
                    .filter(|pe| {
                        let test_x = corner.x;
                        let test_y = corner.y;

                        // The edge must be to the right of our point
                        let edge_x = pe.start.x; // Vertical edges have constant X
                        if edge_x <= test_x {
                            return false;
                        }
                        
                        // Get the Y-range of this vertical edge
                        let edge_min_y = pe.start.y.min(pe.end.y);
                        let edge_max_y = pe.start.y.max(pe.end.y);
                        
                        // Check if our ray's Y-coordinate intersects the edge's Y-range
                        // Use < and >= to handle edge cases consistently
                        test_y >= edge_min_y && test_y < edge_max_y
                    })
                    .count();
                //println!("Got {} edge crossings for {:?}->{:?})", edge_crossings, rect.tile_a, rect.tile_b);
                edge_crossings % 2 == 1
            })
        }
    }

    //Note, the set _must not be empty_ otherwise this will panic
    fn find_topleft<'a>(set: &'a Vec<Tile>) -> &'a Tile {
        let mut min:&'a Tile;

        match set.split_first() {
            Some((first, others))=>{
                min = first;

                for tile in others {
                    if tile.x <= min.x && tile.y <= min.y {
                        min = tile;
                    }
                }
            },
            None=>panic!("cannot find topleft with an empty set")
        }

        min
    }

    fn next_controlpoint<'a, 'b>(current:&'a Tile, set:&'b HashSet<&'a Tile>, direction:Direction) -> Option<&'a Tile> {
        match direction {
            Direction::LR=>{
                //If traversing left-right we can only move to another point on the same row (y)
                let candidates:Vec<&&Tile> = set.iter().filter(|tile| tile.y==current.y).collect();

                //Find any points to the right
                let mut ordered_candidates:Vec<(i64, &&Tile)> = candidates.into_iter()
                    .filter_map(|point| {
                        let dist = point.x - current.x; //we are looking for points to the right, i.e. in increasing x
                        if dist>0 {
                            Some((dist, point))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                //Order by distance
                ordered_candidates.sort_by(|(dist_a, _), (dist_b, _)| {
                    dist_a.cmp(dist_b)
                });

                //Take the first
                ordered_candidates.first().map(|(_, tile)| &***tile)
            },
            Direction::RL=>{
                //If traversing right-left we can only move to another point on the same row (y)
                let candidates:Vec<&&Tile> = set.iter().filter(|tile| tile.y==current.y).collect();

                //Find any points to the right
                let mut ordered_candidates:Vec<(i64, &&Tile)> = candidates.into_iter()
                    .filter_map(|point| {
                        let dist = current.x-point.x; //we are looking for points to the left, i.e. in decreasing x
                        if dist>0 {
                            Some((dist, point))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                //Order by distance
                ordered_candidates.sort_by(|(dist_a, _), (dist_b, _)| {
                    dist_a.cmp(dist_b)
                });

                //Take the first
                ordered_candidates.first().map(|(_, tile)| &***tile)
            },
            Direction::TB=>{
                //If traversing top-bottom we can only move to another point on the same column(x)
                let candidates:Vec<&&Tile> = set.iter().filter(|tile| tile.x==current.x).collect();

                let mut ordered_candidates:Vec<(i64, &&Tile)> = candidates.into_iter()
                    .filter_map(|point| {
                        let dist = point.y - current.y; //we are looking for points below, i.e. in increasing y
                        if dist > 0 {
                            Some((dist, point))
                        } else {
                            None
                        }
                    })
                    .collect();

                ordered_candidates.sort_by(|(dist_a, _), (dist_b, _)| {
                    dist_a.cmp(dist_b)
                });

                ordered_candidates.first().map(|(_, tile)| &***tile)
            },
            Direction::BT=>{
                //If traversing bottom-top we can only move to another point on the same column(x)
                let candidates:Vec<&&Tile> = set.iter().filter(|tile| tile.x==current.x).collect();

                let mut ordered_candidates:Vec<(i64, &&Tile)> = candidates.into_iter()
                    .filter_map(|point| {
                        let dist = current.y - point.y; //we are looking for points above, i.e. in decreasing y
                        if dist > 0 {
                            Some((dist, point))
                        } else {
                            None
                        }
                    })
                    .collect();

                ordered_candidates.sort_by(|(dist_a, _), (dist_b, _)| {
                    dist_a.cmp(dist_b)
                });

                ordered_candidates.first().map(|(_, tile)| &***tile)
            }
        }
    }

    /**
     * Constructs a perimeter from the given control points
     */
    pub fn new<'a>(control_points: &'a Vec<Tile>) -> Option<Perimeter> {
        if control_points.len() < 2 {
            return None
        }

        let mut edges: Vec<Edge> = vec![];
        
        //Let's assume that the control_points vec is already correctly ordered
        let (start_point, remaining_points) = control_points.split_first().unwrap();    //unwrap is safe as we already checked is_empty
        let mut next_point = start_point;
        for cp in remaining_points {
            let next_edge = Edge::new(next_point, cp);
            edges.push(next_edge);
            next_point = cp;
        }
        let end = control_points.last().unwrap();
        let final_edge = Edge::new(end, start_point);
        edges.push(final_edge);

        Some(Perimeter { edges })
    }
}

pub fn pair_up<'a>(tiles: &'a Vec<Tile>) -> Vec<Rectangle<'a>> {
    let top:usize = tiles.len();

    (0_usize..top).into_par_iter()
        .flat_map(|i| {
            match tiles[i..top].split_first() {
                Some((tile_a, others))=>{
                    others.iter().map(|tile_b| {
                        Rectangle::new(tile_a, tile_b)
                    }).collect()
                },
                None=>{
                    panic!("improperly configured tile list, this should not happen");
                    vec![]
                }
            }
        })
        .collect()
}

pub fn parse_input(input:&str) -> Result<Vec<Tile>, Box<dyn Error>> {
    input.lines()
        .into_iter()
        .map(|l| Tile::from_string(l))
        .collect()
}

fn main() ->Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let tiles = parse_input(&content)?;
    let mut pairs = pair_up(&tiles);
    pairs.sort();

    match pairs.last() {
        Some(last_pair)=>println!("The largest area is {}", last_pair.area_of_rectangle()),
        None=>println!("ERROR! The list of pairs was empty :-/")
    }

    let perimeter = Perimeter::new(&tiles).expect("Provided points did not form a closed perimeter");

    let mut pairs_in_perim:Vec<&Rectangle<'_>> = pairs.par_iter()
        .filter(|rec| perimeter.is_inside(rec))
        .collect();
    pairs_in_perim.sort_by(|ra, rb| ra.area_of_rectangle().cmp(&rb.area_of_rectangle()));
    match pairs_in_perim.last() {
        Some(last_pair)=>println!("The largest area in the perimeter is {}", last_pair.area_of_rectangle()),
        None=>println!("ERROR! There were no pairs inside the perimeter :-/")
    }

    Ok( () )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example() {
        let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";
        let tiles = parse_input(&input).unwrap();
        let mut pairs = pair_up(&tiles);
        pairs.sort();

        pairs.iter().for_each(|p| {
            println!("({}, {}) -> ({}, {}): area {}", p.tile_a.x, p.tile_a.y, p.tile_b.x, p.tile_b.y, p.area_of_rectangle())
        });
        assert_eq!(pairs.last().unwrap().area_of_rectangle(), 50);
    }

    #[test]
    fn test_find_topleft() {
        let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";
        let tiles = parse_input(&input).unwrap();

        let topleft = Perimeter::find_topleft(&tiles);
        assert_eq!(topleft, &Tile{ x:7, y:1 });
    }

    #[test]
    fn test_next_controlpoint() {
        let input = "11,1
11,7
9,7
9,5
2,5
2,3
7,3";
        let tiles = parse_input(&input).unwrap();
        let tile_set: HashSet<&Tile, RandomState> = HashSet::from_iter(tiles.iter());

        let next = Perimeter::next_controlpoint(&Tile{ x:7, y:1}, &tile_set, Direction::LR);
        assert!(next.is_some());
        let next_tile = next.unwrap();
        assert_eq!(next_tile, &Tile { x:11, y: 1});
    }

    #[test]
    fn test_perimeter() {
        let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";
        let tiles = parse_input(&input).unwrap();

        let perimeter = Perimeter::new(&tiles);
        assert!(perimeter.is_some());
        let perimeter = perimeter.unwrap();
        assert_eq!(perimeter.edges.len(), 8);
        assert_eq!(perimeter.edges[0], Edge { start: Tile { x: 7, y: 1 }, end: Tile { x: 11, y: 1}, direction: Direction::LR});
        assert_eq!(perimeter.edges[1], Edge { start: Tile { x: 11, y: 1}, end: Tile { x: 11, y: 7}, direction: Direction::TB});
        assert_eq!(perimeter.edges[2], Edge { start: Tile { x: 11, y: 7 }, end: Tile { x: 9, y: 7}, direction: Direction::RL});
        assert_eq!(perimeter.edges[3], Edge { start: Tile { x: 9, y: 7}, end: Tile { x: 9, y: 5}, direction: Direction::BT});
        assert_eq!(perimeter.edges[4], Edge { start: Tile { x: 9, y: 5 }, end: Tile { x:2, y: 5}, direction: Direction::RL});
        assert_eq!(perimeter.edges[5], Edge { start: Tile { x: 2, y: 5}, end: Tile { x: 2, y: 3}, direction: Direction::BT});
        assert_eq!(perimeter.edges[6], Edge { start: Tile { x: 2, y: 3 }, end: Tile { x: 7, y: 3}, direction: Direction::LR});
        assert_eq!(perimeter.edges[7], Edge { start: Tile { x: 7, y: 3}, end: Tile { x: 7, y: 1}, direction: Direction::BT});
        
    }

    #[test]
    fn test_direction_eq() {
        assert!(Direction::LR==Direction::LR);
        assert!(! (Direction::RL==Direction::LR));
    }

    #[test]
    fn test_rectangle_edges() {
        let rect = Rectangle { tile_a: &Tile {x: 1, y:1}, tile_b: &Tile {x: 4, y:4}};
        let edges = rect.edges();

        assert_eq!(edges.len(), 4);
        assert_eq!(edges[0], Edge { start: Tile { x: 1, y: 1 }, end: Tile {x: 4, y: 1}, direction: Direction::LR});
        assert_eq!(edges[1], Edge { start: Tile { x: 4, y: 1 }, end: Tile {x: 4, y: 4}, direction: Direction::TB});
        assert_eq!(edges[2], Edge { start: Tile { x: 4, y: 4 }, end: Tile {x: 1, y: 4}, direction: Direction::RL});
        assert_eq!(edges[3], Edge { start: Tile { x: 1, y: 4 }, end: Tile {x: 1, y: 1}, direction: Direction::BT});
    }

    #[test]
    fn test_edge_perpendicular() {
        let e1 = Edge { start: Tile { x:3, y: 3}, end: Tile { x:5, y: 3}, direction: Direction::LR};
        let e2 = Edge { start: Tile { x:4, y: 2}, end: Tile { x:4, y: 9}, direction: Direction::TB};
        let e3 = Edge { start: Tile { x:5, y: 5}, end: Tile { x:9, y: 5}, direction: Direction::RL};
        
        assert!(e1.is_perpendicular(&e2));
        assert!(! e3.is_perpendicular(&e1));
    }

    #[test]
    fn test_edge_intersection() {
        let e1 = Edge { start: Tile { x:3, y: 3}, end: Tile { x:5, y: 3}, direction: Direction::LR};
        let e2 = Edge { start: Tile { x:5, y: 4}, end: Tile { x:5, y: 9}, direction: Direction::TB};
        let e3 = Edge { start: Tile { x:4, y: 2}, end: Tile { x:4, y: 6}, direction: Direction::TB};

        assert!(! e1.intersects(&e2));
        assert!(! e2.intersects(&e1));
        assert!(e3.intersects(&e1));
        assert!(e1.intersects(&e3));
    }

    #[test]
    fn test_perimeter_is_inside() {
        let input = "7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";
        let tiles = parse_input(&input).unwrap();

        let perimeter = Perimeter::new(&tiles);
        assert!(perimeter.is_some());
        let perimeter = perimeter.unwrap();

        let rects = vec![
            Rectangle::new(&Tile { x: 7, y: 3}, &Tile {x:11, y:1}),
            Rectangle::new(&Tile { x: 9, y: 7}, &Tile {x:9, y:5}),
            Rectangle::new(&Tile { x: 9, y: 5}, &Tile {x:2, y:3}),
        ];

        assert!(rects.par_iter().all(|r| perimeter.is_inside(r)));

    }

    /// A rectangle whose centre is inside the perimeter but whose right edge
    /// extends beyond the perimeter boundary. With correct logic this should
    /// be classified as outside, but the current implementation will report
    /// it as inside, exposing the limitation described in the discussion.
    #[test]
    fn test_perimeter_is_inside_overhangs_outside() {
        // Simple axis-aligned rectangular perimeter from (0,0) to (10,10)
        let input = "0,0
10,0
10,10
0,10";
        let tiles = parse_input(&input).unwrap();

        let perimeter = Perimeter::new(&tiles).unwrap();

        // Rectangle that shares three sides with the perimeter but extends
        // one unit beyond the right edge to x=11. Its centre is at (5,5),
        // which lies inside the perimeter, but part of the rectangle is
        // outside, so a correct implementation should return false here.
        let rect = Rectangle::new(&Tile { x: 0, y: 0 }, &Tile { x: 11, y: 10 });

        assert!(!perimeter.is_inside(&rect));
    }
}