use std::collections::LinkedList;

#[derive(Debug, Clone, Copy)]
pub struct Value(pub usize, pub f64);

#[derive(Debug, Clone, Copy)]
pub struct Shape {
    rows: u64,
    columns: u64,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub rows: LinkedList<Value>,
}

#[derive(Debug, Clone)]
pub enum CSCErr {
    ShapeColumn(Shape, usize),
    Thread(String),
    SendErr,
    Epsilon(f64),
    Shape(usize, usize),
}

#[derive(Debug, Clone)]
pub enum CLIErr {
    Alpha(String),
    Epsilon(String),
    File(String),
}

impl Shape {
    pub fn new(rows: u64, columns: u64) -> Shape {
        Shape { rows, columns }
    }

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

    pub fn get_value(&self, row_idx: usize) -> Option<&Value> {
        self.rows.iter().find(|v| v.0 == row_idx)
    }
}
