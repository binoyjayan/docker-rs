use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(clap::Subcommand)]
pub(crate) enum Commands {
    /// Initialize git repository
    Init,
    /// cat file with pretty print
    CatFile {
        args: Vec<String>,        
    },    
}