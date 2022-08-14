# Almost a crypto

This project implements many of the features present in the operation of cryptocurrencies.
These features include:
    - the blockchain
    - the merkle tree
    - the coinbase user
    - numerous checks to validate blocks and transactions
These projects totally ignore the network part of crypto-currencies and the different types of nodes that can be part of it.


## Built With

To be able to build this project a classical Rust installation is enough. 

The libraries to use (visible in the Cargo.toml file) are:
    sha2 = "0.10.2
    ed25519-dalek = "1.0.1
    rand = "0.7.0
    num = "0.4.0"

The **sha2** library is used for the hash calculation of blocks. Indeed, the mining in this project is done on CPU for simplicity reasons.
The **ed25519-dalek** library is used to sign transactions.
The **rand** library is used for mining to avoid iterating on the proof of concept, a more solid rand may be more relevant, but, again, it is for the example. A stronger rand is used for the generation of public/private key pairs.
The **num** library allows transactions to be performed with infinite precision using rational numbers. Perhaps BigInt in Qxx (e.g. Q64) may be more relevant. 



