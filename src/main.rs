mod command;
use command::*;

use clap::{IntoApp, Parser};
use logs::process_logs;
use ping::process_ping;
use solana_clap_v3_utils::{
    input_parsers::signer::SignerSource, input_validators::normalize_to_url_if_moniker,
    keypair::signer_from_path,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_remote_wallet::remote_wallet::RemoteWalletManager;
use solana_sdk::{commitment_config::CommitmentConfig, native_token::Sol, signer::Signer};
use std::{process::exit, rc::Rc};

struct Config {
    commitment_config: CommitmentConfig,
    default_signer: Box<dyn Signer>,
    json_rpc_url: String,
    verbose: bool,
    websocket_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();
    let mut wallet_manager: Option<Rc<RemoteWalletManager>> = None;

    let config = {
        let cli_config = if let Some(config_file) = args.config_file {
            solana_cli_config::Config::load(&config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        if let Some((_, matches)) = Arguments::command().get_matches().subcommand() {
            let default_signer = if let Ok(Some((signer, _))) =
                SignerSource::try_get_signer(matches, "keypair", &mut wallet_manager)
            {
                Box::new(signer)
            } else {
                signer_from_path(
                    matches,
                    &cli_config.keypair_path,
                    "keypair",
                    &mut wallet_manager,
                )?
            };

            let json_rpc_url =
                normalize_to_url_if_moniker(args.json_rpc_url.unwrap_or(cli_config.json_rpc_url));
            let websocket_url = solana_cli_config::Config::compute_websocket_url(&json_rpc_url);

            Config {
                commitment_config: CommitmentConfig::confirmed(),
                default_signer,
                json_rpc_url,
                verbose: args.verbose,
                websocket_url,
            }
        } else {
            panic!("No subcommand provided");
        }
    };
    solana_logger::setup_with_default("solana=info");

    if config.verbose {
        println!("JSON RPC URL: {}", config.json_rpc_url);
        println!("Websocket URL: {}", config.websocket_url);
    }
    let rpc_client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), config.commitment_config);

    match args.command {
        Commands::Balance { address } => {
            let address = address.unwrap_or_else(|| config.default_signer.pubkey());
            println!(
                "{} has a balance of {}",
                address,
                Sol(rpc_client.get_balance(&address).await?)
            );
        }
        Commands::Logs => {
            process_logs(&config.websocket_url)
                .await
                .unwrap_or_else(|err| {
                    eprintln!("error: {err}");
                    exit(1);
                });
        }
        Commands::Ping => {
            let signature = process_ping(&rpc_client, config.default_signer.as_ref())
                .await
                .unwrap_or_else(|err| {
                    eprintln!("error: send transaction: {err}");
                    exit(1);
                });
            println!("Signature: {signature}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use {super::*, solana_test_validator::*};

    #[tokio::test]
    async fn test_ping() {
        let (test_validator, payer) = TestValidatorGenesis::default().start_async().await;
        let rpc_client = test_validator.get_async_rpc_client();

        assert!(process_ping(&rpc_client, &payer).await.is_ok());
    }
}
