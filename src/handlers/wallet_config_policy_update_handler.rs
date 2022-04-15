use crate::handlers::utils::{
    finalize_multisig_op, get_clock_from_next_account, next_program_account_info,
    start_multisig_config_op,
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
    update: &WalletConfigPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let initiator_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    let wallet = Wallet::unpack(&wallet_account_info.data.borrow())?;

    wallet.validate_config_initiator(initiator_account_info)?;
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

    Ok(())
}

pub fn finalize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    update: &WalletConfigPolicyUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_program_account_info(accounts_iter, program_id)?;
    let account_to_return_rent_to = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    finalize_multisig_op(
        &multisig_op_account_info,
        &account_to_return_rent_to,
        clock,
        MultisigOpParams::UpdateWalletConfigPolicy {
            wallet_address: *wallet_account_info.key,
            update: update.clone(),
        },
        || -> ProgramResult {
            let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow_mut())?;
            wallet.update_config_policy(update)?;
            Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;
            Ok(())
        },
    )?;

    Ok(())
}
