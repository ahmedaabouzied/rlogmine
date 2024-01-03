use colored::Colorize;
use std::fmt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{self, Duration};

/// Default maximum distance between two messages in a cluster.
const DEFAULT_MAX_DIST: f64 = 0.7;
const DISPLAY_MIN_COUNT: u64 = 100;
const DISPLAY_REFRESH_INTERVAL: u64 = 4;
const SCREEN_LINES: u64 = 8;

#[derive(Clone, Debug)]
struct Cluster {
    /// Cluster representative parts message.
    r: Vec<String>,

    /// The maximum distance between two messages in the cluster.
    max_dist: f64,

    /// Count of messages in the cluster.
    count: u64,
}

impl Cluster {
    /// Creates a new cluster with the given parts message.
    fn new(r: &String) -> Cluster {
        let r = r.split(" ").map(|s| s.to_string()).collect::<Vec<String>>();
        Cluster {
            r: r,
            max_dist: DEFAULT_MAX_DIST,
            count: 1,
        }
    }

    /// Returns the distance between two parts messages.
    /// The distace between two parts messages is defined as:
    ///
    /// Dist(P, Q) = 1 - SUM[i=1..Min(len(P), len(Q))] Score(P[i], Q[i]) / Max(len(P), len(Q)))])
    ///
    fn distance(&self, p1: &Vec<String>, p2: &Vec<String>) -> f64 {
        let mut total = 0.0;
        let max = p1.len().max(p2.len()) as f64;
        for (p1i, p2i) in p1.iter().zip(p2.iter()) {
            total += self.score((p1i, p2i)) / max as f64;
            if (1.0 - total) < self.max_dist {
                return 1.0 - total;
            }
            if total > self.max_dist {
                return 1.0;
            }
        }
        1.0 - total
    }

    /// Returns the score of two strings
    /// The score goes by the formula:
    /// Score(x,y) = 1 if x == y, 0 otherwise
    fn score(&self, (x, y): (&String, &String)) -> f64 {
        if x == y {
            1.0
        } else {
            0.0
        }
    }

    /// Checks if the given message is within a suitable distance from the current cluster.
    /// If the message is within the distance, the cluster is updated and true is returned.
    /// Otherwise, false is returned.
    fn process(&mut self, msg: &String) -> bool {
        let msg = msg
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let dist = self.distance(&self.r, &msg);
        if dist < self.max_dist {
            self.count += 1;
            return true;
        } else {
            return false;
        }
    }
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.count, self.r.join(" "))
    }
}

/// A list of clusters.
#[derive(Clone, Debug)]
pub struct Clusters {
    list: Vec<Cluster>,
}

impl fmt::Display for Clusters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for cluster in self.list.iter().take(SCREEN_LINES as usize) {
            if cluster.count < DISPLAY_MIN_COUNT{
                continue;
            }
            write!(f, "{}\n", cluster)?;
        }
        Ok(())
    }
}

impl Clusters {
    /// Creates a new empty clusters list.
    pub fn new() -> Clusters {
        Clusters { list: Vec::new() }
    }
    pub async fn start(
        &mut self,
        input_receiver: &mut Receiver<String>,
        output_sender: Sender<String>,
    ) {
        println!("Starting clusterer");

        let output_sender = output_sender.clone();
        let (watch_sender, watch_receiver) = tokio::sync::watch::channel("".to_string());

        tokio::spawn(async move {
            println!("Starting display thread");
            let mut interval = time::interval(Duration::from_secs(DISPLAY_REFRESH_INTERVAL));
            loop {
                interval.tick().await;
                let update = watch_receiver.borrow().clone();
                output_sender.send(update).await.unwrap();
            }
        });
        while let Some(message) = input_receiver.recv().await {
            self.process(message);        
            self.sort();
            watch_sender.send(self.to_string()).unwrap();
        }
    }

    /// Processes the given message. If the message is within a suitable distance from
    /// any of the clusters, the cluster is updated. Otherwise, a new cluster is created.
    pub fn process(&mut self, msg: String) {
        for cluster in self.list.iter_mut() {
            if cluster.process(&msg) {
                return; // We found a suitable cluster.
            }
        }
        // If we get here, we didn't find a suitable cluster.
        // Create a new one.
        self.list.push(Cluster::new(&msg));
    }

    /// Sorts the clusters by the number of messages in each cluster.
    fn sort(&mut self) {
        self.list.sort_by(|a, b| b.count.cmp(&a.count));
    }

    /// Prints the clusters output
    pub fn print(&mut self) {
        self.sort();
        for cluster in self.list.iter() {
            println!(
                "{:5} {}",
                format!("{}", cluster.count).green(),
                cluster.r.join(" ")
            );
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_score() {
        let c = Cluster::new(&"".to_string());
        let x = "a".to_string();
        let y = "a".to_string();
        let z = "b".to_string();
        assert_eq!(c.score((&x, &y)), 1.0);
        assert_eq!(c.score((&x, &z)), 0.0);
    }

    #[test]
    fn test_distance() {
        let parts1 =
            "2015-07-09 12:32:46,806 INFO action=insert user=tom id=201923 record=abf343rf"
                .to_string()
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
        let parts2 =
            "2015-07-09 12:32:46,806 INFO action=insert user=tom id=201923 record=abf343df"
                .to_string()
                .split(" ")
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
        let parts3 = "2015-07-09 12:32:46,806 ERROR action=update error=invalid user"
            .to_string()
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let parts4 = "2015-07-09 12:32:46,806 ERROR action=update error=invalid DB connection"
            .to_string()
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let c = Cluster::new(&parts1.join(" "));

        assert!(c.distance(&parts1, &parts2) < c.distance(&parts1, &parts3));
        assert!(c.distance(&parts3, &parts4) < c.distance(&parts3, &parts1));
    }

    #[test]
    fn test_process() {
        let mut logs1 = Vec::new();
        for i in 0..100 {
            logs1.push(format!(
                "2015-07-09 12:32:46,806 INFO action=insert user=tom{} id=2019{} record=abf343rf{}",
                i, i, i
            ));
        }
        let mut logs2 = Vec::new();
        for i in 0..43 {
            logs2.push(format!(
                "2015-07-09 12:32:46,806 WARN action=update user=tom{} id=2019{} record=abf343df{}",
                i, i, i
            ));
        }

        let mut logs3 = Vec::new();
        for i in 0..80 {
            logs3.push(format!(
                "2015-07-09 12:32:46,806 ERROR action=insert user=tom{} id=2019{} record=abf343rf{} error=invalid user",
                i, i, i
            ));
        }

        let mut c = Clusters::new();
        for log in [&logs1[..], &logs2[..], &logs3[..]].concat() {
            c.process(log);
        }
        c.sort();
        println!("{}", c);
    }
}
