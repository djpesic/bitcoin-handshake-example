# Bitcoin minimal handshake example

This software represents minimal example of p2p handshake with bitcoin nodes. Handshake protocol is desribed desribed at [bitcoin p2p specification](https://developer.bitcoin.org/devguide/p2p_network.html#p2p-network).
This example implements only mandatory handshaking:


1. Peer discovery
2. Connecting to peers.

## Peer discovery.

[Peer discovery](https://developer.bitcoin.org/devguide/p2p_network.html#peer-discovery) is process of finding Bitcoin peer list, starting from DNS seed. 

## Connecting to peers

After finding peer list, local client do the handshake: interchange of [Version](https://developer.bitcoin.org/reference/p2p_networking.html#version) and [Verack](https://developer.bitcoin.org/reference/p2p_networking.html#verack) messages. After successfull handshake, client is considered to be [connected to peer](https://developer.bitcoin.org/devguide/p2p_network.html#connecting-to-peers). No more work is done, altought, client can be extended in the future.

## Usage

Client can be configured to use Bitcoin mainnet or testnet. Configuration is placed inside toml file. Default configuration is *mainnet_config.toml*. Also repo contains *testnet_config.toml*. You can choose config file with command line arguments.

### Default usage

`cargo run`

### Choose config file

`cargo run -- -c testnet_config.toml`

### Full usage info

```
Usage: bitcoin-p2p-example [OPTIONS]

Options:
  -c, --config-file <CONFIG_FILE>  Config file path, relative to the project root [default: mainnet_config.toml]
  -h, --help                       Print help
  -V, --version                    Print version
```

Client tracks number of successfull and failed handshakes, and prints these numbers to output:
```
2023-03-14T11:07:13.116Z INFO  [bitcoin_p2p_example] Number of successfull handshakes: 25, number of failed handshakes 14
```


