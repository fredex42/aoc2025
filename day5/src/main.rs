use regex::Regex;
use std::{collections::VecDeque, error::Error, fs::File, io::Read};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/**
 * Represents a range of product IDs, inclusive
 */
#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub struct ProductIdRange {
    start: u64,
    end: u64
}

impl Ord for ProductIdRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for ProductIdRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.start.cmp(&other.start))
    }
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
        //println!("Size of {:?} is {}", self, self.end-self.start+1);
        self.end-self.start+1   //+1 because the range is inclusive
    }

    pub fn overlaps(&self, other:&ProductIdRange) -> bool {
        (self.start >=other.start && self.start <= other.end) || (self.end <= other.end && self.end >= other.start) ||
            (other.start >= self.start && other.start <= self.end) || (other.end <= self.end && other.end >= self.start)
    }

    /**
     * If the two ranges overlap, returns a new range that encompasses both.
     * If they do not overlap, then returns None
     */
    pub fn coalesce(&self, other:&ProductIdRange) -> Option<ProductIdRange> {
        if self.overlaps(other) {
            let start = if self.start<=other.start {
                self.start
             } else {
                other.start
             };
             let end = if self.end>=other.end {
                self.end
             } else {
                other.end
             };

            //println!("{:?} overlaps with {:?} to give {} {}", self, other, start, end);
            Some(ProductIdRange { start, end })
        } else {
            //println!("{:?} does not overlap with {:?}", self, other);
            None
        }
    }
}

/**
 * Consumes a Vec of ProductIdRange, sorts it and coalesces overlapping regions
 */
pub fn coalesce_overlapping_ranges(mut ranges:Vec<ProductIdRange>) -> Vec<ProductIdRange> {
    if ranges.is_empty() {
        return ranges;
    }

    let mut result:Vec<ProductIdRange> = Vec::with_capacity(ranges.len());
    ranges.sort();

    let mut q:VecDeque<ProductIdRange> = ranges.into();

    let mut current:ProductIdRange = q.pop_front().expect("There should be at least one item to coalesce!");
    while !q.is_empty() {
        let next = q.pop_front().expect("this should not happen");
        match current.coalesce(&next) {
            Some(combined)=>current = combined, //The ranges overlap, so combine them and keep going
            None=>{
                //We reached the end of the overlap.  `current` should now be a combination of every overlapping region up to this point
                result.push(current);
                current = next; //Resume starting with the next non-overlapping chunk
            }
        }
    }
    //When we get to the end, we still have the last range in play
    result.push(current);
    result
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

    let deduplicated_ranges= coalesce_overlapping_ranges(ranges);
    let total:u64 = deduplicated_ranges.par_iter().map(|r| r.size()).sum();

    println!("Total fresh ingredients: {}", total);
    Ok( () )
}

#[cfg(test)]
mod test {
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
        // let highest_id:u64 = ranges.iter().fold(0_u64, |max, elem| if max<elem.end {
        //     elem.end
        // } else {
        //     max
        // });

        // let total = (0..highest_id+1).into_par_iter()
        //     .filter(|id| ranges.par_iter().any(|range| range.contains(*id)))
        //     .count();

        //Proper way of doing it... hopefully!
        let deduplicated_ranges = coalesce_overlapping_ranges(ranges);
        let total:u64 = deduplicated_ranges.iter().map(|r| r.size()).sum();
        assert_eq!(total, 14);
    }

    #[test]
    fn test_overlap_complete() {
        //Handle completely overlapping regions
        let range_a = ProductIdRange::from_string("123-456").unwrap();
        let range_b = ProductIdRange::from_string("200-300").unwrap();
        let combined = range_a.coalesce(&range_b);
        assert_eq!(combined, Some(ProductIdRange { start: 123, end: 456 }));
    }

    #[test]
    fn test_overlap_partial_hi() {
        //Handle partially overlapping regions at the high end
        let range_a = ProductIdRange::from_string("123-456").unwrap();
        let range_b = ProductIdRange::from_string("400-500").unwrap();
        let combined = range_a.coalesce(&range_b);
        assert_eq!(combined, Some(ProductIdRange { start: 123, end: 500 }));
    }

    #[test]
    fn test_overlap_partial_lo() {
        //Handle partially overlapping regions at the low end
        let range_a = ProductIdRange::from_string("123-456").unwrap();
        let range_b = ProductIdRange::from_string("100-150").unwrap();
        let combined = range_a.coalesce(&range_b);
        assert_eq!(combined, Some(ProductIdRange { start: 100, end: 456 }));
    }

    #[test]
    fn test_overlap_partial_none() {
        //Handle non-overlapping regions
        let range_a = ProductIdRange::from_string("123-456").unwrap();
        let range_b = ProductIdRange::from_string("567-789").unwrap();
        let combined = range_a.coalesce(&range_b);
        assert_eq!(combined, None);
    }

}