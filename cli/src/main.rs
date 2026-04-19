use cli::{command, AppContext, Cli, Command};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = match Cli::parse(std::env::args()) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let context = match AppContext::initialize(&cli).await {
        Ok(context) => context,
        Err(error) => {
            eprintln!("initialization failed: {error}");
            std::process::exit(1);
        }
    };

    let result = match &cli.command {
        Command::Run { request } => command::run::execute(&context, request).await,
        Command::Memory(memory_command) => command::memory::execute(&context, memory_command).await,
        Command::Config(config_command) => command::config::execute(&context, config_command).await,
        Command::Doctor => command::doctor::execute(&context).await,
        Command::Daemon(daemon_command) => command::daemon::execute(&context, daemon_command).await,
    };

    match result {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
