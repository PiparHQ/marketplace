extern crate uuid;
use near_sdk::{self, collections::{LookupSet, Vector}, borsh::{self, BorshDeserialize, BorshSerialize}, PublicKey, Balance};
use near_sdk::{env, near_bindgen, assert_one_yocto, AccountId, Gas, Promise, PanicOnDefault, json_types::U128, is_promise_success,};
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;

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
pub struct Transaction {
    transaction_id: String,
    product_id: String,
    store_contract_id: AccountId,
    buyer_contract_id: AccountId,
    buyer_value_locked: u128,
    product_quantity: u128,
    is_discount: bool,
    is_reward: bool,
    approved: bool,
    shipped: bool,
    delivered: bool,
    disputed: bool,
    canceled: bool,
    hashed_billing_address: String,
    nonce: String,
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
    owner_id: String,
    contract_id: AccountId,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct TokenData {
    product_id: String,
    quantity: u128,
    buyer_account_id: AccountId,
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

    pub fn check_contains_store(&self, store_id: String) -> bool {
        self.stores.contains(&store_id)
    }

    pub fn get_store_cost(&self) -> U128 {
        self.store_cost.into()
    }

    pub fn get_transaction_count(&self) -> usize {
        self.transactions.iter().count()
    }

    pub fn get_all_transactions(&self) {
        // let num: usize = self.transactions.iter().count();
        // let transactions = self.transactions.iter().take(num);
        // println!("{:?}", transactions)
        for t in self.transactions.iter() {
            println!("{:?}", t)
        }
    }

    pub fn get_buyer_transactions(&self, account_id: AccountId) {
        for t in self.transactions.iter() {
            if t.buyer_contract_id == account_id {
                println!("{:?}", t)
            }
        }
    }

    pub fn get_seller_transactions(&self, account_id: AccountId) {
        for t in self.transactions.iter() {
            if t.store_contract_id == account_id {
                println!("{:?}", t)
            }
        }
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
    pub fn create_account(&mut self, new_account_id: String, new_public_key: PublicKey, keypom_args: KeypomArgs) -> Promise {
        let prefix = &new_account_id[0..new_account_id.len()-8];
        let public_key: PublicKey = new_public_key;
        let _keypom = keypom_args;
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: new_account_id.clone(),
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
                        prefix.to_string(),
                    )
            )
    }

    #[private]
    pub fn deploy_store_keypom_callback(
        &mut self,
        prefix: String,
    ) {
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
        self.assert_no_store_with_id(prefix.clone());
        self.account_name_is_valid(prefix.clone());
        assert_ne!(prefix.clone(), "market");
        assert_ne!(prefix.clone(), "pipar");
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: env::signer_account_id().to_string(),
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
                        prefix.clone(),
                        env::attached_deposit().into(),
                    )
            )
    }

    pub fn buy(
        &mut self,
        product_id: String,
        store_contract_id: AccountId,
        product_quantity: u128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
    ) -> Promise {
        let check_existing = self.transactions
            .iter()
            .position(|t| t.product_id == product_id && t.store_contract_id == store_contract_id && t.buyer_contract_id == env::predecessor_account_id())
            .unwrap();

        match self.transactions.get(check_existing as u64) {
            Some(t) => panic!("Cannot escrow buy twice on the same product with the same seller, you must complete one first: {:?}", t),
            None => {
                let args = serde_json::to_vec(&Buy {
                    product_id: product_id.clone(),
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
                                product_id.clone(),
                                store_contract_id,
                                product_quantity,
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
        attached_deposit: u128,
        product_id: String,
        store_contract_id: AccountId,
        product_quantity: u128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
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
                hashed_billing_address: hashed_billing_address,
                nonce: nonce,
                time_created: env::block_timestamp(),
            });
            env::log_str("Successful purchased product")
        } else {
            Promise::new(buyer_account_id)
                .transfer(attached_deposit);
            env::log_str("Product purchase failed, returning funds")
        }
    }

    pub fn complete_purchase(
        &mut self,
        transaction_id: String,
        store_contract_id: AccountId,
    ) -> Promise {
        let check_existing = self.transactions
            .iter()
            .position(|t| t.transaction_id == transaction_id && t.store_contract_id == store_contract_id && t.buyer_contract_id == env::predecessor_account_id() && t.approved == true && t.shipped == true && t.delivered == false && t.disputed == false && t.canceled == false)
            .unwrap();

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                if t.is_reward == true {
                    let args = serde_json::to_vec(&TokenData {
                        product_id: t.product_id,
                        quantity: t.product_quantity,
                        buyer_account_id: env::current_account_id(),
                    })
                        .unwrap();
                    Promise::new(store_contract_id.clone())
                        .function_call("reward_with_token".to_owned(), args, NO_DEPOSIT, PGAS)
                        .then(
                            Self::ext(env::current_account_id())
                                .complete_purchase_callback(
                                    check_existing as u64,
                                )
                        )
                } else {
                    Promise::new(env::current_account_id())
                        .then(
                            Self::ext(env::current_account_id())
                                .complete_purchase_callback(
                                    check_existing as u64,
                                )
                        )
                }
            }
            None => panic!("Cannot complete transaction at this time, please try again later"),
        }
    }

    #[private]
    pub fn complete_purchase_callback(
        &mut self,
        check_existing: u64,
    ) {
        if is_promise_success() {
            match self.transactions.get(check_existing as u64) {
                Some(t) => {
                        self.transactions.replace(check_existing, &Transaction {
                            transaction_id: t.transaction_id,
                            product_id: t.product_id,
                            store_contract_id: t.store_contract_id.clone(),
                            buyer_contract_id: t.buyer_contract_id.clone(),
                            buyer_value_locked: t.buyer_value_locked,
                            product_quantity: t.product_quantity,
                            is_discount: t.is_discount,
                            is_reward: t.is_reward,
                            approved: t.approved,
                            shipped: t.shipped,
                            delivered: true,
                            disputed: t.disputed,
                            canceled: t.canceled,
                            hashed_billing_address: t.hashed_billing_address,
                            nonce: t.nonce,
                            time_created: t.time_created,
                        });
                    Promise::new(t.store_contract_id)
                        .transfer(t.buyer_value_locked);
                    env::log_str("Successful transaction completion")
                }
                None => panic!("Transaction not found"),
            }
        } else {
            env::log_str("Product purchase failed, returning funds")
        }
    }

    pub fn dispute_purchase(
        &mut self,
        transaction_id: String,
        store_contract_id: AccountId,
    ) {
        let check_existing = self.transactions
            .iter()
            .position(|t| t.transaction_id == transaction_id && t.store_contract_id == store_contract_id && t.buyer_contract_id == env::predecessor_account_id() && t.approved == true && t.shipped == true && t.delivered == false && t.disputed == false && t.canceled == false)
            .unwrap();

            match self.transactions.get(check_existing as u64) {
                Some(t) => {
                    self.transactions.replace(check_existing as u64, &Transaction {
                        transaction_id: t.transaction_id,
                        product_id: t.product_id,
                        store_contract_id: t.store_contract_id,
                        buyer_contract_id: t.buyer_contract_id,
                        buyer_value_locked: t.buyer_value_locked,
                        product_quantity: t.product_quantity,
                        is_discount: t.is_discount,
                        is_reward: t.is_reward,
                        approved: t.approved,
                        shipped: t.shipped,
                        delivered: t.delivered,
                        disputed: false,
                        canceled: t.canceled,
                        hashed_billing_address: t.hashed_billing_address,
                        nonce: t.nonce,
                        time_created: t.time_created,
                    });
                    env::log_str("Transaction has been marked disputed")
                }
                None => panic!("Transaction not found"),
            }
    }

}


