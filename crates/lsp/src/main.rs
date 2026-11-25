//! `compmd-lsp`, the CompMD language server, over stdio.

use clap::Parser;
use lsp_server::Connection;

/// The CompMD language server: an editor starts it and speaks LSP over its standard input and output.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Speak over stdio, the only transport; editors pass it, so it is taken and changes nothing.
    #[arg(long)]
    stdio: bool,
}

fn main() -> anyhow::Result<()> {
    let _ = Args::parse();
    let (connection, io_threads) = Connection::stdio();
    compmd_lsp::serve(&connection)?;
    drop(connection);
    io_threads.join()?;
    Ok(())
}
