# dusk moat

[![Repository](https://img.shields.io/badge/github-moat-blueviolet?logo=github)](https://github.com/dusk-network/moat)
![Build Status](https://github.com/dusk-network/moat/workflows/dusk_ci/badge.svg)
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

## moat-core
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

## license-provider
Provides functionality needed for implementors of license provider, including:
- license issuer
- blockchain scanner for relevant request
The crate allows for implementation of a license provider, whose task is to periodically check for license requests in the blockchain, and the to process the request and issue licenses.

## integration-tests
As most of the functionality provided by Moat deals with a blockchain, integration tests play critical role.
As in the case of moat-core functionality, tests include both Citadel-specific tests and blockchain generic test.

## wallet accessor
This is a low-level crate which provides wallet (Blockchain) connectivity for functions of moat-core.
Users of moat-core do not need to be aware of this crate, yet for maintainers and extenders, the crate
provides a convenient low level interface between the higher-level moat-core library and the blockchain.
Note that this crate deals only with contract method calling, it does not deal with contract queries.

## moat-core

### citadel requests

Class: RequestCreator
Methods: 
    create, 
    create_from_hex_args
Both methods allow for creation of a request, given user's secret spend key and license provider's public spend key.
The request can then be sent to license provider, off-chain or on-chain.

Class: RequestSender
Methods: send_request
Submits the request into blockchain.
It does so by calling a dummy contract method with request as an argument.

Class: RequestScanner
Methods:
    scan_transactions,
    scan_last_blocks, 
    scan_block_range
Scan requests in a given collection of transactions, 
contained in a given range of blocks or in a given number of most recent blocks.

### citadel queries

Class: CitadelInquirer
Methods: 
    get_licenses, 
    get_merkle_opening, 
    get_session, 
    get_info
Execute citadel-specific query methods of the license contract method. 

### blockchain payloads

Class: PayloadExtractor
Methods: payload_from_tx
Extracts a payload from the given transaction,
errors if payload of a given type is not present or the transaction is not a contract calling transaction.

Class: PayloadRetriever
Methods: retrieve_payload
Retrieves payload of a given transaction id, 
errors if transaction is not found, or it does not contain a payload
(for example, given transaction is not a contract calling transaction)

Class: PayloadSender
Methods: execute_contract_method
Executes given method of a given contract (identified by a contract id), passing to it the payload as an argument.

### contract queries

Class: ContractInquirer
Methods: 
    query_contract, 
    query_contract_with_feeder
query_contract - accepts a generic argument, contract id and contract query method name, returns a generic value result
query_contract_with_feeder - accepts a generic argument, contract id and method name, returns result as a Stream of bytes

### blockchain queries

Class: BcInquirer
Methods: 
    gql_query, 
    block_height
gql_query - executes a GQL query and returns result as a vector of bytes
block_height - returns the current block height as u64

Class: TxAwaiter
Methods: 
    wait_for, 
    wait_for_tx
Waits for a transaction identified by transaction id to be confirmed on the blockchain.

Class: TxInquirer
Methods: 
    txs_from_block, 
    txs_from_block_range,
    txs_from_last_n_blocks,
    retrieve_tx
Retrieve transaction identified by transaction id, or transactions contained in a given block, or a collection of blocks.
