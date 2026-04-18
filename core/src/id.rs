use uuid::Uuid;

/// Generate a UUID v4 without hyphens.
#[inline]
pub fn generate_id() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Generate a prefixed trace identifier.
#[inline]
pub fn generate_trace_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4().simple())
}
