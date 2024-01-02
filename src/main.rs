use std::io::BufRead;

mod cluster;

fn main() {
    let mut clusters = cluster::Clusters::new();

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        clusters.process(line);
    }   
    clusters.print();
}
