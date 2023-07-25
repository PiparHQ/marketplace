use near_sdk::{
    self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupSet, Vector, UnorderedMap},
    Balance, PublicKey,
};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, is_promise_success, json_types::U128, json_types::U64, near_bindgen, AccountId, Gas, PanicOnDefault, Promise, PromiseResult, serde_json::json
};

// Constants
pub const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
pub const STORE_BALANCE: u128 = 7_000_000_000_000_000_000_000_000;
pub const ONE_YOCTO: u128 = 10_000_000_000_000_000_000_000;
pub const NO_DEPOSIT: Balance = 0;
pub const TGAS: u64 = 1_000_000_000_000;

pub const fn tgas(n: u64) -> Gas {
    Gas(n * 10u64.pow(12))
}
pub const PGAS: Gas = tgas(35 + 5);

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Eq, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum TransactionStatus {
    Approved,
    Shipped,
    Delivered,
    Disputed,
    Canceled,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Transaction {
    pub transaction_id: U128,
    pub product_id: U64,
    pub store_contract_id: AccountId,
    pub buyer_id: AccountId,
    pub buyer_value_locked: U128,
    pub price: Balance,
    pub token_id: String,
    pub timeout: U128,
    pub affiliate: bool,
    pub affiliate_id: Option<AccountId>,
    pub affiliate_percentage: Option<u32>,
    pub is_discount: bool,
    pub is_reward: bool,
    pub is_keypom: bool,
    pub status: TransactionStatus,
    pub hashed_billing_address: String,
    pub nonce: String,
    pub ipfs: String
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
    id: U64,
    receiver_id: AccountId,
    attached_deposit: U128,
    color: String,
    affiliate: Option<AccountId>,
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
pub struct EmptyData {
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FtData {
    owner_id: AccountId,
    marketplace_contract_id: AccountId,
    name: String,
    symbol: String,
    icon: String,
    bg_icon: String,
    category: String,
    description: String,
    facebook: String,
    twitter: String,
    instagram: String,
    tiktok: String,
    youtube: String,
    zip: String,
    city: String,
    state: String,
    country: String,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenData {
    id: U64,
    receiver_id: AccountId,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    token_id: String,
}

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketplaceData {
    price: Balance,
    affiliate: bool,
    affiliate_id: Option<AccountId>,
    affiliate_percentage: Option<u32>,
    token_id: String,
    token_owner: AccountId,
    store_owner: AccountId,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PiparContractFactory {
    pub stores: LookupSet<String>,
    pub transactions: Vector<Transaction>,
    pub store_cost: U128,
    pub stores_stats: UnorderedMap<AccountId, U128>,
}

#[near_bindgen]
impl PiparContractFactory {

    pub fn assert_no_store_with_id(&self, prefix: String) -> bool {
        let current_account = env::current_account_id().to_string();
        let account: AccountId = format!("{prefix}.{current_account}").parse().unwrap();

        return if !self.check_contains_store(prefix) && env::is_valid_account_id(account.as_bytes()) {
            true
        } else {
            false
        }
    }

    pub fn assert_only_buyer(&self, buyer_account_id: AccountId) {
        assert_eq!(
            env::signer_account_id(),
            buyer_account_id,
            "Only transaction buyer can call this method"
        )
    }

    pub fn assert_only_seller(&self, store_account_id: AccountId) {
        assert_eq!(
            env::signer_account_id(),
            store_account_id,
            "Only transaction seller can call this method"
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
        let transactions: Vec<Transaction> = self.transactions.iter().filter(|x| x.buyer_id == account_id).collect();

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
            stores_stats: UnorderedMap::new(b"w".to_vec()),
        }
    }

    #[payable]
    pub fn create_account(
        &mut self,
        new_account_id: String,
        new_public_key: PublicKey,
        keypom_args: KeypomArgs,
        name: String,
        symbol: String,
        icon: String,
        bg_icon: String,
        category: String,
        description: String,
        facebook: String,
        twitter: String,
        instagram: String,
        tiktok: String,
        youtube: String,
        zip: String,
        city: String,
        state: String,
        country: String
    ) -> Promise {
        let prefix = &new_account_id[0..new_account_id.len() - 8];
        let public_key: PublicKey = new_public_key;
        let _keypom = keypom_args;
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: new_account_id.parse().unwrap(),
            marketplace_contract_id: env::current_account_id(),
            name,
            symbol,
            icon,
            bg_icon,
            category,
            description,
            facebook,
            twitter,
            instagram,
            tiktok,
            youtube,
            zip,
            city,
            state,
            country,
        })
        .unwrap();

        Promise::new(subaccount.clone())
            .create_account()
            .add_full_access_key(public_key)
            .transfer(STORE_BALANCE)
            .deploy_contract(include_bytes!("../wasm/store.wasm").to_vec())
            .function_call("new_default_meta".to_owned(), init_args, NO_DEPOSIT, PGAS)
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
    pub fn create_store(&mut self, prefix: String, name: String,
                        symbol: String,
                        icon: String,
                        bg_icon: String,
                        category: String,
                        description: String,
                        facebook: String,
                        twitter: String,
                        instagram: String,
                        tiktok: String,
                        youtube: String,
                        zip: String,
                        city: String,
                        state: String,
                        country: String) -> Promise {
        assert!(
            env::attached_deposit() > STORE_BALANCE,
            "To cover the storage required for your store, you need to attach at least {} yoctoNEAR to this transaction.",
            STORE_BALANCE
        );
        self.assert_no_store_with_id(prefix.clone());
        self.assert_no_store_with_id(prefix.clone());
        assert_ne!(prefix.clone(), "market", "cannot use name for store, choose another name");
        assert_ne!(prefix.clone(), "pipar", "cannot use name for store, choose another name");
        assert_ne!(prefix.clone(), "dao", "cannot use name for store, choose another name");
        assert_ne!(prefix.clone(), "auction", "cannot use name for store, choose another name");
        let current_account = env::current_account_id().to_string();
        let subaccount: AccountId = format!("{prefix}.{current_account}").parse().unwrap();
        let init_args = serde_json::to_vec(&FtData {
            owner_id: env::signer_account_id(),
            marketplace_contract_id: env::current_account_id(),
            name,
            symbol,
            icon,
            bg_icon,
            category,
            description,
            facebook,
            twitter,
            instagram,
            tiktok,
            youtube,
            zip,
            city,
            state,
            country,
        })
        .unwrap();

        Promise::new(subaccount.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(STORE_BALANCE)
            .deploy_contract(include_bytes!("../wasm/store.wasm").to_vec())
            .function_call("new_default_meta".to_owned(), init_args, NO_DEPOSIT, PGAS)
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
    pub fn keypom_buy(
        &mut self,
        product_id: U64,
        store_contract_id: AccountId,
        color: String,
        timeout: U128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
        affiliate: Option<AccountId>,
        keypom_id: AccountId,
    ) -> Promise {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.product_id == product_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_id == keypom_id
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => panic!("Cannot escrow buy twice on the same product with the same seller, you must complete one first: {:?}", t),
            None => {
                let args = serde_json::to_vec(&Buy {
                    id: product_id.clone(),
                    receiver_id: keypom_id.clone(),
                    attached_deposit: env::attached_deposit().into(),
                    color: color,
                    affiliate: affiliate,
                })
                    .unwrap();
                Promise::new(store_contract_id.clone())
                    .function_call("nft_mint".to_owned(), args, ONE_YOCTO, PGAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .buy_callback(
                                keypom_id.clone(),
                                U128::from(env::attached_deposit()),
                                product_id.clone(),
                                store_contract_id,
                                timeout,
                                is_discount,
                                is_reward,
                                true,
                                hashed_billing_address,
                                nonce,
                            )
                    )
            }
        }
    }

    #[payable]
    pub fn buy(
        &mut self,
        product_id: U64,
        store_contract_id: AccountId,
        color: String,
        timeout: U128,
        is_discount: bool,
        is_reward: bool,
        hashed_billing_address: String,
        nonce: String,
        affiliate: Option<AccountId>
    ) -> Promise {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.product_id == product_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_id == env::predecessor_account_id()
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => panic!("Cannot escrow buy twice on the same product with the same seller, you must complete one first: {:?}", t),
            None => {
                let args = serde_json::to_vec(&Buy {
                    id: product_id.clone(),
                    receiver_id: env::predecessor_account_id(),
                    attached_deposit: env::attached_deposit().into(),
                    color: color,
                    affiliate: affiliate,
                })
                    .unwrap();
                Promise::new(store_contract_id.clone())
                    .function_call("nft_mint".to_owned(), args, ONE_YOCTO, PGAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .buy_callback(
                                env::predecessor_account_id(),
                                U128::from(env::attached_deposit()),
                                product_id.clone(),
                                store_contract_id,
                                timeout,
                                is_discount,
                                is_reward,
                                false,
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
        product_id: U64,
        store_contract_id: AccountId,
        timeout: U128,
        is_discount: bool,
        is_reward: bool,
        is_keypom: bool,
        hashed_billing_address: String,
        nonce: String,
    ) -> Option<MarketplaceData> {
        let attached_deposit: u128 = attached_deposit.into();
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!();
        },
            PromiseResult::Successful(value) => {
                if let Ok(result) = serde_json::from_slice::<MarketplaceData>(&value) {
                    self.transactions.push(&Transaction {
                        transaction_id: U128::from(env::block_timestamp() as u128),
                        product_id: product_id,
                        store_contract_id,
                        buyer_id: buyer_account_id,
                        buyer_value_locked: attached_deposit.into(),
                        price: result.price,
                        token_id: result.token_id,
                        timeout,
                        affiliate: result.affiliate,
                        affiliate_id: result.affiliate_id,
                        affiliate_percentage: result.affiliate_percentage,
                        is_discount,
                        is_reward,
                        is_keypom,
                        status: TransactionStatus::Approved,
                        hashed_billing_address,
                        nonce,
                        ipfs: String::from(""),
                    });
                    env::log_str("Successfully purchased product");
                    Some(result.clone())
                } else {
                    env::log_str("The batch call failed and all calls got reverted");
                    None
                }
            },
            PromiseResult::Failed => {
                Promise::new(buyer_account_id).transfer(attached_deposit);
                None
            },
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
                    && t.buyer_id == env::predecessor_account_id()
                    && t.status == TransactionStatus::Shipped
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                if t.is_reward == true {
                    let args = serde_json::to_vec(&TokenData {
                        id: t.product_id,
                        receiver_id: t.buyer_id,
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
                            .complete_purchase_callback(check_existing.clone() as u64),
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
                            buyer_id: t.buyer_id.clone(),
                            buyer_value_locked: t.buyer_value_locked,
                            price: t.price,
                            token_id: t.token_id.clone(),
                            timeout: t.timeout,
                            affiliate: t.affiliate.clone(),
                            affiliate_id: t.affiliate_id.clone(),
                            affiliate_percentage: t.affiliate_percentage.clone(),
                            is_discount: t.is_discount,
                            is_reward: t.is_reward,
                            is_keypom: t.is_keypom,
                            status: TransactionStatus::Delivered,
                            hashed_billing_address: t.hashed_billing_address,
                            nonce: t.nonce,
                            ipfs: t.ipfs
                        },
                    );
                    let payout: u128 = t.buyer_value_locked.into();
                    let percent = payout as f64 * 0.98;
                    let seller_funds = percent as u128;
                    if let Some(affix) = t.affiliate_id.clone() {
                        if t.affiliate == true {
                            let args = serde_json::to_vec(&Token {
                                token_id: t.token_id.clone(),
                            })
                                .unwrap();
                            let percentage = t.affiliate_percentage.unwrap_or(0);
                            let affiliate_payout = seller_funds as f64 / 100.0 * percentage as f64;
                            Promise::new(t.store_contract_id.clone()).transfer(seller_funds - affiliate_payout as Balance)
                                .function_call("unlock_token".to_owned(), args, NO_DEPOSIT, PGAS)
                                .then(
                                    Promise::new(affix).transfer(affiliate_payout as Balance)
                                );
                        } else {
                            Promise::new(t.store_contract_id.clone()).transfer(seller_funds);
                        }
                    }
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
                    && t.buyer_id == env::predecessor_account_id()
                    && t.status == TransactionStatus::Shipped
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
                        buyer_id: t.buyer_id,
                        buyer_value_locked: t.buyer_value_locked,
                        price: t.price,
                        token_id: t.token_id,
                        timeout: t.timeout,
                        affiliate: t.affiliate,
                        affiliate_id: t.affiliate_id,
                        affiliate_percentage: t.affiliate_percentage,
                        is_discount: t.is_discount,
                        is_reward: t.is_reward,
                        is_keypom: t.is_keypom,
                        status: TransactionStatus::Disputed,
                        hashed_billing_address: t.hashed_billing_address,
                        nonce: t.nonce,
                        ipfs: t.ipfs
                    },
                );
                env::log_str("Transaction has been marked disputed")
            }
            None => panic!("Transaction not found"),
        }
    }

    pub fn mark_shipped(&mut self, transaction_id: U128, buyer_id: AccountId, store_contract_id: AccountId, ipfs: String) -> Promise {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_id == buyer_id
                    && t.status == TransactionStatus::Approved
            })
            .unwrap_or_else(|| 11111111);

        match self.transactions.get(check_existing as u64) {
            Some(t) => {
                let args = serde_json::to_vec(&EmptyData {})
                    .unwrap();
                Promise::new(t.store_contract_id.clone())
                    .function_call("assert_store_owner".to_owned(), args, NO_DEPOSIT, PGAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .mark_shipped_callback(check_existing.clone() as u64, ipfs),
                    )
            }
            None => panic!("Transaction not found"),
        }
    }

    #[private]
    pub fn mark_shipped_callback(&mut self, check_existing: u64, ipfs: String) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                let result: bool = serde_json::from_slice::<bool>(&*val).unwrap();
                if result {
                    match self.transactions.get(check_existing.clone() as u64) {
                        Some(t) => {
                            self.transactions.replace(
                                check_existing as u64,
                                &Transaction {
                                    transaction_id: t.transaction_id,
                                    product_id: t.product_id,
                                    store_contract_id: t.store_contract_id,
                                    buyer_id: t.buyer_id,
                                    buyer_value_locked: t.buyer_value_locked,
                                    price: t.price,
                                    token_id: t.token_id,
                                    timeout: t.timeout,
                                    affiliate: t.affiliate,
                                    affiliate_id: t.affiliate_id,
                                    affiliate_percentage: t.affiliate_percentage,
                                    is_discount: t.is_discount,
                                    is_reward: t.is_reward,
                                    is_keypom: t.is_keypom,
                                    status: TransactionStatus::Shipped,
                                    hashed_billing_address: t.hashed_billing_address,
                                    nonce: t.nonce,
                                    ipfs
                                },
                            );
                            env::log_str("Transaction has been marked shipped")
                        }
                        None => panic!("Transaction not found"),
                    }
                } else {
                    env::panic_str("Product Marked shipped failed, please try again")
                }
            },
            PromiseResult::Failed => env::panic_str("Product Marked shipped failed, please try again"),
        }
    }

    pub fn get_refund(&mut self, transaction_id: U128, store_contract_id: AccountId) {
        let check_existing = self
            .transactions
            .iter()
            .position(|t| {
                t.transaction_id == transaction_id
                    && t.store_contract_id == store_contract_id
                    && t.buyer_id == env::predecessor_account_id()
                    && t.status == TransactionStatus::Approved
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
