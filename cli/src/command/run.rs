use crate::init::AppContext;

pub async fn execute(context: &AppContext, request: &str) -> Result<String, String> {
    let memory_summary = context
        .memory
        .memory_summary()
        .await
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "run command received request:\n{request}\n\ncurrent default provider: {}\nproject root: {}\n\nmemory summary:\n{}",
        context.config.provider.default_provider,
        context.project_root.display(),
        memory_summary
    ))
}
