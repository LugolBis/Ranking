use std::{collections::LinkedList, sync::Arc, thread, time::Duration};

use crossbeam_channel::unbounded;

use crate::{
    errors::CSCErr,
    maths::{compute_norm, uniform_vector},
    matrix::types::{Column, Shape, Value},
    matrix::utils::{compute_mult, get_f, get_surfer},
    pool::ThreadPool,
};

/// An immutable thread-safe reference to a `Column`.
pub type RefCol = Arc<Column>;

/// Represent a Sparse Matrix as a `Compressed Sparse Column`.
#[derive(Debug)]
pub struct CSC {
    shape: Shape,
    columns: Vec<Option<RefCol>>,
    /// f is a line vector of the same len of rows of the matrix.<br>
    /// It's constructed as following :<br>
    /// if the row i in P contains only 0 { f\[i] = 1 }<br>
    /// else { f\[i] = 0 }
    f: Vec<f64>,
    alpha: f64,
    /// Thread pool used to parallelize operations
    pool: Arc<ThreadPool>,
}

impl CSC {
    pub fn from(
        shape: Shape,
        columns: Vec<Option<LinkedList<Value>>>,
        row_count: Vec<u64>,
        alpha: f64,
    ) -> Result<CSC, CSCErr> {
        let nb_threads = thread::available_parallelism()
            .map_err(|_| {
                CSCErr::Thread("Failed to get the number of availlable parallelism.".into())
            })?
            .get();
        let pool: ThreadPool = ThreadPool::new(nb_threads);

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
                pool: Arc::new(pool),
            })
        }
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn columns(&self) -> &Vec<Option<RefCol>> {
        &self.columns
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn pool(&self) -> Arc<ThreadPool> {
        Arc::clone(&self.pool)
    }

    /// Return the count of non zero value.
    pub fn count(&self) -> u64 {
        self.columns
            .iter()
            .flatten()
            .map(|c| c.rows.len() as u64)
            .sum::<u64>()
    }

    pub fn set_alpha(&mut self, alpha: f64) {
        self.alpha = alpha;
    }

    /// Compute the following operation :<br>
    /// pi * M (with M the matrix `CSC` itself)<br>
    /// Arguments `csx` and `csy` are coeficients used to compute the random surfer coeficient.
    pub fn mult_vec(&self, pi: &[f64], csx: f64, csy: f64) -> Result<Vec<f64>, CSCErr> {
        let rows_len = self.shape.rows() as usize;

        if rows_len != pi.len() {
            return Err(CSCErr::ShapeVec(rows_len, pi.len()));
        }

        let nb_threads = &self.pool.num_workers();
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

            let _ = &self
                .pool
                .execute(move || compute_mult(tx_c, pi_c, columns_c, alpha, chunk_id, start, end))
                .map_err(|e| CSCErr::Thread(format!("Thread Pool error : {}", e)))?;
        }
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
            }

            need_check = !need_check;
            step += 1;
        }

        Ok((pi_even, step * 2))
    }
}
