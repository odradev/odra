//! Deploys an [odra_examples::contracts::tlw::TimeLockWallet] contract, then deposits and withdraw some CSPRs.
use odra::casper_types::{AsymmetricType, PublicKey, U512};
use odra::host::{Deployer, HostRef};
use odra_examples::contracts::tlw::{TimeLockWallet, TimeLockWalletInitArgs};
use Address;

const DEPOSIT: u64 = 100;
const WITHDRAWAL: u64 = 99;
const GAS: u64 = 20u64.pow(9);

fn main() {
    let env = odra_casper_livenet_env::env();
    let caller = env.get_account(0);

    env.set_caller(caller);
    env.set_gas(GAS);

    let mut contract = TimeLockWallet::deploy(
        &env,
        TimeLockWalletInitArgs {
            lock_duration: 60 * 60
        }
    );

    contract.with_tokens(U512::from(DEPOSIT)).deposit();

    println!("Owner's balance: {:?}", contract.get_balance(&caller));
    contract.withdraw(&U512::from(WITHDRAWAL));
    println!("Remaining balance: {:?}", contract.get_balance(&caller));
}
