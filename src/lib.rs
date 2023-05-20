use near_sdk::{
    self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupSet, Vector},
    Balance, PublicKey,
};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json};
use near_sdk::{
    assert_one_yocto, env, is_promise_success, json_types::U128, near_bindgen, AccountId, Gas, PanicOnDefault, Promise,
};

// Constants
pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const STORE_BALANCE: u128 = 10_000_000_000_000_000_000_000_000;
pub const NO_DEPOSIT: Balance = 0;
pub const TGAS: u64 = 1_000_000_000_000;

pub const fn tgas(n: u64) -> Gas {
    Gas(n * 10u64.pow(12))
}
pub const PGAS: Gas = tgas(65 + 5);

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Transaction {
    pub transaction_id: U128,
    pub product_id: U128,
    pub store_contract_id: AccountId,
    pub buyer_contract_id: AccountId,
    pub buyer_value_locked: U128,
    pub product_quantity: U128,
    pub timeout: U128,
    pub is_discount: bool,
    pub is_reward: bool,
    pub approved: bool,
    pub shipped: bool,
    pub delivered: bool,
    pub disputed: bool,
    pub canceled: bool,
    pub hashed_billing_address: String,
    pub nonce: String
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct KeypomArgs {
    account_id_field: Option<String>,
    drop_id_field: Option<String>,
    key_id_field: Option<String>,
    funder_id_field: Option<String>,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Buy {
    product_id: U128,
    product_quantity: U128,
    buyer_account_id: AccountId,
    attached_near: U128,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Metadata {
    receiver_id: String,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FtData {
    owner_id: AccountId,
    contract_id: AccountId,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenData {
    product_id: U128,
    quantity: U128,
    buyer_account_id: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PiparContractFactory {
    pub stores: LookupSet<String>,
    pub transactions: Vector<Transaction>,
    pub store_cost: U128,
}

#[near_bindgen]
impl PiparContractFactory {

    pub fn assert_no_store_with_id(&self, store_id: String) {
        assert!(
            !self.check_contains_store(store_id),
            "Store with that ID already exists"
        );
    }

    pub fn assert_only_buyer(&self, buyer_account_id: AccountId) {
        assert_one_yocto();
        assert_eq!(
            env::signer_account_id(),
            buyer_account_id,
            "Only transaction buyer can call this method"
        )
    }

    pub fn assert_only_seller(&self, store_account_id: AccountId) {
        assert_one_yocto();
        assert_eq!(
            env::signer_account_id(),
            store_account_id,
            "Only transaction seller can call this method"
        )
    }

    pub fn account_name_is_valid(&self, prefix: String) {
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        assert!(
            env::is_valid_account_id(subaccount.as_bytes()),
            "Account is invalid"
        )
    }

    pub fn calculate_timeout(&self, timeout: U128, timestamp: U128) -> u128 {
        let timeout: u128 = timeout.into();
        let timestamp: u128 = timestamp.into();
        let calc_seconds: u128 = timeout * 24 * 60 * 60;
        let calc_seconds_in_nano: u128 = calc_seconds * 1000000000;
        let total_timestamp: u128 = timestamp + calc_seconds_in_nano;

        total_timestamp
    }

    pub fn check_contains_store(&self, store_id: String) -> bool {
        self.stores.contains(&store_id)
    }

    pub fn get_store_cost(&self) -> u128 {
        self.store_cost.into()
    }

    pub fn get_transaction_count(&self) -> usize {
        self.transactions.iter().count()
    }

    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let transactions: Vec<Transaction> = self.transactions.iter().map(|x| x).collect();

        transactions
    }

    pub fn get_buyer_transactions(&self, account_id: AccountId) -> Vec<Transaction> {
        let transactions: Vec<Transaction> = self.transactions.iter().filter(|x| x.buyer_contract_id == account_id).collect();

        transactions
    }

    pub fn get_seller_transactions(&self, account_id: AccountId) -> Vec<Transaction> {
        let transactions: Vec<Transaction> = self.transactions.iter().filter(|x| x.store_contract_id == account_id).collect();

        transactions
    }

    #[init]
    pub fn new() -> Self {
        Self {
            stores: LookupSet::new(b"s".to_vec()),
            transactions: Vector::new(b"v".to_vec()),
            store_cost: U128::from(STORE_BALANCE),
        }
    }

    #[payable]
    pub fn create_account(
        &mut self,
        new_account_id: String,
        new_public_key: PublicKey,
        keypom_args: KeypomArgs,
    ) -> Promise {
        let prefix = &new_account_id[0..new_account_id.len() - 8];
        let public_key: PublicKey = new_public_key;
        let _keypom = keypom_args;
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: new_account_id.parse().unwrap(),
            contract_id: env::current_account_id(),
        })
        .unwrap();

        Promise::new(subaccount.clone())
            .create_account()
            .add_full_access_key(public_key)
            .transfer(STORE_BALANCE)
            .deploy_contract(include_bytes!("../wasm/store.wasm").to_vec())
            .function_call("new".to_owned(), init_args, NO_DEPOSIT, PGAS)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .deploy_store_keypom_callback(prefix.to_string()),
            )
    }

    #[private]
    pub fn deploy_store_keypom_callback(&mut self, prefix: String) {
        if is_promise_success() {
            self.stores.insert(&prefix);
            env::log_str("Successful token deployment")
        } else {
            env::log_str("failed token deployment & funds returned")
        }
    }

    #[private]
    pub fn deploy_store_callback(
        &mut self,
        store_creator_id: AccountId,
        prefix: String,
        attached_deposit: U128,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            self.stores.insert(&prefix);
            env::log_str("Successful store deployment")
        } else {
            Promise::new(store_creator_id).transfer(attached_deposit);
            env::log_str("failed store deployment & funds returned")
        }
    }

    #[payable]
    pub fn create_store(&mut self, prefix: String) -> Promise {
        assert!(
            env::attached_deposit() >= STORE_BALANCE,
            "To cover the storage required for your store, you need to attach at least {} yoctoNEAR to this transaction.",
            STORE_BALANCE
        );
        self.assert_no_store_with_id(prefix.clone());
        self.account_name_is_valid(prefix.clone());
        assert_ne!(prefix.clone(), "market");
        assert_ne!(prefix.clone(), "pipar");
        assert_ne!(prefix.clone(), "dao");
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: env::signer_account_id(),
            contract_id: env::current_account_id(),
        })
        .unwrap();

        Promise::new(subaccount.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(STORE_BALANCE)
            .deploy_contract(include_bytes!("../wasm/store.wasm").to_vec())
            .function_call("new".to_owned(), init_args, NO_DEPOSIT, PGAS)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .deploy_store_callback(
                        env::signer_account_id(),
                        prefix.clone(),
                        U128::from(env::attached_deposit()),
                    ),
            )
    }

    #[payable]
    pub fn buy(
        &mut self,
        product_id: U128,
        store_contract_id: AccountId,
        product_quantity: U128,
        timeout: U128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
    ) -> Promise {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.product_id == product_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_contract_id == env::predecessor_account_id()
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => panic!("Cannot escrow buy twice on the same product with the same seller, you must complete one first: {:?}", t),
            None => {
                let args = serde_json::to_vec(&Buy {
                    product_id: product_id.clone(),
                    product_quantity: product_quantity.clone(),
                    buyer_account_id: env::predecessor_account_id(),
                    attached_near: env::attached_deposit().into(),
                })
                    .unwrap();
                Promise::new(store_contract_id.clone())
                    .function_call("store_purchase_product".to_owned(), args, NO_DEPOSIT, PGAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .buy_callback(
                                env::predecessor_account_id(),
                                U128::from(env::attached_deposit()),
                                product_id.clone(),
                                store_contract_id,
                                product_quantity,
                                timeout,
                                is_discount,
                                is_reward,
                                hashed_billing_address,
                                nonce,
                            )
                    )
            }
        }
    }

    #[private]
    pub fn buy_callback(
        &mut self,
        buyer_account_id: AccountId,
        attached_deposit: U128,
        product_id: U128,
        store_contract_id: AccountId,
        product_quantity: U128,
        timeout: U128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            self.transactions.push(&Transaction {
                transaction_id: U128::from(env::block_timestamp() as u128),
                product_id,
                store_contract_id,
                buyer_contract_id: buyer_account_id,
                buyer_value_locked: attached_deposit.into(),
                product_quantity,
                timeout,
                is_discount,
                is_reward,
                approved: true,
                shipped: false,
                delivered: false,
                disputed: false,
                canceled: false,
                hashed_billing_address,
                nonce
            });
            env::log_str("Successful purchased product")
        } else {
            Promise::new(buyer_account_id).transfer(attached_deposit);
            env::log_str("Product purchase failed, returning funds")
        }
    }

    pub fn complete_purchase(
        &mut self,
        transaction_id: U128,
        store_contract_id: AccountId,
    ) -> Promise {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_contract_id == env::predecessor_account_id()
                    && t.approved == true
                    && t.shipped == true
                    && t.delivered == false
                    && t.disputed == false
                    && t.canceled == false
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                if t.is_reward == true {
                    let args = serde_json::to_vec(&TokenData {
                        product_id: t.product_id,
                        quantity: t.product_quantity,
                        buyer_account_id: t.buyer_contract_id,
                    })
                    .unwrap();
                    Promise::new(store_contract_id.clone())
                        .function_call("reward_with_token".to_owned(), args, NO_DEPOSIT, PGAS)
                        .then(
                            Self::ext(env::current_account_id())
                                .complete_purchase_callback(check_existing as u64),
                        )
                } else {
                    Promise::new(env::current_account_id())
                        .then(
                        Self::ext(env::current_account_id())
                            .complete_purchase_callback(check_existing as u64),
                    )
                }
            }
            None => panic!("Cannot complete transaction at this time, please try again later"),
        }
    }

    #[private]
    pub fn complete_purchase_callback(&mut self, check_existing: u64) {
        if is_promise_success() {
            match self.transactions.get(check_existing.clone() as u64) {
                Some(t) => {
                    self.transactions.replace(
                        check_existing as u64,
                        &Transaction {
                            transaction_id: t.transaction_id,
                            product_id: t.product_id,
                            store_contract_id: t.store_contract_id.clone(),
                            buyer_contract_id: t.buyer_contract_id.clone(),
                            buyer_value_locked: t.buyer_value_locked,
                            product_quantity: t.product_quantity,
                            timeout: t.timeout,
                            is_discount: t.is_discount,
                            is_reward: t.is_reward,
                            approved: t.approved,
                            shipped: t.shipped,
                            delivered: true,
                            disputed: t.disputed,
                            canceled: t.canceled,
                            hashed_billing_address: t.hashed_billing_address,
                            nonce: t.nonce
                        },
                    );
                    let payout: u128 = t.buyer_value_locked.into();
                    let percent = payout as f64 * 0.98;
                    let seller_funds = percent as u128;
                    Promise::new(t.store_contract_id.clone()).transfer(seller_funds);
                    env::log_str("Successful transaction completion")
                }
                None => panic!("Transaction not found"),
            }
        } else {
            env::log_str("Product Purchase Completion failed, returning funds")
        }
    }

    pub fn dispute_purchase(&mut self, transaction_id: U128, store_contract_id: AccountId) {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_contract_id == env::predecessor_account_id()
                    && t.approved == true
                    && t.shipped == true
                    && t.delivered == false
                    && t.disputed == false
                    && t.canceled == false
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing.clone() as u64) {
            Some(t) => {
                self.transactions.replace(
                    check_existing as u64,
                    &Transaction {
                        transaction_id: t.transaction_id,
                        product_id: t.product_id,
                        store_contract_id: t.store_contract_id,
                        buyer_contract_id: t.buyer_contract_id,
                        buyer_value_locked: t.buyer_value_locked,
                        product_quantity: t.product_quantity,
                        timeout: t.timeout,
                        is_discount: t.is_discount,
                        is_reward: t.is_reward,
                        approved: t.approved,
                        shipped: t.shipped,
                        delivered: t.delivered,
                        disputed: false,
                        canceled: t.canceled,
                        hashed_billing_address: t.hashed_billing_address,
                        nonce: t.nonce
                    },
                );
                env::log_str("Transaction has been marked disputed")
            }
            None => panic!("Transaction not found"),
        }
    }

    pub fn mark_shipped(&mut self, transaction_id: U128, buyer_contract_id: AccountId) {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == env::predecessor_account_id()
                    && t.buyer_contract_id == buyer_contract_id
                    && t.approved == true
                    && t.shipped == false
                    && t.delivered == false
                    && t.disputed == false
                    && t.canceled == false
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                self.transactions.replace(
                    check_existing as u64,
                    &Transaction {
                        transaction_id: t.transaction_id,
                        product_id: t.product_id,
                        store_contract_id: t.store_contract_id,
                        buyer_contract_id: t.buyer_contract_id,
                        buyer_value_locked: t.buyer_value_locked,
                        product_quantity: t.product_quantity,
                        timeout: t.timeout,
                        is_discount: t.is_discount,
                        is_reward: t.is_reward,
                        approved: t.approved,
                        shipped: true,
                        delivered: t.delivered,
                        disputed: t.disputed,
                        canceled: t.canceled,
                        hashed_billing_address: t.hashed_billing_address,
                        nonce: t.nonce
                    },
                );
                env::log_str("Transaction has been marked shipped")
            }
            None => panic!("Transaction not found"),
        }
    }

    pub fn get_refund(&mut self, transaction_id: U128, store_contract_id: AccountId) {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_contract_id == env::predecessor_account_id()
                    && t.approved == true
                    && t.shipped == false
                    && t.delivered == false
                    && t.disputed == false
                    && t.canceled == false
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                let timeout = self.calculate_timeout(t.timeout, t.transaction_id);
                let current_timestamp = env::block_timestamp() as u128;

                if current_timestamp >= timeout {
                    let attached_deposit: u128 = t.buyer_value_locked.into();
                    env::log_str("Transaction time has elapsed, returning funds to the buyer");
                    Promise::new(env::predecessor_account_id()).transfer(attached_deposit);
                } else {
                    panic!("Transaction time is yet to elapsed, please try again later")
                }
            }
            None => panic!("Transaction not found"),
        }
    }

}
