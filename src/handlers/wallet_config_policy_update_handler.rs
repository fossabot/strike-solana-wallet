use crate::handlers::utils::{
    finalize_multisig_op, get_clock_from_next_account, next_program_account_info,
    start_multisig_config_op, validate_wallet_account,
};
use crate::instruction::WalletConfigPolicyUpdate;
use crate::model::multisig_op::MultisigOpParams;
use crate::model::wallet::Wallet;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

pub fn init(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wallet_account_bump_seed: u8,
    update: &WalletConfigPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let initiator_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    validate_wallet_account(
        wallet_account_info.key,
        wallet_account_bump_seed,
        program_id,
    )?;

    let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow())?;

    wallet.validate_config_initiator(initiator_account_info)?;
    wallet.lock_config_policy_updates()?;
    wallet.validate_config_policy_update(update)?;

    start_multisig_config_op(
        &multisig_op_account_info,
        &wallet,
        clock,
        MultisigOpParams::UpdateWalletConfigPolicy {
            wallet_address: *wallet_account_info.key,
            update: update.clone(),
        },
        *initiator_account_info.key,
    )?;

    Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;

    Ok(())
}

pub fn finalize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wallet_account_bump_seed: u8,
    update: &WalletConfigPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let account_to_return_rent_to = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    validate_wallet_account(
        wallet_account_info.key,
        wallet_account_bump_seed,
        program_id,
    )?;

    let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow_mut())?;

    finalize_multisig_op(
        &multisig_op_account_info,
        &account_to_return_rent_to,
        clock,
        MultisigOpParams::UpdateWalletConfigPolicy {
            wallet_address: *wallet_account_info.key,
            update: update.clone(),
        },
        || -> ProgramResult {
            wallet.update_config_policy(update)?;
            Ok(())
        },
    )?;

    wallet.unlock_config_policy_updates();
    Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;

    Ok(())
}
