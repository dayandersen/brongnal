use anyhow::Result;
use brongnal::traits::{X3DHClient, X3DHServerClient};
use brongnal::x3dh::*;
use brongnal::MemoryClient;
use tarpc::{client, context};

#[tokio::main]
async fn main() -> Result<()> {
    let (client_transport, _server_transport) = tarpc::transport::channel::unbounded();
    let rpc_client = X3DHServerClient::new(client::Config::default(), client_transport).spawn();
    let mut bob = MemoryClient::new();
    rpc_client
        .set_spk(
            context::current(),
            "Bob".to_owned(),
            bob.get_identity_key()?.verifying_key(),
            bob.get_spk()?,
        )
        .await??;

    rpc_client
        .publish_otk_bundle(
            context::current(),
            "Bob".to_owned(),
            bob.get_identity_key()?.verifying_key(),
            bob.add_one_time_keys(100),
        )
        .await??;

    let bundle = rpc_client
        .fetch_prekey_bundle(context::current(), "Bob".to_owned())
        .await??;

    let alice = MemoryClient::new();
    let (_send_sk, message) = x3dh_initiate_send(bundle, &alice.get_identity_key()?, b"Hi Bob")?;
    rpc_client
        .send_message(context::current(), "Bob".to_owned(), message)
        .await??;

    let messages = rpc_client
        .retrieve_messages(context::current(), "Bob".to_owned())
        .await?;
    let message = &messages.get(0).unwrap();

    let (_recv_sk, msg) = x3dh_initiate_recv(
        &bob.get_identity_key()?.clone(),
        &bob.get_pre_key()?.clone(),
        &message.sender_identity_key,
        message.ephemeral_key,
        message
            .otk
            .map(|otk_pub| bob.fetch_wipe_one_time_secret_key(&otk_pub).unwrap()),
        &message.ciphertext,
    )?;

    println!("Alice sent to Bob: {}", String::from_utf8(msg)?);

    Ok(())
}
