pub use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Reset the MAC address to the permaddr of the interface
    Reset { ifname: String },
    /// Set the MAC address to a specified value
    Set { ifname: String, addr: String },
    /// Set a random MAC address
    Random { ifname: String },
    /// Get current MAC address
    Get { ifname: String },
    /// Get permanent MAC address
    Perm { ifname: String },
    /// Get a list of all available interfaces
    Inames {
        /// List all interfaces in a single line rather than one on each line
        #[clap(short = '1', long)]
        single_line: bool,
    },
    /// Print shell completion definitions
    Completions {
        /// Print completions for the specified shell
        #[clap(subcommand)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum Shell {
    Fish,
}
