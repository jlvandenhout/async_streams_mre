use futures::TryStreamExt;
use rand::{self, Rng};
use streams::{id::Ed25519, transport::utangle::Client, MessageContent, User};
use tokio::{sync::mpsc::channel, task::LocalSet};

fn create_user() -> User<Client> {
    let seed: [u8; 32] = rand::thread_rng().gen();
    let identity = Ed25519::from_seed(seed);
    let transport = Client::default();
    User::builder()
        .with_identity(identity)
        .with_transport(transport)
        .build()
}

#[tokio::main]
async fn main() {
    const TOPIC: &str = "TOPIC";

    let local = LocalSet::new();
    let (transmitter, mut receiver) = channel(32);

    let mut author = create_user();
    let announcement = author.create_stream(TOPIC).await.unwrap();
    local.spawn_local(async move {
        while let Some(message) = receiver.recv().await {
            author
                .message()
                .public()
                .with_topic(TOPIC)
                .with_payload(message)
                .send()
                .await
                .unwrap();

            println!("> Published {}", message);
        }
    });
    println!("> Created the author");

    let mut recipient = create_user();
    recipient
        .receive_message(announcement.address())
        .await
        .unwrap();
    local.spawn_local(async move {
        loop {
            if let Some(message) = recipient.messages().try_next().await.unwrap() {
                match message.content() {
                    MessageContent::TaggedPacket(content) => {
                        println!(
                            "> Received {}",
                            std::str::from_utf8(&content.public_payload).unwrap()
                        );
                    }
                    _ => {
                        println!("! Unrecognized message type");
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });
    println!("> Created the recipient");

    tokio::spawn(async move {
        let messages = ["one", "two", "three"];
        for message in messages {
            transmitter.send(message).await.unwrap();
            println!("> Transmitted {}", message);
        }
    });
    println!("> Started transmitting");

    local.await;
}
