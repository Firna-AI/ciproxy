//! The VM tool of the macOS image build: install, run with VNC, and the guest
//! address. `image/macos/build.sh` drives it. It runs only on macOS.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

#[cfg(target_os = "macos")]
mod macos;

#[derive(Parser)]
#[command(about = "Install and run the VM of the macOS image build")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a bundle and install macOS into it from a restore image.
    Install {
        /// The restore image (.ipsw).
        #[arg(long)]
        restore_image: PathBuf,
        /// The new bundle directory. It must not exist.
        #[arg(long)]
        bundle: PathBuf,
        /// The logical size of the sparse startup disk.
        #[arg(long)]
        disk_bytes: u64,
        #[command(flatten)]
        machine: Machine,
    },
    /// Run the VM with a VNC server until the guest turns off or the stop
    /// file appears.
    Run {
        #[arg(long)]
        bundle: PathBuf,
        /// Start in macOS Recovery.
        #[arg(long)]
        recovery: bool,
        /// A file with the VNC password. VNC listens on every interface.
        #[arg(long)]
        vnc_password_file: PathBuf,
        /// The tool writes the VNC port here, as one JSON line, after start.
        #[arg(long)]
        status_file: PathBuf,
        /// When this file appears, the tool stops the VM.
        #[arg(long)]
        stop_file: PathBuf,
        #[command(flatten)]
        machine: Machine,
    },
    /// Print the IPv4 address of the VM from the DHCP leases.
    Ip {
        #[arg(long)]
        bundle: PathBuf,
        /// Wait up to this many seconds for a lease.
        #[arg(long, default_value_t = 0)]
        wait_seconds: u64,
    },
}

/// The VM's CPU, memory, and display.
#[derive(Args, Clone, Copy)]
struct Machine {
    #[arg(long, default_value_t = 4)]
    cpus: u32,
    #[arg(long, default_value_t = 8)]
    memory_gib: u64,
    #[arg(long, default_value_t = 1920)]
    width: u32,
    #[arg(long, default_value_t = 1080)]
    height: u32,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    #[cfg(target_os = "macos")]
    {
        match macos::run(cli.command) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("ciproxy-image-vm: {error}");
                ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _unused = cli.command;
        eprintln!("ciproxy-image-vm runs only on macOS");
        ExitCode::FAILURE
    }
}
