# Smart Contract Indexer

This is a Rust application which indexes Ethereum the [nft-marketplace](https://github.com/DanielDimitrov5/NFT-Marketplace-Project/tree/main) smart contract. It insert data in a DB and it starts listening for smart contract events and updating the data.

## Getting Started

### Prerequisites

You will need to have Rust and MongoDB installed on your machine. For Rust, you can refer to the [official guide](https://www.rust-lang.org/tools/install). For MongoDB, you can refer to the [official guide](https://docs.mongodb.com/manual/installation/).

### Installation

1. Clone the repository.
`git clone <https://github.com/DanielDimitrov5/NFT-Marketplace-Indexer-Rust.git>`

2. Navigate into the cloned repository. `cd <repository_directory>`

3. Compile the project with `cargo build`


### Configuration

The indexer needs to connect to a MongoDB instance and an Ethereum network.

1. MongoDB:
This can be configured by modifying the connection string passed to `ClientOptions::parse` in the `main` function. By default, it is set to `mongodb://localhost:27017`.

2. Ethereum:
This is configured through the `RPC_URL` and `CONTRACT_ADDRESS` variables. `RPC_URL` should contain the URL of the Ethereum RPC endpoint and `CONTRACT_ADDRESS` should be the address of the smart contract. RPC_URL can be set in a `.env` file in the root directory of the project. If a `.env` file is not present, please create one and add the following lines: `RPC_URL=<your_rpc_url>`

### Usage

To run the indexer, execute `cargo run`


This command will start the application, which will bulk data into MongoDB and start listening for events from now on.

## Code Overview

The main modules in the application are `services`, `models`, and the `main` script.

`services` contains the following submodules:
- `data_loader`: handles fetching data from the smart contract.
- `logging`: handles logging of events.
- `listener`: handles listening to new events from the smart contract.

`models` contains the data models for the smart contract entities like `NFTCollection`, `Item` and `Offer`.

In the main script, `main` function is defined which does the following:
- Setups the environment variables
- Establishes a connection to the MongoDB database
- Initializes the logging
- Drops existing MongoDB collections
- Connects to the Ethereum network using the provided RPC endpoint
- Initializes the contract
- Fetches all collections, items, and offers from the smart contract
- Bulks the fetched data into MongoDB
- Starts listening for events and updating the data accordingly

## Contact

If you have any questions or comments, please feel free to contact me.

## Acknowledgements

This project is powered by several open-source projects, notably:
- [rust](https://www.rust-lang.org/)
- [mongodb](https://www.mongodb.com/)
- [ethers-rs](https://github.com/gakonst/ethers-rs)

## License
Distributed under the [MIT License](./LICENSE). See `LICENSE` for more information.