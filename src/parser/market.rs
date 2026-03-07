use std::str::FromStr;

use crate::{
    parser::api::{ParseErr, Parsed},
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

            let result = iterator
                .enumerate()
                .map(parse_line)
                .collect::<Result<Vec<Parsed>, ParseErr>>()?;

            Ok((shape, result))
        }
        _ => Err(ParseErr::Header(format!(
            "Parser isn't yet implemented for : {:?}",
            header
        ))),
    }
}

fn parse_line(couple: (usize, String)) -> Result<Parsed, ParseErr> {
    let (index, line) = couple;
    let mut iter = line.split(' ').flat_map(|v| v.parse::<f64>());

    let row_idx = iter
        .next()
        .ok_or_else(|| ParseErr::Value("Failed to get the row index.".to_string(), index))?
        as usize
        - 1;
    let col_idx = iter
        .next()
        .ok_or_else(|| ParseErr::Value("Failed to get the column index.".to_string(), index))?
        as usize
        - 1;
    let val = iter.next().unwrap_or(1.0);

    Ok(Parsed::new(row_idx, col_idx, val))
}
