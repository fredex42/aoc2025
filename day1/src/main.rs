use std::{error::Error, fs::File, io::Read};
use regex::Regex;

pub struct SafeDial {
    pub position: u32,
    size: u32,
    pub zero_counter: u32,
    pub zero_click_counter: u32
}

#[derive(Debug)]
pub enum Movement {
    Left(u32),
    Right(u32)
}

impl SafeDial {
    pub fn new(initial_position: u32, size: u32) -> Self {
        SafeDial { 
            position: initial_position, 
            size,
            zero_counter: 0,
            zero_click_counter: 0
        }
    }

    pub fn turn(&mut self, movement: Movement) {
        let wrapped = match movement {
            Movement::Left(delta)=>{
                let steps = delta % self.size;  //if we go once around then we go to the same place, so we only want remainder
                let whole_rotations = delta / self.size;
                self.zero_click_counter += whole_rotations;
                //println!("{:?}: {} whole rotations and {} steps", movement, whole_rotations, steps);
                let started_at_0 = self.position==0;

                let wrapped = i32::try_from(self.position).unwrap() - i32::try_from(steps).unwrap() < 0;
                if wrapped {    //we are wrapping around at 0
                    self.position = u32::try_from( 
                        i32::try_from(self.position).unwrap() - i32::try_from(steps).unwrap() + i32::try_from(self.size).unwrap()
                    ).unwrap();
                    if !started_at_0 {
                        self.zero_click_counter += 1;
                    }
                } else {
                    self.position -= steps;
                }
                wrapped
            },
            Movement::Right(delta) => {
                let steps = delta % self.size;
                let whole_rotations = delta / self.size;
                self.zero_click_counter += whole_rotations;

                self.position += steps;
                let wrapped =self.position >= self.size;
                if wrapped {
                    //We wrapped around
                    self.zero_click_counter += 1;
                    self.position -= self.size;
                }
                wrapped
            }
        };
        //println!("{:?} goes to {} ({})", movement, self.position, self.zero_click_counter);
        if self.position==0 {
            self.zero_counter += 1;
            if !wrapped {   //don't double-count wraparounds.  If we already detected a zero wrap don't count it again here.
                self.zero_click_counter += 1;
            }
        }
    }
}

pub fn parse_input(file_content: &str) -> Result<Vec<Movement>, Box<dyn Error>> {
    let splitter = Regex::new(r"([LR])(\d+)").unwrap();
    let mut results:Vec<Movement> = vec![];

    for (line, [dirn, steps]) in splitter.captures_iter(file_content).map(|c| c.extract()) {
        match steps.parse::<u32>() {
            Ok(step_count)=>{
                let m = match dirn {
                    "L"=>Ok(Movement::Left(step_count)),
                    "R"=>Ok(Movement::Right(step_count)),
                    _=>Err(format!("invalid direction: {}", dirn))
                };
                match m {
                    Ok(entry)=>results.push(entry),
                    Err(e)=>
                        return Err(format!("Could not parse line '{}': {}", line, e).into())
                }
            },
            Err(e)=>{
                println!("Could not parse line '{}': {}", line, e);
                return Err(e.into());
            }
        }
    }

    Ok(results)
}

fn main()->Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    
    let movements = parse_input(&content)?;
    println!("Loaded {} movements from input", movements.len());
    let mut dial = SafeDial::new(50,100);

    for m in movements {
        dial.turn(m);
    }
    println!("The final position of the dial is {}", dial.position);
    println!("The dial landed on zero {} times", dial.zero_counter);
    println!("The dial passed zero {} times", dial.zero_click_counter);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turns() {
        let mut dial = SafeDial::new(5, 10);
        dial.turn(Movement::Left(1));
        assert_eq!(dial.position, 4);
        dial.turn(Movement::Right(2));
        assert_eq!(dial.position, 6);
        dial.turn(Movement::Left(10));
        assert_eq!(dial.position, 6);
        dial.turn(Movement::Right(25));
        assert_eq!(dial.position, 1);
    }

    #[test]
    fn test_rightedge() {
        let mut dial = SafeDial::new(5, 10);
        dial.turn(Movement::Right(5));
        assert_eq!(dial.position, 0);

        let mut dial = SafeDial::new(9, 10);
        dial.turn(Movement::Right(1));
        assert_eq!(dial.position, 0);
    }

    #[test]
    fn test_leftedge() {
        let mut dial = SafeDial::new(10, 100);
        dial.turn(Movement::Left(45));
        assert_eq!(dial.position,65);
    }

    #[test]
    fn test_example() {
        let mut dial = SafeDial::new(50, 100);
        dial.turn(Movement::Left(68));
        assert_eq!(dial.position, 82);
        assert_eq!(dial.zero_click_counter, 1); // Should be 1 after first move
        
        dial.turn(Movement::Left(30));
        assert_eq!(dial.position, 52);
        assert_eq!(dial.zero_click_counter, 1); // Still 1
        
        dial.turn(Movement::Right(48));
        assert_eq!(dial.position, 0);
        assert_eq!(dial.zero_click_counter, 2); // Now 2 (wrap to 0)
        
        dial.turn(Movement::Left(5));
        assert_eq!(dial.position, 95);
        assert_eq!(dial.zero_click_counter, 2);

        dial.turn(Movement::Right(60));
        assert_eq!(dial.position, 55);
        assert_eq!(dial.zero_click_counter, 3);
        
        dial.turn(Movement::Left(55));
        assert_eq!(dial.position, 0);
        assert_eq!(dial.zero_click_counter, 4);
        
        dial.turn(Movement::Left(1));
        assert_eq!(dial.position, 99);
        assert_eq!(dial.zero_click_counter, 4);
        
        dial.turn(Movement::Left(99));
        assert_eq!(dial.position, 0);
        assert_eq!(dial.zero_click_counter, 5);
        
        dial.turn(Movement::Right(14));
        assert_eq!(dial.position, 14);
        assert_eq!(dial.zero_click_counter, 5);
        
        dial.turn(Movement::Left(82));
        assert_eq!(dial.position, 32);

        assert_eq!(dial.zero_click_counter, 6);
        
        // Part 1: should be 3 (steps 4, 7, 9 end on 0)
        assert_eq!(dial.zero_counter, 3);
    }
}