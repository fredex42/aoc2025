use std::error::Error;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

#[derive(Copy, Clone, PartialEq, Eq)]
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