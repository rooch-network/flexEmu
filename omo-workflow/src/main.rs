use clap::Parser;
use log::LevelFilter;
use omo::{config::OmoConfig, parse_key_val};
use omo_workflow::RunUnit;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, ValidCryptoMaterialStringExt};
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use std::{fs::read, path::PathBuf};
#[derive(Parser)]
struct Options {
    #[clap(long = "config")]
    /// config file of the emulation.
    config_file: PathBuf,
    /// exec file
    #[clap(long = "exec")]
    exec: PathBuf,
    #[clap(long = "arg")]
    args: Vec<String>,
    #[clap(long = "env", parse(try_from_str=parse_key_val))]
    envs: Vec<(String, String)>,

    #[clap(long = "keyfile")]
    key_file: PathBuf,

    #[clap(
        long = "nodeurl",
        help = "starcoin node rpc url",
        default_value = "ws://localhost:9870"
    )]
    node_url: String,

    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Parser)]
enum Commands {
    Proposer {
        fault_step: Option<usize>,
    },
    Challenger {
        #[clap(long = "proposer")]
        /// proposer address
        proposer: AccountAddress,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let Options {
        config_file,
        exec,
        args,
        envs,
        key_file,
        node_url,
        cmd,
    } = Options::parse();
    let config: OmoConfig =
        toml::from_str(&std::fs::read_to_string(&config_file).unwrap()).unwrap();
    let binary = read(exec.as_path()).unwrap();
    let argv = {
        let mut a = args;
        a.insert(0, exec.file_name().unwrap().to_string_lossy().to_string());
        a
    };
    let account = Ed25519PrivateKey::from_encoded_string(
        std::fs::read_to_string(key_file.as_path())?.trim(),
    )?;
    let client = RpcClient::connect_websocket(&node_url).unwrap();
    match cmd {
        Commands::Proposer { fault_step } => {
            let p = omo_workflow::Proposer::new(
                config,
                RunUnit {
                    env: envs,
                    argv,
                    binary,
                },
                account,
                client,
                fault_step,
            );
            p.run()?;
        }
        Commands::Challenger { proposer } => {
            let challenger = omo_workflow::Challenger::new(
                config,
                RunUnit {
                    env: envs,
                    argv,
                    binary,
                },
                account,
                client,
                proposer,
            );
            challenger.run()?;
        }
    }

    Ok(())
}
