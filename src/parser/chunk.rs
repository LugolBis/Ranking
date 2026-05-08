use crate::parser::api::Parsed;

/// Represent the coordinate parsed in the Matrix file
#[derive(Debug, Clone, Copy)]
pub struct Coord {
    row_idx: usize,
    column_idx: usize,
}

/// Represent a parsed part of the Matrix file
#[derive(Debug)]
pub struct Chunk {
    id: usize,
    coords: Vec<Coord>,
    row_count: Vec<u64>,
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
    pub fn from(chunk_id: usize, coords: Vec<Coord>, row_count: Vec<u64>) -> Chunk {
        Chunk {
            id: chunk_id,
            coords,
            row_count,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_row_count(&self) -> &Vec<u64> {
        &self.row_count
    }

    /// Evenly distribute the probability according to the number of row values
    pub fn parse(&self, row_count: &[u64]) -> Vec<Parsed> {
        self.coords
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
