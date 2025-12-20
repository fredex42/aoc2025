use regex::Regex;
use std::{error::Error, fs::File, io::Read};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator, IntoParallelIterator};

/**
 * Represents a range of product IDs, inclusive
 */
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProductIdRange {
    start: u64,
    end: u64
}

impl ProductIdRange {
    pub fn from_string(input:&str) -> Result<ProductIdRange, Box<dyn Error>> {
        let splitter = Regex::new(r"^(\d+)-(\d+)$").unwrap();
        match splitter.captures(input).map(|c| c.extract()) {
            Some((_, [start_str, end_str]))=> {
                match (start_str.parse::<u64>(), end_str.parse::<u64>()) {
                    (Ok(start), Ok(end)) => Ok(
                        ProductIdRange { start, end }
                    ),
                    (_, _)=>Err(format!("a number in '{}' was not valid", input).into())
                }
            },
            None=>Err(format!("the range line '{}' was malformatted", input).into())
        }
    }

    pub fn contains(&self, id:u64) -> bool {
        id >= self.start && id <= self.end
    }

    pub fn size(&self) -> u64 {
        self.end-self.start+1   //+1 because the range is inclusive
    }
}

/**
 * Parses the input file contents, consisting of a set of ranges and a set of IDs to test
 */
pub fn parse(input:&str) -> Result<(Vec<ProductIdRange>, Vec<u64>), Box<dyn Error>> {
    let mut ranges:Vec<ProductIdRange> = vec![];
    let mut ids:Vec<u64> = vec![];
    let mut section:u16 = 0;

    for l in input.split("\n") {
        if l=="" {
            section += 1;
        } else if section==0 {
            let range = ProductIdRange::from_string(l)?;
            ranges.push(range);
        } else if section==1 {
            let id = l.parse::<u64>()?;
            ids.push(id)
        } else {
            if l!="" {
                return Err(format!("unparseable line '{}'", l).into());
            }
        }
    }

    Ok( (ranges, ids) )
}

/**
 * A "spoiled" ingredient is defined as one which does NOT fall into any of the available ranges
 */
fn find_spoiled(ranges:&Vec<ProductIdRange>, ids: &Vec<u64>) -> Vec<u64> {
    ids.par_iter()
        .filter(|id| {
            let is_good = ranges
                .par_iter()
                .any(|range| range.contains(**id));
            ! is_good
        })
        .map(|id| id.to_owned())
        .collect()
}

fn main() ->Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let (ranges, ids) = parse(&content)?;
    let spoiled = find_spoiled(&ranges, &ids);
    //println!("Spoiled ingredient IDs: {}", spoiled.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(";"));

    let fresh_count = ids.len() - spoiled.len();
    println!("Out of a total of {} ingredients, {} are fresh", ids.len(), fresh_count);

    //Bit of a hack... let's find the largest ingredient ID, and just brute-force our way through the
    //lot using Rayon
    let highest_id:u64 = ranges.iter().fold(0_u64, |max, elem| if max<elem.end {
        elem.end
    } else {
        max
    });

    let total = (0..highest_id+1).into_par_iter()
        .filter(|id| ranges.par_iter().any(|range| range.contains(*id)))
        .count();

    println!("Total fresh ingredients: {}", total);
    Ok( () )
}

#[cfg(test)]
mod test {
    use rayon::iter::IntoParallelIterator;

    use super::*;

    #[test]
    fn test_read() {
        let example_data = "3-5
10-14
16-20
12-18

1
5
8
11
17
32
";
        let (ranges, ids) = parse(example_data).unwrap();
        assert_eq!(ranges[0], ProductIdRange { start: 3, end: 5});
        assert_eq!(ranges[1], ProductIdRange { start: 10, end: 14});
        assert_eq!(ranges[2], ProductIdRange { start: 16, end: 20});
        assert_eq!(ranges[3], ProductIdRange { start: 12, end: 18});
        assert_eq!(ids[0], 1);
        assert_eq!(ids[1], 5);
        assert_eq!(ids[2], 8);
        assert_eq!(ids[3], 11);
        assert_eq!(ids[4], 17);
        assert_eq!(ids[5], 32);
        assert_eq!(ranges.len(), 4);
        assert_eq!(ids.len(), 6);
        
    }

    #[test]
    fn test_example() {
        let example_data = "3-5
10-14
16-20
12-18

1
5
8
11
17
32
";
        let (ranges, ids) = parse(example_data).unwrap();
        let spoiled = find_spoiled(&ranges, &ids);
        assert_eq!(spoiled[0], 1);
        assert_eq!(spoiled[1], 8);
        assert_eq!(spoiled[2], 32);
        assert_eq!(spoiled.len(), 3)
    }

    #[test]
    fn test_size() {
        let example_data = "3-5
10-14
16-20
12-18

1
5
8
11
17
32
";
        let (ranges, _) = parse(&example_data).unwrap();
        assert_eq!(ranges[0].size(), 3);    //3, 4, 5
        assert_eq!(ranges[1].size(), 5);    //10, 11, 12, 13, 14

        // //Doesn't work; some ids are in multiple ranges.  So, we need to de-duplicate the ranges first
        // let total:u64 = ranges.iter().map(|r| r.size()).sum();

        //Bit of a hack... let's find the largest ingredient ID, and just brute-force our way through the
        //lot using Rayon
        let highest_id:u64 = ranges.iter().fold(0_u64, |max, elem| if max<elem.end {
            elem.end
        } else {
            max
        });

        let total = (0..highest_id+1).into_par_iter()
            .filter(|id| ranges.par_iter().any(|range| range.contains(*id)))
            .count();

        assert_eq!(total, 14);
    }
}