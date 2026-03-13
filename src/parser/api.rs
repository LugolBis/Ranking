use std::collections::LinkedList;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use crate::errors::ParseErr;
use crate::matrix::CSC;
use crate::types::Shape;
use crate::types::Value;

#[derive(Debug)]
pub struct Parsed {
    row_idx: usize,
    col_idx: usize,
    val: f64,
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

/// Provide a generic API to unify matrix file format and their implementation.
pub fn parse_file<F>(path: PathBuf, fn_parse: F, alpha: f64) -> Result<CSC, ParseErr>
where
    F: Fn(&mut dyn Iterator<Item = String>) -> Result<(Shape, Vec<Parsed>, Vec<u64>), ParseErr>,
{
    let file = File::open(path).map_err(|e| ParseErr::File(e.to_string()))?;
    let mut buffer = BufReader::new(file).lines().flatten();

    let (shape, parsed, row_count) = fn_parse(&mut buffer)?;
    let mut values: Vec<Option<LinkedList<Value>>> = vec![None; shape.columns() as usize];

    // We supposed that the parsed values are sorted by rows.
    for ps in parsed {
        match &mut values[ps.col_idx] {
            Some(col) => {
                col.push_back(Value::from(ps.val, ps.row_idx));
            }
            empty => {
                *empty = Some(LinkedList::from([Value::from(ps.val, ps.row_idx)]));
            }
        }
    }
    CSC::from(shape, values, row_count, alpha).map_err(|_| ParseErr::CSC)
}
