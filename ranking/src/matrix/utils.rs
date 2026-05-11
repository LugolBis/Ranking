use std::{collections::LinkedList, sync::Arc};

use crate::{
    errors::{CSCErr, RefErr},
    maths::RngSeq,
    matrix::{
        core::RefCol,
        partition::{GroupPartition, Partition},
        types::Value,
    },
};
use crossbeam_channel::Sender;

/// Compute the multiplication between `pi` and few columns of the matrix.
pub fn compute_mult(
    tx_c: Sender<(usize, Vec<f64>)>,
    pi_c: Arc<Vec<f64>>,
    columns_c: Arc<Vec<Option<RefCol>>>,
    group: Arc<GroupPartition>,
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
                local += alpha
                    * pi_c[group.index(value.get_row_index().try_into().unwrap())]
                    * value.get_value();
            }
            local_vec[col_idx] = local;
        }
    }
    tx_c.send((chunk_id, local_vec))
        .map_err(|_| Box::new(CSCErr::SendErr) as RefErr)?;
    Ok(())
}

fn normalize(value: f64) -> f64 {
    1.0 / (1.0 + (-(8.0 * value - 3.0)).exp())
}

/// Remove edges of the matrix based on the `treshold` in input.
pub fn filter_edges(
    tx_c: Sender<(usize, Vec<Option<LinkedList<Value>>>, Vec<u64>)>,
    columns_c: Arc<Vec<Option<RefCol>>>,
    partition: Arc<Partition>,
    chunk_id: usize,
    start: usize,
    end: usize,
    rows_len: usize,
    seed: u64,
) -> Result<(), RefErr> {
    let mut local_cols = vec![None; end - start];
    let mut row_count = vec![0u64; rows_len];
    let mut rdseq = RngSeq::from(seed);

    for (col_idx, opt) in columns_c[start..end].iter().enumerate() {
        if let Some(column) = opt {
            let column_filtered = column
                .rows
                .iter()
                .filter(|row| {
                    rdseq.next() >= normalize(row.get_value())
                        && partition.group_containing(col_idx.try_into().unwrap())
                            != partition.group_containing(row.get_row_index().try_into().unwrap())
                })
                .collect::<LinkedList<Value>>();

            if column_filtered.len() == column.rows.len() {
                println!("No changes here !");
            }

            if !column_filtered.is_empty() {
                for value in column_filtered.iter() {
                    row_count[value.get_row_index()] += 1;
                }

                local_cols[col_idx] = Some(column_filtered);
            }
        }
    }
    tx_c.send((chunk_id, local_cols, row_count))
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
