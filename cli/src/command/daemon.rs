use crate::cli::DaemonCommand;
use crate::init::AppContext;

pub async fn execute(context: &AppContext, command: &DaemonCommand) -> Result<String, String> {
    match command {
        DaemonCommand::Status => {
            let pid = mc_daemon::PidFileGuard::read_pid(&context.config.daemon.pid_file)
                .map_err(|error| error.to_string())?;

            if let Some(pid) = pid {
                Ok(format!(
                    "daemon configured: enabled={}\npid_file: {}\nstatus: running (pid {pid})",
                    context.config.daemon.enabled, context.config.daemon.pid_file
                ))
            } else {
                Ok(format!(
                    "daemon configured: enabled={}\npid_file: {}\nstatus: idle",
                    context.config.daemon.enabled, context.config.daemon.pid_file
                ))
            }
        }
    }
}
