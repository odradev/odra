use odra::{casper_types::U256, prelude::*};
use odra_modules::cep18_token::Cep18;

/// A ballot cast by a voter.
#[odra::odra_type]
struct Ballot {
    voter: Address,
    choice: bool,
    amount: U256
}

/// Errors for the governed token.
#[odra::odra_error]
pub enum GovernanceError {
    /// The vote is already in progress.
    VoteAlreadyOpen = 0,
    /// No vote is in progress.
    NoVoteInProgress = 1,
    /// Cannot tally votes yet.
    VoteNotYetEnded = 2,
    /// Vote ended
    VoteEnded = 3,
    /// Only the token holders can propose a new mint.
    OnlyTokenHoldersCanPropose = 4
}

/// A module definition. Each module struct consists of Vars and Mappings
/// or/and other modules.
#[odra::module]
pub struct OurToken {
    /// A submodule that implements the CEP-18 token standard.
    token: SubModule<Cep18>,
    /// The proposed mint.
    proposed_mint: Var<(Address, U256)>,
    /// The list of votes cast in the current vote.
    votes: List<Ballot>,
    /// Whether a vote is open.
    is_vote_open: Var<bool>,
    /// The time when the vote ends.
    vote_end_time: Var<u64>
}
/// Module implementation.
///
/// To generate entrypoints,
/// an implementation block must be marked as #[odra::module].
#[odra::module]
impl OurToken {
    /// Initializes the contract with the given metadata and initial supply.
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        // We put the token address as an admin, so it can govern itself. Self-governing token!
        self.token
            .init(symbol, name, decimals, initial_supply, vec![], vec![], None);
    }

    // Delegate all Cep18 functions to the token submodule.
    delegate! {
        to self.token {
            /// Admin EntryPoint to manipulate the security access granted to users.
            /// One user can only possess one access group badge.
            /// Change strength: None > Admin > Minter
            /// Change strength meaning by example: If a user is added to both Minter and Admin, they will be an
            /// Admin, also if a user is added to Admin and None then they will be removed from having rights.
            /// Beware: do not remove the last Admin because that will lock out all admin functionality.
            fn change_security(
                &mut self,
                admin_list: Vec<Address>,
                minter_list: Vec<Address>,
                none_list: Vec<Address>
            );

            /// Returns the name of the token.
            fn name(&self) -> String;

            /// Returns the symbol of the token.
            fn symbol(&self) -> String;

            /// Returns the number of decimals the token uses.
            fn decimals(&self) -> u8;

            /// Returns the total supply of the token.
            fn total_supply(&self) -> U256;

            /// Returns the balance of the given address.
            fn balance_of(&self, address: &Address) -> U256;

            /// Returns the amount of tokens the owner has allowed the spender to spend.
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;

            /// Approves the spender to spend the given amount of tokens on behalf of the caller.
            fn approve(&mut self, spender: &Address, amount: &U256);

            /// Decreases the allowance of the spender by the given amount.
            fn decrease_allowance(&mut self, spender: &Address, decr_by: &U256);

            /// Increases the allowance of the spender by the given amount.
            fn increase_allowance(&mut self, spender: &Address, inc_by: &U256);

            /// Transfers tokens from the caller to the recipient.
            fn transfer(&mut self, recipient: &Address, amount: &U256);

            /// Transfers tokens from the owner to the recipient using the spender's allowance.
            fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);

            /// Mints new tokens and assigns them to the given address.
            fn mint(&mut self, owner: &Address, amount: &U256);

            /// Burns the given amount of tokens from the given address.
            fn burn(&mut self, owner: &Address, amount: &U256);
        }
    }

    /// Proposes a new mint for the contract.
    pub fn propose_new_mint(&mut self, account: Address, amount: U256) {
        // Only allow proposing a new mint if there is no vote in progress.
        if self.is_vote_open.get_or_default() {
            self.env().revert(GovernanceError::VoteAlreadyOpen);
        }

        // Only the token holders can propose a new mint.
        if self.balance_of(&self.env().caller()) == U256::zero() {
            self.env()
                .revert(GovernanceError::OnlyTokenHoldersCanPropose);
        }

        // Set the proposed mint.
        self.proposed_mint.set((account, amount));
        // Open a vote.
        self.is_vote_open.set(true);
        // Set the vote end time to 10 minutes from now.
        self.vote_end_time
            .set(self.env().get_block_time() + 10 * 60 * 1000);
    }

    /// Votes on the proposed mint.
    pub fn vote(&mut self, choice: bool, amount: U256) {
        // Only allow voting if there is a vote in progress.
        self.assert_vote_in_progress();

        let voter = self.env().caller();
        let contract = self.env().self_address();

        // Transfer the voting tokens from the voter to the contract.
        self.token.transfer(&contract, &amount);

        // Add the vote to the list.
        self.votes.push(Ballot {
            voter,
            choice,
            amount
        });
    }

    /// Count the votes and perform the action
    pub fn tally(&mut self) {
        // Only allow tallying the votes once.
        if !self.is_vote_open.get_or_default() {
            self.env().revert(GovernanceError::NoVoteInProgress);
        }

        // Only allow tallying the votes after the vote has ended.
        let finish_time = self
            .vote_end_time
            .get_or_revert_with(GovernanceError::NoVoteInProgress);
        if self.env().get_block_time() < finish_time {
            self.env().revert(GovernanceError::VoteNotYetEnded);
        }

        // Count the votes
        let mut yes_votes = U256::zero();
        let mut no_votes = U256::zero();

        let contract = self.env().self_address();

        while let Some(vote) = self.votes.pop() {
            if vote.choice {
                yes_votes += vote.amount;
            } else {
                no_votes += vote.amount;
            }

            // Transfer back the voting tokens to the voter.
            self.token
                .raw_transfer(&contract, &vote.voter, &vote.amount);
        }

        // Perform the action if the vote has passed.
        if yes_votes > no_votes {
            let (account, amount) = self
                .proposed_mint
                .get_or_revert_with(GovernanceError::NoVoteInProgress);
            self.token.raw_mint(&account, &amount);
        }

        // Close the vote.
        self.is_vote_open.set(false);
    }

    fn assert_vote_in_progress(&self) {
        if !self.is_vote_open.get_or_default() {
            self.env().revert(GovernanceError::NoVoteInProgress);
        }

        let finish_time = self
            .vote_end_time
            .get_or_revert_with(GovernanceError::NoVoteInProgress);

        if self.env().get_block_time() > finish_time {
            self.env().revert(GovernanceError::VoteEnded);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn it_works() {
        let env = odra_test::env();
        let init_args = OurTokenInitArgs {
            name: "OurToken".to_string(),
            symbol: "OT".to_string(),
            decimals: 0,
            initial_supply: U256::from(1_000u64)
        };

        let mut token = OurToken::deploy(&env, init_args);

        // The deployer, as the only token holder,
        // starts a new voting to mint 1000 tokens to account 1.
        // There is only 1 token holder, so there is one Ballot cast.
        token.propose_new_mint(env.get_account(1), U256::from(2000));
        token.vote(true, U256::from(1000));

        // The tokens should now be staked.
        assert_eq!(token.balance_of(&env.get_account(0)), U256::zero());

        // Wait for the vote to end.
        env.advance_block_time(60 * 11 * 1000);

        // Finish the vote.
        token.tally();

        // The tokens should now be minted.
        assert_eq!(token.balance_of(&env.get_account(1)), U256::from(2000));
        assert_eq!(token.total_supply(), 3000.into());

        // The stake should be returned.
        assert_eq!(token.balance_of(&env.get_account(0)), U256::from(1000));

        // Now account 1 can mint new tokens with their voting power...
        env.set_caller(env.get_account(1));
        token.propose_new_mint(env.get_account(1), U256::from(2000));
        token.vote(true, U256::from(2000));

        // ...Even if the deployer votes against it.
        env.set_caller(env.get_account(0));
        token.vote(false, U256::from(1000));

        env.advance_block_time(60 * 11 * 1000);

        token.tally();

        // The power of community governance!
        assert_eq!(token.balance_of(&env.get_account(1)), U256::from(4000));
    }
}
