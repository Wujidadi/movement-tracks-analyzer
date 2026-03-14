use std::{error::Error, fmt, io};

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
    /// KMZ 檔案處理錯誤
    KmzError(String),
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
            AnalyzerError::KmzError(msg) => write!(f, "KMZ error: {}", msg),
            AnalyzerError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for AnalyzerError {}

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

/// 從 ZipError 自動轉換為 AnalyzerError
impl From<zip::result::ZipError> for AnalyzerError {
    fn from(error: zip::result::ZipError) -> Self {
        AnalyzerError::KmzError(error.to_string())
    }
}

/// 便捷的 Result 類型別名
pub type Result<T> = std::result::Result<T, AnalyzerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_error_display() {
        let err = AnalyzerError::FileNotFound("/path/to/file.kml".to_string());
        assert_eq!(err.to_string(), "File not found: /path/to/file.kml");
    }

    #[test]
    fn test_analyzer_error_parsing() {
        let err = AnalyzerError::ParsingError("Invalid XML format".to_string());
        assert_eq!(err.to_string(), "Parsing error: Invalid XML format");
    }

    #[test]
    fn test_analyzer_error_from_string() {
        let err: AnalyzerError = "Test error".into();
        assert!(matches!(err, AnalyzerError::Other(_)));
        assert_eq!(err.to_string(), "Test error");
    }

    #[test]
    fn test_analyzer_error_from_io_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: AnalyzerError = io_err.into();
        assert!(matches!(err, AnalyzerError::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        let result: Result<String> = Ok("Success".to_string());
        assert!(result.is_ok());

        let error_result: Result<String> = Err(AnalyzerError::Other("Error".to_string()));
        assert!(error_result.is_err());
    }

    #[test]
    fn test_analyzer_error_kmz() {
        let err = AnalyzerError::KmzError("No KML file found in KMZ archive".to_string());
        assert_eq!(
            err.to_string(),
            "KMZ error: No KML file found in KMZ archive"
        );
    }

    #[test]
    fn test_analyzer_error_from_zip_error() {
        let zip_err = zip::result::ZipError::FileNotFound;
        let err: AnalyzerError = zip_err.into();
        assert!(matches!(err, AnalyzerError::KmzError(_)));
    }
}
