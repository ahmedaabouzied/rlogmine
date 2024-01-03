use tokio::sync::mpsc;
use std::io::stdout as stdioout;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use crossterm::{
    terminal::{self, ClearType},
    execute,
    cursor,
};

use colored::Colorize;

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
        while let Some(message) = output_receiver.recv().await {
            clear_terminal().unwrap();
            let lines = message.lines();
            for line in lines {
                let (freq, msg) = line.split_once(",").unwrap();
                println!("{} {}", freq.bold().bright_red(), msg);
            }
        }
    });


    let mut lines = BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        input_sender.send(line).await.unwrap();
    }
}

fn clear_terminal() -> anyhow::Result<()> {
    let mut stdout = stdioout();
    execute!(stdout, terminal::Clear(ClearType::All))?;
    execute!(stdout, cursor::MoveTo(0, 0))?;
    print_header();
    Ok(())
}

fn print_header(){
    println ! ("{}","=== Log Miner ".bold().bright_cyan());
    println!("{}: {}", "=== Input".bold().bright_cyan(), "STDIN".bold().bright_red());
    println!("{}: {}", "=== Refresh interval".bold().bright_cyan(), "4s".bold().bright_red());
    println!("{}: {}", "=== Max distance (clustering factor)".bold().bright_cyan(), "0.7".bold().bright_red());
    println!("{}: {}", "=== Output lines per screen".bold().bright_cyan(), "5".bold().bright_red());
    println!("{}: {}", "=== Min frequency displayed".bold().bright_cyan(), "100".bold().bright_red());
    println!("");
}
