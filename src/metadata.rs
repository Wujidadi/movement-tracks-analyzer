use chrono::NaiveDateTime;

/// 軌跡 Placemark 詮釋資料結構
///
/// 包含軌跡的所有相關資訊，包括時間、座標、分類和距離計算。
///
/// # Fields
///
/// * `name` - 軌跡名稱（通常為時間戳或人工標記）
/// * `start_time` - 軌跡開始時間
/// * `end_time` - 軌跡結束時間
/// * `coordinates` - 軌跡點座標陣列（經度、緯度）
/// * `category` - 活動大分類（如「戶外運動」）
/// * `activity` - 活動細分類（如「步行」）
/// * `year` - 活動年份
/// * `month` - 活動月份（YYYY-MM 格式）
///
/// # Example
///
/// ```ignore
/// use movement_tracks_analyzer::TrackMetadata;
/// use chrono::NaiveDateTime;
///
/// let metadata = TrackMetadata {
///     name: "Morning Walk".to_string(),
///     start_time: NaiveDateTime::parse_from_str("2025-03-11 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     end_time: NaiveDateTime::parse_from_str("2025-03-11 09:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     coordinates: vec![(120.5, 24.7), (120.51, 24.71)],
///     category: "戶外運動".to_string(),
///     activity: "步行".to_string(),
///     year: "2025".to_string(),
///     month: "2025-03".to_string(),
/// };
///
/// let distance = metadata.calculate_distance();
/// let duration = metadata.duration_seconds();
/// ```
#[derive(Debug, Clone)]
pub struct TrackMetadata {
    /// 軌跡名稱
    pub name: String,
    /// 開始時間
    pub start_time: NaiveDateTime,
    /// 結束時間
    pub end_time: NaiveDateTime,
    /// 座標點
    pub coordinates: Vec<(f64, f64)>,
    /// 分類
    pub category: String,
    /// 活動
    pub activity: String,
    /// 年度
    pub year: String,
    /// 月份
    pub month: String,
}

impl TrackMetadata {
    /// 計算軌跡總距離（公尺）
    ///
    /// 使用半正矢（Haversine）公式計算地球表面上兩點間的大圓距離。
    ///
    /// # Returns
    ///
    /// 軌跡總距離，單位為公尺（m）
    ///
    /// # Algorithm
    ///
    /// 半正矢公式計算球面兩點距離：
    /// - 地球半徑：6371 km
    /// - 精度：適合一般 GPS 應用（誤差 < 1%）
    pub fn calculate_distance(&self) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        let mut total_distance = 0.0;

        for i in 0..self.coordinates.len() - 1 {
            let (lon1, lat1) = self.coordinates[i];
            let (lon2, lat2) = self.coordinates[i + 1];

            let lat1_rad = lat1.to_radians();
            let lat2_rad = lat2.to_radians();
            let delta_lat = (lat2 - lat1).to_radians();
            let delta_lon = (lon2 - lon1).to_radians();

            let a = (delta_lat / 2.0).sin().powi(2)
                + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

            total_distance += EARTH_RADIUS_KM * c;
        }

        total_distance * 1000.0 // 轉換為公尺
    }

    /// 計算軌跡持續時間（秒）
    ///
    /// 計算開始時間和結束時間之間的時間差。
    ///
    /// # Returns
    ///
    /// 持續時間，單位為秒（s）
    ///
    /// # Note
    ///
    /// 若 `end_time` 早於 `start_time`，返回負數。
    pub fn duration_seconds(&self) -> i64 {
        self.end_time
            .signed_duration_since(self.start_time)
            .num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn create_test_metadata() -> TrackMetadata {
        TrackMetadata {
            name: "Test Track".to_string(),
            start_time: NaiveDateTime::parse_from_str("2025-03-11 10:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            end_time: NaiveDateTime::parse_from_str("2025-03-11 11:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            coordinates: vec![
                (120.5, 24.7),
                (120.51, 24.71),
                (120.52, 24.72),
            ],
            category: "戶外運動".to_string(),
            activity: "步行".to_string(),
            year: "2025".to_string(),
            month: "2025-03".to_string(),
        }
    }

    #[test]
    fn test_duration_seconds() {
        let metadata = create_test_metadata();
        let duration = metadata.duration_seconds();
        assert_eq!(duration, 5400); // 1 hour 30 minutes = 5400 seconds
    }

    #[test]
    fn test_duration_same_time() {
        let mut metadata = create_test_metadata();
        metadata.end_time = metadata.start_time;
        let duration = metadata.duration_seconds();
        assert_eq!(duration, 0);
    }

    #[test]
    fn test_duration_negative() {
        let mut metadata = create_test_metadata();
        // Swap start and end times
        let temp = metadata.start_time;
        metadata.start_time = metadata.end_time;
        metadata.end_time = temp;
        let duration = metadata.duration_seconds();
        assert!(duration < 0);
    }

    #[test]
    fn test_calculate_distance_multiple_points() {
        let metadata = create_test_metadata();
        let distance = metadata.calculate_distance();
        // Distance should be positive and reasonable for nearby coordinates
        assert!(distance > 0.0);
        assert!(distance < 10000.0); // Less than 10 km
    }

    #[test]
    fn test_calculate_distance_single_point() {
        let mut metadata = create_test_metadata();
        metadata.coordinates = vec![(120.5, 24.7)];
        let distance = metadata.calculate_distance();
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_calculate_distance_two_points() {
        let mut metadata = create_test_metadata();
        metadata.coordinates = vec![(120.5, 24.7), (120.51, 24.71)];
        let distance = metadata.calculate_distance();
        assert!(distance > 0.0);
        assert!(distance < 2000.0); // Less than 2 km for nearby points
    }

    #[test]
    fn test_metadata_creation() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.name, "Test Track");
        assert_eq!(metadata.category, "戶外運動");
        assert_eq!(metadata.activity, "步行");
        assert_eq!(metadata.coordinates.len(), 3);
    }
}
