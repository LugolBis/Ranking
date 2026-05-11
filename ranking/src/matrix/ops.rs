use std::{
    collections::LinkedList,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::Arc,
};

use crossbeam_channel::unbounded;

use crate::{
    errors::CSCErr,
    matrix::{core::CSC, types::Value, utils::filter_edges},
};

impl CSC {
    /// Write the CSC matrix as the Matrix Format in the given `path` file.
    pub fn dump(&self, path: &PathBuf) -> Result<(), CSCErr> {
        let file = File::create(path)
            .map_err(|e| CSCErr::Dump(format!("{} with path {}", e, path.display())))?;

        let mut buffer = BufWriter::new(file);
        buffer
            .write_all("%%MatrixMarket matrix coordinate pattern general\n".as_bytes())
            .map_err(|e| CSCErr::Dump(format!("Failed to write headers due to : {}", e)))?;

        writeln!(buffer, "{} {} {}", self.size(), self.size(), &self.count())
            .map_err(|e| CSCErr::Dump(format!("Failed to write shape due to : {}", e)))?;

        let iterator = self.columns().iter().enumerate();

        for (col_idx, opt_col) in iterator {
            if let Some(column) = opt_col {
                for value in column.rows.iter() {
                    writeln!(buffer, "{} {}", value.get_row_index() + 1, col_idx + 1).map_err(
                        |e| {
                            CSCErr::Dump(format!(
                                "Failed to write the value {} at row={} column={} due to {}",
                                value.get_value(),
                                value.get_row_index(),
                                col_idx,
                                e
                            ))
                        },
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn remove_edges(&self, treshold: f64, seed: u64) -> Result<CSC, CSCErr> {
        let mut local_seed = seed;
        let nb_threads = &self.pool().num_workers();
        let (tx, rx) = unbounded();

        let total_len = self.size() as usize;
        let chunk_size = (total_len / (nb_threads * 2)) + 1;

        let columns = Arc::new(self.columns().clone());

        for chunk_id in 0..nb_threads * 2 {
            let start = chunk_id * chunk_size;
            if start >= total_len {
                break;
            }
            let end = (start + chunk_size).min(total_len);

            let tx_c = tx.clone();
            let columns_c = Arc::clone(&columns);

            let _ = &self
                .pool()
                .execute(move || {
                    filter_edges(
                        tx_c,
                        columns_c,
                        treshold,
                        chunk_id,
                        start,
                        end,
                        total_len,
                        local_seed,
                    )
                })
                .map_err(|e| CSCErr::Thread(format!("Thread Pool error : {}", e)))?;
            local_seed += 1;
        }
        drop(tx);

        let mut filtered_columns = vec![Vec::new(); nb_threads * 2];
        let mut rows_count = vec![0u64; self.size() as usize];
        for (chunk_id, chunk_cols, chunk_rows_count) in rx.iter() {
            filtered_columns[chunk_id] = chunk_cols;
            rows_count = rows_count
                .iter()
                .zip(chunk_rows_count)
                .map(|(x, y)| x + y)
                .collect();
        }

        let renormalized: Vec<Option<LinkedList<Value>>> = filtered_columns
            .into_iter()
            .flatten()
            .map(|opt_col| {
                opt_col.map(|col| {
                    col.into_iter()
                        .map(|v| {
                            let row_idx = v.get_row_index();
                            let count = rows_count[row_idx];
                            Value::from(if count == 0 { 0.0 } else { 1.0 / count as f64 }, row_idx)
                        })
                        .collect::<LinkedList<Value>>()
                })
            })
            .collect();

        CSC::from(self.size(), renormalized, rows_count, self.alpha())
    }
}
