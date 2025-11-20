//! Validator-related methods.

use casper_client::get_auction_info;
use casper_types::system::auction::BidAddr;
use casper_types::system::auction::ValidatorBid;
use casper_types::{Key, PublicKey, StoredValue, U512};
use odra_core::prelude::*;

/// Validator-related methods implementation for CasperClient.
impl super::CasperClient {
    pub async fn get_validator(&self, index: usize) -> PublicKey {
        let auction_info = get_auction_info(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            None
        )
        .await;
        let auction_info = auction_info.unwrap_or_else(|e| {
            panic!(
                "Couldn't get auction info from node: {:?}, reason: {:?}",
                self.configuration.node_address(),
                e
            )
        });
        let auction_info = auction_info.result;
        let validator = auction_info
            .auction_state
            .era_validators()
            .nth(0)
            .unwrap_or_else(|| panic!("Couldn't get auction state",))
            .validator_weights()
            .nth(index)
            .unwrap_or_else(|| panic!("Validator index {} out of bounds", index))
            .public_key();
        validator.clone()
    }

    pub async fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        let purse_uref = match self.get_main_purse(&delegator).await {
            Ok(uref) => uref,
            Err(_) => return U512::zero()
        };
        let account_hash = validator.to_account_hash();
        let key = Key::BidAddr(BidAddr::DelegatedPurse {
            validator: account_hash,
            delegator: purse_uref.addr()
        });

        let stored_value = self.query_global_state_maybe(key, None).await;
        match stored_value {
            None => U512::zero(),
            Some(sv) => match sv {
                StoredValue::BidKind(bid_kind) => bid_kind.staked_amount().unwrap_or_default(),
                _ => {
                    panic!(
                        "Couldn't get delegated amount for address: {:?}",
                        delegator.to_formatted_string()
                    )
                }
            }
        }
    }

    pub async fn get_validator_info(&self, _validator: PublicKey) -> Option<ValidatorBid> {
        todo!("Implement get_validator_info")
    }

    pub async fn auction_delay(&self) -> u64 {
        let chainspec = self.chainspec().await;

        let auction_delay = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get auction from chainspec"))
            .get("auction_delay")
            .unwrap_or_else(|| {
                panic!("Couldn't get era_delay from chainspec");
            });

        let era_duration = Self::era_duration(&chainspec)
            .unwrap_or_else(|e| panic!("Failed to get era_duration: {}", e));

        let auction_delay_int = auction_delay.as_integer().unwrap_or_else(|| {
            panic!(
                "auction_delay is not an integer in chainspec: {:?}",
                auction_delay
            )
        });
        era_duration * auction_delay_int as u64
    }

    pub async fn unbonding_delay(&self) -> u64 {
        let chainspec = self.chainspec().await;

        let unbonding_delay = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get core from chainspec"))
            .get("unbonding_delay")
            .unwrap_or_else(|| {
                panic!("Couldn't get unbonding_delay from chainspec");
            });

        let era_duration = Self::era_duration(&chainspec)
            .unwrap_or_else(|e| panic!("Failed to get era_duration: {}", e));

        let unbonding_delay_int = unbonding_delay.as_integer().unwrap_or_else(|| {
            panic!(
                "unbonding_delay is not an integer in chainspec: {:?}",
                unbonding_delay
            )
        });
        unbonding_delay_int as u64 * era_duration
    }
}
