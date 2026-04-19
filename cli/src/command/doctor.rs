use std::process::Command;

use crate::init::AppContext;

pub async fn execute(context: &AppContext) -> Result<String, String> {
    let git = command_version("git", &["--version"]);
    let rustc = command_version("rustc", &["--version"]);
    let memory_state = context
        .memory
        .load_project_memory_state()
        .await
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "project_root: {}\nconfig_loaded: true\ngit: {}\nrustc: {}\nmemory_state: {:?}",
        context.project_root.display(),
        git,
        rustc,
        memory_state
    ))
}

fn command_version(program: &str, args: &[&str]) -> String {
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            format!("unavailable ({stderr})")
        }
        Err(error) => format!("unavailable ({error})"),
    }
}
