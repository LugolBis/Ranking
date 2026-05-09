use crate::errors::ParseErr;

#[derive(Debug, Clone, Copy)]
pub enum Object {
    Matrix,
    Vector,
}

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Coordinate,
    Array,
}

#[derive(Debug, Clone, Copy)]
pub enum Field {
    Real,
    Double,
    Complex,
    Integer,
    Pattern,
}

#[derive(Debug, Clone, Copy)]
pub enum Symmetry {
    General,
    Symmetric,
    SkewSymmetric,
    Hermitian,
}

/// Represent the Header information of the Matrix Market format
#[derive(Debug, Clone, Copy)]
pub struct Header(pub Object, pub Format, pub Field, pub Symmetry);

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
