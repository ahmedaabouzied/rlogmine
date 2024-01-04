use clap::Parser;
use colored::Colorize;
use crossterm::{
    cursor, execute,
    terminal::{self, ClearType},
};
use std::io::stdout as stdioout;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

mod cluster;

#[derive(Parser, Debug, Clone)]
#[clap(version = "0.1.0")]
#[clap(
    about = "A tool for mining logs. It clusters similar log lines together and displays the most frequent ones."
)]
#[clap(
    long_about = "A tool for mining logs. It clusters similar log lines together and displays the most frequent ones. It is useful for monitoring logs in real-time."
)]
#[clap(after_help = "
       Example:
            tail -f /var/log/syslog | logminer -M 0.7 -m 100 -l 5 -i 4

       Credits:
            The clustering is using the LogMine algorithm described in the paper: https://www.cs.unm.edu/~mueen/Papers/LogMine.pdf.
")]
struct Args {
    #[clap(
        short = 'M',
        long,
        default_value = "0.7",
        help = "The maximum distance between two log messages in the same cluster. It must be between 0.0 and 1.0"
    )]
    max_distance: f64,

    #[clap(
        short = 'm',
        long,
        default_value = "100",
        help = "The minimum frequency to be considered for printing to the screen in the output"
    )]
    min_frequency: u64,

    #[clap(
        short = 'l',
        long,
        default_value = "5",
        help = "The maximim number of lines to be printed on each screen output refresh"
    )]
    output_lines: u64,

    #[clap(
        short = 'i',
        long,
        default_value = "0",
        help = "Screen output refresh interval in seconds"
    )]
    refresh_interval: u64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut clusters =
        cluster::Clusters::new(args.max_distance, args.min_frequency, args.output_lines);

    // TODO: Asses the ideal capacity of the channels.
    let (input_sender, mut input_receiver) = mpsc::channel::<String>(10);
    let (output_sender, mut output_receiver) = mpsc::channel::<String>(10);
    let (screen_sender, mut screen_receiver) =
        tokio::sync::watch::channel::<String>("".to_string());
    let (done_sender, mut done_receiver) = mpsc::channel::<()>(1);

    // Spawn an async background task for the clusters.
    tokio::spawn(async move {
        clusters.start(&mut input_receiver, output_sender).await;
    });

    if args.refresh_interval == 0 {
        clear_terminal(&args).unwrap();
        let output = tokio::spawn(async move {
            let mut latest_output = String::new();
            while let Some(output) = output_receiver.recv().await {
                latest_output = output;
            }
            let lines = latest_output.lines();
            for line in lines {
                let (freq, msg) = line.split_once(",").unwrap();
                println!("{} {}", freq.bold().bright_red(), msg);
            }
        });
        // Read lines from STDIN.
        let mut lines = BufReader::new(io::stdin()).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            input_sender.send(line).await.unwrap();
        }
        drop(input_sender);
        output.await.unwrap();
        return;
    }

    // Spawn a background task to print output to the screen.
    let screen = tokio::spawn(async move {
        let args = args.clone();
        let mut interval = time::interval(Duration::from_secs(args.refresh_interval));
        loop {
            interval.tick().await;
            let message = screen_receiver.borrow_and_update();
            clear_terminal(&args).unwrap();
            let lines = message.lines();
            for line in lines {
                let (freq, msg) = line.split_once(",").unwrap();
                println!("{} {}", freq.bold().bright_red(), msg);
            }
            if done_receiver.try_recv().is_ok() {
                break;
            }
        }
    });

    let output_handler = tokio::spawn(async move {
        while let Some(message) = output_receiver.recv().await {
            screen_sender.send(message).unwrap();
        }
        done_sender.send(()).await.unwrap();
    });

    // Read lines from STDIN.
    let mut lines = BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        input_sender.send(line).await.unwrap();
    }
    drop(input_sender);
    output_handler.await.unwrap();
    screen.await.unwrap();
}

/// clear_terminal clears the terminal and print the current execution info as a header of
/// the new terminal cleared screen.
fn clear_terminal(args: &Args) -> anyhow::Result<()> {
    let mut stdout = stdioout();
    execute!(stdout, terminal::Clear(ClearType::All))?;
    execute!(stdout, cursor::MoveTo(0, 0))?;
    print_header(args);
    Ok(())
}

fn print_header(args: &Args) {
    let max_distance = format!("{:.2}", args.max_distance);
    let refresh_interval = format!("{}", args.refresh_interval);
    let min_frequency = format!("{}", args.min_frequency);
    let output_lines = format!("{}", args.output_lines);

    println!("{}", "=== Log Miner ".bold().bright_cyan());
    println!(
        "{}: {}",
        "=== Input".bold().bright_cyan(),
        "STDIN".bold().bright_red()
    );
    println!(
        "{}: {}",
        "=== Refresh interval".bold().bright_cyan(),
        refresh_interval.bold().bright_red()
    );
    println!(
        "{}: {}",
        "=== Max distance (clustering factor)".bold().bright_cyan(),
        max_distance.bold().bright_red()
    );
    println!(
        "{}: {}",
        "=== Output lines per screen".bold().bright_cyan(),
        output_lines.bold().bright_red()
    );
    println!(
        "{}: {} \n",
        "=== Min frequency displayed".bold().bright_cyan(),
        min_frequency.bold().bright_red()
    );
}
