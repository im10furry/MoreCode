use std::path::Path;

use crate::layer::PromptLayer;

pub fn is_supported_prompt_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase()),
        Some(extension) if matches!(extension.as_str(), "md" | "yaml" | "json")
    )
}

pub fn infer_layer_from_path(path: &Path) -> PromptLayer {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect::<Vec<_>>();

    if components
        .iter()
        .any(|part| part == "system" || part == "tools")
    {
        PromptLayer::Global
    } else if components
        .iter()
        .any(|part| part == "org" || part == "organization")
    {
        PromptLayer::Organization
    } else if components.iter().any(|part| part == "session") {
        PromptLayer::Session
    } else {
        PromptLayer::Project
    }
}
