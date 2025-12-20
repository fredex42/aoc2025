use std::{error::Error, num::ParseIntError};
use regex::Regex;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Operation {
    Mul,
    Add,
    Sub,
    Div
}

#[derive(Clone, Debug, PartialEq)]
pub struct MathProblem {
    terms: Vec<i64>,
    op: Operation
}

impl MathProblem {
    pub fn calculate(&self) -> Option<i64> {
        match self.op {
            Operation::Mul=>self.terms.clone().into_iter().reduce(|total, term| total * term),
            Operation::Add=>self.terms.clone().into_iter().reduce(|total, term| total+term),
            Operation::Sub=>self.terms.clone().into_iter().reduce(|total, term| total - term),
            Operation::Div=>self.terms.clone().into_iter().reduce(|total, term| total - term)
        }
    }
}

pub fn parse_input(input:&str) -> Result<Vec<MathProblem>, Box<dyn Error>> {
    let is_space = Regex::new("\\s+").unwrap();

    let values:Vec<Vec<&str>> = input
        .split("\n")
        .map(|line| {
            is_space.split(line.trim()).into_iter().collect()
        })
        .collect();

    //println!("values: {:?}", values);

    //OK, so values currently goes row -> column (outer to inner).  We need to reverse it, into column -> row (outer to inner)
    
    //We know the sizes, so pre-allocate
    if values.is_empty() {
        return Err("there was no data to parse".into());
    }
    let row_count = values.len();
    let col_count = values[0].len();
    //Sanity-check col_count
    if values.iter().any(|row| row.len()!=col_count) {
        return Err("the incoming data was not square".into());
    }
    let mut new_shape:Vec<Vec<&str>> = vec![vec![""; col_count]; row_count];

    //Flip them over into the new vecs
    for row_idx in 0..(&values).len() {
        let row = &values[row_idx];
        
        for col_idx in 0..row.len() {
            new_shape[col_idx][row_idx] = values[row_idx][col_idx];
        }
    }
    
    //println!("new_shape: {:?}", new_shape);

    //Now construct the domain objects
    let mut results:Vec<MathProblem> = Vec::with_capacity(new_shape.len());
    for col in new_shape {
        match col.split_last() {
            Some( (last, others) )=>{
                let op = match *last {
                    "*"=>Operation::Mul,
                    "+"=>Operation::Add,
                    "-"=>Operation::Sub,
                    "/"=>Operation::Div,
                    _=>return Err(format!("invalid operation specifier '{}'", last).into())
                };
                let values:Result<Vec<i64>, ParseIntError> = others.into_iter().map(|s| s.parse::<i64>()).collect();
                match values {
                    Ok(terms)=>results.push(
                        MathProblem {op, terms}
                    ),
                    Err(e)=>return Err(e.into())
                }
            },
            None=>return Err("There were no problems to build".into())
        }
    }
    Ok( results )
}

fn main() {

}

#[cfg(test)]
mod test {
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use super::*;

    #[test]
    fn test_example_parse() {
        let input = "123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  ";

        let probs = parse_input(&input).unwrap();
        assert_eq!(probs[0], MathProblem { terms: vec![123, 45, 6], op: Operation::Mul});
        assert_eq!(probs[1], MathProblem { terms: vec![328, 64, 98], op: Operation::Add});
        assert_eq!(probs[2], MathProblem { terms: vec![51, 387, 215], op: Operation::Mul});
        assert_eq!(probs[3], MathProblem { terms: vec![64, 23, 314], op: Operation::Add});
    }

    #[test]
    fn test_example() {
                let input = "123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  ";

        let probs = parse_input(&input).unwrap();
        let final_result:i64 = probs.par_iter().map(|p| p.calculate().expect("the problem was empty?")).sum();
        assert_eq!(final_result, 4277556);
    }
}