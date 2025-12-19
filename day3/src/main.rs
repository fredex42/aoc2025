use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;
use std::fmt::format;

#[derive(Debug)]
pub struct BatteryBank {
    content: Vec<u32>
}

impl BatteryBank {
    pub fn from_string(input:&str) -> Result<BatteryBank, Box<dyn Error>> {
        let content:Vec<Result<u32, ParseIntError>> = input.chars().map(|ch| {
            let s:String = ch.into();
            s.parse::<u32>()
        }).collect();
        let failures = content.iter().filter(|r| r.is_err()).map(|r| r.as_ref().unwrap_err()).count();
        if failures > 0 {
            Err(format!("{} chars failed to parse", failures).into())
        } else {
            Ok(
                BatteryBank {
                    content: content.into_iter().filter(|r| r.is_ok()).map(|r| r.unwrap()).collect()
                }
            )
        }
    }

    /**
     * Calculate the "joltage" (defined in the problem) for the battery bank.
     * This is the highest two-numbered value that can be gained by taking one
     * digit as the tens and another to the right of it as the ones
     */
    pub fn max_joltage(&self) -> u32 {
        //Step one - where is the highest digit
        let mut highest_index = 0;
        let mut highest_val = 0;
        if self.content.len() < 2 { //can't make a two-digit number if we have less than 2 digits to start with!
            return 0;   
        }
        for i in 0..(self.content.len()-1) {    //-1, because we can't make a two-digit number in order from the last number in the set
            if self.content[i] > highest_val {
                highest_index = i;
                highest_val = self.content[i];
            }
        }
        
        //Step two - find the next highest digit after that
        let mut second_highest = 0;
        for i in (highest_index+1)..self.content.len() {
            if self.content[i] > second_highest {
                second_highest = self.content[i];
            }
        }

        (highest_val * 10) + second_highest
    }
}

pub fn parse_input(content:&str) -> Result<Vec<BatteryBank>, Box<dyn Error>> {
    content
        .split("\n")
        .into_iter()
        .map(|s| BatteryBank::from_string(s))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut f = File::open("input.txt")?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;
    
    let banks = parse_input(&content)?;

    println!("Loaded {} battery bank definitions", banks.len());

    let max_jolt:u32 = banks.iter().map(|b| b.max_joltage()).sum();

    println!("Maximum joltage is {}", max_jolt);

    Ok( () )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example1() {
        let bank = BatteryBank::from_string("987654321111111").unwrap();
        assert_eq!(bank.max_joltage(), 98);
    }

    #[test]
    fn test_example2() {
        let bank = BatteryBank::from_string("811111111111119").unwrap();
        assert_eq!(bank.max_joltage(), 89);
    }

    #[test]
    fn test_example3() {
        let bank = BatteryBank::from_string("234234234234278").unwrap();
        assert_eq!(bank.max_joltage(), 78);
    }

    #[test]
    fn test_example4() {
        let bank = BatteryBank::from_string("818181911112111").unwrap();
        assert_eq!(bank.max_joltage(), 92);
    }
}