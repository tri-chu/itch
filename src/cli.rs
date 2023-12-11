use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Prune,
    Delete(DeleteArgs),
    Diff(DiffArgs),
    List,
    Load(LoadArgs),
    Log,
    Merge,
    New(NewArgs),
    Save(SaveArgs),
    Status(StatusArgs),
    Squash,
    Sync(SyncArgs),
}

#[derive(Args)]
pub struct NewArgs {
    pub name: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    pub names: Vec<String>,
}

#[derive(Args)]
pub struct SyncArgs {
    pub names: Vec<String>,
}

#[derive(Args)]
pub struct SaveArgs {
    pub message: Vec<String>,
}

#[derive(Args)]
pub struct LoadArgs {
    pub target: String,
}

#[derive(Args)]
pub struct StatusArgs {
    pub target: Option<String>,
}

#[derive(Args)]
pub struct DiffArgs {
    pub target: Option<String>,
}
