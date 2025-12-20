use std::{error::Error, fs::File, io::Read};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Slot {
    Empty,
    Occupied
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SlotMobility {
    Empty,
    Accessible,
    Immovable
}

pub struct WarehouseGrid {
    contents: Vec<Vec<Slot>>
}

pub struct WarehouseAvailability {
    contents: Vec<Vec<SlotMobility>>
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

    pub fn at(&self, row:i32, col:i32) -> Option<Slot> {
        if row<0 || col<0 {
            None
        } else {
            self.contents.get(row as usize)?.get(col as usize).copied()
        }
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

    fn count_total(&self) -> usize {
        self.contents.iter().map(|row| row.iter().filter(|slot| match slot {
            Slot::Empty=>false,
            Slot::Occupied=>true
        }).count()).sum()
    }

    fn availability_for(&self, row:i32, col:i32) -> Result<SlotMobility, Box<dyn Error>> {
        match self.at(row, col) {
            Some(Slot::Occupied)=>{
                    let surrounding_count = vec![
                        self.at(row-1, col-1),
                        self.at(row, col-1),
                        self.at(row+1, col-1),
                        self.at(row-1, col),
                        self.at(row+1, col),
                        self.at(row-1, col+1),
                        self.at(row, col+1),
                        self.at(row+1, col+1)
                    ].into_iter().filter(|s| match s {
                        Some(Slot::Occupied)=>true,
                        _=>false
                    }).count();
                    //Instructions say that if there are less than for adjacent occupied slots, the slot is accessible
                    if surrounding_count < 4 {
                        Ok(SlotMobility::Accessible)
                    } else {
                        Ok(SlotMobility::Immovable)
                    }
            },
            Some(Slot::Empty)=> Ok(SlotMobility::Empty),
            None=>return Err("Grid was improperly shaped".into())
        }
    }

    pub fn map_accessible(&self) -> Result<WarehouseAvailability, Box<dyn Error>> {
        match self.contents.first().map(|v| v.len()) {
            None=>Err("there was no content to search".into()),
            Some(width)=>{
                let height = self.contents.len();
                let mut new_cols:Vec<Vec<SlotMobility>> = vec![];

                for row in 0..height {
                    let mut new_row:Vec<SlotMobility> = vec![];
                    for col in 0..width {
                        let availability = self.availability_for(row.try_into().unwrap(), col.try_into().unwrap())?;
                        // if row==0 {
                        //     println!("{}: {:?}", row, availability);
                        // }
                        new_row.push(availability);
                    }
                    new_cols.push(new_row);
                }
                Ok(WarehouseAvailability { contents: new_cols })
            }
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
                        let availability = self.availability_for(row.try_into().unwrap(), col.try_into().unwrap())?;
                        match availability {
                            SlotMobility::Accessible=>count+=1,
                            SlotMobility::Immovable=>{},
                            SlotMobility::Empty=>{}
                        }
                    }
                }
                Ok(count)
            },
            None=>Err("there was no content to search".into())
        }
    }
    
    pub fn render(&self) -> String {
        let mut temp:Vec<String> = vec![];
        for row in 0..self.height() {
            let mut temprow:Vec<char> = vec![];
            for col in 0..self.width() {
                match self.at(row.try_into().unwrap(), col.try_into().unwrap()) {
                    Some(Slot::Empty)=>temprow.push('.'),
                    Some(Slot::Occupied)=>temprow.push('@'),
                    None=>temprow.push('!')
                }
            }
            temp.push(temprow.iter().collect())
        }
        temp.join("\n")
    }
}

impl WarehouseAvailability {
    pub fn at(&self, row:usize, col:usize) -> Option<SlotMobility> {
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

    pub fn render(&self) -> String {
        let mut temp:Vec<String> = vec![];
        for row in 0..self.height() {
            let mut temprow:Vec<char> = vec![];
            for col in 0..self.width() {
                match self.at(row, col) {
                    Some(SlotMobility::Empty)=>temprow.push('.'),
                    Some(SlotMobility::Accessible)=>temprow.push('x'),
                    Some(SlotMobility::Immovable)=>temprow.push('@'),
                    None=>temprow.push('!')
                }
            }
            temp.push(temprow.iter().collect())
        }
        temp.join("\n")
    }

    /**
     * Removes the accessible rolls and returns the new warehouse state
     */
    pub fn next_state(&self) -> WarehouseGrid {
        let contents:Vec<Vec<Slot>> = self.contents.iter().map(|row| {
            row.iter().map(|slot| match slot {
                SlotMobility::Empty=>Slot::Empty,
                SlotMobility::Accessible=>Slot::Empty,
                SlotMobility::Immovable=>Slot::Occupied
            }).collect()
        }).collect();

        WarehouseGrid { contents }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let mut grid = WarehouseGrid::from_string(&content)?;
    let mut i = 0;
    let mut total_moved = 0;

    loop {
        i += 1;
        let accessible_count = grid.count_accessible()?;
        let availability = grid.map_accessible()?;
        grid = availability.next_state();
        total_moved += accessible_count;
        println!("Step {}: there are {} accessible rolls with {} remaining in warehouse", i, accessible_count, grid.count_total());
        if accessible_count==0 {
            break;
        }
    }
    println!("A total of {} rolls were moved", total_moved);
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

    #[test]
    fn test_at() {
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
        assert_eq!(grid.at(0, 0), Some(Slot::Empty));
        assert_eq!(grid.at(0, 1), Some(Slot::Empty));
        assert_eq!(grid.at(0, 2), Some(Slot::Occupied));
        assert_eq!(grid.at(0, 3), Some(Slot::Occupied));
        assert_eq!(grid.at(0, 4), Some(Slot::Empty));
    }

    #[test]
    fn test_read() {
        let grid_desc = "..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.";
        let grid = WarehouseGrid::from_string(grid_desc).unwrap();
        assert_eq!(grid.render(), grid_desc);
    }

    #[test]
    fn test_showmap() {
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

        let availability = grid.map_accessible().unwrap();
        println!("{}", availability.render());
    }

    #[test]
    fn test_state_removal() {
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
        let second_state = "..xx.xx@x.
x@@.@.@.@@
@@@@@.x.@@
@.@@@@..@.
x@.@@@@.@x
.@@@@@@@.@
.@.@.@.@@@
x.@@@.@@@@
.@@@@@@@@.
x.x.@@@.x.";

        let third_state = ".......x..
.@@.x.x.@x
x@@@@...@@
x.@@@@..x.
.@.@@@@.x.
.x@@@@@@.x
.x.@.@.@@@
..@@@.@@@@
.x@@@@@@@.
....@@@...";

        let initial_grid = WarehouseGrid::from_string(grid_desc).unwrap();
        let availability = initial_grid.map_accessible().unwrap();
        assert_eq!(availability.render(), second_state);
        assert_eq!(initial_grid.count_accessible().unwrap(), 13);
        let next_grid = availability.next_state();
        assert_eq!(next_grid.count_accessible().unwrap(), 12);
        let next_availability = next_grid.map_accessible().unwrap().render();
        assert_eq!(next_availability, third_state);
    }
}