use std::collections::LinkedList;

/// Represent a value of a column of the matrix. So Value.0 is the row index.
#[derive(Debug, Clone, Copy)]
pub struct Value(pub usize, pub f64);

/// Represent the shape of a matrix.
#[derive(Debug, Clone, Copy)]
pub struct Shape {
    rows: u64,
    columns: u64,
}

/// Represent the column of a Sparse Matrix.
#[derive(Debug, Clone)]
pub struct Column {
    pub rows: LinkedList<Value>,
}

impl Shape {
    pub fn new(rows: u64, columns: u64) -> Shape {
        Shape { rows, columns }
    }

    /// Construct a Shape from a line.
    pub fn from(
        line: Option<String>,
        pattern: &str,
        row_idx: usize,
        col_idx: usize,
    ) -> Result<Shape, String> {
        if let Some(content) = line {
            let parts = content.split(pattern).collect::<Vec<&str>>();
            if let (Some(rows), Some(cols)) = (parts.get(row_idx), parts.get(col_idx)) {
                Ok(Shape {
                    rows: rows.parse::<u64>().map_err(|e| e.to_string())?,
                    columns: cols.parse::<u64>().map_err(|e| e.to_string())?,
                })
            } else {
                Err(format!(
                    "Failed to retrieve the Shape at indexes (row_idx={row_idx}, col_idx={col_idx})."
                ))
            }
        } else {
            Err("There isn't any String content.".to_string())
        }
    }

    pub fn rows(&self) -> u64 {
        self.rows
    }

    pub fn columns(&self) -> u64 {
        self.columns
    }
}

impl Column {
    pub fn from(rows: LinkedList<Value>) -> Column {
        Column { rows }
    }

    /// Return the value at the given index.
    pub fn get_value(&self, row_idx: usize) -> Option<&Value> {
        self.rows.iter().find(|v| v.0 == row_idx)
    }
}
