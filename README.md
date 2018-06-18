# rusty-ports

This command line utility allows you to monitor a range of TCP ports on a computer.

This can help reverse-engineering when guessing which port a client will use when connecting to its server.

The program also outputs an escaped version of the messages sent by the clients to help reverse-engineering the protocol.

## Setup

```
cargo build
```

## Usage

```
./target/release/rusty-ports RANGE_START RANGE_END
```

**Example**:

Monitor ports 2000 to 2050 (inclusive).

```
./target/release/rusty-ports 2000 2050
```

Note that you can also monitor a unique port by repeating the same value (e.g. `rusty-ports 2000 2000`).

You need to be root to be able to monitor ports below 1025 (the program will throw an error otherwise).

## License

Mozilla Public License Version 2.0, Copyright (c) Louis Brunner
