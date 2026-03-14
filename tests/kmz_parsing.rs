use movement_tracks_analyzer::{extract_placemarks_with_paths, AnalyzerError};
use std::path::PathBuf;

#[test]
fn test_parse_kmz_with_doc_kml() {
    let path = PathBuf::from("tests/fixtures/tracks.kmz");
    let result = extract_placemarks_with_paths(&path);
    assert!(result.is_ok(), "Failed to parse KMZ: {:?}", result.err());
    let data = result.unwrap();
    assert!(!data.is_empty(), "KMZ parsing should return placemarks");
}

#[test]
fn test_parse_kmz_without_doc_kml() {
    // KMZ 中的 KML 名稱不是 doc.kml，應透過 fallback 邏輯找到
    let path = PathBuf::from("tests/fixtures/tracks_no_doc.kmz");
    let result = extract_placemarks_with_paths(&path);
    assert!(result.is_ok(), "Failed to parse KMZ without doc.kml: {:?}", result.err());
    let data = result.unwrap();
    assert!(!data.is_empty(), "KMZ fallback parsing should return placemarks");
}

#[test]
fn test_parse_kmz_no_kml_inside() {
    // KMZ 中不含 KML 檔案，應回傳 KmzError
    let path = PathBuf::from("tests/fixtures/empty.kmz");
    let result = extract_placemarks_with_paths(&path);
    assert!(result.is_err(), "Parsing KMZ without KML should fail");
    let err = result.unwrap_err();
    assert!(
        matches!(err, AnalyzerError::KmzError(_)),
        "Expected KmzError, got: {:?}",
        err
    );
}

#[test]
fn test_kmz_and_kml_produce_same_results() {
    let kml_path = PathBuf::from("tests/fixtures/tracks.kml");
    let kmz_path = PathBuf::from("tests/fixtures/tracks.kmz");

    let kml_result = extract_placemarks_with_paths(&kml_path).unwrap();
    let kmz_result = extract_placemarks_with_paths(&kmz_path).unwrap();

    assert_eq!(
        kml_result.len(),
        kmz_result.len(),
        "KML and KMZ should produce the same number of placemarks"
    );

    // 逐一比對每個 Placemark 的名稱與路徑
    for (i, ((kml_path, kml_meta), (kmz_path, kmz_meta))) in
        kml_result.iter().zip(kmz_result.iter()).enumerate()
    {
        assert_eq!(
            kml_path, kmz_path,
            "Placemark {} folder path mismatch", i
        );
        assert_eq!(
            kml_meta.name, kmz_meta.name,
            "Placemark {} name mismatch", i
        );
        assert_eq!(
            kml_meta.category, kmz_meta.category,
            "Placemark {} category mismatch", i
        );
    }
}

