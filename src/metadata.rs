use chrono::NaiveDateTime;

/// 軌跡 Placemark 詮釋資料結構
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
    /// 計算軌跡距離（公尺）- 使用半正矢（Haversine）公式
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
    pub fn duration_seconds(&self) -> i64 {
        (self.end_time - self.start_time).num_seconds()
    }
}
