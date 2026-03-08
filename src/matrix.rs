use std::{
    collections::LinkedList,
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

use mylog::error;

use crate::{
    maths::{compute_norm, uniform_vector},
    pool::ThreadPool,
    types::{Shape, Value},
};

pub type RefCol = Arc<Column>;

#[derive(Debug, Clone)]
pub struct Column {
    rows: LinkedList<Value>,
}

#[derive(Debug, Clone)]
pub struct CSC {
    shape: Shape,
    columns: Vec<Option<RefCol>>,
}

impl Column {
    pub fn from(rows: LinkedList<Value>) -> Column {
        Column { rows }
    }

    pub fn get_value(&self, row_idx: usize) -> Option<&Value> {
        self.rows.iter().find(|v| v.0 == row_idx)
    }
}

impl CSC {
    pub fn from(shape: Shape, columns: Vec<Option<LinkedList<Value>>>) -> Result<CSC, ()> {
        if shape.columns() as usize != columns.len() {
            Err(error!(
                "Invalide shape {:?} for the columns of len {}",
                shape,
                columns.len()
            ))
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

    pub fn mult_vec(&self, pi: &[f64]) -> Result<Vec<f64>, ()> {
        let rows_len = self.shape.rows() as usize;

        if rows_len != pi.len() {
            return Err(error!(
                "Matrix ({:?}) can't be multiplied with a vector of {:?}",
                self.shape,
                pi.len()
            ));
        }

        let nb_threads = thread::available_parallelism().map_err(|_| ())?.get();
        let columns = self.columns.clone();
        let pi_shared = Arc::new(pi.to_vec());

        let (tx, rx) = mpsc::channel();
        let mut pool = ThreadPool::new(nb_threads);

        for (col_idx, opt) in columns.into_iter().enumerate() {
            if let Some(column) = opt {
                let tx = tx.clone();
                let pi_c = Arc::clone(&pi_shared);
                pool.execute(move || {
                    let mut local = 0.0;

                    for &value in column.rows.iter() {
                        local += pi_c[value.0] * value.1
                    }

                    tx.send((col_idx, local))
                        .map_err(|_| Box::<dyn std::error::Error>::from("Send error."))?;
                    Ok(())
                })
                .expect("Failed to execute job");
            }
        }

        pool.shutdown(Duration::from_secs(2))
            .map_err(|e| error!("{:?}", e))?;
        drop(tx);

        let mut result = vec![0.0; self.shape.columns() as usize];
        for (index, coef) in rx.iter() {
            result[index] = coef;
        }
        Ok(result)
    }

    pub fn stationary_distribution(&self, epsilon: f64) -> Result<(Vec<f64>, usize), ()> {
        assert_ne!(1f64 - (1f64 - epsilon), 0_f64);

        let mut pi_even = uniform_vector(self.shape.rows() as usize);
        let mut pi_odd = pi_even.clone();
        let mut counter = 0usize;
        let mut need_check = false;
        let mut norm = 1.0;

        while norm > epsilon {
            pi_odd = self.mult_vec(&pi_even)?;
            pi_even = self.mult_vec(&pi_odd)?;

            if need_check {
                norm = compute_norm(&pi_even, &pi_odd);
            }

            need_check = !need_check;
            counter += 1;
        }

        Ok((pi_even, counter))
    }
}

#[test]
fn test() {
    // Création de quelques colonnes avec des listes chaînées
    let mut entries0 = LinkedList::new();
    entries0.push_back(Value(1, 3.14));
    entries0.push_back(Value(5, 2.71));

    let mut entries1 = LinkedList::new();
    entries1.push_back(Value(0, 1.41));
    entries1.push_back(Value(2, 1.73));

    let columns = vec![Some(entries0), Some(entries1), None];

    let shape = Shape::new(6, 3);
    let matrix = CSC::from(shape, columns).unwrap();

    let vector: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    println!("{:#?}", matrix.mult_vec(&vector));
}
