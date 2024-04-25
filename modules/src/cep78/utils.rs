#![allow(dead_code)]
use odra::{args::Maybe, prelude::*, Address, Var};

#[odra::module]
struct MockDummyContract;

#[odra::module]
impl MockDummyContract {}


#[odra::module]
pub struct MockTransferFilterContract {
    value: Var<u8>
}

#[odra::module]
impl MockTransferFilterContract {
    pub fn set_return_value(&mut self, return_value: u8) {
        self.value.set(return_value);
    }

    pub fn can_transfer(&self) -> u8 {
        self.value.get_or_default()
    }
}

#[odra::module]
struct MockContract {
    nft_contract: Var<Address>
}

#[odra::module]
impl MockContract {
    pub fn set_address(&mut self, nft_contract: &Address) {
        self.nft_contract.set(*nft_contract);
    }

    pub fn mint(
        &mut self,
        token_metadata: String,
        is_reverse_lookup_enabled: bool
    ) -> (String, Address, String) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        if is_reverse_lookup_enabled {
            NftContractContractRef::new(self.env(), nft_contract_address)
                .register_owner(Maybe::Some(self.env().self_address()));
        }

        NftContractContractRef::new(self.env(), nft_contract_address).mint(
            self.env().self_address(),
            token_metadata,
            Maybe::None
        )
    }

    pub fn mint_with_hash(
        &mut self,
        token_metadata: String,
        token_hash: String
    ) -> (String, Address, String) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address).mint(
            self.env().self_address(),
            token_metadata,
            Maybe::Some(token_hash)
        )
    }

    pub fn burn(&mut self, token_id: u64) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address)
            .burn(Maybe::Some(token_id), Maybe::None)
    }

    pub fn mint_for(
        &mut self,
        token_owner: Address,
        token_metadata: String
    ) -> (String, Address, String) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address).mint(
            token_owner,
            token_metadata,
            Maybe::None
        )
    }

    pub fn transfer(&mut self, token_id: u64, target: Address) -> (String, Address) {
        let address = self.env().self_address();
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address).transfer(
            Maybe::Some(token_id),
            Maybe::None,
            address,
            target
        )
    }
    pub fn transfer_from(
        &mut self,
        token_id: u64,
        source: Address,
        target: Address
    ) -> (String, Address) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address).transfer(
            Maybe::Some(token_id),
            Maybe::None,
            source,
            target
        )
    }

    pub fn approve(&mut self, spender: Address, token_id: u64) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address).approve(
            spender,
            Maybe::Some(token_id),
            Maybe::None
        )
    }

    pub fn revoke(&mut self, token_id: u64) {
        let nft_contract_address = self.nft_contract.get().unwrap();
        NftContractContractRef::new(self.env(), nft_contract_address)
            .revoke(Maybe::Some(token_id), Maybe::None)
    }
}

#[odra::external_contract]
trait NftContract {
    fn mint(
        &mut self,
        token_owner: Address,
        token_metadata: String,
        token_hash: Maybe<String>
    ) -> (String, Address, String);
    fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
    fn register_owner(&mut self, token_owner: Maybe<Address>) -> String;
    fn transfer(
        &mut self,
        token_id: Maybe<u64>,
        token_hash: Maybe<String>,
        source: Address,
        target: Address
    ) -> (String, Address);
    fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>);
    fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>);
}
