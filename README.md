# What Is Rustcurrent?
Rust current is my attempt at completing the [fly.io](https://fly.io/dist-sys/) distributed systems challenges.

# Testing

## Installing Maelstrom

Maelstrom is built in Clojure so you’ll need to install OpenJDK. It also provides some plotting and graphing utilities which rely on Graphviz & gnuplot. If you’re using Homebrew, you can install these with this command:

```sh
brew install openjdk graphviz gnuplot
```

Next, you’ll need to download Maelstrom itself. These challenges have been tested against the [Maelstrom 0.2.3](https://github.com/jepsen-io/maelstrom/releases/tag/v0.2.3). Download the tarball & unpack it. You can run the maelstrom binary from inside this directory.

## Running nodes in Maelstrom
We can now start up Maelstrom and pass it the full path to our binary:

```sh
path/to/maelstrom test -w echo --bin path/to/target/debug/echo --node-count 1 --time-limit 10
```

# TODO
Consensus algorithms in rust. Also Byzantine Fault Tolerant State Machine Replication

[3:01:30](https://www.youtube.com/watch?v=gboGyccRVXI&t=10890s) instead of random you can use a module counter so you send n, n+1 and n+2 additional alreadyknowns. Or you add the first x of all already now since in the next message they won’t be contained

implementing traditional leader based (Paxos or Raft) or even new leaderless alternatives (Epaxos, Atlas, Tempo, Accord) consensus algorithms, query routing, distributed hash tables, distributed storage, etc.

An idea for sync across nodes - form a probabilistic data structure(like a bloom filter) from your node’s values and send it to neighbours, so they can test for known values, to figure out the data that we might be interested in

Sure, there may be false positives(we don’t know about the value, but the bloom filter says we do), but I feel like they can be dealt with.

Or use a lossy hash table for inverse behaviour(only false negatives)
