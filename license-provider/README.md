# dusk moat - license provider - reference implementation

`license-provider` is a reference implementation of a hipothetical license provider, the following points summarize the 
functionality provided by the license provider:

- initializes itself with a given identity (as defined by Citadel, a view key to the owned public key) 
- scans the entire blockchain for requests addressed to "self" (is able to filter requests)
- alternatively, scans the last N blocks for requests addressed to "self"
- holds a collection of requests to process along with a set of requests hashes to avoid duplication caused by multiple scanning of the same blocks

Note that from the cryptographic point of view, the license provider implementation is secure, in a sense, that the
only key it holds and uses is a view key, which contains data from one half of the public key. The implementation
does not hold the license provider secret key, and does not hold the public key.

## how to test

to run a local unit tests with a hardcoded set of transactions do:

```sh
cd license-provider
cargo t --release
```

to run an integration test against a life blockchain do:

```sh
cd integration-tests
cargo t lp --release --features exp_tests
```
Make sure blockchain access data in `config.toml` in integration-tests/tests/config is up to date.
