use std::fmt;

use crate::types::Shape;

pub type RefErr = Box<dyn std::error::Error + Send + 'static>;

#[derive(Debug, Clone)]
pub enum ParseErr {
    File(String),
    Header(String),
    Shape(String),
    Value(String, usize),
    Index(String, usize),
    Thread(String),
    CSC,
}

#[derive(Debug)]
pub enum ThreadPoolErr {
    ShutdownTimeout,
    ThreadExec(String),
    ThreadJoin(String),
    JobSignal(String),
}

#[derive(Debug, Clone)]
pub enum CSCErr {
    ShapeColumn(Shape, usize),
    Thread(String),
    SendErr,
    Epsilon(f64),
    ShapeVec(usize, usize),
}

#[derive(Debug, Clone)]
pub enum CLIErr {
    Alpha(String),
    Epsilon(String),
    File(String),
}

impl std::error::Error for ParseErr {}
impl std::error::Error for ThreadPoolErr {}
impl std::error::Error for CSCErr {}
impl std::error::Error for CLIErr {}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErr::CSC => write!(
                f,
                "Failed to construct the Compressed Sparse Column (CSC) matrix."
            ),
            ParseErr::File(s) => write!(f, "Failed to process the matrix file due to : [{}]", s),
            ParseErr::Header(s) => write!(f, "Failed to parse Header due to : [{}]", s),
            ParseErr::Index(s, line) => {
                write!(f, "Failed to parse index due to [{}] at line {}", s, line)
            }
            ParseErr::Shape(s) => write!(f, "Failed to parse shape due to : [{}]", s),
            ParseErr::Thread(s) => write!(
                f,
                "Encountered the following thread issue while parsing : [{}]",
                s
            ),
            ParseErr::Value(s, line) => {
                write!(f, "Failed to parse value at line {} due to : {}", line, s)
            }
        }
    }
}

impl fmt::Display for ThreadPoolErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreadPoolErr::JobSignal(s) => write!(f, "Job signal issue : [{}]", s),
            ThreadPoolErr::ShutdownTimeout => write!(
                f,
                "Failed to Shutdown the ThreadPool before the end of the timeout."
            ),
            ThreadPoolErr::ThreadExec(s) => write!(f, "Thread execution error : [{}]", s),
            ThreadPoolErr::ThreadJoin(s) => write!(f, "Thread join issue : [{}]", s),
        }
    }
}

impl fmt::Display for CSCErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CSCErr::ShapeColumn(shape, columns) => {
                write!(
                    f,
                    "The shape isn't the same as columns : {:?}, columns {}",
                    shape, columns
                )
            }
            CSCErr::Thread(s) => write!(f, "Thread error : {}", s),
            CSCErr::SendErr => write!(f, "Send error"),
            CSCErr::Epsilon(eps) => write!(f, "Epsilon error : {}", eps),
            CSCErr::ShapeVec(rows, vec_size) => write!(
                f,
                "Matrix shape is incompatible with vector size : Matrix rows {}, columnar vector size {}",
                rows, vec_size
            ),
        }
    }
}

impl fmt::Display for CLIErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CLIErr::Alpha(s) => write!(f, "{}", s),
            CLIErr::Epsilon(s) => write!(f, "{}", s),
            CLIErr::File(s) => write!(f, "File error : [{}]", s),
        }
    }
}
