# knock-rs: A port-knocking implementation in Rust

<img src="https://raw.githubusercontent.com/darktohka/knock-rs/master/images/knock.png" width="600">

## What is port-knocking?

Port-knocking is a method of externally opening ports on a firewall by generating a connection attempt on a set of prespecified closed ports. Once a correct sequence of connection attempts is received, the firewall rules are dynamically modified to allow the host which sent the connection attempts to connect over specific port(s).

`knock-rs` **only detects the SYN packets and doesn't listen to the opened ports**, it uses the [pnet](https://docs.rs/pnet/latest/pnet/) crate to capture the raw packets.

A common use of this technique is to secure connections to an SSH server by only allowing access to the SSH port after a successful port-knocking sequence has been executed.

This project is inspired by another [knock](https://github.com/jvinet/knock) project which is written in C, but it is written in Rust and has a different configuration format.

## Why use port-knocking?

Port-knocking is a simple and effective way to secure your server from unauthorized access. It is a lightweight and secure method to protect your server from unauthorized access.

## Common Use Cases

- Secure your SSH server from brute-force attacks
- Open and close any ports on your firewall dynamically based on your needs

## Download

You can download the pre-built binaries from the [releases](https://github.com/darktohka/knock-rs/releases) page.

## Build

```bash
cargo build --release
```

## Configuration

### Server Configuration

Create a configuration file named `config.json` in the same directory as the `knockd` binary.

```json
{
  "interface": "eth0",
  "timeout": 5,
  "rules": [
    {
      "name": "enable_ssh",
      "command": "/usr/sbin/iptables -I INPUT -s %IP% -p tcp --dport 22 -j ACCEPT",
      "setup": "iptables -A INPUT -p tcp --dport 22 -j DROP",
      "teardown": "iptables -D INPUT -p tcp --dport 22 -j DROP",
      "sequence": [15523, 17767, 32768, 28977, 51234]
    },
    {
      "name": "disable_ssh",
      "command": "/usr/sbin/iptables -D INPUT -s %IP% -p tcp --dport 22 -j ACCEPT",
      "sequence": [51234, 28977, 32768, 17767, 15523]
    }
  ]
}
```

- `interface`: The network interface to listen on
- `timeout`: The timeout in seconds to wait for the client to send the complete sequence
- `rules`: The rules to apply when the correct sequence is received
  - `name`: The name of the rule
  - `command`: The command to execute when the correct sequence is received. `%IP%` will be replaced with the client's IP address
  - `setup`: The command to execute when starting the `knockd` daemon. This can be used to block the port by default
  - `teardown`: The command to execute when stopping the `knockd` daemon. This can be used to allow ports to be used without knocking again
  - `sequence`: The sequence of ports that the client should knock

### Client Configuration

Create a configuration file named `config.json` in the same directory as the `knock-cli` binary.

**Do make sure that the client has the same sequence as the server.**

```json
{
  "rules": [
    {
      "name": "enable_ssh",
      "sequence": [12345, 54321, 32768, 18933]
    },
    {
      "name": "disable_ssh",
      "sequence": [18933, 32768, 54321, 12345]
    }
  ]
}
```

- `rules`: The rules to apply when the correct sequence is sent
  - `name`: The name of the rule, the name doesn't need to match the server's rule name, but the sequence does. And also, the name should be unique in the client's configuration file
  - `sequence`: The sequence of ports to knock

## Usage

### Server

```bash
./knockd -c config.json
```

The default config path is `config.json`, you can also specify the config file path by using the `-c` option.

### Client

```bash
./knock-cli -c config.json -r enable_ssh -h example.com
```

The default config path is `config.json`, you can also specify the config file path by using the `-c` option.

The `-r` option is used to specify the rule name to knock.

The `-h` option is used to specify the host to knock on.

## Run Server as docker container

```bash
docker run --network host --cap-add=NET_RAW --cap-add=NET_BIND_SERVICE --cap-add=NET_ADMIN -d --restart=always --name=knockd -v ./config.json:/config.json:ro ghcr.io/darktohka/knockd:latest
```

Since the server needs to listen to the raw packets, you need to add the `NET_RAW`, `NET_BIND_SERVICE` and `NET_ADMIN` capabilities to the container.

## Examples

Assume that you have already added one firewall rule to block all incoming connections to the SSH port. E.g.:

```bash
iptables -A INPUT -p tcp --dport 22 -j DROP
```

Use the following command to enable the SSH port on the server:

```bash
./knock-cli -r enable_ssh -h example.com
```

After the correct sequence is sent, the SSH port will be opened for the client's IP address. Now you can connect to the SSH server.

To close the SSH port, use the following command:

```bash
./knock-cli -r disable_ssh -h example.com
```
