/// 残り時間を推定する（分単位）
pub fn estimate_remaining_time(processed: usize, total: usize, elapsed_seconds: f64) -> f64 {
    if processed == 0 || elapsed_seconds == 0.0 {
        return 0.0;
    }

    let items_per_second = processed as f64 / elapsed_seconds;
    let remaining_items = (total - processed) as f64;

    if items_per_second > 0.0 {
        remaining_items / items_per_second / 60.0
    } else {
        0.0
    }
}
