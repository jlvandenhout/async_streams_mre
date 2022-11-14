use rand::{self, Rng};
use streams::{id::Ed25519, transport::utangle::Client, User};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    const topic: &str = "TOPIC";
    let (transmitter, mut receiver) = channel(32);

    tokio::spawn(async move {
        let messages = ["one", "two", "three"];
        for message in messages {
            transmitter.send(message).await.unwrap()
        }
    });

    tokio::spawn(async move {
        let seed: [u8; 32] = rand::thread_rng().gen();
        let identity = Ed25519::from_seed(seed);
        let transport: Client = Client::default();
        let mut user = User::builder()
            .with_identity(identity)
            .with_transport(transport)
            .build();

        user.create_stream(topic).await.unwrap();

        while let Some(message) = receiver.recv().await {
            user.message()
                .with_topic(topic)
                .with_payload(message)
                .send()
                .await
                .unwrap();
        }
    });
}
