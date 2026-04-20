pub fn render_progress_bar(width: usize, progress_percent: u8) -> String {
    let width = width.max(1);
    let progress_percent = progress_percent.min(100);
    let filled = width * progress_percent as usize / 100;
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
    use super::render_progress_bar;

    #[test]
    fn progress_bar_renders_bounds() {
        assert_eq!(render_progress_bar(10, 0), "[----------]   0%");
        assert_eq!(render_progress_bar(10, 50), "[#####-----]  50%");
        assert_eq!(render_progress_bar(10, 100), "[##########] 100%");
    }
}
