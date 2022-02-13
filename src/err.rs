use std::fmt::Display;

/// Errors that can occur when accessing HDFS
#[derive(thiserror::Error, Debug)]
pub enum HdfsErr {
    /// File path
    FileNotFound(String),
    /// File path
    FileAlreadyExists(String),
    /// Namenode address
    CannotConnectToNameNode(String),
    /// URL
    InvalidUrl(String),
    /// Description
    Miscellaneous(String),
}

impl Display for HdfsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
