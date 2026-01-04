use std::{error::Error, fs::File, io::Read};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
pub struct TilePair<'a> {
    tile_a: &'a Tile,
    tile_b: &'a Tile
}

impl TilePair<'_> {
    pub fn new<'a>(tile_a: &'a Tile, tile_b: &'a Tile) -> TilePair<'a> {
        TilePair { tile_a, tile_b }
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

    pub fn edges_of_rectangle(&self) -> Vec<Edge> {
        let tl = Tile { x: self.tile_a.x.min(self.tile_b.x), y: self.tile_a.y.min(self.tile_b.y)};
        let bl = Tile { x: self.tile_a.x.min(self.tile_b.x), y: self.tile_a.y.max(self.tile_b.y)};
        let tr = Tile { x: self.tile_a.x.max(self.tile_b.x), y: self.tile_a.y.min(self.tile_b.y)};
        let br = Tile { x: self.tile_a.x.max(self.tile_b.x), y: self.tile_b.y.max(self.tile_b.y)};

        vec![
            Edge { start: tl, end: tr},
            Edge { start: bl, end: br},
            Edge { start: tl, end: bl},
            Edge { start: tr, end: br}
        ]
    }

    pub fn centre_of_rectangle(&self) -> Tile {
        Tile {
            x: (self.tile_a.x + self.tile_b.x) / 2,
            y: (self.tile_a.y + self.tile_b.y) / 2
        }
    }
}

impl PartialOrd for TilePair<'_> {
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

impl Ord for TilePair<'_> {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edge {
    start: Tile,
    end: Tile
}

impl Edge {
    pub fn new(start: &Tile, end: &Tile) -> Edge {
        Edge { start: *start, end: *end }
    }

    /**
     * Returns true if the other edge is at 90 degrees to this one.
     * Since all edges in the problem are axis aligned we only check horizontal and vertical
     */
    pub fn is_perpendicular(&self, other: &Edge) -> bool {
        (self.start.x==self.end.x && other.start.y==other.end.y) || (self.start.y==self.end.y && other.start.x==other.end.x)
    }

    pub fn min_x(&self) -> i64 {
        return self.start.x.min(self.end.x)
    }

    pub fn max_x(&self) -> i64 {
        return self.start.x.max(self.end.x)
    }

    pub fn min_y(&self) -> i64 {
        self.start.y.min(self.end.y)
    }

    pub fn max_y(&self) -> i64 {
        self.start.y.max(self.end.y)
    }

    pub fn is_vertical(&self) -> bool {
        self.start.x==self.end.x
    }
}

#[derive(Clone, Debug)]
pub struct Perimeter {
    edges: Vec<Edge>
}

impl Perimeter {
        /**
     * Constructs a perimeter from the given control points
     */
    pub fn new<'a>(control_points: &'a Vec<Tile>) -> Option<Perimeter> {
        if control_points.len() < 3 {
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

    /**
     * Returns true if the given edge crosses any segment of the perimeter.
     * Touching is fine.
     */
    pub fn edge_crosses(&self, edge: &Edge) -> bool {
        self.edges.par_iter()
            .filter(|perim_edge| perim_edge.is_perpendicular(edge))
            .any(|perim_edge| {
                if edge.is_vertical() {
                    let x = edge.start.x;
                    let y = perim_edge.start.y;

                    x > perim_edge.min_x() && x < perim_edge.max_x() &&
                    y > edge.min_y()      && y < edge.max_y()
                } else {
                    let y = edge.start.y;
                    let x = perim_edge.start.x;

                    y > perim_edge.min_y() && y < perim_edge.max_y() &&
                    x > edge.min_x()       && x < edge.max_x()
                }
            })
    }

    /**
     * Test if a point is inside the polygon.  If a line cast to the right intersects an
     * odd number of edges, this sould be the case
     */
    pub fn point_is_inside(&self, point: &Tile) -> bool {
        let edge_crossings = self.edges.par_iter()
            .filter(|edge| {
                edge.is_vertical() && edge.start.x >= point.x &&
                    point.y >= edge.min_y() && point.y < edge.max_y()
            })
            .count();
        edge_crossings % 2 == 1
    }
}

pub fn pair_up<'a>(tiles: &'a Vec<Tile>) -> Vec<TilePair<'a>> {
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

    let perimeter = Perimeter::new(&tiles).expect("Could not construct perimeter");

    let largest_fitting_rect = pairs.iter()
        .filter(|rect| {
            let crosses_perimeter = rect.edges_of_rectangle().par_iter().any(|edge| perimeter.edge_crosses(edge));
            ! crosses_perimeter
        })
        .filter(|rect| {
            let centre = rect.centre_of_rectangle();
            perimeter.point_is_inside(&centre)
        })
        .next();
    
    match largest_fitting_rect {
        Some(rect)=>println!("The largest rectangle that fitted was {}", rect.area_of_rectangle()),
        None=>println!("No available rectangles fitted")
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
}