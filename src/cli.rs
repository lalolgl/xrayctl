use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "xrayctl",
    version,
    about = "Xray subscription manager",
    long_about = "A simple command-line manager for Xray subscriptions and connections."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        name = "sub",
        alias = "subscription",
        about = "Manage subscriptions and Xray"
    )]
    Subscription(SubArgs),
}

#[derive(clap::Args)]
pub struct SubArgs {
    /// Start Xray
    #[arg(
        short = 's',
        long = "start",
        conflicts_with_all = ["stop", "status", "list", "update", "generate"]
    )]
    pub start: bool,

    /// Stop Xray
    #[arg(
        short = 'x',
        long = "stop",
        conflicts_with_all = ["start", "status", "list", "update", "generate"]
    )]
    pub stop: bool,

    /// Show Xray status
    #[arg(
        short = 't',
        long = "status",
        conflicts_with_all = ["start", "stop", "list", "update", "generate"]
    )]
    pub status: bool,

    /// List profiles
    #[arg(
        short = 'l',
        long = "list",
        conflicts_with_all = ["start", "stop", "status", "update", "generate"]
    )]
    pub list: bool,

    /// Update subscription
    #[arg(
        short = 'u',
        long = "update",
        conflicts_with_all = ["start", "stop", "status", "list", "generate"]
    )]
    pub update: bool,

    /// Generate Xray configuration
    #[arg(
        short = 'g',
        long = "generate",
        conflicts_with_all = ["start", "stop", "status", "list", "update"]
    )]
    pub generate: bool,

    #[command(subcommand)]
    pub command: Option<SubscriptionCommand>,
}

#[derive(Subcommand)]
pub enum SubscriptionCommand {
    #[command(about = "Add a subscription")]
    Add { url: String },

    #[command(about = "Show subscription URL")]
    Show,

    #[command(about = "Download subscription")]
    Fetch,

    #[command(about = "Show subscription information")]
    Info,

    #[command(about = "List available profiles")]
    List,

    #[command(about = "Debug subscription profiles")]
    Debug,

    #[command(about = "Show detailed profile information")]
    ShowProfile { index: usize },

    #[command(about = "Select active profile")]
    Use { index: usize },

    #[command(about = "Generate Xray configuration")]
    Generate,

    #[command(about = "Start Xray")]
    Start { index: Option<usize> },

    #[command(about = "Stop Xray")]
    Stop,

    #[command(about = "Show Xray status")]
    Status,

    #[command(about = "Update subscription")]
    Update,
}
