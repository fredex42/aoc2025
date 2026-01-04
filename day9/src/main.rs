use std::{collections::HashSet, error::Error, fs::File, hash::RandomState, io::Read, mem::take};
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TilePair {
    tile_a: Tile,
    tile_b: Tile
}

impl TilePair {
    pub fn new(tile_a: &Tile, tile_b: &Tile) -> TilePair {
        TilePair { tile_a: *tile_a, tile_b: *tile_b }
    }

    pub fn area_of_rectangle(&self) -> u64 {
        (
            //(x1-x2)*(y1-y2) does not include the last row and column because the co-ordinates are exclusive.
            //We want the area _bounded by_ the co-ordinates inclusively, i.e. include 1 extra column on the end, 
            //and 1 extra row on the end
            //
            ((self.tile_a.x - self.tile_b.x).abs()+ 1) * ((self.tile_a.y - self.tile_b.y).abs() + 1)
        ).try_into().expect("there was an integer overflow calculating area")
    }

    pub fn corners_of_rectangle(&self) -> Vec<Tile> {
        vec![
            Tile { x: self.tile_a.x, y: self.tile_b.y},
            Tile { x: self.tile_a.x, y: self.tile_a.y},
            Tile { x: self.tile_b.x, y: self.tile_a.y},
            Tile { x: self.tile_b.x, y: self.tile_b.y}
        ]
    }

    pub fn diagonals_of_rectangle(&self) -> Vec<Edge> {
        vec![
            Edge { 
                start: Tile { x: self.tile_a.x.min(self.tile_b.x), y: self.tile_a.y.min(self.tile_b.y)},
                end: Tile { x: self.tile_a.x.max(self.tile_b.x), y: self.tile_a.y.max(self.tile_b.y)},
                direction: Direction::Diag
            },
            Edge {
                start: Tile { x: self.tile_a.x.min(self.tile_b.x), y: self.tile_a.y.max(self.tile_b.y)},
                end: Tile { x: self.tile_a.x.max(self.tile_b.x), y: self.tile_a.y.min(self.tile_b.y)},
                direction: Direction::Diag
            }
        ]
    }
}

impl PartialOrd for TilePair {
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

impl Ord for TilePair {
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
    BT,  //Bottom-top
    Diag
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
            Self::BT=>Self::LR,
            Self::Diag=>Self::Diag
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
            Self::BT=>*other==Self::TB,
            Self::Diag=>false
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
    // /**
    //  * Checks if a given point lies within the polygon defined by this vector.
    //  * This assumes that the polygon was constructed in a clockwise-following manner;
    //  * if it was anti-clockwise then the test must be reversed
    //  */
    // pub fn is_inside(&self, point: &Tile) -> bool {
    //     println!("checking {:?} against {:?}", point, self);

    //     match self.direction {
    //         Direction::LR=>point.y >= self.start.y,
    //         Direction::RL=>point.y <= self.start.y,
    //         Direction::TB=>point.x <= self.start.x,
    //         Direction::BT=>point.x >= self.start.x
    //     }
    // }

    //When ray-casting, the interval must be half-open otherwise we can double count
    pub fn x_in_range(&self, point: &Tile) -> bool {
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);

        point.x >= min_x && point.x <= max_x
    }

    pub fn y_in_range(&self, point: &Tile) -> bool {
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        point.y >= min_y && point.y <= max_y
    }

    //Return a list of points in this line, at integer resolution
    pub fn points(&self) -> Vec<Tile> {
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        (min_x..max_x).flat_map(|x| {
            (min_y..max_y).map(move |y| Tile {x, y})
        }).collect()
    }

    pub fn minimal_points(&self) -> Vec<Tile> {
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        //Try for 
        let first_try:Vec<Tile> = vec![
            Tile { x: min_x+1, y: min_y+1},
            Tile { x: max_x-1, y: min_y+1},
            Tile { x: max_x-1, y: max_y-1},
            Tile { x: min_x+1, y: max_y-1},
            Tile { x: (min_x + max_x)/2, y: (min_y + max_y)/2}
        ]
            .into_iter()
            .filter(|tile| tile.x>=min_x && tile.x<=max_x && tile.y>=min_y &&tile.y<=max_y).collect();

        if first_try.is_empty() {
            vec![
                Tile {x: min_x, y:min_y},
                Tile {x: max_x, y: max_y}
            ]
        } else {
            first_try
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Perimeter {
    edges: Vec<Edge>,
    x_max: i64,
    y_max: i64
}

impl Perimeter {
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
            },
            Direction::Diag=>{
                panic!("There should not be diagonals in the perimeter")
            }
        }
    }

    /**
     * Checks if the given point is inside the polygon
     */
    pub fn sits_inside(&self, point: &Tile) -> bool {
        let mut crossings = 0;
        let mut on_edge = false;

        //We project an imaginary "ray" to the right.  This means we are only measuring intersections with the _vertical_ edges and
        //can exclude _horizontal_ edges
        for edge in self.edges.iter().filter(|edge| edge.direction==Direction::TB || edge.direction==Direction::BT) {
            let (x1, y1) = (edge.start.x as f64, edge.start.y as f64);
            let (x2, y2) = (edge.end.x as f64, edge.end.y as f64);

            // // Check if the horizontal ray intersects this edge.  This does not handle the case where the point is _on_ the edge.
            // let intersects = (edge.start.y > point.y) != (edge.end.y > point.y) //y coord of point must lie in the edge so ray intersects
            //     && (point.x as f64) < (x2 - x1) * (point.y as f64 - y1) / (y2 - y1) + x1;   //x coord of point must be less than the intersection point. 
            //                                                                 //Calculate intersection point by solving parametric co-ordinates of the ray

            // if intersects {
            //     crossings += 1;
            // }
            
            //The case where point.x==edge.start.x is explicitly handled below
            if point.x < edge.start.x && ((edge.start.y <= point.y && edge.end.y > point.y) || (edge.end.y <= point.y && edge.start.y > point.y)) {   //edge.start.x === edge.end.x as the direction is vertical
                crossings += 1
            }

            //We lie on the edge, if P is between start and end, AND it lies on a straight line between the two
            //The second of those conditions is determined by the area of the related triangle being 0, but is implicit if the edges are axis aligned
            if edge.x_in_range(point) && edge.y_in_range(point) {
                on_edge = true;
                break;
            }
        }

        crossings % 2 == 1 || on_edge
    }

    pub fn rectangle_sits_inside(&self, pair: &TilePair) -> bool {
        //pair.corners_of_rectangle().par_iter().all(|corner| self.sits_inside(corner))
        pair.diagonals_of_rectangle().par_iter()
            .flat_map(|d| d.minimal_points())
            .all(|point| self.sits_inside(&point))
    }

    /**
     * Constructs a perimeter from the given control points
     */
    pub fn new<'a>(control_points: &'a Vec<Tile>) -> Option<Perimeter> {
        let mut edges: Vec<Edge> = vec![];

        let mut cp_set:HashSet<&Tile, RandomState> = HashSet::from_iter(control_points.iter());
        if cp_set.is_empty() {
            return None
        }

        //Find the top-left of the incoming control points
        let start_point = Self::find_topleft(control_points);
        //Don't remove the initial controlpoint from the set. It should be the last one we come to.
        let mut current = start_point;

        let mut current_direction = Direction::LR;  //start going left-right
        let mut starting_direction = current_direction;
        while ! cp_set.is_empty() {
            match Self::next_controlpoint(current, &cp_set, current_direction) {
                Some(tile)=>{
                    //println!("Found {:?} following {:?} going {:?}", tile, current, current_direction);
                    //Great, we found a control point.
                    let next_edge = Edge { start: *current, end: *tile, direction: current_direction };
                    edges.push(next_edge);
                    //If we got back to where we started, we have a perimeter!
                    if tile == start_point {
                        break;
                    }
                    //Remove this now connected control point from the set and iterate to find the next one
                    cp_set.remove(tile);
                    current = tile;
                    starting_direction = current_direction;
                    //Look for another control point 90 degrees further around the circle
                    current_direction = current_direction.turn();
                },
                None=>{
                    //println!("Nothing found for {:?} going {:?}. Starting direction was {:?}", current, current_direction, starting_direction);

                    current_direction = current_direction.turn();
                    if current_direction==starting_direction {
                        //We went through 360 degrees without finding any control point to link to.
                        //Crucially we did check if there was another point on the same line
                        println!("ERROR! Ran out of valid control points after {:?}", current);
                        return None
                    } else if current_direction.is_inverse(&starting_direction) {
                        //Don't traverse back the way we came otherwise we get stuck in a loop. Nudge on to the next possible direction
                        current_direction = current_direction.turn();
                    }
                }
            }
        }

        let x_max:i64 = control_points.iter().map(|cp| cp.x).max().unwrap();
        let y_max:i64 = control_points.iter().map(|cp| cp.y).max().unwrap();
        Some(Perimeter { edges, x_max, y_max })
    }
}

pub fn pair_up<'a>(tiles: &'a Vec<Tile>) -> Vec<TilePair> {
    let top:usize = tiles.len();

    (0_usize..top).into_par_iter()
        .flat_map(|i| {
            match tiles[i..top].split_first() {
                Some((tile_a, others))=>{
                    others.iter().map(|tile_b| {
                        TilePair::new(tile_a, tile_b)
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
    pairs.reverse();

    match pairs.first() {
        Some(last_pair)=>println!("The largest area is {}", last_pair.area_of_rectangle()),
        None=>println!("ERROR! The list of pairs was empty :-/")
    }

    let total_count = pairs.len();

    let permimeter = Perimeter::new(&tiles).expect("Could not join points into a perimeter");
    // let valid_rectangles:Vec<&TilePair> = pairs.iter().filter(|rec| permimeter.rectangle_sits_inside(rec)).collect();

    // valid_rectangles.iter().for_each(|rec| {
    //     println!("{:?} -> {:?}; {}", rec.tile_a, rec.tile_b, rec.area_of_rectangle());
    // });
    // match valid_rectangles.first() {
    //     Some(last_pair)=>println!("The largest rectangle inside the perimeter has an area of {}", last_pair.area_of_rectangle()),
    //     None=>println!("ERROR! No rectangles lay within the perimeter :-/")
    // }

    println!("{} rectangles to check", total_count);
    let mut i = 0;

    //Since the rectangles are already sorted by area, we just need to find the first one that fits
    let largest_in_perim = pairs.iter().filter(|rec| {
        i += 1;
        // if (i % 10)==0 {
            println!("{} %", ((i as f64) / (total_count as f64)) * 100.0);
        //}
        permimeter.rectangle_sits_inside(rec)
    }).next();
    match largest_in_perim {
        Some(last_pair)=>println!("The largest rectangle inside the perimeter has an area of {}", last_pair.area_of_rectangle()),
        None=>println!("ERROR! No rectangles lay within the perimeter :-/")
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
    fn test_sits_inside() {
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
        assert!(perimeter.sits_inside(&Tile { x: 4, y: 4}));
        assert!(perimeter.sits_inside(&Tile { x: 10, y: 5}));
        assert!(!perimeter.sits_inside(&Tile { x: 0, y: 0}));
        assert!(!perimeter.sits_inside(&Tile { x: 3, y: 7}));
    }

    #[test]
    fn test_rectangle_sits_inside() {
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

        let rec1 = TilePair::new(&Tile { x: 7, y:3 }, &Tile {x: 11, y: 1});
        assert!(perimeter.rectangle_sits_inside(&rec1));
        let rec2 = TilePair::new(&Tile { x: 9, y:7 }, &Tile {x: 9, y: 5});
        assert!(perimeter.rectangle_sits_inside(&rec2));
        let rec3 = TilePair::new(&Tile { x: 2, y:1 }, &Tile {x: 6, y: 4});
        assert!(! perimeter.rectangle_sits_inside(&rec3));
        let rec4 = TilePair::new(&Tile { x: 5, y:6 }, &Tile {x: 10, y: 4});
        assert!(! perimeter.rectangle_sits_inside(&rec4));
    }

    #[test]
    fn test_example_2() {
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
        let perimeter = Perimeter::new(&tiles).expect("Could not join all points into a perimeter");

        pairs.sort();
        pairs.reverse();
        let valid_pairs:Vec<&TilePair> = pairs.iter().filter(|rect| perimeter.rectangle_sits_inside(rect)).collect();

        let result = valid_pairs.first().expect("No rectangles were valid in the perimeter");
        assert_eq!(result.area_of_rectangle(), 24);
    }

    #[test]
    fn test_direction_eq() {
        assert!(Direction::LR==Direction::LR);
        assert!(! (Direction::RL==Direction::LR));
    }
}