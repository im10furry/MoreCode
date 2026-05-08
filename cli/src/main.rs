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
        Command::Run(command) => command::run::execute(&context, command).await,
        Command::Review(command) => command::review::execute(&context, command).await,
        Command::Replay(command) => command::replay::execute(&context, command).await,
        Command::Export(command) => command::export::execute(&context, command).await,
        Command::Tui(command) => command::tui::execute(&context, command).await,
        Command::Web(command) => command::web::execute(&context, command).await,
        Command::Memory(memory_command) => command::memory::execute(&context, memory_command).await,
        Command::Config(config_command) => command::config::execute(&context, config_command).await,
        Command::Doctor => command::doctor::execute(&context).await,
        Command::Daemon(daemon_command) => command::daemon::execute(&context, daemon_command).await,
        Command::OtherCli => command::othercli::execute(&context).await,
        Command::OtherCliAutoMigrate => command::othercli::execute_auto_migrate(&context).await,
        Command::Taskpile(taskpile_command) => command::taskpile::execute(&context, taskpile_command).await,
        Command::Help => {
            println!("{}", cli::usage());
            Ok(String::new())
        }
    };

    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
