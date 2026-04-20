pub fn progress_ratio(progress_percent: u8) -> f64 {
    f64::from(progress_percent.min(100)) / 100.0
}

pub fn progress_label(progress_percent: u8) -> String {
    format!("{:>3}%", progress_percent.min(100))
}

pub fn render_progress_bar(width: usize, progress_percent: u8) -> String {
    let width = width.max(1);
    let progress_percent = progress_percent.min(100);
    let filled = width * usize::from(progress_percent) / 100;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}] {:>3}%",
        "#".repeat(filled),
        "-".repeat(empty),
        progress_percent
    )
}

#[cfg(test)]
mod tests {
    use super::{progress_label, progress_ratio, render_progress_bar};

    #[test]
    fn progress_bar_renders_bounds() {
        assert_eq!(render_progress_bar(10, 0), "[----------]   0%");
        assert_eq!(render_progress_bar(10, 50), "[#####-----]  50%");
        assert_eq!(render_progress_bar(10, 100), "[##########] 100%");
    }

    #[test]
    fn helpers_clamp_progress() {
        assert_eq!(progress_label(120), "100%");
        assert_eq!(progress_ratio(50), 0.5);
    }
}
