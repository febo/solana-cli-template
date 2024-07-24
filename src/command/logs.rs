use futures_util::StreamExt;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::commitment_config::CommitmentConfig;

pub async fn process_logs(websocket_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pubsub_client = PubsubClient::new(websocket_url).await?;

    let (mut logs, logs_unsubscribe) = pubsub_client
        .logs_subscribe(
            RpcTransactionLogsFilter::All,
            RpcTransactionLogsConfig {
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .await?;

    while let Some(log) = logs.next().await {
        println!("Transaction executed in slot {}:", log.context.slot);
        println!("  Signature: {}:", log.value.signature);
        println!(
            "  Status: {}",
            log.value
                .err
                .map(|err| err.to_string())
                .unwrap_or_else(|| "Success".into())
        );
        println!("  Log Messages:");
        for msg in log.value.logs {
            println!("    {msg}");
        }
    }
    logs_unsubscribe().await;
    Ok(())
}
