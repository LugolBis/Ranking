use std::{collections::LinkedList, sync::Arc, thread, time::Duration};

use crossbeam_channel::{Sender, unbounded};

use crate::{
    errors::{CSCErr, RefErr},
    maths::{compute_norm, uniform_vector},
    pool::ThreadPool,
    types::{Column, Shape, Value},
};

/// An immutable thread-safe reference to a `Column`.
pub type RefCol = Arc<Column>;

/// Represent a Sparse Matrix as a `Compressed Sparse Column`.
#[derive(Debug, Clone)]
pub struct CSC {
    shape: Shape,
    columns: Vec<Option<RefCol>>,
    /// f is a line vector of the same len of rows of the matrix.<br>
    /// It's constructed as following :<br>
    /// if the row i in P contains only 0 { f\[i] = 1 }<br>
    /// else { f\[i] = 0 }
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

    pub fn get_shape(&self) -> Shape {
        self.shape
    }

    pub fn get_columns(&self) -> &Vec<Option<RefCol>> {
        &self.columns
    }

    /// Return the count of non zero value.
    pub fn get_count(&self) -> u64 {
        self.columns
            .iter()
            .flatten()
            .map(|c| c.rows.len() as u64)
            .sum::<u64>()
    }

    /// Compute the following operation :<br>
    /// pi * M (with M the matrix `CSC` itself)<br>
    /// Arguments `csx` and `csy` are coeficients used to compute the random surfer coeficient.
    pub fn mult_vec(&self, pi: &[f64], csx: f64, csy: f64) -> Result<Vec<f64>, CSCErr> {
        let rows_len = self.shape.rows() as usize;

        if rows_len != pi.len() {
            return Err(CSCErr::ShapeVec(rows_len, pi.len()));
        }

        let nb_threads = thread::available_parallelism()
            .map_err(|_| {
                CSCErr::Thread("Failed to get the number of availlable parallelisme.".into())
            })?
            .get();
        let mut pool: ThreadPool = ThreadPool::new(nb_threads);
        let (tx, rx) = unbounded();

        let total_len = self.shape.rows() as usize;
        let chunk_size = (total_len / (nb_threads * 2)) + 1;

        let columns = Arc::new(self.columns.clone());
        let pi_shared = Arc::new(pi.to_vec());

        for chunk_id in 0..nb_threads * 2 {
            let start = chunk_id * chunk_size;
            if start >= total_len {
                break;
            }
            let end = (start + chunk_size).min(total_len);

            let tx_c = tx.clone();
            let pi_c = Arc::clone(&pi_shared);
            let columns_c = Arc::clone(&columns);
            let alpha = *&self.alpha;

            pool.execute(move || compute_mult(tx_c, pi_c, columns_c, alpha, chunk_id, start, end))
                .map_err(|e| CSCErr::Thread(format!("Thread Pool error : {}", e)))?;
        }

        pool.shutdown(Duration::from_secs(2))
            .map_err(|e| CSCErr::Thread(format!("ThreadPool error : {}", e)))?;
        drop(tx);

        let mut result = vec![Vec::new(); nb_threads * 2];
        for (chunk_id, chunk) in rx.iter() {
            result[chunk_id] = chunk;
        }

        Ok(result
            .into_iter()
            .flatten()
            .zip(get_surfer(csx, csy, self.shape.rows(), pi, &self.f[..]))
            .map(|(x, y)| x + y)
            .collect())
    }

    /// Compute the stationary distribution with the `epsilon` parameter which define the target precision.<br>
    /// Note that it use the random surfer and `self.alpha` for computations.
    pub fn stationary_distribution(&self, epsilon: f64) -> Result<(Vec<f64>, usize), CSCErr> {
        if 1f64 - (1f64 - epsilon) == 0f64 {
            return Err(CSCErr::Epsilon(epsilon));
        }

        let mut pi_even = uniform_vector(self.shape.rows() as usize);
        let mut pi_odd: Vec<f64>;
        let n = 1f64 / self.shape.rows() as f64;
        let csx = (1f64 - &self.alpha) * n;
        let csy = &self.alpha * n;

        let mut step = 0usize;
        let mut need_check = false;
        let mut norm = 1.0;

        while norm > epsilon {
            pi_odd = self.mult_vec(&pi_even, csx, csy)?;
            pi_even = self.mult_vec(&pi_odd, csx, csy)?;

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

/// Compute the multiplication between `pi` and few columns of the matrix.
fn compute_mult(
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

/// Compute the surfer coeficient :<br>
/// surfer_coef = (1-alpha) * (1/N) + alpha * (1/N) * (pi * f^t)<br>
///             = csx + csy * (pi * f^t)<br>
/// So csx = (1-alpha) * (1/N) et csy = alpha * (1/N)
fn get_surfer(csx: f64, csy: f64, rows: u64, pi: &[f64], f: &[f64]) -> Vec<f64> {
    let coef = csx + csy * pi.iter().zip(f.iter()).map(|(x, y)| x * y).sum::<f64>();
    vec![coef; rows as usize]
}

/// Construct the f line vector based on the count of values for each row of the matrix
fn get_f(row_count: Vec<u64>) -> Vec<f64> {
    row_count
        .iter()
        .map(|x| if *x == 0u64 { 1f64 } else { 0f64 })
        .collect()
}
