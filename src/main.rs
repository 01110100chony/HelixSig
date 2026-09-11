use clap::{Parser, Subcommand};
use helix::config::RunConfig;
use std::io::Write;

#[derive(Parser)]
#[command(version, about = "Bounded sequential signal-processing experiment")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Run(RunConfig),
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = if error.use_stderr() { 64 } else { 0 };
            let _ = error.print();
            std::process::exit(code);
        }
    };
    let Command::Run(config) = cli.command;
    let summary = helix::runtime::run(config);
    let mut stdout = std::io::stdout().lock();
    if serde_json::to_writer(&mut stdout, &summary).is_err()
        || stdout
            .write_all(b"\n")
            .and_then(|()| stdout.flush())
            .is_err()
    {
        eprintln!("failed to emit final summary to stdout");
        std::process::exit(1);
    }
    std::process::exit(summary.exit_code());
}
