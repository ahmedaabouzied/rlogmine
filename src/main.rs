use tokio::sync::mpsc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

mod cluster;

#[tokio::main]
async fn main() {
    let (input_sender, mut input_receiver) = mpsc::channel::<String>(100);
    let (output_sender, mut output_receiver) = mpsc::channel::<String>(100);
    let mut clusters = cluster::Clusters::new();

    tokio::spawn(async move {
        clusters.start(&mut input_receiver, output_sender).await;
    });

    // Spawn the background task
    tokio::spawn(async move {
        println!("Started receiving process");
        while let Some(message) = output_receiver.recv().await {
            println!("Received: {}", message);
        }
    });


    let mut lines = BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        input_sender.send(line).await.unwrap();
    }
}
