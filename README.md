# Moat: The Citadel SDK

[![Repository](https://img.shields.io/badge/github-moat-blueviolet?logo=github)](https://github.com/dusk-network/moat)

**Moat** (a.k.a. the Citadel SDK) contains all the required tools for using and implementing self-sovereign identity systems using the Citadel protocol integrated into the Blockchain of Dusk.

## Prerequisites

**Moat** requires a reachable Rusk node installed and running, or selecting a trusted one. You can set up a node as explained [here](https://wiki.dusk.network/en/setting-up-node). It also requires an installed wallet connected to the given Rusk node, as explained [here](https://github.com/dusk-network/wallet-cli/blob/main/src/bin/README.md). Then you should specify the Rusk node address in `config.toml`.

## Testing the environment

You can test if the environment you set up and the library are working properly by specifying the testing Rusk node address in `integration-tests/tests/config/config.toml`, and executing the following (note that the wallet needs to use `password` as password for the tests to succeed):

```
cargo t --release --features="exp_tests" -- --test-threads=1
cargo t --release --features="int_tests" -- --test-threads=1
```

## Usage

The moat-cli utility can be used from the POV of any of the parties involved in the Citadel protocol, let them be:
- **User:** A party requesting licenses onchain to LPs, and being able to use the licenses onchain as well.
- **License Provider (LP):** A party receiving onchain requests from users to issue licenses onchain addressed to them.
- **Service Provider (SP):** A party receiving offchain requests from users to grant services.


### User

Users can request licenses and use them. To run the user CLI, simply run:

```sh
cargo r --release --bin moat-cli-user -- --wallet-pass <PASSWORD>
```

### License Provider

LPs can scan the Blockchain for requests and issue licenses if the requests are valid. To run the LP CLI, simply run:

```sh
cargo r --release --bin moat-cli-lp -- --wallet-pass <PASSWORD>
```

### Service Provider

SPs can get requests from users to grant their services, and accept or deny them by checking if the session cookies provided by the users are valid. To run the SP CLI, simply run:

```sh
cargo r --release --bin moat-cli-sp -- --wallet-pass <PASSWORD>
```
