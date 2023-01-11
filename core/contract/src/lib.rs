// Find all NEAR documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, env, near_bindgen, AccountId, Promise, Balance};
use std::collections::BTreeMap;



// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    betcontext_count: u128,
    betcontext_titles: BTreeMap<u128, String>,
    betcontext_amounts: BTreeMap<u128, Vec<(AccountId, u128)>>,
    betcontext_bets: BTreeMap<u128, Vec<(AccountId, u128)>>,
    betcontext_fate: BTreeMap<u128, u128>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract{
    fn default() -> Self{
        Self{
            betcontext_count: 0, 
            betcontext_titles: BTreeMap::new(), 
            betcontext_amounts: BTreeMap::new(), 
            betcontext_bets: BTreeMap::new(), 
            betcontext_fate: BTreeMap::new(),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // Public method - returns the current number of bet contexts
    pub fn get_betcontext_count(&self) -> u128 {
        return self.betcontext_count.clone();
    }

    // Public method - returns all bet contexts stored
    pub fn get_all_betcontexts(&self) -> BTreeMap<u128, String> {
        return self.betcontext_titles.clone();
    }

    // Public method - get all the votes for given betcontext ID
    pub fn get_all_bets(&self, betcontext_id: u128) -> Vec<(AccountId, u128)> {
       if  self.betcontext_bets.get(&betcontext_id).is_none() {
        return Vec::new();
       }
        return self.betcontext_bets.get(&betcontext_id).clone().unwrap().to_vec();
    }
    
    // Public method - creates a new bet context
    pub fn create_betcontext(&mut self, betcontext_text: String) {
        let creator: AccountId = env::predecessor_account_id();
        assert_eq!(creator, "admin.near".parse().unwrap(), "Unauthorized Lead Validator");
        log!("Creating New Context: {}", betcontext_text);
        let new_context_count: u128 = self.betcontext_count.clone() + 1;
        self.betcontext_count = new_context_count;
        self.betcontext_titles.insert(new_context_count, betcontext_text);
        self.betcontext_bets.insert(new_context_count, Vec::new());
        self.betcontext_amounts.insert(new_context_count, Vec::new());
    }

    // Public method - allows placing a bet on a betcontext 
    pub fn bet_on_betcontext(&mut self, betcontext_id: u128, bet_choice: u128, amt: u128) {
        assert!(amt > 0, "Amount bet cannot be this low");
        let betcontext_exists = self.betcontext_titles.get(&betcontext_id);
        assert!(!betcontext_exists.is_none(), "Context does not Exist");
        let betcontext_status = self.betcontext_fate.get(&betcontext_id);
        assert!(betcontext_status.is_none(), "Betting has Already Closed!");
        let bet_deductible: Balance = env::attached_deposit();
        assert!(bet_deductible > amt, "Attach at least {} yoctoNEAR", amt);
        let better: AccountId = env::predecessor_account_id();
        let mut bet_vec = self.betcontext_bets.get(&betcontext_id).unwrap().clone();
        bet_vec.push((better.clone(), bet_choice.clone()));
        self.betcontext_bets.remove(&betcontext_id);
        self.betcontext_bets.insert(betcontext_id, (bet_vec).clone().to_vec());
        let mut amt_vec = self.betcontext_amounts.get(&betcontext_id).unwrap().clone();
        amt_vec.push((better.clone(), amt.clone()));
        self.betcontext_amounts.remove(&betcontext_id);
        self.betcontext_amounts.insert(betcontext_id, (amt_vec).clone().to_vec());

    }

    // Public method - closing of the betcontext by the admin
    pub fn close_betcontext(&mut self, betcontext_id: u128, winning_bet: u128) -> u128{
        let creator: AccountId = env::predecessor_account_id();
        assert_eq!(creator, "admin.near".parse().unwrap(), "Unauthorized Lead Validator");
        log!("Ending Context: {}", betcontext_id);
        let betcontext_exists = self.betcontext_titles.get(&betcontext_id);
        assert!(!betcontext_exists.is_none(), "Context does not Exist");
        let betcontext_status = self.betcontext_fate.get(&betcontext_id);
        assert!(betcontext_status.is_none(), "Bet has Already Closed!");
        let amt_vec = self.betcontext_amounts.get(&betcontext_id).unwrap().clone();
        let mut total_amount : u128 = 0;
        for item in amt_vec.clone() {
            total_amount += item.1;
        }
        let bet_vec = self.betcontext_bets.get(&betcontext_id).unwrap().clone();
        let mut winner_amount : u128 = 0;
        for item in bet_vec.clone() {
            if item.1 == winning_bet {
                for item2 in amt_vec.clone() {
                    if item2.0 == item.0 {
                        winner_amount = winner_amount + item2.1; 
                        break;
                    }
                }
                
            }
        }
        let mut payable : u128 = 0;
        for item in bet_vec.clone() {
            if item.1 == winning_bet {
                for item2 in amt_vec.clone() {
                    if item2.0 == item.0 {
                        payable = item2.1 + total_amount*((item2.1)/winner_amount); 
                        Promise::new((item2.0).clone()).transfer(payable.clone());
                        break;
                    }
                }
                
                
            }
        }
        
        self.betcontext_fate.insert(betcontext_id, winning_bet);
        return winner_amount;
        
        
    }

   
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::testing_env;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::Balance;

    //const BENEFICIARY: &str = "beneficiary";
    const NEAR: u128 = 1000000000000000000000000;

    #[test]
    fn test_get_default_betcontexts() {
        let contract = Contract::default();
        // to check if the initialization is with no bet contexts already stored
        assert_eq!(
            contract.get_betcontext_count(),
            0
        );
    }

    #[test]
    fn test_create_new_betcontext() {
        let mut contract = Contract::default();
        let acc: AccountId = "admin.near".parse().unwrap();
        set_context(acc, 10*NEAR);
        contract.create_betcontext("What will be the max temperature tomorrow?".to_string());
        assert_eq!(
            contract.get_betcontext_count(),
            1
        );
    }

    #[test]
    fn test_bet_on_betcontext() {
        let mut contract = Contract::default();
        let acc1: AccountId = "admin.near".parse().unwrap();
        set_context(acc1, 10*NEAR);
        contract.create_betcontext("Who will score the first goal in Roma vs Milan tomorrow?".to_string());
        let acc2: AccountId = "mikky.near".parse().unwrap();
        set_context(acc2, 55*NEAR);
        contract.bet_on_betcontext(1, 3, 50*NEAR);
        assert_eq!(
            1,
            1
        );
        // Just checks if the code finishes execution 
        // and the said steps complete without panicking
        // This is taken to imply success.
    }

    #[test]
    fn test_close_betcontext() {
        let mut contract = Contract::default();
        let acc1: AccountId = "admin.near".parse().unwrap();
        set_context(acc1.clone(), 10*NEAR);
        contract.create_betcontext("Who will win the Grammys this time?".to_string());
        let acc2: AccountId = "kurt.near".parse().unwrap();
        set_context(acc2, 10*NEAR);
        contract.bet_on_betcontext(1, 1, 5*NEAR);
        let acc3: AccountId = "weiler.near".parse().unwrap();
        set_context(acc3, 10*NEAR);
        contract.bet_on_betcontext(1, 2, 7*NEAR);
        let acc4: AccountId = "brandon.near".parse().unwrap();
        set_context(acc4, 10*NEAR);
        contract.bet_on_betcontext(1, 2, 6*NEAR);
        let acc5: AccountId = "snow.near".parse().unwrap();
        set_context(acc5, 10*NEAR);
        contract.bet_on_betcontext(1, 3, 8*NEAR);
        set_context(acc1.clone(), 10*NEAR);
        let result = contract.close_betcontext(1, 2);
        assert_eq!(
            result,
            13*NEAR
        );
       
    }

    

    

    fn set_context(predecessor: AccountId, amount: Balance) {
        let mut builder = VMContextBuilder::new();
        
        builder.predecessor_account_id(predecessor);
        builder.attached_deposit(amount);
    
        testing_env!(builder.build());
      }
}