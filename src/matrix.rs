use std::{
    collections::LinkedList,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Sender, unbounded};

use crate::{
    maths::{compute_norm, uniform_vector},
    types::{CSCErr, Column, Shape, Value},
};

pub type RefCol = Arc<Column>;

#[derive(Debug, Clone)]
pub struct CSC {
    shape: Shape,
    columns: Vec<Option<RefCol>>,
    // f est un vecteur ligne de taille N tel que f [i] = 1 si la ligne i de P ne contient que des zéros et sinon f [i] = 0
    f: Vec<f64>,
    alpha: f64,
}

impl CSC {
    pub fn from(
        shape: Shape,
        columns: Vec<Option<LinkedList<Value>>>,
        row_count: Vec<u64>,
        alpha: f64,
    ) -> Result<CSC, CSCErr> {
        if shape.columns() as usize != columns.len() {
            Err(CSCErr::ShapeColumn(shape, columns.len()))
        } else {
            Ok(CSC {
                shape,
                columns: columns
                    .into_iter()
                    .map(|col| {
                        if let Some(rows) = col {
                            Some(Arc::new(Column::from(rows)))
                        } else {
                            None
                        }
                    })
                    .collect(),
                f: get_f(row_count),
                alpha,
            })
        }
    }

    pub fn get_column(&self, column_idx: usize) -> Option<RefCol> {
        if let Some(ref_col) = self.columns.get(column_idx) {
            ref_col.clone()
        } else {
            None
        }
    }

    pub fn get_value(&self, row_idx: usize, column_idx: usize) -> Option<Value> {
        if let Some(column) = self.get_column(column_idx) {
            if let Some(value) = column.get_value(row_idx) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn mult_vec(&self, pi: &[f64]) -> Result<Vec<f64>, CSCErr> {
        let rows_len = self.shape.rows() as usize;

        if rows_len != pi.len() {
            return Err(CSCErr::Shape(rows_len, pi.len()));
        }

        let nb_threads = thread::available_parallelism()
            .map_err(|_| {
                CSCErr::Thread("Failed to get the number of availlable parallelisme.".into())
            })?
            .get();
        let (tx, rx) = unbounded();
        let mut pool: Vec<JoinHandle<Result<(), CSCErr>>> = Vec::new();

        let total_len = self.shape.rows() as usize;
        let chunk_size = (total_len + nb_threads - 1) / nb_threads;

        let columns = Arc::new(self.columns.clone());
        let pi_shared = Arc::new(pi.to_vec());

        for chunk_id in 0..nb_threads {
            let start = chunk_id * chunk_size;
            if start >= total_len {
                break;
            }
            let end = (start + chunk_size).min(total_len);

            let tx_c = tx.clone();
            let pi_c = Arc::clone(&pi_shared);
            let columns_c = Arc::clone(&columns);
            let alpha = *&self.alpha;

            pool.push(thread::spawn(move || {
                compute_mult(tx_c, pi_c, columns_c, alpha, chunk_id, start, end)
            }));
        }

        for thread_join in pool {
            let _ = thread_join
                .join()
                .map_err(|e| CSCErr::Thread(format!("{:?}", e)))?;
        }
        drop(tx);

        let mut result = vec![Vec::new(); nb_threads];
        for (chunk_id, chunk) in rx.iter() {
            result[chunk_id] = chunk;
        }

        Ok(result
            .into_iter()
            .flatten()
            .zip(get_surfer(self.alpha, self.shape.rows(), pi, &self.f[..]))
            .map(|(x, y)| x + y)
            .collect())
    }

    pub fn stationary_distribution(&self, epsilon: f64) -> Result<(Vec<f64>, usize), CSCErr> {
        if 1f64 - (1f64 - epsilon) == 0f64 {
            return Err(CSCErr::Epsilon(epsilon));
        }

        let mut pi_even = uniform_vector(self.shape.rows() as usize);
        let mut pi_odd = pi_even.clone();
        let mut step = 0usize;
        let mut need_check = false;
        let mut norm = 1.0;

        while norm > epsilon {
            pi_odd = self.mult_vec(&pi_even)?;
            pi_even = self.mult_vec(&pi_odd)?;

            if need_check {
                norm = compute_norm(&pi_even, &pi_odd);
                println!("Step = {} - Norm = {}", step, norm);
            }

            need_check = !need_check;
            step += 1;
        }

        Ok((pi_even, step * 2))
    }
}

fn compute_mult(
    tx_c: Sender<(usize, Vec<f64>)>,
    pi_c: Arc<Vec<f64>>,
    columns_c: Arc<Vec<Option<RefCol>>>,
    alpha: f64,
    chunk_id: usize,
    start: usize,
    end: usize,
) -> Result<(), CSCErr> {
    let mut local_vec = vec![0.0; end - start];

    for (col_idx, opt) in columns_c[start..end].iter().enumerate() {
        if let Some(column) = opt {
            let mut local = 0.0;
            for &value in column.rows.iter() {
                local += alpha * pi_c[value.0] * value.1;
            }
            local_vec[col_idx] = local;
        }
    }
    tx_c.send((chunk_id, local_vec))
        .map_err(|_| CSCErr::SendErr)?;
    Ok(())
}

fn get_surfer(alpha: f64, rows: u64, pi: &[f64], f: &[f64]) -> Vec<f64> {
    let coef = (1f64 - alpha) * (1f64 / rows as f64)
        + alpha * (1f64 / rows as f64) * pi.iter().zip(f.iter()).map(|(x, y)| x * y).sum::<f64>();
    vec![coef; rows as usize]
}

fn get_f(row_count: Vec<u64>) -> Vec<f64> {
    row_count
        .iter()
        .map(|x| if *x == 0u64 { 1f64 } else { 0f64 })
        .collect()
}
