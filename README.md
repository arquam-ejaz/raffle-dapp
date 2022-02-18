# Onchain Raffle Dapp

## Concept

Onchain Raffle Dapp is a decentralized application built on the NEAR blockchain using Rust programming language.

The idea of this dapp is to allow organizations and individuals to organize raffles in a decentralized, trust-less, fraud-proof, and transparent manner by leveraging the power of blockchain and smart contracts.


## Features

1. Enables organizations and individuals to register their raffles with prize money greater than 2 NEAR.

2. Once registered, the raffle organizer's prize money is locked till the end of the raffle.

3. Onchain Raffle Dapp reserves a charge of 2 NEAR for storage and service fees from the prize money.
   
   So, Prize money = NEAR attached - 2 NEAR

4. Users can participate in the raffle, by locking at least 1 NEAR which reflects their confidence to win this raffle. The more NEAR they lock the more confident they are to win this raffle.

5. The participant's locked amount (confidence) will be refunded, once the raffle is finalized by the raffle organizer, irrespective of whether they win or lose.

6. The raffle organizer can only finalize the raffle after the raffle ends.

7. The raffle winner is decided randomly by leveraging the 'unbiased and unpredictable' random seed available at each block.

For knowing more features of this Dapp look at the smart contract file `./src/lib.rs`


## Prerequisites

Install [Rust](https://rustup.rs/) and [NEAR CLI](https://docs.near.org/docs/tools/near-cli#setup) globally before trying any of the below-mentioned steps.


## Getting started

To get started with Onchain Raffle Dapp:

1. Clone this repository
2. Test the contract 

    `cargo test -- --nocapture`

3. Build the contract
        
    `./build.sh`

4. Deploy the contract to the NEAR testnet

    `near dev-deploy --wasmFile ./res/raffle_dapp.wasm`

5. Create shell variable `CONTRACT_NAME` by copying and pasting the content of `./neardev/dev-account.env` in the shell

7. Initialize the contract

    `near call $CONTRACT_NAME new --accountId $CONTRACT_NAME`

8. Register a raffle

    `near call $CONTRACT_NAME register_raffle '{"start": <raffle start time in ms>, "end" : <raffle end time in ms>}' --accountId $CONTRACT_NAME --amount <prize money (including service fee) in NEAR>`

    This [website](https://currentmillis.com/) is useful to convert your local time to milliseconds (ms)

9. Participate in the raffle (repeat this step with different testnet accounts)

    `near call $CONTRACT_NAME participate '{"raffle_id":"$CONTRACT_NAME"}' --accountId <participant account id> --amount <participant confidence in NEAR>`

10. Finalize the raffle

    `near call $CONTRACT_NAME finalize_raffle '{"raffle_id":"$CONTRACT_NAME"}' --accountId $CONTRACT_NAME --gas=300000000000000`