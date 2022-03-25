use crate::handlers::{
    address_book_update_handler, approval_disposition_handler, balance_account_creation_handler,
    balance_account_name_update_handler, balance_account_policy_update_handler,
    balance_account_settings_update_handler, dapp_book_update_handler, dapp_transaction_handler,
    init_wallet_handler, spl_token_accounts_creation_handler, transfer_handler,
    update_signer_handler, wallet_config_policy_update_handler, wrap_unwrap_handler,
};
use crate::instruction::ProgramInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ProgramInstruction::unpack(instruction_data)?;

        match instruction {
            ProgramInstruction::InitWallet {
                wallet_account_bump_seed,
                initial_config: update,
            } => {
                init_wallet_handler::handle(program_id, accounts, wallet_account_bump_seed, &update)
            }

            ProgramInstruction::InitWalletConfigPolicyUpdate {
                wallet_account_bump_seed,
                update,
            } => wallet_config_policy_update_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::FinalizeWalletConfigPolicyUpdate {
                wallet_account_bump_seed,
                update,
            } => wallet_config_policy_update_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::InitBalanceAccountCreation {
                wallet_account_bump_seed,
                account_guid_hash,
                creation_params,
            } => balance_account_creation_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &creation_params,
            ),

            ProgramInstruction::FinalizeBalanceAccountCreation {
                wallet_account_bump_seed,
                account_guid_hash,
                creation_params,
            } => balance_account_creation_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &creation_params,
            ),

            ProgramInstruction::InitBalanceAccountNameUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                account_name_hash,
            } => balance_account_name_update_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &account_name_hash,
            ),

            ProgramInstruction::FinalizeBalanceAccountNameUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                account_name_hash,
            } => balance_account_name_update_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &account_name_hash,
            ),

            ProgramInstruction::InitBalanceAccountPolicyUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                update,
            } => balance_account_policy_update_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &update,
            ),

            ProgramInstruction::FinalizeBalanceAccountPolicyUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                update,
            } => balance_account_policy_update_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                &update,
            ),

            ProgramInstruction::InitTransfer {
                wallet_account_bump_seed,
                account_guid_hash,
                amount,
                destination_name_hash,
            } => transfer_handler::init(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                amount,
                &destination_name_hash,
            ),

            ProgramInstruction::FinalizeTransfer {
                wallet_account_bump_seed,
                account_guid_hash,
                amount,
                token_mint,
            } => transfer_handler::finalize(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                amount,
                token_mint,
            ),

            ProgramInstruction::SetApprovalDisposition {
                disposition,
                params_hash,
            } => approval_disposition_handler::handle(
                program_id,
                &accounts,
                disposition,
                params_hash,
            ),

            ProgramInstruction::InitWrapUnwrap {
                wallet_account_bump_seed,
                account_guid_hash,
                amount,
                direction,
            } => wrap_unwrap_handler::init(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                amount,
                direction,
            ),

            ProgramInstruction::FinalizeWrapUnwrap {
                wallet_account_bump_seed,
                account_guid_hash,
                amount,
                direction,
            } => wrap_unwrap_handler::finalize(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                amount,
                direction,
            ),

            ProgramInstruction::InitUpdateSigner {
                wallet_account_bump_seed,
                slot_update_type,
                slot_id,
                signer,
            } => update_signer_handler::init(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                slot_update_type,
                slot_id,
                signer,
            ),

            ProgramInstruction::FinalizeUpdateSigner {
                wallet_account_bump_seed,
                slot_update_type,
                slot_id,
                signer,
            } => update_signer_handler::finalize(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                slot_update_type,
                slot_id,
                signer,
            ),

            ProgramInstruction::InitDAppTransaction {
                wallet_account_bump_seed,
                ref account_guid_hash,
                dapp,
                instruction_count,
            } => dapp_transaction_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                account_guid_hash,
                dapp,
                instruction_count,
            ),

            ProgramInstruction::SupplyDAppTransactionInstructions {
                instructions,
                starting_index,
            } => dapp_transaction_handler::supply_instructions(
                program_id,
                accounts,
                starting_index,
                instructions,
            ),

            ProgramInstruction::FinalizeDAppTransaction {
                wallet_account_bump_seed,
                ref account_guid_hash,
                ref params_hash,
            } => dapp_transaction_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                account_guid_hash,
                params_hash,
            ),

            ProgramInstruction::InitAccountSettingsUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                whitelist_enabled,
                dapps_enabled,
            } => balance_account_settings_update_handler::init(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                whitelist_enabled,
                dapps_enabled,
            ),

            ProgramInstruction::FinalizeAccountSettingsUpdate {
                wallet_account_bump_seed,
                account_guid_hash,
                whitelist_enabled,
                dapps_enabled,
            } => balance_account_settings_update_handler::finalize(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &account_guid_hash,
                whitelist_enabled,
                dapps_enabled,
            ),

            ProgramInstruction::InitDAppBookUpdate {
                wallet_account_bump_seed,
                update,
            } => dapp_book_update_handler::init(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::FinalizeDAppBookUpdate {
                wallet_account_bump_seed,
                update,
            } => dapp_book_update_handler::finalize(
                program_id,
                &accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::InitAddressBookUpdate {
                wallet_account_bump_seed,
                update,
            } => address_book_update_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::FinalizeAddressBookUpdate {
                wallet_account_bump_seed,
                update,
            } => address_book_update_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &update,
            ),

            ProgramInstruction::InitSPLTokenAccountsCreation {
                wallet_account_bump_seed,
                payer_account_guid_hash,
                account_guid_hashes,
            } => spl_token_accounts_creation_handler::init(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &payer_account_guid_hash,
                &account_guid_hashes,
            ),

            ProgramInstruction::FinalizeSPLTokenAccountsCreation {
                wallet_account_bump_seed,
                payer_account_guid_hash,
                account_guid_hashes,
            } => spl_token_accounts_creation_handler::finalize(
                program_id,
                accounts,
                wallet_account_bump_seed,
                &payer_account_guid_hash,
                &account_guid_hashes,
            ),
        }
    }
}
