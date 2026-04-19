const BARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

pub fn render_sparkline(values: &[u64]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let max = values.iter().copied().max().unwrap_or(1).max(1);
    values
        .iter()
        .map(|value| {
            let idx = ((*value as f64 / max as f64) * (BARS.len() as f64 - 1.0)).round() as usize;
            BARS[idx.min(BARS.len() - 1)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::render_sparkline;

    #[test]
    fn sparkline_handles_empty_and_scaled_data() {
        assert_eq!(render_sparkline(&[]), "");
        let output = render_sparkline(&[0, 5, 10]);
        assert_eq!(output.len(), 3);
    }
}
