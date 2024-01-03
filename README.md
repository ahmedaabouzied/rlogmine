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
## Theory
*This is 100% based on the [LogMine paper](https://www.cs.unm.edu/~mueen/Papers/LogMine.pdf).*

*If we define the distance function `Dist(P, Q)` between two log messages as:*

$$
Dist(P, Q) = 1 - \sum_{i=1}^{\min(\text{len}(P),\text{len}(Q))} \frac{\text{Score}(P_i,Q_i)}{\max(\text{len}(P),\text{len}(Q))}
$$

*where the `Score` function is given by:*

$$
\text{Score}(x,y) = 
\begin{cases} 
k_1 & \text{if } x=y \\
0 & \text{otherwise}
\end{cases}
$$

*`Pi` is the `i`th field of log `P`, and `len(P)` is the number of fields of log `P`. `k1` is a tunable parameter. We set `k_1 = 1` in our default log distance function, but this parameter can be changed to put more or less weight on the matched fields in two log messages.*

*The distance `D(P1,P2)` would be 0, only if `P1 == P2`, ie, the two log messages are duplicate. But, if two log messages `P1, P2` follow the same pattern with only fields are chaning as in the example logs in the previous section, then it follows that*


$$ 1 > Dist(P1, P2) > 0 $$

*Now, given an arbitray $r$ indicating the maximum distance between two log messages in the same cluster, then `D(P1, P2) < r` if `P1` and `P2` belong to the same cluster.* 

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
