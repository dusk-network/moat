# dusk moat

[![Repository](https://img.shields.io/badge/github-piecrust-blueviolet?logo=github)](https://github.com/dusk-network/moat)
![Build Status](https://github.com/dusk-network/moat/workflows/build/badge.svg)
[![Documentation](https://img.shields.io/badge/docs-moat-blue?logo=rust)](https://docs.rs/moat/)

`moat` is a Rust workspace containing the following crates: `moat-cli`, `moat-core`, `wallet-accessor`, `integration-tests` and `license-provider`.

- `moat-cli`: Command line interface (CLI) for submitting license requests to the Dusk blockchain.
- `moat-core`: Library for submitting and scanning license requests in the Dusk blockchain.
- `wallet-accessor`: Library for submitting contract-call transactions to the Dusk blockchain.
- `integration-tests`: Moat library integration tests (test which require access to Dusk Wallet and a live Dusk cluser).
- `license-prvider`: A reference implementation of a license provider.

Example usage of the moat-cli utility:
```sh
cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password password ./moat-cli/request.json
```
where:
- `wallet-path`: path to your installed wallet directory
- `config-path`: path to your blockchain access configuration (refer to the configuration section)
- `password`: password for your wallet
- `...request.json`: path to your request json file (as explained in the request section)

Example usage of the moat-core library:
```rust
use moat_core::{JsonLoader, RequestCreator, RequestJson, RequestSender};
use wallet_accessor::BlockchainAccessConfig;
//...
    let request_json: RequestJson = RequestJson::from_file(json_path)?;
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;

    let wallet_path = WalletPath::from(wallet_path.join("wallet.dat"));
    let blockchain_access_config =
        BlockchainAccessConfig::load_path(config_path)?;

    RequestSender::send(
        request,
        &blockchain_access_config,
        wallet_path,
        password,
        gas_limit,
        gas_price,
    )
    .await?;
//...
```
In the above example, the user is expected to have a request json file prepared and available
at some filesystem path. The code loads contents of the file, extracts request arguments from it and 
instantiates a request object. Subsequently, request is being sent to blockchain via
a provided wallet (utilizing wallet path and password, as well as a blockchain access configuration file).

Example usage of the wallet-accessor library:
```rust
use wallet_accessor::{BlockchainAccessConfig, WalletAccessor};
//...
        let wallet_accessor = WalletAccessor {
            path: wallet_path,
            pwd: password,
        };
        let tx_id = wallet_accessor
            .send(
                request,
                YOUR_CONTRACT_ID,
                YOUR_METHOD_NAME.to_string(),
                cfg,
                gas_limit,
                gas_price,
            )
            .await?;
//...
```
In the above example, the code issues a contract call with request as an argument. The contract is identified by a given contract id,
while contract's method is identified by method's name. Note that request sent as argument of the contract
call is generic, so it can be any type declared by the user of the library, as long as it is rkyv-serializable.

##Request
`moat-cli` requires a path to a json request file. An example request json file looks as follows:
```json
{
  "user_ssk": "c6afd78c8b3902b474d4c0972b62888e4b880dccf8da68e86266fefa45ee7505926f06ab82ac200995f1239d518fdb74903f225f4460d8db62f2449f6d4dc402",
  "provider_psk": "29c4336ef24e585f4506e32e269c5363a71f7dcd74586b210c56e569ad2644e832c785f102dd3c985c705008ec188be819bac85b65c9f70decb9adcf4a72cc43"
}
```
It contains the user secret spend key (user_ssk) and provider public spend key (provider_psk), both in a form of a hexadecimal string.

##Scanning for Requests
The following code illustrates how to scan for all requests in the blockchain:
```rust
    let mut height = 0;
    loop {
        let height_end = height + 10000;
        let (requests, top) =
            RequestScanner::scan_block_range(height, height_end, &cfg).await?;

        println!(
            "{} requests in range ({},{}) top={}",
            requests.len(),
            height,
            height_end,
            top
        );

        if top <= height_end {
            break;
        }

        height = height_end;
    }
```
In the above example, calls scanning the blockchain are executed in a loop, in effect, scanning the entire blockchain. The user may also want 
to arrange the calls differently. User may also scan only the last n blocks - an appropriate API for that is also available.

##Configuration
`moat-cli` requires a configuration file with the urls which allow for blockchain access.
An example configuration file looks as follows:
```toml
rusk_address = "https://devnet.nodes.dusk.network:8585"
prover_address = "https://devnet.provers.dusk.network:8686"
graphql_address = "http://devnet.nodes.dusk.network:9500/graphql"
```
The file is also used by the `moat-core` library.

##Testing
To build and run unit tests:
```sh
cargo t
```

To build and run unit tests and integration tests:
```sh
cargo t --features integration_tests
```

To build and run unit tests and expensive tests (like scanning the entire Dusk blockchain for requests):
```sh
cargo t --features expensive_tests
```
