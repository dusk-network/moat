# dusk moat

[![Repository](https://img.shields.io/badge/github-piecrust-blueviolet?logo=github)](https://github.com/dusk-network/moat)
![Build Status](https://github.com/dusk-network/moat/workflows/build/badge.svg)
[![Documentation](https://img.shields.io/badge/docs-moat-blue?logo=rust)](https://docs.rs/moat/)

`moat` is a Rust workspace containing the following crates:

- `moat-cli`: Command line interface (CLI) for submitting license requests to the Dusk blockchain.
- `moat-core`: Library providing an SDK required for implementing Citadel scenarios with the Dusk blockchain.
- `license-provider`: Reference implementation of a license provider.
- `integration-tests`: Moat library integration tests (test which require access to Dusk Wallet and live Dusk cluster).
- `wallet-accessor`: Library for submitting contract-call transactions to the Dusk blockchain.


## moat-cli

moat-cli utility can be used to submit license request to the Dusk blockchain.
Any license provider can then scan the blockchain for requests, filter out relevant requests and process them accordingly.

Example usage of the moat-cli utility:
```sh
cargo r --release --bin moat-cli -- --wallet-path ~/.dusk/rusk-wallet --config-path ./moat-cli/config.toml --password password ./moat-cli/request.json
```
where:
- `wallet-path`: path to your installed wallet directory
- `config-path`: path to your blockchain access configuration (refer to the configuration section)
- `password`: password for your wallet
- `...request.json`: path to your request json file (as explained in the request section)

Request sending can also be performed programmatically, using moat-core, described below:
```rust
//...
    let request_json: RequestJson = RequestJson::from_file(json_path)?;
    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        request_json.user_ssk,
        request_json.provider_psk,
        rng,
    )?;
//...
   PayloadSender::send_to_contract_method(
        request,
        &blockchain_access_config,
        &wallet_path,
        &psw,
        gas_limit,
        gas_price,
        LICENSE_CONTRACT_ID,
        NOOP_METHOD_NAME,
    )
    .await?;
//...
```
In the above example, the user is expected to have a request json file available
at some filesystem path. The code loads contents of the file, extracts request arguments from it and 
instantiates a request object. Subsequently, request is being sent to blockchain via
a provided wallet (utilizing wallet path and password, as well as a blockchain access configuration file).

As the `moat-cli` requires a path to a json request file. An example request json file looks as follows:
```json
{
  "user_ssk": "c6afd78c8b3902b474d4c0972b62888e4b880dccf8da68e86266fefa45ee7505926f06ab82ac200995f1239d518fdb74903f225f4460d8db62f2449f6d4dc402",
  "provider_psk": "29c4336ef24e585f4506e32e269c5363a71f7dcd74586b210c56e569ad2644e832c785f102dd3c985c705008ec188be819bac85b65c9f70decb9adcf4a72cc43"
}
```
It contains the user secret spend key (user_ssk) and provider public spend key (provider_psk), both in a form of a hexadecimal string.

##moat-core
Provides an SDK for writing user, LP, and SP applications based on the Dusk blockchain.
It provides the following functionality, related to license contract:
- creating requests
- sending license requests
- scanning blockchain for requests
- performing license contract queries: _get_licenses, get_merkle_opening, get_session, get_info_

In addition, it provides the following generic Dusk blockchain functionality, not necessarily related to license contract:
- sending payloads to any method of any contract (e.g., can be used for _issue_license_ and _use_license_)
- retrieving payloads of any type from the blockchain
- performing queries on any method of any contract (with return values passed directly or via a feeder/stream)
- retrieving transactions from blockchain (e.g., by block range)

In addition, websocket functionality for the queries is also provided.

##license-provider
Provides functionality needed for implementors of license provider, including:
- license issuer
- blockchain scanner for relevant request
The crate allows for implementation of a license provider, whose task is to periodically check for license requests in the blockchain, and the to process the request and issue licenses.

##integration-tests
As most of the functionality provided by Moat deals with a blockchain, integration tests play critical role. The crate contains the following groups of tests:
- license-provider related tests including scanning and license issuing
- License contract round-trip scenario test (this test contains LP, SP and user scenario)
- contract queries tests
- request retrieval tests
- request submitting tests
- generic payload sending tests
As in the case of moat-core, tests include both Citadel-specific tests and blockchain generic test.

##wallet accessor
This is a low-level crate which provides wallet (Blockchain) connectivity for functions of moat-core.
Users of moat-core do not need to be aware of this crate, yet for maintainers and extenders, the crate
provides a convenient low level interface between the higher-level moat-core library and the blockchain.
Note that this crate deals only with contract method calling, it does not deal with contract queries.
