# dusk moat

[![Repository](https://img.shields.io/badge/github-piecrust-blueviolet?logo=github)](https://github.com/dusk-network/moat)
![Build Status](https://github.com/dusk-network/moat/workflows/build/badge.svg)
[![Documentation](https://img.shields.io/badge/docs-moat-blue?logo=rust)](https://docs.rs/moat/)

`moat` is a Rust workspace containing the following crates: `moat-cli`, `moat-core` and `wallet-accessor`.

- `moat-cli`: Command line interface (CLI) for submitting license requests to the Dusk blockchain.
- `moat-core`: Library for submitting and scanning license requests in the Dusk blockchain.
- `wallet-accessor`: Library for submitting contract-call transactions to the Dusk blockchain. 

Example usage of the moat-cli utility:
```sh
cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password password ./moat-cli/request.json
```
where:
- `wallet-path`: path to your installed wallet directory
- `config-path`: path to your blockchain access configuration (refer to the configuration section)
- `password`: password for your wallet
- `...request.json`: path to your request json file (refer to the request section)

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

##Configuration
`moat-cli` requires a configuration file with the urls which allow for blockchain access.
An example configuration file looks as follows:
```toml
rusk_address = "https://devnet.nodes.dusk.network:8585"
prover_address = "https://devnet.provers.dusk.network:8686"
graphql_address = "http://devnet.nodes.dusk.network:9500/graphql"
```
The file is also used by the `moat-core` and `wallet-accessor` libraries.

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
