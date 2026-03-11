use std::{fmt, io};

/// 分析器錯誤類型
#[derive(Debug)]
pub enum AnalyzerError {
    /// IO 錯誤
    Io(io::Error),
    /// KML 檔案解析錯誤
    ParsingError(String),
    /// 時間解析錯誤
    TimeParsingError(String),
    /// 座標解析錯誤
    CoordinateParsingError(String),
    /// 檔案不存在
    FileNotFound(String),
    /// 其他錯誤
    Other(String),
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalyzerError::Io(e) => write!(f, "IO error: {}", e),
            AnalyzerError::ParsingError(msg) => write!(f, "Parsing error: {}", msg),
            AnalyzerError::TimeParsingError(msg) => write!(f, "Time parsing error: {}", msg),
            AnalyzerError::CoordinateParsingError(msg) => {
                write!(f, "Coordinate parsing error: {}", msg)
            }
            AnalyzerError::FileNotFound(path) => write!(f, "File not found: {}", path),
            AnalyzerError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AnalyzerError {}

/// 從 io::Error 自動轉換為 AnalyzerError
impl From<io::Error> for AnalyzerError {
    fn from(error: io::Error) -> Self {
        AnalyzerError::Io(error)
    }
}

/// 從 String 自動轉換為 AnalyzerError
impl From<String> for AnalyzerError {
    fn from(error: String) -> Self {
        AnalyzerError::Other(error)
    }
}

/// 從 &str 自動轉換為 AnalyzerError
impl From<&str> for AnalyzerError {
    fn from(error: &str) -> Self {
        AnalyzerError::Other(error.to_string())
    }
}

/// 便捷的 Result 類型別名
pub type Result<T> = std::result::Result<T, AnalyzerError>;
