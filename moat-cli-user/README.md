# Moat CLI User

Command line interface to Dusk Citadel User

Available commands:

- submit a license request to blockchain (USER)
- list requests present on blockchain (USER)
- list user's licenses (USER)
- compute proof and use license (USER)
- obtain service from SP (USER & SP)

## Submit a license request to blockchain (User)

arguments:
- data for wallet connection
- data for Rusk cluster connection
- user SSK (Secret Spend Key)
- provider PSK (Public Spend Key)
- gas limit
- gas price

## Retrieve from blockchain the requests which were sent by the user (User)

arguments:
- scope: either an entire blockchain, or block range, or N last blocks
- data for Rusk cluster connection
- user's view key (created from user's SSK)

## List user's licenses (User)

arguments:
- scope: block height range
- data for Rusk cluster connection
- user's view key (created from user's SSK)

## Use license (User)

arguments:
- data for wallet connection
- data for Rusk cluster connection
- license
- more - TBD

## Obtains service from SP (User and SP)

arguments:
TBD as we need to mock SP

