use std::{sync::Arc, thread, time::Duration};

use crossbeam_channel::unbounded;

use crate::{
    errors::{ParseErr, RefErr},
    parser::{
        api::Parsed,
        chunk::{Chunk, Coord},
    },
    pool::ThreadPool,
    types::Shape,
};

#[derive(Debug, Clone, Copy)]
enum Object {
    Matrix,
    Vector,
}

#[derive(Debug, Clone, Copy)]
enum Format {
    Coordinate,
    Array,
}

#[derive(Debug, Clone, Copy)]
enum Field {
    Real,
    Double,
    Complex,
    Integer,
    Pattern,
}

#[derive(Debug, Clone, Copy)]
enum Symmetry {
    General,
    Symmetric,
    SkewSymmetric,
    Hermitian,
}

/// Represent the Header information of the Matrix Market format
#[derive(Debug, Clone, Copy)]
struct Header(Object, Format, Field, Symmetry);

impl TryFrom<&str> for Object {
    type Error = ParseErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "matrix" => Ok(Object::Matrix),
            "vector" => Ok(Object::Vector),
            unknow => Err(ParseErr::Header(format!("Unknow Object : '{unknow}'."))),
        }
    }
}

impl TryFrom<&str> for Format {
    type Error = ParseErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "coordinate" => Ok(Format::Coordinate),
            "array" => Ok(Format::Array),
            unknow => Err(ParseErr::Header(format!("Unknow Format : '{unknow}'."))),
        }
    }
}

impl TryFrom<&str> for Field {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "real" => Ok(Field::Real),
            "double" => Ok(Field::Double),
            "complex" => Ok(Field::Complex),
            "integer" => Ok(Field::Integer),
            "pattern" => Ok(Field::Pattern),
            unknow => Err(ParseErr::Header(format!("Unknow Field : '{unknow}'."))),
        }
    }
}

impl TryFrom<&str> for Symmetry {
    type Error = ParseErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "general" => Ok(Symmetry::General),
            "symmetric" => Ok(Symmetry::Symmetric),
            "skew_symmetric" => Ok(Symmetry::SkewSymmetric),
            "hermitian" => Ok(Symmetry::Hermitian),
            unknow => Err(ParseErr::Header(format!("Unknow Symmetry : '{unknow}'."))),
        }
    }
}

impl TryFrom<Option<String>> for Header {
    type Error = ParseErr;
    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        let content = value.ok_or(ParseErr::Header("There isn't the header line.".to_string()))?;
        let mut parts = content.split(' ').skip(1);
        let obj: Object = parts
            .next()
            .ok_or(ParseErr::Header("There isn't the object header.".into()))?
            .try_into()?;

        let fmt = parts
            .next()
            .ok_or(ParseErr::Header("There isn't the format header.".into()))?
            .try_into()?;

        let field: Field = parts
            .next()
            .ok_or(ParseErr::Header("There isn't the field header.".into()))?
            .try_into()?;

        let sym = parts
            .next()
            .ok_or(ParseErr::Header("There isn't the symmetry header.".into()))?
            .try_into()?;

        Ok(Header(obj, fmt, field, sym))
    }
}

/// Parse a Matrix Market file.
pub fn market_parser(
    iterator: &mut dyn Iterator<Item = String>,
) -> Result<(Shape, Vec<Parsed>, Vec<u64>), ParseErr> {
    let header = iterator.next().try_into()?;

    let mut iterator = iterator.skip_while(|l| l.starts_with('%'));

    if let Header(
        Object::Matrix,
        Format::Coordinate,
        Field::Double | Field::Real | Field::Integer | Field::Pattern,
        Symmetry::General,
    ) = header
    {
        let shape = Shape::from(iterator.next(), " ", 0, 1).map_err(|e| ParseErr::Shape(e))?;

        let nb_threads = thread::available_parallelism()
            .map_err(|_| ParseErr::Thread("Failed to get availlable threds.".into()))?
            .get();
        let mut pool = ThreadPool::new(nb_threads);
        let (tx, rx) = unbounded::<Result<Chunk, ParseErr>>();

        let lines = Arc::new(iterator.enumerate().collect::<Vec<(usize, String)>>());
        let total_len = lines.len();
        let chunk_size = (total_len + nb_threads - 1) / nb_threads;

        for chunk_id in 0..nb_threads {
            let start = chunk_id * chunk_size;
            if start >= total_len {
                break;
            }
            let end = (start + chunk_size).min(total_len);

            let tx_c = tx.clone();
            let lines_ref = Arc::clone(&lines);

            pool.execute(move || {
                match parse_chunk(chunk_id, &lines_ref[start..end], shape.rows() as usize) {
                    Ok(chunk) => {
                        tx_c.send(Ok(chunk)).map_err(|_| {
                            Box::new(ParseErr::Thread("Sender error.".into())) as RefErr
                        })?;
                    }
                    Err(err) => {
                        tx_c.send(Err(err)).map_err(|_| {
                            Box::new(ParseErr::Thread("Sender error.".into())) as RefErr
                        })?;
                    }
                }

                Ok(())
            })
            .map_err(|e| ParseErr::Thread(format!("ThreadPool error {:?}", e)))?;
        }

        pool.shutdown(Duration::from_secs(2))
            .map_err(|e| ParseErr::Thread(format!("ThreadPool error : {:?}", e)))?;
        drop(tx);

        let mut chunks_opt: Vec<Option<Chunk>> = (0..nb_threads).map(|_| None).collect();
        let mut step = 0f64;
        for res in rx {
            let chunk = res?;
            let index = &chunk.get_id();
            chunks_opt[*index] = Some(chunk);

            step += 1f64;
            println!("[Parsing {}%]", (step / nb_threads as f64) * 100f64)
        }
        let chunks = chunks_opt.iter().flatten().collect::<Vec<&Chunk>>();
        let row_count = join_row_count(&chunks)?;

        Ok((
            shape,
            chunks
                .into_iter()
                .flat_map(|chunk| chunk.into_parsed(&row_count[..]))
                .collect::<Vec<Parsed>>(),
            row_count,
        ))
    } else {
        Err(ParseErr::Header(format!(
            "Parser isn't yet implemented for : {:?}",
            header
        )))
    }
}

/// Join the row count of each computed chunk
fn join_row_count(chunks: &Vec<&Chunk>) -> Result<Vec<u64>, ParseErr> {
    let mut iterator = chunks.into_iter();

    let mut row_count = iterator
        .next()
        .ok_or(ParseErr::Thread("There isn't any Chunk.".into()))?
        .get_row_count()
        .clone();

    while let Some(chunk) = iterator.next() {
        for (r, v) in row_count.iter_mut().zip(chunk.get_row_count()) {
            *r += v;
        }
    }

    Ok(row_count)
}

/// Try to parse an array of lines (with their index) into Chunks.
fn parse_chunk(
    chunk_id: usize,
    chunk: &[(usize, String)],
    rows_len: usize,
) -> Result<Chunk, ParseErr> {
    let mut row_count = vec![0u64; rows_len];
    let mut coords = Vec::new();

    for couple in chunk {
        match parse_line(couple) {
            Ok((row_idx, col_idx)) => {
                coords.push(Coord::from(row_idx, col_idx));
                row_count[row_idx] += 1;
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(Chunk::from(chunk_id, coords, row_count))
}

/// Try to parse a line to extract the : (row index, column index)
fn parse_line(couple: &(usize, String)) -> Result<(usize, usize), ParseErr> {
    let (index, line) = couple;
    let mut parts = line.split_whitespace();

    let row_idx = parts
        .next()
        .ok_or(ParseErr::Value(
            "Failed to get the row index".to_string(),
            *index,
        ))?
        .parse::<usize>()
        .or(Err(ParseErr::Value(
            "Failed to parse row index".to_string(),
            *index,
        )))?
        .checked_sub(1)
        .ok_or(ParseErr::Index("Too low index (0)".to_string(), *index))?;
    let col_idx = parts
        .next()
        .ok_or(ParseErr::Value(
            "Failed to get the column index".to_string(),
            *index,
        ))?
        .parse::<usize>()
        .or(Err(ParseErr::Value(
            "Failed to parse column index".to_string(),
            *index,
        )))?
        .checked_sub(1)
        .ok_or(ParseErr::Index("Too low index (0)".to_string(), *index))?;

    Ok((row_idx, col_idx))
}
