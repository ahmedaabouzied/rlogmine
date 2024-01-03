# rlogmine

A utility to cluster log messages into patterns and count of occurance of each pattern. It's using the LogMine algorithm mentioned in this [paper](https://www.cs.unm.edu/~mueen/Papers/LogMine.pdf).

## Usecase

Given a large log file generated by a service such as a web service, the logs messages usually follow a set of certain patterns of fixed strings and fields. This utility will cluster them into groups based on the patterns of log messages and show a count of occurances of each pattern.

Example:
```txt
2015-07-09 12:32:46,806 INFO action=insert user=tom id=20193 record=abf343rffd
2015-07-09 12:32:46,806 INFO action=upsert user=tom id=20193 record=abf343rffd
2015-07-09 12:32:46,806 ERROR action=insert id=2019 record=abf343rf error=invalid user
2015-07-09 12:32:46,806 INFO action=update user=tom id=126363 record=abd0kdwkldh
2015-07-09 12:32:46,806 INFO action=insert user=tom id=20193 record=abf343rffd
2015-07-09 12:32:46,806 ERROR action=update id=2019 record=abf343rf error=invalid user
2015-07-09 12:32:46,806 INFO action=update user=tom id=126363 record=abd0kdwkldh
2015-07-09 12:32:46,806 ERROR action=update id=2219 record=abf343rf error=invalid user
2015-07-09 12:32:46,806 INFO action=delete user=tom id=20193 record=abf343rffd
```

Output:
```txt
6  2015-07-09 12:32:46,806 INFO action=insert user=tom id=20193 record=abf343rffd
3  2015-07-09 12:32:46,806 ERROR action=insert id=2019 record=abf343rf error=invalid user
```

## Running
Due to always using a refresh interval for the output screen. It expects a continous stream of input.

### Example usage
```bash
tail -f ./example_log_files/example.log | ./target/debug/rlogmine -M 0.45 -l 1000 -m 1 -i 1
```

### Options
The help output shows all the availabale options and what they indicate.

```bash
A tool for mining logs. It clusters similar log lines together and displays the most frequent ones. It is useful for monitoring logs in real-time.

Usage: rlogmine [OPTIONS]

Options:
  -M, --max-distance <MAX_DISTANCE>
          The maximum distance between two log messages in the same cluster. It must be between 0.0 and 1.0

          [default: 0.7]

  -m, --min-frequency <MIN_FREQUENCY>
          The minimum frequency to be considered for printing to the screen in the output

          [default: 100]

  -l, --output-lines <OUTPUT_LINES>
          The maximim number of lines to be printed on each screen output refresh

          [default: 5]

  -i, --refresh-interval <REFRESH_INTERVAL>
          Screen output refresh interval in seconds

          [default: 4]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version


       Example:
            tail -f /var/log/syslog | logminer -M 0.7 -m 100 -l 5 -i 4

       Credits:
            The clustering is using the LogMine algorithm described in the paper: https://www.cs.unm.edu/~mueen/Papers/LogMine.pdf.
```


## Building

### Debug build
```
cargo build
```

### Release build
```
cargo build --Release
```
