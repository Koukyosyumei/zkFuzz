# Telepathy Subgraphs
This directory contains the subgraphs for telepathy.

# Installation instructions

    yarn install
    yarn run codegen
    yarn run build

# Subgraph info
|Network|Subgraph name|Graph network ID|GraphQL query endpoint
|--|--|--|--|
|Polygon|succinctlabs/telepathy-polygon|matic|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-polygon
|Optimism|succinctlabs/telepathy-optimism|optimism|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-optimism
|Gnosis|succinctlabs/telepathy-gnosis|gnosis|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-gnosis
|Arbitrum|succinctlabs/telepathy-arbitrum|arbitrum-one|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-arbitrum
|Binance Smart Chain|succinctlabs/telepathy-bsc|bsc|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-bsc
|Avalanche|succinctlabs/telepathy-avalanche|avalanche|https://api.thegraph.com/subgraphs/name/succinctlabs/telepathy-avalanche

# Deployment command
Run the following command to deploy a subgraph

    graph deploy --product hosted-service --deploy-key <deploy key> --network <Graph network ID> --network-file ./networks.json <subgraph name>
