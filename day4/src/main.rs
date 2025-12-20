use std::error::Error;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Copy)]
pub enum Slot {
    Empty,
    Occupied
}

pub struct WarehouseGrid {
    contents: Vec<Vec<Slot>>
}

impl WarehouseGrid {
    /**
     * Populates a WarehouseGrid from string input.  This should be a 2d array of chars like this:
     *  ..@@.@@@@.
        @@@.@.@.@@
        @@@@@.@.@@
        @.@@@@..@.
        @@.@@@@.@@
        .@@@@@@@.@
        .@.@.@.@@@
        @.@@@.@@@@
        .@@@@@@@@.
        @.@.@@@.@.
     * . represents an empty slot and @ represents an occupied slot.
     * Any invalid characters will result in a parsing error
     */
    pub fn from_string(input:&str) -> Result<WarehouseGrid, Box<dyn Error>> {
        let content:Result<Vec<Vec<Slot>>, String> = input
            .split("\n")
            .into_iter()
            .map(|row| {
                let row_content:Result<Vec<Slot>, String> = row.chars().into_iter().map(|ch| match ch {
                    '.'=>Ok(Slot::Empty),
                    '@'=>Ok(Slot::Occupied),
                    other@_=>Err(format!("Unparseable character '{}'", other))
                }).collect();
                row_content
            })
            .filter(|r| match r {
                Err(_)=>true,
                Ok(v)=>v.len() > 0
            })
            .collect();
        match content {
            Ok(c)=>Ok(WarehouseGrid { contents: c }),
            Err(e)=>Err(e.into())
        }
    }

    pub fn at(&self, row:usize, col:usize) -> Option<Slot> {
        self.contents.get(row).map(|r| r.get(col)).flatten().map(|s| *s)
    }

    pub fn height(&self)->usize {
        self.contents.len()
    }

    pub fn width(&self)->usize { 
        match self.contents.first() {
            Some(row)=>row.len(),
            None=>0
        }
    }

    /**
     * The forklifts can only access a roll of paper if there are fewer than four rolls of paper in the eight adjacent positions. 
     * Count how many occupied slots have less than 4 rolls of paper around them
     */
    pub fn count_accessible(&self) -> Result<usize, Box<dyn Error>> {
        match self.contents.first().map(|v| v.len()) {
            Some(width)=>{
                let mut count:usize = 0;
                let height = self.contents.len();
                for col in 0..width {
                    for row in 0..height {
                        match self.at(row, col) {
                            Some(Slot::Occupied)=>{
                                if row==0 || row==height || col==0 || col==width {
                                    //If a slot on the edge is occupied, it's available by definition
                                    count += 1;
                                } else {
                                    let surrounding_count = vec![
                                        self.at(row-1, col-1),
                                        self.at(row, col-1),
                                        self.at(row+1, col-1),
                                        self.at(row-1, col),
                                        self.at(row+1, col),
                                        self.at(row-1, col+1),
                                        self.at(row, col+1),
                                        self.at(row+1, col+1)
                                    ].into_par_iter().filter(|s| match s {
                                        Some(Slot::Occupied)=>true,
                                        _=>false
                                    }).count();
                                    //Instructions say that if there are less than for adjacent occupied slots, the slot is accessible
                                    if surrounding_count < 4 {
                                        count += 1
                                    }
                                }
                            },
                            Some(Slot::Empty)=> {},
                            None=>return Err("Grid was improperly shaped".into())
                        }
                    }
                }
                Ok(count)
            },
            None=>Err("there was no content to search".into())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok( () )
}

#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_example() {
        let grid_desc = "..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
";

        let grid = WarehouseGrid::from_string(grid_desc).unwrap();
        assert_eq!(grid.height(), 10);
        assert_eq!(grid.width(), 10);

        assert_eq!(grid.count_accessible().unwrap(), 13);
    }
}