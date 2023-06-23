# Dusk Moat

Command line interface (CLI) and a library for submitting license requests to the Dusk network.

License requests are being sent as arguments to a noop method of the license contract.
The noop method ignores the arguments, yet transactions along with arguments are stored in blockchain.
They can later be retrieved using the explorer API.

To run unit tests:
`cargo t`

To run all tests including integration tests:
`cargo t --features integration_tests`

