use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::sysvar::Sysvar;

use crate::handlers::utils::validate_wallet_account;
use crate::instruction::InitialWalletConfig;
use crate::model::signer::Signer;
use crate::model::wallet::Wallet;
use crate::version::VERSION;

pub fn handle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    wallet_account_bump_seed: u8,
    update: &InitialWalletConfig,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let wallet_account_info = next_account_info(accounts_iter)?;
    let assistant_account_info = next_account_info(accounts_iter)?;
    let fee_payer_account_info = next_account_info(accounts_iter)?;

    validate_wallet_account(
        wallet_account_info.key,
        wallet_account_bump_seed,
        program_id,
    )?;

    let rent = Rent::get()?;
    let wallet_account_rent = rent.minimum_balance(Wallet::LEN);
    invoke_signed(
        &system_instruction::create_account(
            fee_payer_account_info.key,
            &wallet_account_info.key,
            wallet_account_rent,
            Wallet::LEN as u64,
            program_id,
        ),
        &[fee_payer_account_info.clone(), wallet_account_info.clone()],
        &[&[
            b"version",
            &VERSION.to_le_bytes(),
            &[wallet_account_bump_seed],
        ]],
    )?;

    let mut wallet = Wallet::unpack_unchecked(&wallet_account_info.data.borrow())?;

    if wallet.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    wallet.is_initialized = true;
    wallet.assistant = Signer {
        key: *assistant_account_info.key,
    };
    wallet.rent_return_address = *fee_payer_account_info.key;
    wallet.initialize(update)?;
    Wallet::pack(wallet, &mut wallet_account_info.data.borrow_mut())?;

    Ok(())
}
