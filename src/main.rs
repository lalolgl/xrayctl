use clap::Parser;

mod cli;
mod commands;
mod config;
mod subscription;
mod subscription_client;
mod ui;
mod xray;

use cli::{Cli, Commands, SubscriptionCommand};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Subscription(args) => {
            let command = if args.start {
                Some(SubscriptionCommand::Start { index: None })
            } else if args.stop {
                Some(SubscriptionCommand::Stop)
            } else if args.status {
                Some(SubscriptionCommand::Status)
            } else if args.list {
                Some(SubscriptionCommand::List)
            } else if args.update {
                Some(SubscriptionCommand::Update)
            } else if args.generate {
                Some(SubscriptionCommand::Generate)
            } else {
                args.command
            };

            commands::subscription::run(command);
        }
    }
}
