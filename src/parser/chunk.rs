use std::ops::DivAssign;

use crate::parser::api::Parsed;

#[derive(Debug, Clone, Copy)]
pub struct Coord {
    row_idx: usize,
    column_idx: usize,
}

pub struct Chunk {
    id: usize,
    values: Vec<Coord>,
}

impl Coord {
    pub fn from(row_idx: usize, column_idx: usize) -> Coord {
        Coord {
            row_idx,
            column_idx,
        }
    }
}

impl Chunk {
    pub fn from(chunk_id: usize, chunk_c: Vec<Coord>) -> Chunk {
        Chunk {
            id: chunk_id,
            values: chunk_c,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn into_parsed(&self, row_count: &[u64]) -> Vec<Parsed> {
        self.values
            .clone()
            .into_iter()
            .map(|coord| {
                let val = row_count[coord.row_idx];
                if val == 0 {
                    Parsed::new(coord.row_idx, coord.column_idx, val as f64)
                } else {
                    Parsed::new(coord.row_idx, coord.column_idx, 1f64 / val as f64)
                }
            })
            .collect()
    }
}
