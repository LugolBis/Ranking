use std::{collections::LinkedList, sync::Arc};

use crate::{
    errors::{CSCErr, RefErr},
    maths::random,
    matrix::core::RefCol,
    matrix::types::Value,
};
use crossbeam_channel::Sender;

/// Compute the multiplication between `pi` and few columns of the matrix.
pub fn compute_mult(
    tx_c: Sender<(usize, Vec<f64>)>,
    pi_c: Arc<Vec<f64>>,
    columns_c: Arc<Vec<Option<RefCol>>>,
    alpha: f64,
    chunk_id: usize,
    start: usize,
    end: usize,
) -> Result<(), RefErr> {
    let mut local_vec = vec![0.0; end - start];

    for (col_idx, opt) in columns_c[start..end].iter().enumerate() {
        if let Some(column) = opt {
            let mut local = 0.0;
            for &value in column.rows.iter() {
                local += alpha * pi_c[value.get_row_index()] * value.get_value();
            }
            local_vec[col_idx] = local;
        }
    }
    tx_c.send((chunk_id, local_vec))
        .map_err(|_| Box::new(CSCErr::SendErr) as RefErr)?;
    Ok(())
}

/// Compute the multiplication between `pi` and few columns of the matrix.
pub fn filter_edges(
    tx_c: Sender<(usize, Vec<Option<LinkedList<Value>>>, Vec<u64>)>,
    columns_c: Arc<Vec<Option<RefCol>>>,
    treshold: f64,
    chunk_id: usize,
    start: usize,
    end: usize,
) -> Result<(), RefErr> {
    let mut local_cols = vec![None; end - start];
    let mut local_rows_count = vec![0u64; end - start];

    for (col_idx, opt) in columns_c.iter().enumerate() {
        if let Some(column) = opt {
            let column_filtered = column
                .rows
                .iter()
                .filter(|_| random() >= treshold)
                .collect::<LinkedList<Value>>();

            if !column_filtered.is_empty() {
                local_rows_count[col_idx] = column_filtered.len() as u64;
                local_cols[col_idx] = Some(column_filtered);
            }
        }
    }
    tx_c.send((chunk_id, local_cols, local_rows_count))
        .map_err(|_| Box::new(CSCErr::SendErr) as RefErr)?;
    Ok(())
}

/// Compute the surfer coeficient :<br>
/// surfer_coef = (1-alpha) * (1/N) + alpha * (1/N) * (pi * f^t)<br>
///             = csx + csy * (pi * f^t)<br>
/// So csx = (1-alpha) * (1/N) et csy = alpha * (1/N)
pub fn get_surfer(csx: f64, csy: f64, rows: u64, pi: &[f64], f: &[f64]) -> Vec<f64> {
    let coef = csx + csy * pi.iter().zip(f.iter()).map(|(x, y)| x * y).sum::<f64>();
    vec![coef; rows as usize]
}

/// Construct the f line vector based on the count of values for each row of the matrix
pub fn get_f(row_count: Vec<u64>) -> Vec<f64> {
    row_count
        .iter()
        .map(|x| if *x == 0u64 { 1f64 } else { 0f64 })
        .collect()
}
