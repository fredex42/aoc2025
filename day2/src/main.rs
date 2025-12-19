use std::{error::Error, fs::File, io::Read};
use regex::Regex;

#[derive(Debug)]
pub struct ProductIdRange {
    start: u64,
    end: u64
}

fn split_into_n_segments(s: &str, n: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut segments = Vec::with_capacity(n);

    let mut start = 0;
    let base_size = len / n;
    let remainder = len % n;

    for i in 0..n {
        let extra = if i < remainder { 1 } else { 0 };
        let end = start + base_size + extra;
        segments.push(chars[start..end].iter().collect());
        start = end;
    }

    segments
}

impl ProductIdRange {
        /**
         * OK. The instructions say:
         * Since the young Elf was just doing silly patterns, you can find the invalid IDs by looking for any ID which is made only 
         * of some sequence of digits repeated twice. So, 55 (5 twice), 6464 (64 twice), and 123123 (123 twice) would all be invalid IDs.
        */
    fn is_borken(id: &u64) -> bool {
        let id_str= id.to_string();
        //println!("is_borken testing {}", id_str);

        let len = id_str.len();
        if len <2 || (len % 2) !=0 { //we can't get a repeating pattern if it is not long enough, or if the number is odd
            false
        } else {
            for chunk_count in 2..len+1 {
                //Test the ID.  We start by splitting in half, then checking if the two halves are equal to each other.
                //If so we return true; if not, we reduce the half-length and try again.
                //We keep going until we find a point at which all splits are equal or we run out of string
                let parts= split_into_n_segments(&id_str, chunk_count);
                //println!(". at {} parts are {:?}", chunk_count, parts);
                let matches = match parts.first() {
                    Some(first)=>{
                        //println!("elem is {}",first);
                        if *first != id_str {
                            parts.iter().all(|ent| ent==first)
                        } else {
                            false
                        }
                    },
                    None=>false
                };
                if matches {
                    return true
                }
            }
            false
        }
    }

    pub fn find_broken_ids(&self) -> Vec<u64> {
        (self.start..self.end)
            .filter(ProductIdRange::is_borken)
            .collect()
    }

    pub fn from_string(input: &str) -> Result<ProductIdRange, Box<dyn Error>> {
        let splitter = Regex::new(r"(\d+)-(\d+)")?;
        match splitter.captures(input).map(|c| c.extract()) {
            Some((_, [start_str, end_str]))=>{
                let start = start_str.parse::<u64>()?;
                let end = end_str.parse::<u64>()? + 1;
                Ok(ProductIdRange { start, end })
            },
            None=>Err(format!("Input line {} was improperly formatted", input).into())
        }
    }
}

pub fn parse_input(input: &str) -> Result<Vec<ProductIdRange>, Box<dyn Error>> {
    input
        .split(",")
        .into_iter()
        .map(|s| ProductIdRange::from_string(s))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    
    let ranges = parse_input(&content)?;
    println!("Got {} ranges to check", ranges.len());

    let broken:Vec<u64> = ranges.iter().map(|r| r.find_broken_ids()).flatten().collect();
    println!("Found {} broken ids:", broken.len());
    broken.iter().for_each(|id| println!("  {}", id));
    let sum:u64 = broken.iter().sum();
    println!("The total was {:?}", sum);
    Ok( () )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_broken_id_123123() {
        let range = ProductIdRange { start: 123123, end: 123124 };
        let ids = range.find_broken_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 123123);
    }

    #[test]
    fn test_broken_id_6464() {
        let range = ProductIdRange { start: 6464, end: 6465 };
        let ids = range.find_broken_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 6464);
    }

    #[test]
    fn test_broken_id_55() {
        let range = ProductIdRange { start: 55, end: 56 };
        let ids = range.find_broken_ids();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 55);
    }

    #[test]
    fn test_working_id_7654() {
        let range = ProductIdRange { start: 7654, end: 7655 };
        let ids = range.find_broken_ids();
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_parser_inclusive() {
        //The ranges we are given are inclusive, but the domain object range is exclusive.  Therefore end must be 1 more than the end value given
        let r = ProductIdRange::from_string("222220-222224").unwrap();
        assert_eq!(r.start, 222220);
        assert_eq!(r.end, 222225);
    }
    #[test]
    fn test_example() {
        /*
        11-22 has two invalid IDs, 11 and 22.
        95-115 has one invalid ID, 99.
        998-1012 has one invalid ID, 1010.
        1188511880-1188511890 has one invalid ID, 1188511885.
        222220-222224 has one invalid ID, 222222.
        1698522-1698528 contains no invalid IDs.
        446443-446449 has one invalid ID, 446446.
        38593856-38593862 has one invalid ID, 38593859.
        */
        let ranges:Vec<ProductIdRange> = ["11-22","95-115","998-1012","1188511880-1188511890","222220-222224","1698522-1698528","446443-446449","38593856-38593862"]
            .into_iter()
            .map(|s| ProductIdRange::from_string(s))
            .map(|result| result.unwrap())  //meh, we can crash a test :D
            .collect();

        assert_eq!(ranges[0].find_broken_ids(), vec![11,22]);
        assert_eq!(ranges[1].find_broken_ids(), vec![99]);
        assert_eq!(ranges[2].find_broken_ids(), vec![1010]);
        assert_eq!(ranges[3].find_broken_ids(), vec![1188511885]);
        assert_eq!(ranges[4].find_broken_ids(), vec![222222]);
        assert_eq!(ranges[5].find_broken_ids(), vec![]);
        assert_eq!(ranges[6].find_broken_ids(), vec![446446]);
        assert_eq!(ranges[7].find_broken_ids(), vec![38593859]);

        let sum = ranges.into_iter()
            .map(|r| r.find_broken_ids())
            .flatten()
            .reduce(|sum, elem| sum+elem);

        assert_eq!(sum, Some(1227775554));
    }
}