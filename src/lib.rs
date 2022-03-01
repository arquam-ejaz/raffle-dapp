use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId, Balance, Gas, Promise, Timestamp};
use serde_json::json;

// constant representing 1 NEAR in yoctoNear
const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;

// constant to convert milliseconds to nanoseconds and vice versa
const TO_FROM_NANOSECONDS: u64 = 1_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RaffleDetails {
    prize: Balance,
    start: Timestamp,
    end: Timestamp,
    participants: UnorderedMap<AccountId, Balance>,
    attempts: u8,
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
            "Only the contract account can initialize the smart contract"
        );
        Self {
            raffles: UnorderedMap::new(b"r"),
        }
    }

    #[payable]
    pub fn register_raffle(&mut self, start: Timestamp, end: Timestamp) {
        // Check if the attached deposit is greater than 2 NEAR to cover storage and service fees
        // Thus, Prize = attached depost (in NEAR) - 2 NEAR
        assert!(
            env::attached_deposit() > 2 * ONE_NEAR,
            "Prize money should be greater than 2 NEAR"
        );

        // Allowing to register only one raffle per account,
        // they can register a new raffle after their current raffle ends
        assert!(
            self.raffles.get(&env::predecessor_account_id()).is_none(),
            "You have already registered a raffle"
        );

        assert!(
            end > start,
            "The raffle's end date should be greater than its start date"
        );

        let raffle_details = RaffleDetails {
            prize: env::attached_deposit() - 2 * ONE_NEAR,
            start: start * TO_FROM_NANOSECONDS,
            end: end * TO_FROM_NANOSECONDS,
            participants: UnorderedMap::new(env::sha256(&env::predecessor_account_id().as_bytes())),
            attempts: 0,
        };

        self.raffles
            .insert(&env::predecessor_account_id(), &raffle_details);

        let raffle_details: RaffleDetails =
            self.raffles.get(&env::predecessor_account_id()).unwrap();

        env::log_str(&format!(
            "Raffle registered succesfully for {:?} with prize money {:?} NEAR starting from {:?} ms till {:?} ms",
            env::predecessor_account_id().to_string(),
            raffle_details.prize / ONE_NEAR,
            raffle_details.start/TO_FROM_NANOSECONDS,
            raffle_details.end/TO_FROM_NANOSECONDS
        ));
    }

    #[payable]
    pub fn participate(&mut self, raffle_id: String) {
        // Users can participate in the raffle by locking at least 1 NEAR token to prevent spam or duplicate entries to some extent.
        // The participant's locked NEAR tokens plays no role while deciding the winner to conduct an unbiased raffle.
        assert!(
            env::attached_deposit() >= 1 * ONE_NEAR,
            "The locked amount should be at least 1 NEAR"
        );

        let raffle_account_id: AccountId = AccountId::try_from(raffle_id).unwrap();

        assert_ne!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "The contract account cannot participate in raffles for security reasons"
        );

        assert_ne!(
            env::predecessor_account_id(),
            raffle_account_id,
            "You cannot participate in your own raffle"
        );

        assert!(
            self.raffles.get(&raffle_account_id).is_some(),
            "Sorry, no raffle is being conducted by {:?}",
            raffle_account_id.to_string()
        );

        assert!(
            self.raffles
                .get(&raffle_account_id)
                .unwrap()
                .participants
                .get(&env::predecessor_account_id())
                .is_none(),
            "You have already participated in this raffle"
        );

        assert!(
            self.raffles
                .get(&raffle_account_id)
                .unwrap()
                .participants
                .len()
                <= 256,
            "Sorry, the raffle's maximum participants limit reached"
        );

        let mut raffle_details = self.raffles.get(&raffle_account_id).unwrap();

        let current_timestamp = env::block_timestamp();
        assert!(
            current_timestamp > raffle_details.start && current_timestamp < raffle_details.end,
            "The raffle has either not started yet or has finished already"
        );

        let locked_tokens = env::attached_deposit();
        raffle_details
            .participants
            .insert(&env::predecessor_account_id(), &locked_tokens);

        self.raffles.insert(&raffle_account_id, &raffle_details);

        env::log_str(&format!(
            "{:?} has sucessfully participated in the raffle of {:?} with {:?} NEAR token(s) locked",
            env::predecessor_account_id().to_string(),
            raffle_account_id.to_string(),
            self.raffles
                .get(&raffle_account_id)
                .unwrap()
                .participants
                .get(&env::predecessor_account_id())
                .unwrap()
                / ONE_NEAR
        ));
    }

    pub fn finalize_raffle(&mut self, raffle_id: String) {
        let raffle_account_id: AccountId = AccountId::try_from(raffle_id.clone()).unwrap();
        assert!(
            env::predecessor_account_id() == raffle_account_id
                || env::predecessor_account_id() == env::current_account_id(),
            "Only the raffle's owner or the contract account can finalize the raffle"
        );

        assert!(
            self.raffles.get(&raffle_account_id).is_some(),
            "No raffle registered from this account"
        );

        let raffle_detail: RaffleDetails = self.raffles.get(&raffle_account_id).unwrap();
        let current_time = env::block_timestamp();

        assert!(
            current_time > raffle_detail.end,
            "You can only finalize raffle after it ends"
        );

        let participants: UnorderedMap<AccountId, Balance> = raffle_detail.participants;

        if participants.len() == 0 {
            self.raffles.remove(&raffle_account_id);
            Promise::new(raffle_account_id).transfer(raffle_detail.prize);
            env::log_str("Nobody participated in your raffle");
            return;
        }

        let participants_vec = participants.to_vec();

        env::log_str(&format!(
            "Total number of participants: {:?}",
            participants.len()
        ));

        let length = participants_vec.len() as u8;
        let random_seed = env::random_seed();
        env::log_str(&format!("env::random_seed = {:?}", random_seed));

        let mut random_index: u8 = random_seed[0];
        let mut found = false;

        for x in random_seed.iter() {
            if *x < length {
                random_index = *x;
                found = true;
                break;
            }
        }

        let mut raffle_detail: RaffleDetails = self.raffles.get(&raffle_account_id).unwrap();
        raffle_detail.attempts += 1;

        if !found {
            self.raffles.insert(&raffle_account_id, &raffle_detail);
            env::log_str(
                "Failed to discover Random index in this block, searching it in the future blocks...",
            );
            Promise::new(env::current_account_id()).function_call(
                "finalize_raffle".to_string(),
                json!({ "raffle_id": raffle_account_id.to_string() })
                    .to_string()
                    .into_bytes(),
                0,
                Gas::from(env::prepaid_gas() - env::used_gas() * 2),
            );
            return;
        }

        let winner_id = (participants_vec[random_index as usize].0).to_string();
        let winner_locked_tokens = participants_vec[random_index as usize].1;

        Promise::new(AccountId::try_from(winner_id.clone()).unwrap())
            .transfer(raffle_detail.prize + winner_locked_tokens);

        env::log_str(&format!(
            "The winner for this raffle is {:?} and his locked tokens was {:?} NEAR",
            winner_id,
            winner_locked_tokens / ONE_NEAR
        ));

        env::log_str(&format!(
            "The Random index {:?} was discovered in {:?} attempt(s)",
            random_index, raffle_detail.attempts
        ));

        for (participants_account_id, locked_tokens) in participants_vec {
            if participants_account_id.to_string() == winner_id {
                continue;
            }
            Promise::new(participants_account_id).transfer(locked_tokens);
        }

        self.raffles.remove(&raffle_account_id);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package raffle-dapp -- --nocapture
 * Note: 'raffle-dapp' comes from Cargo.toml's 'name' key
 */

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId};

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

    fn jacob_account_id() -> AccountId {
        AccountId::new_unchecked("jacob.testnet".to_string())
    }

    fn mike_account_id() -> AccountId {
        AccountId::new_unchecked("mike.testnet".to_string())
    }

    fn jack_account_id() -> AccountId {
        AccountId::new_unchecked("jack.testnet".to_string())
    }

    #[test]
    #[should_panic(expected = "The smart contract should be initialized before usage")]
    fn check_default() {
        RaffleDapp::default().register_raffle(1644353705121, 1644353705130);
    }

    #[test]
    #[should_panic(expected = "Only the contract account can initialize the smart contract")]
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

    #[test]
    fn check_participate() {
        let mut context = get_context();
        context.predecessor_account_id(raffle_dapp_account_id());
        testing_env!(context.build());

        let mut contract = RaffleDapp::new();

        context.predecessor_account_id(alice_account_id());
        context.attached_deposit(17 * ONE_NEAR);
        testing_env!(context.build());

        contract.register_raffle(1644353705121, 1644353705130);

        context.block_timestamp(1644353705125 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(bob_account_id());
        context.attached_deposit(2 * ONE_NEAR);
        testing_env!(context.build());

        contract.participate(alice_account_id().to_string());
    }

    #[test]
    fn check_randomness() {
        let mut map: UnorderedMap<String, u8> = UnorderedMap::new(b"c");

        map.insert(&"alice 1".to_string(), &1);
        map.insert(&"alice 2".to_string(), &2);
        map.insert(&"alice 3".to_string(), &3);
        map.insert(&"alice 4".to_string(), &4);
        map.insert(&"alice 5".to_string(), &5);
        map.insert(&"alice 6".to_string(), &6);
        map.insert(&"alice 7".to_string(), &7);
        map.insert(&"alice 8".to_string(), &8);
        map.insert(&"alice 9".to_string(), &9);
        map.insert(&"alice 10".to_string(), &10);

        let mut context = get_context();
        let v = vec![
            150, 255, 1, 8, 45, 32, 101, 50, 123, 221, 58, 3, 127, 202, 56, 16, 32, 9, 111, 255,
            49, 45, 77, 17, 25, 26, 37, 79, 210, 159, 31, 56,
        ];
        context.random_seed(v);
        testing_env!(context.build());

        let map_vec = map.to_vec();
        let length = map_vec.len() as u8;
        let random_seed = env::random_seed();
        let mut random_index = random_seed[0];
        let mut found = false;

        for x in random_seed.iter() {
            if *x < length {
                random_index = *x;
                found = true;
                break;
            }
        }

        // Should print RANDOM: ["alice 2" , 2] | INDEX: 1
        if !found {
            println!("Failed to discover Random index in this block, searching it in the future blocks...");
        } else {
            println!(
                "RANDOM: [{:?} , {:?}] | INDEX: {:?}",
                map_vec[random_index as usize].0,
                map_vec[random_index as usize].1,
                random_index as usize
            );
        }
    }

    #[test]
    fn check_finalize_raffle() {
        let mut context = get_context();
        context.predecessor_account_id(raffle_dapp_account_id());
        testing_env!(context.build());

        let mut contract = RaffleDapp::new();

        context.predecessor_account_id(alice_account_id());
        context.attached_deposit(17 * ONE_NEAR);
        testing_env!(context.build());

        contract.register_raffle(1644353705121, 1644353705521);

        context.block_timestamp(1644353705125 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(bob_account_id());
        context.attached_deposit(2 * ONE_NEAR);
        testing_env!(context.build());

        contract.participate(alice_account_id().to_string());

        context.block_timestamp(1644353705135 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(jacob_account_id());
        context.attached_deposit(10 * ONE_NEAR);
        testing_env!(context.build());

        contract.participate(alice_account_id().to_string());

        context.block_timestamp(1644353705145 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(mike_account_id());
        context.attached_deposit(15 * ONE_NEAR);
        testing_env!(context.build());

        contract.participate(alice_account_id().to_string());

        context.block_timestamp(1644353705150 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(jack_account_id());
        context.attached_deposit(27 * ONE_NEAR);
        testing_env!(context.build());

        contract.participate(alice_account_id().to_string());

        context.block_timestamp(1644353705600 * TO_FROM_NANOSECONDS);
        context.predecessor_account_id(alice_account_id());
        let v = vec![
            150, 255, 1, 8, 45, 32, 101, 50, 123, 221, 58, 3, 127, 202, 56, 16, 32, 9, 111, 255,
            49, 45, 77, 17, 25, 26, 37, 79, 210, 159, 31, 56,
        ];
        context.random_seed(v);
        testing_env!(context.build());

        contract.finalize_raffle(alice_account_id().to_string());
    }
}
