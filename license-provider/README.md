# dusk moat - license provider - reference implementation

`license-provider` is a reference implementation of a hipothetical license provider, the following points summarize the 
functionality provided by the license provider:

- initializes itself with a given identity (as defined by Citadel, a public spend key and a secret spend key) 
- scans the entire blockchain for requests addressed to "self" (is able to filter requests)
- alternatively, scans the last N blocks for requests addressed to "self"
- holds a collection of requests to process along with a set of requests hashes to avoid duplication caused by multiple scanning of the same blocks

## how to test

to run a local unit tests with a hardcoded set of transactions do:

```sh
cd license-provider
cargo t --release
```

to run integration test against a life blockchain do:

```sh
cd integration-tests
cargo t lp --release --features exp_tests
```
Make sure blockchain access data in `config.toml` in integration-tests/tests/config is up-to-date.
