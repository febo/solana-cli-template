use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    message::Message, signature::Signature, signer::Signer, system_instruction,
    transaction::Transaction,
};

pub async fn process_ping(
    rpc_client: &RpcClient,
    signer: &dyn Signer,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let from = signer.pubkey();
    let to = signer.pubkey();
    let amount = 0;

    let mut transaction = Transaction::new_unsigned(Message::new(
        &[system_instruction::transfer(&from, &to, amount)],
        Some(&signer.pubkey()),
    ));

    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .map_err(|err| format!("error: unable to get latest blockhash: {err}"))?;

    transaction
        .try_sign(&vec![signer], blockhash)
        .map_err(|err| format!("error: failed to sign transaction: {err}"))?;

    let signature = rpc_client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .await
        .map_err(|err| format!("error: send transaction: {err}"))?;

    Ok(signature)
}
