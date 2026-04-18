use mc_core::ChangeType;

pub(crate) fn infer_change_type(path: &str) -> ChangeType {
    if path.ends_with("Cargo.toml")
        || path.ends_with("Cargo.lock")
        || path.ends_with("package.json")
        || path.ends_with("pyproject.toml")
        || path.ends_with("go.mod")
    {
        ChangeType::ModifyConfig
    } else {
        ChangeType::ModifyFile
    }
}
