# Moat CLI LP

Command line interface to Dusk Citadel License Provider

Available commands:

- list relevant license requests
- issue license for a given request
- list licenses

## Retrieve relevant license requests (LP)

arguments:
- scope: either an entire blockchain, or block range, or N last blocks
- data for Rusk cluster connection
- LP's view key (created from LP's SSK)

## Issue license for a given request (LP)

arguments:
- data for wallet connection
- data for Rusk cluster connection
- gas limit
- gas price
- license (created from the relevant request and LP's SSK)

## List licenses

arguments:
- scope: block height range
- data for Rusk cluster connection
