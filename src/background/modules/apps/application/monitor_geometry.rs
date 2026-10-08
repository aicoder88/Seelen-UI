/// Translate a normal window between work areas, accounting for DPI and keeping it visible.
pub fn relocate(rect: [i32; 4], source: [i32; 4], target: [i32; 4], scale: f64) -> [i32; 4] {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let width = (((rect[2] - rect[0]) as f64 * scale).round() as i32)
        .clamp(1, (target[2] - target[0]).max(1));
    let height = (((rect[3] - rect[1]) as f64 * scale).round() as i32)
        .clamp(1, (target[3] - target[1]).max(1));
    let left = (target[0] + ((rect[0] - source[0]) as f64 * scale).round() as i32)
        .clamp(target[0], target[2] - width);
    let top = (target[1] + ((rect[1] - source[1]) as f64 * scale).round() as i32)
        .clamp(target[1], target[3] - height);
    [left, top, left + width, top + height]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitor_left_of_primary_keeps_window_visible() {
        assert_eq!(
            relocate(
                [100, 100, 900, 700],
                [0, 0, 1920, 1040],
                [-1920, 0, 0, 1040],
                1.0
            ),
            [-1820, 100, -1020, 700]
        );
    }

    #[test]
    fn smaller_monitor_clamps_oversized_window_to_work_area() {
        assert_eq!(
            relocate(
                [100, 100, 2500, 1500],
                [0, 0, 3840, 2120],
                [3840, 30, 5120, 1024],
                1.0
            ),
            [3840, 30, 5120, 1024]
        );
    }

    #[test]
    fn dpi_change_preserves_logical_size_and_offset() {
        assert_eq!(
            relocate(
                [100, 100, 900, 700],
                [0, 0, 1920, 1040],
                [1920, 40, 4480, 1400],
                1.5
            ),
            [2070, 190, 3270, 1090]
        );
    }
}
