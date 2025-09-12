use anyhow::Result;
use clap::Parser;
use proto_sign::spec::{Compatibility, Spec};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "proto-sign")]
#[command(about = "Check Protobuf file compatibility and generate semantic fingerprints")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Compare two .proto files for compatibility")]
    Compare {
        #[arg(help = "Path to the old .proto file")]
        old_file: PathBuf,
        #[arg(help = "Path to the new .proto file")]
        new_file: PathBuf,
    },
    #[command(about = "Generate semantic fingerprint for a .proto file")]
    Fingerprint {
        #[arg(help = "Path to the .proto file")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Compare { old_file, new_file } => {
            let old_content = fs::read_to_string(&old_file).map_err(|e| {
                anyhow::anyhow!("Failed to read old file '{}': {}", old_file.display(), e)
            })?;
            let new_content = fs::read_to_string(&new_file).map_err(|e| {
                anyhow::anyhow!("Failed to read new file '{}': {}", new_file.display(), e)
            })?;

            let old_spec = Spec::try_from(old_content.as_str())?;
            let new_spec = Spec::try_from(new_content.as_str())?;

            let compatibility = old_spec.compare_with(&new_spec);

            match compatibility {
                Compatibility::Green => {
                    println!("Green: Files are semantically identical");
                    std::process::exit(0);
                }
                Compatibility::Yellow => {
                    println!("Yellow: New file is backward-compatible with old file");
                    std::process::exit(0);
                }
                Compatibility::Red => {
                    println!("Red: Breaking change detected");
                    std::process::exit(1);
                }
            }
        }
        Commands::Fingerprint { file } => {
            let content = fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file.display(), e))?;

            let fingerprint = proto_sign::generate_fingerprint(&content)?;
            println!("{}", fingerprint);
        }
    }

    Ok(())
}
