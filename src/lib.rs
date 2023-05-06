extern crate uuid;
use near_sdk::{self, assert_one_yocto, collections::{LookupSet, LookupMap, Vector}, borsh::{self, BorshDeserialize, BorshSerialize}, PublicKey, Balance};
use near_sdk::{ext_contract, env, log, near_bindgen, AccountId, Gas, Promise, PromiseError, PanicOnDefault, json_types::U128, is_promise_success,};
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;
use std::fmt;

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
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, fmt::Debug)]
pub struct Transaction {
    transaction_id: String,
    product_id: String,
    store_contract_id: AccountId,
    buyer_contract_id: AccountId,
    buyer_value_locked: u128,
    product_quantity: u32,
    is_discount: bool,
    is_reward: bool,
    approved: bool,
    shipped: bool,
    delivered: bool,
    disputed: bool,
    canceled: bool,
    time_created: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PiparContractFactory {
    pub stores: LookupSet<String>,
    pub transactions: Vector<Transaction>,
    pub store_cost: u128
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct KeypomArgs {
    account_id_field: Option<String>,
    drop_id_field: Option<String>,
    key_id_field: Option<String>,
    funder_id_field: Option<String>,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct Buy {
    product_id: String,
    buyer_account_id: AccountId,
    attached_near: Balance
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct Metadata {
    receiver_id: String,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct FtData {
    owner_id: AccountId,
    contract_id: AccountId,
}

#[near_bindgen]
impl PiparContractFactory {
    pub fn assert_no_store_with_id(&self, store_id: String) {
        assert!(
            !self.check_contains_store(store_id),
            "Store with that ID already exists"
        );
    }

    pub fn check_contains_store(&self, store_id: String) -> bool {
        self.stores.contains(&store_id)
    }

    pub fn get_store_cost(&self) -> U128 {
        self.store_cost.into()
    }

    /// Initialization
    #[init(ignore_state)]
    #[private]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            stores: LookupSet::new(b"t".to_vec()),
            transactions: Vector::new(b"vec-uid-1".to_vec()),
            store_cost: STORE_BALANCE
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old = env::state_read().expect("migrating state");
        Self { ..old }
    }

    #[payable]
    pub fn create_account(&mut self, new_account_id: AccountId, new_public_key: PublicKey, keypom_args: KeypomArgs) -> Promise {
        let prefix = &new_account_id[0..new_account_id.len()-8];
        let public_key: PublicKey = new_public_key;
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: new_account_id,
            contract_id: env::current_account_id()
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
                    .with_static_gas(Gas(5*TGAS))
                    .deploy_store_keypom_callback(
                        &new_account_id,
                        &prefix,
                        env::attached_deposit().into(),
                    )
            )
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
            env::log_str("Successful token deployment")
        } else {
            Promise::new(store_creator_id)
                .transfer(attached_deposit);
            env::log_str("failed token deployment & funds returned")
        }
    }

    #[payable]
    pub fn create_store(&mut self, prefix: String) -> Promise {
        assert!(
            env::attached_deposit() >= STORE_BALANCE,
            "To cover the storage required for your store, you need to attach at least {} yoctoNEAR to this transaction.",
            STORE_BALANCE
        );
        self.assert_no_store_with_id(prefix);
        assert_ne!(&prefix, "market");
        assert_ne!(&prefix, "pipar");
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: env::signer_account_id(),
            contract_id: env::current_account_id()
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
                    .with_static_gas(Gas(5*TGAS))
                    .deploy_store_callback(
                        env::signer_account_id(),
                        &prefix,
                        env::attached_deposit().into(),
                    )
            )
    }

    pub fn buy(
        &mut self,
        product_id: String,
        store_contract_id: AccountId,
        product_quantity: u32,
        is_discount: bool,
        is_reward: bool,
    ) -> Promise {
        match self.transactions.iter().position(|t| t.product_id == product_id && t.store_contract_id == store_contract_id && t.buyer_contract_id == env::predecessor_account_id()).unwrap() {
            Some(t) => panic!("Cannot escrow buy twice on the same product with the same seller, you must complete one first: {:?}", t),
            None => {
                let args = serde_json::to_vec(&Buy {
                    product_id: product_id,
                    buyer_account_id: env::predecessor_account_id(),
                    attached_near: env::attached_deposit(),
                })
                    .unwrap();
                Promise::new(store_contract_id.clone())
                    .function_call("store_purchase_product".to_owned(), args, NO_DEPOSIT, PGAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .buy_callback(
                                env::predecessor_account_id(),
                                env::attached_deposit(),
                                &product_id,
                                &store_contract_id,
                                &product_quantity,
                                &is_discount,
                                &is_reward,
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
        product_id: String,
        store_contract_id: AccountId,
        product_quantity: u32,
        is_discount: bool,
        is_reward: bool,
    ) {
        let attached_deposit: u128 = attached_deposit.into();
        if is_promise_success() {
            let id = Uuid::new_v4().to_string();
            self.transactions.push(&Transaction {
                transaction_id: id,
                product_id: product_id,
                store_contract_id: store_contract_id,
                buyer_contract_id: buyer_account_id,
                buyer_value_locked: attached_deposit,
                product_quantity: product_quantity,
                is_discount: is_discount,
                is_reward: is_reward,
                approved: true,
                shipped: false,
                delivered: false,
                disputed: false,
                canceled: false,
                time_created: env::block_timestamp(),
            });
            env::log_str("Successful purchased product")
        } else {
            Promise::new(buyer_account_id)
                .transfer(attached_deposit);
            env::log_str("Product purchase failed, returning funds")
        }
    }

}


