# Onchain Raffle Dapp

## Concept

Onchain Raffle Dapp is a decentralized application built on the [NEAR blockchain](https://near.org/) using [Rust](https://www.rust-lang.org/).

The idea of this [decentralized application](https://en.wikipedia.org/wiki/Decentralized_application) (dapp) is to allow organizations and individuals to organize raffles in a decentralized, trust-less, fraud-proof and transparent manner by leveraging the power of blockchain and smart contracts.

###### What is 'Raffle' in the context of this Dapp?

A Raffle is a contest with prize money where the winner is decided randomly purely based on luck.
The difference between a raffle and a lottery is that to participate in a lottery one has to spend some amount which is non-refundable, while to participate in a raffle one has to 'lock' some amount with the smart contract till the raffle is finalized i.e till the winner is chosen randomly. After the raffle is finalized, the locked amount gets refunded to the participants irrespective of whether one win or lose.


## Features

1. Enables organizations/individuals to register raffles with prize money greater than 2 NEAR tokens (NEAR blockchain's native tokens) by specifying the raffle start and end dates.

2. Once the raffle is registered, the prize money is locked till the end of the raffle to ensure that the raffle is carried out fairly.

3. The smart contract enables Onchain Raffle Dapp to reserve a charge of 2 NEAR tokens for storage and service fees from the attached deposit paid by the raffle organizer.
   
   `Prize money = attached NEAR tokens - 2 NEAR tokens`

4. Users can participate in the raffle by locking at least 1 NEAR token to prevent spam or duplicate entries to some extent. The participant's locked NEAR tokens plays no role while deciding the winner to conduct an unbiased raffle.

5. The participant's locked NEAR tokens will be refunded once the raffle is finalized by the raffle organizer irrespective of whether one win or lose.

6. The raffle organizer can only finalize the raffle after the raffle ends.

7. The raffle winner is decided randomly by leveraging the 'unbiased and unpredictable' random seed available at each block.

To know more features of this Dapp look at the smart contract file [`./src/lib.rs`](https://github.com/arquam-ejaz/raffle-dapp/blob/master/src/lib.rs)


## Prerequisites

Install [Rust](https://rustup.rs/) and [NEAR CLI](https://docs.near.org/docs/tools/near-cli#setup) globally before trying any of the below-mentioned steps.


## Getting started

To get started with Onchain Raffle Dapp:

1. Clone this repository

    `git clone https://github.com/arquam-ejaz/raffle-dapp.git`

2. Test the contract 

    `cargo test -- --nocapture`

3. Build the contract
        
    `./build.sh`

4. Deploy the contract to the NEAR testnet

    `near dev-deploy --wasmFile ./res/raffle_dapp.wasm`

5. Create shell variable `CONTRACT_NAME` by simply copying and pasting the content of `./neardev/dev-account.env` in the shell

7. Initialize the contract

    `near call $CONTRACT_NAME new --accountId $CONTRACT_NAME`

8. Register a raffle

    `near call $CONTRACT_NAME register_raffle '{"start": <raffle start time in ms>, "end" : <raffle end time in ms>}' --accountId $CONTRACT_NAME --amount <prize money (including service fee) in NEAR>`

    Note: This [website](https://currentmillis.com/) is useful to convert your local time to milliseconds (ms)

9. Participate in the raffle (repeat this step with different testnet accounts)

    `near call $CONTRACT_NAME participate '{"raffle_id":"$CONTRACT_NAME"}' --accountId <participant account id> --amount <participant confidence in NEAR>`

10. Finalize the raffle

    `near call $CONTRACT_NAME finalize_raffle '{"raffle_id":"$CONTRACT_NAME"}' --accountId $CONTRACT_NAME --gas=300000000000000`