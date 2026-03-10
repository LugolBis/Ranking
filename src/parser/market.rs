use std::{str::FromStr, sync::Arc, thread, time::Duration};

use crossbeam_channel::unbounded;

use crate::{
    parser::{
        api::{ParseErr, Parsed},
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

#[derive(Debug, Clone, Copy)]
struct Header(Object, Format, Field, Symmetry);

impl FromStr for Object {
    type Err = ParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "matrix" => Ok(Object::Matrix),
            "vector" => Ok(Object::Vector),
            unknow => Err(ParseErr::Header(format!("Unknow Object : '{unknow}'."))),
        }
    }
}

impl FromStr for Format {
    type Err = ParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "coordinate" => Ok(Format::Coordinate),
            "array" => Ok(Format::Array),
            unknow => Err(ParseErr::Header(format!("Unknow Format : '{unknow}'."))),
        }
    }
}

impl FromStr for Field {
    type Err = ParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "real" => Ok(Field::Real),
            "double" => Ok(Field::Double),
            "complex" => Ok(Field::Complex),
            "integer" => Ok(Field::Integer),
            "pattern" => Ok(Field::Pattern),
            unknow => Err(ParseErr::Header(format!("Unknow Field : '{unknow}'."))),
        }
    }
}

impl FromStr for Symmetry {
    type Err = ParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "general" => Ok(Symmetry::General),
            "symmetric" => Ok(Symmetry::Symmetric),
            "skew_symmetric" => Ok(Symmetry::SkewSymmetric),
            "hermitian" => Ok(Symmetry::Hermitian),
            unknow => Err(ParseErr::Header(format!("Unknow Symmetry : '{unknow}'."))),
        }
    }
}

impl Header {
    fn from(line: Option<String>) -> Result<Header, ParseErr> {
        if let Some(content) = line {
            let parts = content.split(' ').collect::<Vec<&str>>();
            let obj =
                Object::from_str(parts.get(1).ok_or_else(|| {
                    ParseErr::Header("There isn't the object header.".to_owned())
                })?)?;

            let fmt =
                Format::from_str(parts.get(2).ok_or_else(|| {
                    ParseErr::Header("There isn't the object header.".to_owned())
                })?)?;

            let field =
                Field::from_str(parts.get(3).ok_or_else(|| {
                    ParseErr::Header("There isn't the object header.".to_owned())
                })?)?;

            let sym =
                Symmetry::from_str(parts.get(4).ok_or_else(|| {
                    ParseErr::Header("There isn't the object header.".to_owned())
                })?)?;

            Ok(Header(obj, fmt, field, sym))
        } else {
            Err(ParseErr::Header("There isn't the header line.".to_string()))
        }
    }
}

pub fn market_parser(
    iterator: &mut dyn Iterator<Item = String>,
) -> Result<(Shape, Vec<Parsed>), ParseErr> {
    let header = Header::from(iterator.next())?;

    let mut iterator = iterator.skip_while(|l| l.starts_with('%'));

    match header {
        Header(
            Object::Matrix,
            Format::Coordinate,
            Field::Double | Field::Real | Field::Integer | Field::Pattern,
            Symmetry::General,
        ) => {
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
                            tx_c.send(Ok(chunk))
                                .map_err(|_| Box::<dyn std::error::Error>::from("Send error."))?;
                        }
                        Err(err) => {
                            tx_c.send(Err(err))
                                .map_err(|_| Box::<dyn std::error::Error>::from("Send error."))?;
                        }
                    }

                    Ok(())
                })
                .expect("Failed to execute job");
            }

            pool.shutdown(Duration::from_secs(10))
                .map_err(|e| ParseErr::Thread(format!("Pool error : {:?}", e)))?;
            drop(tx);

            let mut chunks_opt: Vec<Option<Chunk>> = (0..nb_threads).map(|_| None).collect();
            for res in rx {
                let chunk = res?;
                let index = &chunk.get_id();
                chunks_opt[*index] = Some(chunk);
            }
            let chunks = chunks_opt.iter().flatten().collect::<Vec<&Chunk>>();
            let row_count = join_row_count(&chunks)?;

            Ok((
                shape,
                chunks
                    .into_iter()
                    .flat_map(|chunk| chunk.into_parsed(&row_count[..]))
                    .collect::<Vec<Parsed>>(),
            ))
        }
        _ => Err(ParseErr::Header(format!(
            "Parser isn't yet implemented for : {:?}",
            header
        ))),
    }
}

fn join_row_count(chunks: &Vec<&Chunk>) -> Result<Vec<u64>, ParseErr> {
    let mut iterator = chunks.into_iter();

    let mut row_count = iterator
        .next()
        .ok_or_else(|| ParseErr::Thread("There isn't any Chunk.".into()))?
        .get_row_count()
        .clone();

    while let Some(chunk) = iterator.next() {
        for (r, v) in row_count.iter_mut().zip(chunk.get_row_count()) {
            *r += v;
        }
    }

    Ok(row_count)
}

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

fn parse_line(couple: &(usize, String)) -> Result<(usize, usize), ParseErr> {
    let (index, line) = couple;
    let mut parts = line.split_whitespace();

    let row_idx = parts
        .next()
        .ok_or_else(|| ParseErr::Value("Failed to get the row index.".to_string(), *index))?
        .parse::<usize>()
        .map_err(|_| ParseErr::Value("Failed to parse row index.".to_string(), *index))?
        .checked_sub(1)
        .ok_or_else(|| ParseErr::Index("Too low index (0).".to_string(), *index))?;
    let col_idx = parts
        .next()
        .ok_or_else(|| ParseErr::Value("Failed to get the column index.".to_string(), *index))?
        .parse::<usize>()
        .map_err(|_| ParseErr::Value("Failed to parse column index.".to_string(), *index))?
        .checked_sub(1)
        .ok_or_else(|| ParseErr::Index("Too low index (0).".to_string(), *index))?;

    Ok((row_idx, col_idx))
}
