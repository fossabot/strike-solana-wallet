use crate::handlers::utils::{
    finalize_multisig_op, get_clock_from_next_account, next_program_account_info,
    start_multisig_config_op, validate_wallet_account,
};
use crate::instruction::BalanceAccountPolicyUpdate;
use crate::model::balance_account::BalanceAccountGuidHash;
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
    account_guid_hash: &BalanceAccountGuidHash,
    update: &BalanceAccountPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let initiator_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    validate_wallet_account(
        wallet_account_info,
        wallet_account_bump_seed,
        program_id,
        true,
    )?;

    let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow())?;
    wallet.validate_config_initiator(initiator_account_info)?;
    wallet.lock_balance_account_policy_updates(account_guid_hash)?;
    wallet.validate_balance_account_policy_update(account_guid_hash, update)?;

    start_multisig_config_op(
        &multisig_op_account_info,
        &wallet,
        clock,
        MultisigOpParams::UpdateBalanceAccountPolicy {
            wallet_address: *wallet_account_info.key,
            account_guid_hash: *account_guid_hash,
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
    account_guid_hash: &BalanceAccountGuidHash,
    update: &BalanceAccountPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let rent_collector_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    validate_wallet_account(
        wallet_account_info,
        wallet_account_bump_seed,
        program_id,
        true,
    )?;

    let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow_mut())?;

    finalize_multisig_op(
        &multisig_op_account_info,
        &rent_collector_account_info,
        clock,
        MultisigOpParams::UpdateBalanceAccountPolicy {
            account_guid_hash: *account_guid_hash,
            wallet_address: *wallet_account_info.key,
            update: update.clone(),
        },
        || -> ProgramResult {
            wallet.update_balance_account_policy(account_guid_hash, update)?;
            Ok(())
        },
    )?;

    wallet.unlock_balance_account_policy_updates(account_guid_hash)?;
    Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;

    Ok(())
}
