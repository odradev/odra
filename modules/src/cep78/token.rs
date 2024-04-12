use odra::{args::Maybe, casper_types::{bytesrepr::ToBytes, ContractHash, ContractPackage, Key, URef}, prelude::*, Address, Mapping, Sequence, SubModule, UnwrapOrRevert, Var};
use super::{constants, error::CEP78Error, events::{Approval, ApprovalForAll, ApprovalRevoked, Burn, MetadataUpdated, Mint, RevokedForAll, VariablesSet}, metadata::Metadata, modalities::{BurnMode, EventsMode, MintingMode, NFTIdentifierMode, OwnershipMode, TokenIdentifier, TransferFilterContractResult}, utils::{self, GetAs}, whitelist::ACLWhitelist};



#[odra::module(
    //package_hash_key = "cep78",
    //access_key = "cep78",
)]
pub struct CEP78 {
    installer: Var<Address>,
    collection_name: Var<String>,
    collection_symbol: Var<String>,
    cap: Var<u64>,
    allow_minting: Var<bool>,
    minting_mode: Var<u8>,
    ownership_mode: Var<u8>,
    nft_kind: Var<u8>,
    holder_mode: Var<u8>,
    package_operator_mode: Var<bool>,
    operator_burn_mode: Var<bool>,
    burn_mode: Var<u8>,
    events_mode: Var<u8>,
    whitelist_mode: Var<u8>,
    counter: Sequence<u64>,
    whitelist: SubModule<ACLWhitelist>,
    metadata: SubModule<Metadata>,
    owners: Mapping<String, Address>,
    issuers: Mapping<String, Address>,
    approved: Mapping<String, Option<Address>>,
    token_count: Mapping<Address, u64>,
    burnt_tokens: Mapping<String, ()>,
    operators: Mapping<(Address, Address), bool>
}

#[odra::module]
impl CEP78 {
    /// Initializes the module.
    pub fn init(
        &mut self, 
        collection_name: String, 
        collection_symbol: String, 
        total_token_supply: u64, 
        allow_minting: Maybe<bool>,
        minting_mode: Maybe<u8>,
        ownership_mode: u8,
        nft_kind: u8,
        holder_mode: Maybe<u8>,
        whitelist_mode: Maybe<u8>,
        acl_white_list: Maybe<Vec<Address>>,
        acl_package_mode: Maybe<bool>,
        package_operator_mode: Maybe<bool>,
        json_schema: Maybe<String>,
        receipt_name: Maybe<String>,
        identifier_mode: u8,
        burn_mode: Maybe<u8>,
        operator_burn_mode: Maybe<bool>,
        nft_metadata_kind: u8,
        metadata_mutability: u8,
        owner_reverse_lookup_mode: Maybe<u8>,
        events_mode: u8,
        transfer_filter_contract_contract_key: Maybe<Address>,

        additional_required_metadata: Maybe<Vec<u8>>,
        optional_metadata: Maybe<Vec<u8>>,
    ) {
        let installer = self.env().caller();
        self.installer.set(installer);
        self.collection_name.set(collection_name);
        self.collection_symbol.set(collection_symbol);

        if total_token_supply == 0 {
            self.env().revert(CEP78Error::CannotInstallWithZeroSupply)

        }

        if total_token_supply > constants::MAX_TOTAL_TOKEN_SUPPLY {
            self.env().revert(CEP78Error::ExceededMaxTotalSupply)
        }

        self.cap.set(total_token_supply);
        self.allow_minting.set(allow_minting.unwrap_or(true));
        self.minting_mode.set(minting_mode.clone().unwrap_or(0));
        self.ownership_mode.set(ownership_mode);
        self.nft_kind.set(nft_kind);
        self.holder_mode.set(holder_mode.unwrap_or(2u8));
        self.burn_mode.set(burn_mode.unwrap_or(0));
        self.operator_burn_mode.set(operator_burn_mode.unwrap_or_default());

        self.whitelist.init(
            acl_white_list.unwrap_or_default(),
            whitelist_mode.unwrap_or_default(), 
            acl_package_mode.unwrap_or_default()
        );

       
        // Deprecated in 1.4 in favor of following acl whitelist
        // A whitelist of keys specifying which entity can mint
        // NFTs in the contract holder mode with restricted minting.
        // This value can only be modified if the whitelist lock is
        // set to be unlocked.
        // let contract_white_list: Vec<ContractHash> = utils::get_optional_named_arg_with_user_errors(
        //     ARG_CONTRACT_WHITELIST,
        //     NFTCoreError::InvalidContractWhitelist,
        // )
        // .unwrap_or_default();
    
        self.package_operator_mode.set(package_operator_mode.unwrap_or_default());
        self.metadata.init(nft_metadata_kind, additional_required_metadata, optional_metadata, metadata_mutability, identifier_mode, json_schema);

    
        if identifier_mode == 1 && metadata_mutability == 1 {
            self.env().revert(CEP78Error::InvalidMetadataMutability)
        }

        if ownership_mode == 0 && minting_mode.unwrap_or_default() == 0 && owner_reverse_lookup_mode.unwrap_or_default() == 1 {
            self.env().revert(CEP78Error::InvalidReportingMode)
        }
    
        self.counter.next_value();
        
    }

    /// Exposes all variables that can be changed by managing account post
    /// installation. Meant to be called by the managing account (INSTALLER) post
    /// installation if a variable needs to be changed.
    /// By switching allow_minting to false we pause minting.
    pub fn set_variables(
        &mut self, 
        allow_minting: Maybe<bool>,
        contract_whitelist: Maybe<Vec<ContractHash>>,
        acl_whitelist: Maybe<Vec<Address>>,
        acl_package_mode: Maybe<bool>,
        package_operator_mode: Maybe<bool>,
        operator_burn_mode: Maybe<bool>,
    ) {
        let installer = self.installer.get_or_revert_with(CEP78Error::MissingInstaller);
        
        // Only the installing account can change the mutable variables.
        self.ensure_not_caller(installer);
    
        if let Maybe::Some(allow_minting) = allow_minting {
            self.allow_minting.set(allow_minting);
        }
    
        self.whitelist.update_package_mode(acl_package_mode);
        self.whitelist.update_addresses(acl_whitelist, contract_whitelist);
    
        if let Maybe::Some(package_operator_mode) = package_operator_mode {
            self.package_operator_mode.set(package_operator_mode);
        }
    
        if let Maybe::Some(operator_burn_mode) = operator_burn_mode {
            self.operator_burn_mode.set(operator_burn_mode);
        }

        self.emit_ces_event(VariablesSet::new());
    }


    /// Mints a new token with provided metadata.
    /// Reverts with [CEP78Error::MintingIsPaused] error if `allow_minting` is false.
    /// When a token is minted the calling account is listed as its owner and the token is
    /// automatically assigned an `u64` ID equal to the current `number_of_minted_tokens`.
    /// Before minting the token, checks if `number_of_minted_tokens`
    /// exceeds the `total_token_supply`. If so, it reverts the minting with an error
    /// [CEP78Error::TokenSupplyDepleted]. The `mint` function also checks whether the calling account
    /// is the managing account (the installer) If not, and if `public_minting` is set to
    /// false, it reverts with the error [CEP78Error::InvalidAccount]. 
    /// After minting is successful the number_of_minted_tokens is incremented by one.
    pub fn mint(
        &mut self,
        token_owner: Address,
        token_metadata: String,
        token_hash: Maybe<String>,
    ) -> (String, Address, String) {
        // The contract owner can toggle the minting behavior on and off over time.
        // The contract is toggled on by default.
        let allow_minting = self.allow_minting.get_or_default();

        // If contract minting behavior is currently toggled off we revert.
        if !allow_minting {
            self.env().revert(CEP78Error::MintingIsPaused);
        }

        let total_token_supply = self.cap.get_or_revert_with(CEP78Error::MissingTotalTokenSupply);

        // The minted_tokens_count is the number of minted tokens so far.
        let minted_tokens_count = self.counter.get_current_value();

        // Revert if the token supply has been exhausted.
        if minted_tokens_count >= total_token_supply {
            self.env().revert(CEP78Error::TokenSupplyDepleted);
        }

        let minting_mode: MintingMode = self.minting_mode.get_as(&self.env());

        // let (caller, contract_package): (Key, Option<Key>) =
        //     match self.env().caller() {
        //         Caller::Session(account_hash) => (account_hash.into(), None),
        //         Caller::StoredCaller(contract_hash, contract_package_hash) => {
        //             (contract_hash.into(), Some(contract_package_hash.into()))
        //         }
        //     };

        let (caller, contract_package) = (self.env().caller(), None::<ContractPackage>);

        // Revert if minting is private and caller is not installer.
        if MintingMode::Installer == minting_mode {
            match caller {
                Address::Account(_) => {
                    let installer_account = self.installer.get_or_revert_with(CEP78Error::MissingInstaller);
                    // Revert if private minting is required and caller is not installer.
                    if caller != installer_account {
                        self.env().revert(CEP78Error::InvalidMinter)
                    }
                }
                _ => self.env().revert(CEP78Error::InvalidKey),
            }
        }

        // Revert if minting is acl and caller is not whitelisted.
        if MintingMode::Acl == minting_mode {
            // TODO: Implement the following
            // let acl_package_mode: bool = self.whitelist.is_package_mode();
            // let is_whitelisted = match (acl_package_mode, contract_package) {
            //     (true, Some(contract_package)) => utils::get_dictionary_value_from_key::<bool>(
            //         ACL_WHITELIST,
            //         &utils::encode_dictionary_item_key(contract_package),
            //     )
            //     .unwrap_or_default(),
            //     _ => utils::get_dictionary_value_from_key::<bool>(
            //         ACL_WHITELIST,
            //         &utils::encode_dictionary_item_key(caller),
            //     )
            //     .unwrap_or_default(),
            // };
            let is_whitelisted = false;

            match caller {
                Address::Contract(_) => {
                    if !is_whitelisted {
                        self.env().revert(CEP78Error::UnlistedContractHash);
                    }
                }
                Address::Account(_) => {
                    if !is_whitelisted {
                        self.env().revert(CEP78Error::InvalidMinter);
                    }
                }
            }
        }

        let identifier_mode = self.metadata.get_identifier_mode();

        let optional_token_hash: String = token_hash.unwrap_or_default();
        let token_identifier: TokenIdentifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(minted_tokens_count),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(if optional_token_hash.is_empty() {
                // TODO: Implement the following
                // base16::encode_lower(&runtime::blake2b(token_metadata.clone()))
                "".to_string()
            } else {
                optional_token_hash
            }),
        };

        self.metadata.update_or_revert(&token_metadata, &token_identifier);


        // The contract's ownership behavior (determined at installation) determines,
        // who owns the NFT we are about to mint.()
        let ownership_mode: OwnershipMode = self.ownership_mode.get_as(&self.env());
        let token_owner_key = if let OwnershipMode::Assigned | OwnershipMode::Transferable = ownership_mode {
            token_owner
        } else {
            caller
        };

        let id = token_identifier.to_string();
        self.owners.set(&id, token_owner_key);
        self.issuers.set(&id, caller);

        // TODO: Implement the following
        // if let NFTIdentifierMode::Hash = identifier_mode {
        //     // Update the forward and reverse trackers
        //     utils::insert_hash_id_lookups(minted_tokens_count, token_identifier.clone());
        // }

        //Increment the count of owned tokens.
        self.token_count.add(&token_owner_key, 1);
            
        // Increment number_of_minted_tokens by one
        self.counter.next_value();


        // Emit Mint event.
        self.emit_ces_event(Mint::new(
            token_owner_key,
            token_identifier.clone(),
            token_metadata.clone(),
        ));

        // TODO: Implement the following
        // if let OwnerReverseLookupMode::Complete = utils::get_reporting_mode() {
        //     if (NFTIdentifierMode::Hash == identifier_mode)
        //         && runtime::get_key(OWNED_TOKENS).is_some()
        //         && utils::should_migrate_token_hashes(token_owner_key)
        //     {
        //         utils::migrate_token_hashes(token_owner_key)
        //     }

        //     let (page_table_entry, page_uref) = utils::add_page_entry_and_page_record(
        //         minted_tokens_count,
        //         &owned_tokens_item_key,
        //         true,
        //     );

        //     let receipt_string = utils::get_receipt_name(page_table_entry);
        //     let receipt_address = Key::dictionary(page_uref, owned_tokens_item_key.as_bytes());
        //     let token_identifier_string = token_identifier.get_dictionary_item_key();

        //     (receipt_string, receipt_address, token_identifier_string)
        // }
        (id, token_owner_key, token_metadata)
    }


    /// Burns the token with provided `token_id` argument, after which it is no
    /// longer possible to transfer it.
    /// Looks up the owner of the supplied token_id arg. If caller is not owner we revert with
    /// error [CEP78Error::InvalidTokenOwner]. If the token id is invalid (e.g. out of bounds) it reverts
    /// with error [CEP78Error::InvalidTokenIdentifier]. If the token is listed as already burnt we revert with
    /// error [CEP78Error::PreviouslyBurntToken]. If not the token is then registered as burnt.
    pub fn burn(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        self.ensure_burnable();
    
        let token_identifier = self.token_identifier(token_id, token_hash);
    
        let (caller, contract_package): (Address, Option<Address>) = (self.env().caller(), None);
            // match utils::get_verified_caller().unwrap_or_revert() {
            //     Caller::Session(account_hash) => (account_hash.into(), None),
            //     Caller::StoredCaller(contract_hash, contract_package_hash) => {
            //         (contract_hash.into(), Some(contract_package_hash.into()))
            //     }
            // };
    
        let token_owner = self.owner_of_by_id(&token_identifier);
    
        // Check if caller is owner
        let is_owner = token_owner == caller;
    
        // Check if caller is operator to execute burn
        let is_operator = if !is_owner {
            self.operators.get_or_default(&(token_owner, caller))
        } else {
            false
        };
    
        // With operator package mode check if caller's package is operator to let contract execute burn
        let is_package_operator = if !is_owner && !is_operator {
            match (self.package_operator_mode.get_or_default(), contract_package) {
                (true, Some(contract_package)) => {
                    // TODO: Implement the following
                    // self.operators.get_or_default(&(token_owner, contract_package));
                    true
                }
                _ => false,
            }
        } else {
            false
        };
    
        // Revert if caller is not token_owner nor operator for the owner
        if !is_owner && !is_operator && !is_package_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner)
        };
    
        // It makes sense to keep this token as owned by the caller. It just happens that the caller
        // owns a burnt token. That's all. Similarly, we should probably also not change the
        // owned_tokens dictionary.
        self.ensure_not_burned(&token_identifier);
    
        // Mark the token as burnt by adding the token_id to the burnt tokens dictionary.
        self.burnt_tokens.set(&token_identifier.to_string(), ());
        self.token_count.subtract(&token_owner, 1);
    
 
        // Emit Burn event.
        self.emit_ces_event(Burn::new(token_owner, token_identifier, caller));
    }

    /// Transfers ownership of the token from one account to another.
    /// It looks up the owner of the supplied token_id arg. Reverts if the token is already burnt,
    /// `token_id` is invalid, or if caller is not owner nor an approved account nor operator.
    /// If token id is invalid it reverts with error [CEP78Error::InvalidTokenIdentifier].
    pub fn transfer(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>, source_key: Address, target_key: Address) -> (String, Address) {
        // If we are in minter or assigned mode we are not allowed to transfer ownership of token, hence
        // we revert.
        self.ensure_minter_or_assigned();
       

        let token_identifier = self.checked_token_identifier(token_id, token_hash);

        // We assume we cannot transfer burnt tokens
        self.ensure_not_burned(&token_identifier);


        self.ensure_not_owner(&token_identifier, &source_key);

        let (caller, contract_package): (Address, Option<Key>) = (self.env().caller(), None);
            
        let owner = self.owner_of_by_id(&token_identifier);
        // Check if caller is owner
        let is_owner = owner == caller;

        // Check if caller is approved to execute transfer
        let is_approved = !is_owner
            && match self.approved.get(&token_identifier.to_string()) {
                Some(Some(maybe_approved)) => caller == maybe_approved,
                Some(None) | None => false,
            };

        // Check if caller is operator to execute transfer
        let is_operator = if !is_owner && !is_approved {
            self.operators.get_or_default(&(source_key, caller))
        } else {
            false
        };

        // With operator package mode check if caller's package is operator to let contract execute
        // transfer
        let is_package_operator = if !is_owner && !is_approved && !is_operator {
            match (self.package_operator_mode.get_or_default(), contract_package) {
                (true, Some(contract_package)) => {
                    // TODO: Implement the following
                    // self.operators.get_or_default(&(source_key, contract_package))
                    self.operators.get_or_default(&(source_key, caller))
                }
                _ => false,
            }
        } else {
            false
        };

        if let Some(filter_contract) = utils::get_transfer_filter_contract() {
            let result = TransferFilterContractContractRef::new(self.env(), filter_contract)
                .can_transfer(source_key, target_key, token_identifier);

            if TransferFilterContractResult::DenyTransfer == result {
                self.env().revert(CEP78Error::TransferFilterContractDenied);
            }
        }

        // Revert if caller is not owner nor approved nor an operator.
        if !is_owner && !is_approved && !is_operator && !is_package_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }


        // if NFTIdentifierMode::Hash == identifier_mode && runtime::get_key(OWNED_TOKENS).is_some() {
        //     if utils::should_migrate_token_hashes(source_key) {
        //         utils::migrate_token_hashes(source_key)
        //     }

        //     if utils::should_migrate_token_hashes(target_key) {
        //         utils::migrate_token_hashes(target_key)
        //     }
        // }

        // let target_owner_item_key = utils::encode_dictionary_item_key(target_owner_key);

        // // Updated token_owners dictionary. Revert if token_owner not found.
        // match utils::get_dictionary_value_from_key::<Key>(
        //     TOKEN_OWNERS,
        //     &token_identifier.get_dictionary_item_key(),
        // ) {
        //     Some(token_actual_owner) => {
        //         if token_actual_owner != source_owner_key {
        //             runtime::revert(NFTCoreError::InvalidTokenOwner)
        //         }
        //         utils::upsert_dictionary_value_from_key(
        //             TOKEN_OWNERS,
        //             &token_identifier.get_dictionary_item_key(),
        //             target_owner_key,
        //         );
        //     }
        //     None => runtime::revert(NFTCoreError::MissingOwnerTokenIdentifierKey),
        // }

        // let source_owner_item_key = utils::encode_dictionary_item_key(source_owner_key);

        // // Update the from_account balance
        // let updated_from_account_balance =
        //     match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &source_owner_item_key) {
        //         Some(balance) => {
        //             if balance > 0u64 {
        //                 balance - 1u64
        //             } else {
        //                 // This should never happen...
        //                 runtime::revert(NFTCoreError::FatalTokenIdDuplication);
        //             }
        //         }
        //         None => {
        //             // This should never happen...
        //             runtime::revert(NFTCoreError::FatalTokenIdDuplication);
        //         }
        //     };
        // utils::upsert_dictionary_value_from_key(
        //     TOKEN_COUNT,
        //     &source_owner_item_key,
        //     updated_from_account_balance,
        // );

        // // Update the to_account balance
        // let updated_to_account_balance =
        //     match utils::get_dictionary_value_from_key::<u64>(TOKEN_COUNT, &target_owner_item_key) {
        //         Some(balance) => balance + 1u64,
        //         None => 1u64,
        //     };

        // utils::upsert_dictionary_value_from_key(
        //     TOKEN_COUNT,
        //     &target_owner_item_key,
        //     updated_to_account_balance,
        // );

        // utils::upsert_dictionary_value_from_key(
        //     APPROVED,
        //     &token_identifier.get_dictionary_item_key(),
        //     Option::<Key>::None,
        // );

        // let events_mode = EventsMode::try_from(utils::get_stored_value_with_user_errors::<u8>(
        //     EVENTS_MODE,
        //     NFTCoreError::MissingEventsMode,
        //     NFTCoreError::InvalidEventsMode,
        // ))
        // .unwrap_or_revert();

        // match events_mode {
        //     EventsMode::NoEvents => {}
        //     EventsMode::CEP47 => record_cep47_event_dictionary(CEP47Event::Transfer {
        //         sender: caller,
        //         recipient: target_owner_key,
        //         token_id: token_identifier.clone(),
        //     }),
        //     EventsMode::CES => {
        //         // Emit Transfer event.
        //         let spender = if caller == owner { None } else { Some(caller) };
        //         casper_event_standard::emit(Transfer::new(
        //             owner,
        //             spender,
        //             target_owner_key,
        //             token_identifier.clone(),
        //         ));
        //     }
        // }

        // let reporting_mode = utils::get_reporting_mode();

        // if let OwnerReverseLookupMode::Complete | OwnerReverseLookupMode::TransfersOnly = reporting_mode
        // {
        //     // Update to_account owned_tokens. Revert if owned_tokens list is not found
        //     let tokens_count = utils::get_token_index(&token_identifier);
        //     if OwnerReverseLookupMode::TransfersOnly == reporting_mode {
        //         utils::add_page_entry_and_page_record(tokens_count, &source_owner_item_key, false);
        //     }

        //     let (page_table_entry, page_uref) = utils::update_page_entry_and_page_record(
        //         tokens_count,
        //         &source_owner_item_key,
        //         &target_owner_item_key,
        //     );

        //     let owned_tokens_actual_key = Key::dictionary(page_uref, source_owner_item_key.as_bytes());

        //     let receipt_string = utils::get_receipt_name(page_table_entry);

        //     let receipt = CLValue::from_t((receipt_string, owned_tokens_actual_key))
        //         .unwrap_or_revert_with(NFTCoreError::FailedToConvertToCLValue);
        //     runtime::ret(receipt)
        // }
        todo!()
    }

    /// Approves another token holder (an approved account) to transfer tokens. It
    /// reverts if token_id is invalid, if caller is not the owner nor operator, if token has already
    /// been burnt, or if caller tries to approve themselves as an approved account.
    pub fn approve(&mut self, spender: Address, token_id: Maybe<u64>, token_hash: Maybe<String>, operator: Maybe<Address>) {
        // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
        // revert.
        self.ensure_minter_or_assigned();
        
        let (caller, contract_package): (Address, Option<Key>) =
            (self.env().caller(), None);

        let token_identifier = self.checked_token_identifier(token_id, token_hash);

        let owner = self.owner_of_by_id(&token_identifier);

        // Revert if caller is not token owner nor operator.
        // Only the token owner or an operator can approve an account
        let is_owner = caller == owner;
        let is_operator = !is_owner && self.operators.get_or_default(&(owner, caller));

        let is_package_operator = if !is_owner && !is_operator {
            match (self.package_operator_mode.get_or_default(), contract_package) {
                (true, Some(contract_package)) => {
                    // TODO: Implement the following
                    // self.operators.get_or_default(&(owner, contract_package))
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        if !is_owner && !is_operator && !is_package_operator {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }

        // We assume a burnt token cannot be approved
        self.ensure_not_burned(&token_identifier);

        let spender = match operator {
            Maybe::Some(deprecated_operator) => deprecated_operator,
            Maybe::None => spender
        };

        // If token owner or operator tries to approve itself that's probably a mistake and we revert.
        self.ensure_not_caller(spender);
        self.approved.set(&token_identifier.to_string(), Some(spender));
        self.emit_ces_event(Approval::new(owner, spender, token_identifier));
    }

    /// Revokes an approved account to transfer tokens. It reverts
    /// if token_id is invalid, if caller is not the owner, if token has already
    /// been burnt, if caller tries to approve itself.
    pub fn revoke(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) {
        let env = self.env();
        // If we are in minter or assigned mode it makes no sense to approve an account. Hence we
        // revert.
        self.ensure_minter_or_assigned();

        let (caller, contract_package): (Address, Option<Key>) = (env.caller(), None);
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        
        // Revert if caller is not the token owner or an operator. Only the token owner / operators can
        // revoke an approved account
        let owner = self.owner_of_by_id(&token_identifier);
        let is_owner = caller == owner;
        let is_operator = !is_owner && self.operators.get_or_default(&(owner, caller));

        let is_package_operator = if !is_owner && !is_operator {
            match (
                self.package_operator_mode.get_or_default(),
                contract_package,
            ) {
                (true, Some(contract_package)) => {
                    // TODO: Implement the following
                    // self.operators.get_or_default(&(owner, contract_package))
                    true
                }
                _ => false,
            }
        } else {
            false
        };

        if !is_owner && !is_operator && !is_package_operator {
            env.revert(CEP78Error::InvalidTokenOwner);
        }

        // We assume a burnt token cannot be revoked
        self.ensure_not_burned(&token_identifier);
        self.approved.set(&token_identifier.to_string(), Option::<Address>::None);
        // Emit ApprovalRevoked event.
        self.emit_ces_event(ApprovalRevoked::new(owner, token_identifier));
    }

    /// Approves all tokens owned by the caller and future to another token holder
    /// (an operator) to transfer tokens. It reverts if token_id is invalid, if caller is not the
    /// owner, if caller tries to approve itself as an operator.
    pub fn set_approval_for_all(&mut self, approve_all: bool, operator: Address) {
        let env = self.env();
        // If we are in minter or assigned mode it makes no sense to approve an operator. Hence we
        // revert.
        self.ensure_minter_or_assigned();
        // If caller tries to approve itself as operator that's probably a mistake and we revert.
        self.ensure_not_caller(operator);

        let caller = env.caller();
        // Depending on approve_all we either approve all or disapprove all.
        self.operators.set(&(caller, operator), approve_all);

        let events_mode: EventsMode = self.events_mode.get_as(&env);
        if let EventsMode::CES = events_mode {
            if approve_all {
                env.emit_event(ApprovalForAll::new(caller, operator));
            } else {
                env.emit_event(RevokedForAll::new(caller, operator));
            }
        }
    }

    /// Returns if an account is operator for a token owner
    pub fn is_approved_for_all(&mut self, token_owner: Address, operator: Address) -> bool {    
        self.operators.get_or_default(&(token_owner, operator))
    }
    
    /// Returns the token owner given a token_id. It reverts if token_id
    /// is invalid. A burnt token still has an associated owner.
    pub fn owner_of(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Address {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.owner_of_by_id(&token_identifier)
    }
    
    /// Returns the approved account (if any) associated with the provided token_id
    /// Reverts if token has been burnt.
    pub fn get_approved(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> Option<Address> {
        let token_identifier: TokenIdentifier = self.checked_token_identifier(token_id, token_hash);
    
        self.ensure_not_burned(&token_identifier);
        self.approved.get(&token_identifier.to_string()).flatten()
    }
    
    /// Returns the metadata associated with the provided token_id
    pub fn metadata(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> String {
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.metadata.get_or_revert(&token_identifier)
    }

    /// Updates the metadata if valid.
    pub fn set_token_metadata(&mut self, token_id: Maybe<u64>, token_hash: Maybe<String>, updated_token_metadata: String) {
        self.metadata.ensure_mutability(CEP78Error::ForbiddenMetadataUpdate);
        
        let token_identifier = self.checked_token_identifier(token_id, token_hash);
        self.ensure_owner_not_caller(&token_identifier);
        self.metadata.update_or_revert(&updated_token_metadata, &token_identifier);
    
        self.emit_ces_event(MetadataUpdated::new(token_identifier, updated_token_metadata));
    }
    
    /// Returns number of owned tokens associated with the provided token holder
    pub fn balance_of(&mut self, token_owner: Address) -> u64 {
        self.token_count.get(&token_owner).unwrap_or_default()
    }

    /// This entrypoint will upgrade the contract from the 1_0 version to the
    /// 1_1 version. The contract will insert any addition dictionaries and
    /// sentinel values that were absent in the previous version of the contract.
    /// It will also perform the necessary data transformations of historical
    /// data if needed
    pub fn migrate(&mut self, nft_package_key: String) {
        todo!()
    }
    
    /// This entrypoint will allow NFT owners to update their receipts from
    /// the previous owned_tokens list model to the current pagination model
    /// scheme. Calling the entrypoint will return a list of receipt names
    /// alongside the dictionary addressed to the relevant pages.
    pub fn updated_receipts(&mut self) -> Vec<(String, Address)> {
        todo!()
    }

    /// This entrypoint allows users to register with a give CEP-78 instance,
    /// allocating the necessary page table to enable the reverse lookup
    /// functionality and allowing users to pay the upfront cost of allocation
    /// resulting in more stable gas costs when minting and transferring
    /// Note: This entrypoint MUST be invoked if the reverse lookup is enabled
    /// in order to own NFTs.
    pub fn register_owner(&mut self) -> (String, URef) {
        todo!()
    }
}


impl CEP78 {
    #[inline]
    fn is_minter_or_assigned(&self) -> bool {
        let ownership_mode: OwnershipMode = self.ownership_mode.get_as(&self.env());
        matches!(ownership_mode, OwnershipMode::Minter | OwnershipMode::Assigned)
    }

    #[inline]
    fn ensure_minter_or_assigned(&self) {
        if self.is_minter_or_assigned() {
            self.env().revert(CEP78Error::InvalidOwnershipMode)
        }
    }

    #[inline]
    fn token_identifier(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> TokenIdentifier {
        let env = self.env();
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&env)),
        }
    }

    #[inline]
    fn checked_token_identifier(&self, token_id: Maybe<u64>, token_hash: Maybe<String>) -> TokenIdentifier {
        let env = self.env();
        let identifier_mode: NFTIdentifierMode = self.metadata.get_identifier_mode();
        let token_identifier = match identifier_mode {
            NFTIdentifierMode::Ordinal => TokenIdentifier::Index(token_id.unwrap(&env)),
            NFTIdentifierMode::Hash => TokenIdentifier::Hash(token_hash.unwrap(&env)),
        };

        let number_of_minted_tokens = self.counter.get_current_value();
        if let NFTIdentifierMode::Ordinal = identifier_mode {
            // Revert if token_id is out of bounds
            if token_identifier.get_index().unwrap_or_revert(&env) >= number_of_minted_tokens {
                env.revert(CEP78Error::InvalidTokenIdentifier);
            }
        }
        token_identifier
    }

    #[inline]
    fn owner_of_by_id(&self, id: &TokenIdentifier) -> Address {
        match self.owners.get(&id.to_string()) {
            Some(token_owner) => token_owner,
            None => self.env().revert(CEP78Error::MissingOwnerTokenIdentifierKey),
        }
    }

    #[inline]
    fn is_token_burned(&self, token_identifier: &TokenIdentifier) -> bool {
        self.burnt_tokens.get(&token_identifier.to_string()).is_some()
    }


    #[inline]
    fn ensure_not_owner(&self, token_identifier: &TokenIdentifier, address: &Address) {
        let owner = self.owner_of_by_id(token_identifier);
        if address == &owner {
            self.env().revert(CEP78Error::InvalidAccount);
        }
    }
    
    #[inline]
    fn ensure_owner_not_caller(&self, token_identifier: &TokenIdentifier) {
        let owner = self.owner_of_by_id(token_identifier);
        if self.env().caller() == owner {
            self.env().revert(CEP78Error::InvalidTokenOwner);
        }
    }

    #[inline]
    fn ensure_not_burned(&self, token_identifier: &TokenIdentifier) {
        if self.is_token_burned(token_identifier) {
            self.env().revert(CEP78Error::PreviouslyBurntToken);
        }
    }

    #[inline]
    fn ensure_not_caller(&self, address: Address) {
        if self.env().caller() == address {
            self.env().revert(CEP78Error::InvalidAccount);
        }
    }

    #[inline]
    fn emit_ces_event<T: ToBytes>(&self, event: T) {
        let events_mode: EventsMode = self.events_mode.get_as(&self.env());
        if let EventsMode::CES = events_mode {
            self.env().emit_event(event);
        }
    }

    #[inline]
    fn ensure_burnable(&self) {
        if let BurnMode::NonBurnable = self.burn_mode.get_as(&self.env()) {
            self.env().revert(CEP78Error::InvalidBurnMode)
        }
    }
}


#[odra::external_contract]
pub trait TransferFilterContract {
    fn can_transfer(&self, source_key: Address, target_key: Address, token_id: TokenIdentifier) -> TransferFilterContractResult;
}