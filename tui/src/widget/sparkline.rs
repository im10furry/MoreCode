pub fn compress_history(values: &[u64], width: usize) -> Vec<u64> {
    if values.is_empty() || width == 0 {
        return Vec::new();
    }

    if values.len() <= width {
        return values.to_vec();
    }

    let chunk_size = values.len().div_ceil(width);
    values
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().copied().max().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::compress_history;

    #[test]
    fn sparkline_handles_empty_and_scaled_data() {
        assert!(compress_history(&[], 4).is_empty());
        assert_eq!(compress_history(&[0, 5, 10], 4), vec![0, 5, 10]);
        assert_eq!(compress_history(&[1, 3, 2, 4, 6, 5], 3), vec![3, 4, 6]);
    }
}
