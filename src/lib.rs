use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Timestamp};

// constant representing 1 NEAR in yoctoNear
const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

// constant to convert milliseconds to nanoseconds and vice versa
const TO_NANOSECONDS: u64 = 1_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RaffleDetails {
    prize: Balance,
    start: Timestamp,
    end: Timestamp,
    participants: UnorderedMap<AccountId, Balance>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct RaffleDapp {
    raffles: UnorderedMap<AccountId, RaffleDetails>,
}

impl Default for RaffleDapp {
    fn default() -> Self {
        panic!("The smart contract should be initialized before usage")
    }
}

#[near_bindgen]
impl RaffleDapp {
    #[init]
    pub fn new() -> Self {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            //"raffle-dapp.testnet".to_string(),
            //"Only raffle-dapp.testnet can initialize the smart contract"
            "Only the current account can initialize the smart contract"
        );
        Self {
            raffles: UnorderedMap::new(b"r"),
        }
    }

    #[payable]
    pub fn register_raffle(&mut self, start: Timestamp, end: Timestamp) {
        // Check if the attached deposit is greater than 2 N to cover gas and service fees
        // Thus, Prize = Attached depost (in NEAR) - 2 NEAR
        assert!(
            env::attached_deposit() > 2 * ONE_NEAR,
            "Prize money should be greater than 2 N"
        );

        // Allowing to register only one raffle per account to avoid spamming, they can register a new raffle after their current raffle ends
        assert!(
            self.raffles.get(&env::predecessor_account_id()).is_none(),
            "You have already registered a raffle"
        );

        assert!(
            end > start,
            "The end date should be greater than start date"
        );

        let raffle_details = RaffleDetails {
            prize: env::attached_deposit() - 2 * ONE_NEAR,
            start: start * TO_NANOSECONDS,
            end: end * TO_NANOSECONDS,
            participants: UnorderedMap::new(b"p"),
        };

        self.raffles
            .insert(&env::predecessor_account_id(), &raffle_details);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package raffle-dapp -- --nocapture
 * Note: 'raffle-dapp' comes from Cargo.toml's 'name' key
 */

#[cfg(test)]
#[allow(dead_code)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, Balance};

    // Setting up a mock context with current account as 'raffle-dapp.testnet'
    fn get_context() -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(raffle_dapp_account_id());
        builder
    }

    fn raffle_dapp_account_id() -> AccountId {
        AccountId::new_unchecked("raffle-dapp.testnet".to_string())
    }

    fn alice_account_id() -> AccountId {
        AccountId::new_unchecked("alice.testnet".to_string())
    }

    fn bob_account_id() -> AccountId {
        AccountId::new_unchecked("bob.testnet".to_string())
    }

    #[test]
    #[should_panic(expected = "The smart contract should be initialized before usage")]
    fn check_default() {
        RaffleDapp::default().register_raffle(1644353705121, 1644353705130);
    }

    #[test]
    #[should_panic(expected = "Only the current account can initialize the smart contract")]
    fn check_initialization() {
        let mut context = get_context();
        context.predecessor_account_id(alice_account_id());
        testing_env!(context.build());
        RaffleDapp::new();
    }

    #[test]
    #[should_panic(expected = "Prize money should be greater than 2 N")]
    fn check_register_raffles_less_than_two_near() {
        let mut context = get_context();
        context.predecessor_account_id(raffle_dapp_account_id());
        testing_env!(context.build());
        let mut contract = RaffleDapp::new();

        context.predecessor_account_id(alice_account_id());
        context.attached_deposit(1 * ONE_NEAR);
        testing_env!(context.build());

        contract.register_raffle(1644353705121, 1644353705130);
    }

    #[test]
    fn check_register_raffles() {
        let mut context = get_context();
        context.predecessor_account_id(raffle_dapp_account_id());
        testing_env!(context.build());
        let mut contract = RaffleDapp::new();

        context.predecessor_account_id(alice_account_id());
        context.attached_deposit(3 * ONE_NEAR);
        testing_env!(context.build());

        contract.register_raffle(1644353705121, 1644353705130);

        let prize: Balance = contract
            .raffles
            .get(&env::predecessor_account_id())
            .unwrap()
            .prize;

        println!(
            "Raffle registered succesfully for {:?}, with prize money: {:?}N",
            env::predecessor_account_id().to_string(),
            prize / ONE_NEAR
        );
    }

    #[test]
    #[should_panic(expected = "You have already registered a raffle")]
    fn check_register_raffles_already_registered() {
        let mut context = get_context();
        context.predecessor_account_id(raffle_dapp_account_id());
        testing_env!(context.build());
        let mut contract = RaffleDapp::new();

        context.predecessor_account_id(alice_account_id());
        context.attached_deposit(3 * ONE_NEAR);
        testing_env!(context.build());

        contract.register_raffle(1644353705121, 1644353705130);
        contract.register_raffle(1644353705128, 1644353705140);
    }


    

}
