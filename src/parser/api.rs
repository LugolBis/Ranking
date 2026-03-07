use std::collections::LinkedList;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use crate::matrix::CSC;
use crate::types::Shape;
use crate::types::Value;

#[derive(Debug)]
pub struct Parsed {
    row_idx: usize,
    col_idx: usize,
    val: f64,
}

#[derive(Debug, Clone)]
pub enum ParseErr {
    File(String),
    Header(String),
    Shape(String),
    Value(String, usize),
    CSC,
}

impl Parsed {
    pub fn new(row_idx: usize, col_idx: usize, val: f64) -> Parsed {
        Parsed {
            row_idx,
            col_idx,
            val,
        }
    }
}

pub fn parse_file<F>(path: PathBuf, fn_parse: F) -> Result<CSC, ParseErr>
where
    F: Fn(&mut dyn Iterator<Item = String>) -> Result<(Shape, Vec<Parsed>), ParseErr>,
{
    let file = File::open(path).map_err(|e| ParseErr::File(e.to_string()))?;
    let mut buffer = BufReader::new(file).lines().flatten();

    let (shape, parsed) = fn_parse(&mut buffer)?;
    let mut values: Vec<Option<LinkedList<Value>>> = vec![None; shape.columns() as usize];

    // We supposed that the parsed values are sorted by rows.
    for ps in parsed {
        match &mut values[ps.col_idx] {
            Some(col) => {
                col.push_back(Value(ps.row_idx, ps.val));
            }
            empty => {
                *empty = Some(LinkedList::from([Value(ps.row_idx, ps.val)]));
            }
        }
    }
    CSC::from(shape, values).map_err(|_| ParseErr::CSC)
}
