use crate::handlers::utils::{
    finalize_multisig_op, get_clock_from_next_account, next_program_account_info,
    next_wallet_account_info, start_multisig_config_op,
};
use crate::instruction::AddressBookWhitelistUpdate;
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
    account_guid_hash: &BalanceAccountGuidHash,
    update: &AddressBookWhitelistUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_wallet_account_info(accounts_iter, program_id)?;
    let initiator_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    let wallet = Wallet::unpack(&wallet_account_info.data.borrow())?;
    wallet.validate_config_initiator(initiator_account_info)?;
    wallet.validate_address_book_whitelist_update(account_guid_hash, update)?;

    start_multisig_config_op(
        &multisig_op_account_info,
        &wallet,
        clock,
        MultisigOpParams::UpdateAddressBookWhitelist {
            wallet_address: *wallet_account_info.key,
            account_guid_hash: *account_guid_hash,
            update: update.clone(),
        },
        *initiator_account_info.key,
    )?;

    Ok(())
}

pub fn finalize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    account_guid_hash: &BalanceAccountGuidHash,
    update: &AddressBookWhitelistUpdate,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let multisig_op_account_info = next_program_account_info(accounts_iter, program_id)?;
    let wallet_account_info = next_wallet_account_info(accounts_iter, program_id)?;
    let rent_collector_account_info = next_account_info(accounts_iter)?;
    let clock = get_clock_from_next_account(accounts_iter)?;

    finalize_multisig_op(
        &multisig_op_account_info,
        &rent_collector_account_info,
        clock,
        MultisigOpParams::UpdateAddressBookWhitelist {
            account_guid_hash: *account_guid_hash,
            wallet_address: *wallet_account_info.key,
            update: update.clone(),
        },
        || -> ProgramResult {
            let mut wallet = Wallet::unpack(&wallet_account_info.data.borrow_mut())?;
            wallet.update_address_book_whitelist(account_guid_hash, update)?;
            Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;
            Ok(())
        },
    )?;

    Ok(())
}