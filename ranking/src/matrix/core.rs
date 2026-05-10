use std::{collections::LinkedList, sync::Arc, thread};

use crossbeam_channel::unbounded;

use crate::{
    errors::CSCErr,
    maths::{compute_norm, uniform_vector},
    matrix::{
        partition::{GroupParition, Partition},
        types::{Column, Value},
        utils::{compute_mult, get_f, get_surfer},
    },
    pool::ThreadPool,
};

/// An immutable thread-safe reference to a `Column`.
pub type RefCol = Arc<Column>;

/// Represent a Sparse Matrix as a `Compressed Sparse Column`.
#[derive(Debug)]
pub struct CSC {
    size: u64,
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
        size: u64,
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

        if size as usize != columns.len() {
            Err(CSCErr::ShapeColumn(size, columns.len()))
        } else {
            Ok(CSC {
                size,
                columns: columns
                    .into_iter()
                    .map(|col| col.map(|rows| Arc::new(Column::from(rows))))
                    .collect(),
                f: get_f(row_count),
                alpha,
                pool: Arc::new(pool),
            })
        }
    }

    pub fn size(&self) -> u64 {
        self.size
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
        if self.size as usize != pi.len() {
            return Err(CSCErr::ShapeVec(self.size as usize, pi.len()));
        }

        let nb_threads = &self.pool.num_workers();
        let (tx, rx) = unbounded();

        let total_len = self.size as usize;
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
            let alpha = self.alpha;

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
            .zip(get_surfer(csx, csy, self.size, pi, &self.f[..]))
            .map(|(x, y)| x + y)
            .collect())
    }

    /// Compute the stationary distribution with the `epsilon` parameter which define the target precision.<br>
    /// Note that it use the random surfer and `self.alpha` for computations.
    pub fn stationary_distribution(
        &self,
        epsilon: f64,
        group_count: u64,
    ) -> Result<(Vec<f64>, usize), CSCErr> {
        if 1f64 - (1f64 - epsilon) == 0f64 {
            return Err(CSCErr::Epsilon(epsilon));
        }

        let mut step = 0usize;

        let partition = Partition::new(self.size, group_count);
        let mut stationary_distributions = Vec::new();

        for group in partition.groups() {
            let (stationary_distribution, steps) = self
                .sub_matrix(group)
                .stationary_distribution_from(epsilon, uniform_vector(self.size as usize))?;
            step += steps;
            stationary_distributions.push(stationary_distribution);
        }

        let stationary_distribution =
            partition.fusion_stationary_distributions(stationary_distributions);
        let (final_stationary_distribution, steps) =
            self.stationary_distribution_from(epsilon, stationary_distribution)?;
        step += steps;

        Ok((final_stationary_distribution, step * 2))
    }

    fn stationary_distribution_from(
        &self,
        epsilon: f64,
        pi: Vec<f64>,
    ) -> Result<(Vec<f64>, usize), CSCErr> {
        if 1f64 - (1f64 - epsilon) == 0f64 {
            return Err(CSCErr::Epsilon(epsilon));
        }

        let mut pi_even = pi;
        let mut pi_odd: Vec<f64>;
        let n = 1f64 / self.size as f64;
        let csx = (1f64 - &self.alpha) * n;
        let csy = self.alpha * n;

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

    pub fn sub_matrix(&self, group: &GroupParition) -> CSC {
        let mut sub_matrix_columns = Vec::new();
        let mut f = vec![1.0; self.size as usize];
        for col in 0..self.columns.len() {
            if group.contains(col as u64)
                && let Some(Some(column)) = self.columns.get(col)
            {
                let sub_column = (*column).get_sub_column(group);
                for value in sub_column.rows.iter() {
                    f[value.get_row_index()] = 0.0;
                }
                sub_matrix_columns.push(Some(Arc::new(sub_column)));
            } else {
                sub_matrix_columns.push(None);
            }
        }
        CSC {
            size: self.size,
            f,
            columns: sub_matrix_columns,
            alpha: self.alpha,
            pool: self.pool.clone(),
        }
    }
}
