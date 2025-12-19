use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::num::ParseIntError;

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

    /**
     * Calculate the "joltage" (defined in the problem) for the battery bank.
     * This is now the highest 12-numbered value that can be gained by taking one
     * digit as the 10^x and another to the right of it as the 10^(x-1) etc.
     */
    pub fn max_joltage_v2(&self) -> u64 {
        fn find_next_highest(content: &Vec<u32>, pow:usize,start_pos:usize) -> (u64, usize) {
            //If we are e.g. at power 10, there must be at least 10 other values following us in order to be valid.
            //Practically this means we cut off at len - 10
            if content.len()==0 {
                return (0, 0)
            }
            let mut highest:u64 = 0;
            let mut highest_index:usize = 0;
            for i in start_pos..(content.len() - pow) {
                if u64::from(content[i]) > highest {
                    highest = content[i].into();
                    highest_index = i;
                }
            }

            (highest * 10_u64.pow(pow.try_into().unwrap()), highest_index+1)    //next start position is current highest index + 1
        }

        let mut sum:u64 = 0;
        let mut pos:usize = 0;
        for pow in (0..12).rev() {
            let (next_value, next_pos) = find_next_highest(&self.content, pow, pos);
            sum += next_value;
            pos = next_pos;
        }

        sum
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
    println!("Old maximum joltage is {}", max_jolt);
    let max_jolt_v2:u64 = banks.iter().map(|b| b.max_joltage_v2()).sum();
    println!("New maximum joltage is {}", max_jolt_v2);
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

    #[test]
    fn test_example1v2() {
        let bank = BatteryBank::from_string("987654321111111").unwrap();
        assert_eq!(bank.max_joltage_v2(), 987654321111);
    }

    #[test]
    fn test_example2v2() {
        let bank = BatteryBank::from_string("811111111111119").unwrap();
        assert_eq!(bank.max_joltage_v2(), 811111111119);
    }


    #[test]
    fn test_example2v3() {
        let bank = BatteryBank::from_string("234234234234278").unwrap();
        assert_eq!(bank.max_joltage_v2(), 434234234278);
    }


    #[test]
    fn test_example2v4() {
        let bank = BatteryBank::from_string("818181911112111").unwrap();
        assert_eq!(bank.max_joltage_v2(), 888911112111);
    }


}