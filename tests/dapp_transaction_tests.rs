#![cfg(feature = "test-bpf")]

use std::borrow::BorrowMut;

use bitvec::macros::internal::funty::Fundamental;
use solana_program::hash::{hash, Hash};
use solana_program::instruction::Instruction;
use solana_program::instruction::InstructionError::Custom;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{system_instruction, system_program};
use solana_program_test::tokio;
use solana_sdk::account::ReadableAccount;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer as SdkSigner;
use solana_sdk::transaction::{Transaction, TransactionError};
use solana_sdk::transport;

pub use common::instructions::*;
use common::instructions::{
    finalize_dapp_transaction, init_dapp_transaction, init_transfer, set_approval_disposition,
};
pub use common::utils::*;
use strike_wallet::error::WalletError;
use strike_wallet::model::address_book::{DAppBookEntry, DAppBookEntryNameHash};
use strike_wallet::model::dapp_multisig_data::DAppMultisigData;
use strike_wallet::model::multisig_op::{ApprovalDisposition, BooleanSetting, MultisigOp};

use crate::common::utils;
use crate::utils::BalanceAccountTestContext;

mod common;

struct DAppTest {
    context: BalanceAccountTestContext,
    balance_account: Pubkey,
    multisig_op_account: Keypair,
    multisig_data_account: Keypair,
    inner_instructions: Vec<Instruction>,
    inner_multisig_op_account: Keypair,
    params_hash: Hash,
}

async fn inner_instructions(
    context: &mut BalanceAccountTestContext,
    inner_multisig_op_account: &Pubkey,
    balance_account: &Pubkey,
    amount: u64,
) -> Vec<Instruction> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let multisig_op_account_rent = rent.minimum_balance(MultisigOp::LEN);
    vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            inner_multisig_op_account,
            multisig_op_account_rent,
            MultisigOp::LEN as u64,
            &context.program_id,
        ),
        init_transfer(
            &context.program_id,
            &context.wallet_account.pubkey(),
            &inner_multisig_op_account,
            &context.initiator_account.pubkey(),
            balance_account,
            &context.destination.pubkey(),
            context.balance_account_guid_hash,
            amount,
            context.destination_name_hash,
            &system_program::id(),
            &context.payer.pubkey(),
        ),
    ]
}

async fn setup_dapp_test() -> DAppTest {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(Some(100000)).await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let multisig_op_account_rent = rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();

    account_settings_update(
        &mut context,
        Some(BooleanSetting::Off),
        Some(BooleanSetting::On),
        None,
    )
    .await;

    let inner_multisig_op_account = Keypair::new();
    let dapp = DAppBookEntry {
        address: context.program_id.clone(),
        name_hash: DAppBookEntryNameHash::new(&hash_of(b"Strike Wallet")),
    };

    let inner_instructions = inner_instructions(
        &mut context,
        &inner_multisig_op_account.pubkey(),
        &balance_account,
        123,
    )
    .await;

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_op_account.pubkey(),
                    multisig_op_account_rent,
                    MultisigOp::LEN as u64,
                    &context.program_id,
                ),
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_data_account.pubkey(),
                    multisig_data_account_rent,
                    DAppMultisigData::LEN as u64,
                    &context.program_id,
                ),
                init_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &context.initiator_account.pubkey(),
                    &context.balance_account_guid_hash,
                    dapp,
                    inner_instructions.len().as_u8(),
                ),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &multisig_op_account,
                &multisig_data_account,
                &context.initiator_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();

    // supply the instructions
    // send them in two separate transactions, with the second one sent first
    supply_instructions(
        &mut context,
        &multisig_op_account,
        &multisig_data_account,
        1,
        &vec![inner_instructions[1].clone()],
    )
    .await
    .unwrap();
    supply_instructions(
        &mut context,
        &multisig_op_account,
        &multisig_data_account,
        0,
        &vec![inner_instructions[0].clone()],
    )
    .await
    .unwrap();

    let mut multisig_data =
        DAppMultisigData::unpack_unchecked(&[0; DAppMultisigData::LEN]).unwrap();
    multisig_data
        .init(
            context.wallet_account.pubkey(),
            context.balance_account_guid_hash.clone(),
            dapp,
            inner_instructions.len().as_u8(),
        )
        .unwrap();

    for (ix, instruction) in inner_instructions.iter().enumerate() {
        multisig_data
            .add_instruction(ix.as_u8(), instruction)
            .unwrap()
    }

    DAppTest {
        context,
        balance_account,
        multisig_op_account,
        multisig_data_account,
        inner_instructions,
        inner_multisig_op_account,
        params_hash: multisig_data.hash().unwrap(),
    }
}

#[tokio::test]
async fn test_dapp_transaction_simulation() {
    let mut dapp_test = setup_dapp_test().await;

    let context = dapp_test.context.borrow_mut();

    // attempting to finalize before approval should result in a transaction simulation
    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[finalize_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &dapp_test.multisig_op_account.pubkey(),
                    &dapp_test.multisig_data_account.pubkey(),
                    &dapp_test.balance_account,
                    &context.payer.pubkey(),
                    &context.balance_account_guid_hash,
                    &dapp_test.params_hash,
                    &dapp_test.inner_instructions,
                )],
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &context.initiator_account,
                    &dapp_test.inner_multisig_op_account,
                ],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::SimulationFinished as u32)),
    );
}

#[tokio::test]
async fn test_dapp_transaction_bad_signature() {
    let dapp_test = setup_dapp_test().await;

    let mut context = dapp_test.context;

    let params_hash = utils::get_operation_hash(
        context.banks_client.borrow_mut(),
        dapp_test.multisig_op_account.pubkey(),
    )
    .await;
    let approver = &context.approvers[0];
    let approve_transaction = Transaction::new_signed_with_payer(
        &[set_approval_disposition(
            &context.program_id,
            &dapp_test.multisig_op_account.pubkey(),
            &approver.pubkey(),
            ApprovalDisposition::APPROVE,
            params_hash,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, approver],
        context.recent_blockhash,
    );
    context
        .banks_client
        .process_transaction(approve_transaction)
        .await
        .unwrap();

    // attempt to finalize with bad param hash
    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[finalize_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &dapp_test.multisig_op_account.pubkey(),
                    &dapp_test.multisig_data_account.pubkey(),
                    &dapp_test.balance_account,
                    &context.payer.pubkey(),
                    &context.balance_account_guid_hash,
                    &hash(&[0]),
                    &dapp_test.inner_instructions,
                )],
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &context.initiator_account,
                    &dapp_test.inner_multisig_op_account,
                ],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::InvalidSignature as u32)),
    );
}

#[tokio::test]
async fn test_dapp_transaction() {
    let dapp_test = setup_dapp_test().await;

    let mut context = dapp_test.context;

    let params_hash = utils::get_operation_hash(
        context.banks_client.borrow_mut(),
        dapp_test.multisig_op_account.pubkey(),
    )
    .await;
    let approver = &context.approvers[0];
    let approve_transaction = Transaction::new_signed_with_payer(
        &[set_approval_disposition(
            &context.program_id,
            &dapp_test.multisig_op_account.pubkey(),
            &approver.pubkey(),
            ApprovalDisposition::APPROVE,
            params_hash,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, approver],
        context.recent_blockhash,
    );
    context
        .banks_client
        .process_transaction(approve_transaction)
        .await
        .unwrap();

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[finalize_dapp_transaction(
                &context.program_id,
                &context.wallet_account.pubkey(),
                &dapp_test.multisig_op_account.pubkey(),
                &dapp_test.multisig_data_account.pubkey(),
                &dapp_test.balance_account,
                &context.payer.pubkey(),
                &context.balance_account_guid_hash,
                &dapp_test.params_hash,
                &dapp_test.inner_instructions,
            )],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &context.initiator_account,
                &dapp_test.inner_multisig_op_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();

    let multisig_op = MultisigOp::unpack_from_slice(
        context
            .banks_client
            .get_account(dapp_test.inner_multisig_op_account.pubkey())
            .await
            .unwrap()
            .unwrap()
            .data(),
    )
    .unwrap();
    assert!(multisig_op.is_initialized);
}

#[tokio::test]
async fn test_dapp_transaction_denied() {
    let dapp_test = setup_dapp_test().await;

    let mut context = dapp_test.context;

    let params_hash = utils::get_operation_hash(
        context.banks_client.borrow_mut(),
        dapp_test.multisig_op_account.pubkey(),
    )
    .await;
    let approver = &context.approvers[0];
    let approve_transaction = Transaction::new_signed_with_payer(
        &[set_approval_disposition(
            &context.program_id,
            &dapp_test.multisig_op_account.pubkey(),
            &approver.pubkey(),
            ApprovalDisposition::DENY,
            params_hash,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, approver],
        context.recent_blockhash,
    );
    context
        .banks_client
        .process_transaction(approve_transaction)
        .await
        .unwrap();

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[finalize_dapp_transaction(
                &context.program_id,
                &context.wallet_account.pubkey(),
                &dapp_test.multisig_op_account.pubkey(),
                &dapp_test.multisig_data_account.pubkey(),
                &dapp_test.balance_account,
                &context.payer.pubkey(),
                &context.balance_account_guid_hash,
                &dapp_test.params_hash,
                &dapp_test.inner_instructions,
            )],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &context.initiator_account,
                &dapp_test.inner_multisig_op_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();

    // ensure inner transaction did not execute (so inner multisig op account should not exist)
    assert!(context
        .banks_client
        .get_account(dapp_test.inner_multisig_op_account.pubkey())
        .await
        .unwrap()
        .is_none());

    // outer multisig op should have been cleaned up
    assert!(context
        .banks_client
        .get_account(dapp_test.multisig_op_account.pubkey())
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_dapp_transaction_with_spl_transfers() {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(Some(100000)).await;

    account_settings_update(&mut context, None, Some(BooleanSetting::On), None).await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let multisig_op_account_rent = rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();
    let mint_account_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    let mint = Keypair::new();
    let mint_authority = Keypair::new();
    let source_token_address = spl_associated_token_account::get_associated_token_address(
        &balance_account,
        &mint.pubkey(),
    );

    let inner_instructions = vec![
        system_instruction::create_account(
            &context.payer.pubkey(),
            &mint.pubkey(),
            mint_account_rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ),
        system_instruction::create_account(
            &context.payer.pubkey(),
            &mint_authority.pubkey(),
            0,
            0,
            &system_program::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &mint_authority.pubkey(),
            Some(&mint_authority.pubkey()),
            6,
        )
        .unwrap(),
        spl_associated_token_account::create_associated_token_account(
            &context.payer.pubkey(),
            &balance_account,
            &mint.pubkey(),
        ),
        spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &source_token_address,
            &mint_authority.pubkey(),
            &[],
            1000,
        )
        .unwrap(),
    ];

    let dapp = DAppBookEntry {
        address: context.program_id.clone(),
        name_hash: DAppBookEntryNameHash::new(&hash_of(b"Strike Wallet")),
    };

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_op_account.pubkey(),
                    multisig_op_account_rent,
                    MultisigOp::LEN as u64,
                    &context.program_id,
                ),
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_data_account.pubkey(),
                    multisig_data_account_rent,
                    DAppMultisigData::LEN as u64,
                    &context.program_id,
                ),
                init_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &context.initiator_account.pubkey(),
                    &context.balance_account_guid_hash,
                    dapp,
                    inner_instructions.len().as_u8(),
                ),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &multisig_op_account,
                &multisig_data_account,
                &context.initiator_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();

    // attempting to approve or finalize before supplying the instructions results in an error
    let approver = &context.approvers[0];
    let approve_transaction = Transaction::new_signed_with_payer(
        &[set_approval_disposition(
            &context.program_id,
            &multisig_op_account.pubkey(),
            &approver.pubkey(),
            ApprovalDisposition::APPROVE,
            Hash::new_unique(), // doesn't matter
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, approver],
        context.recent_blockhash,
    );
    assert_eq!(
        context
            .banks_client
            .process_transaction(approve_transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::OperationNotInitialized as u32)),
    );

    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[finalize_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &balance_account,
                    &context.payer.pubkey(),
                    &context.balance_account_guid_hash,
                    &Hash::new_unique(), // doesn't matter
                    &inner_instructions,
                )],
                Some(&context.payer.pubkey()),
                &[&context.payer, &mint, &mint_authority],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::OperationNotInitialized as u32)),
    );

    supply_instructions(
        &mut context,
        &multisig_op_account,
        &multisig_data_account,
        0,
        &inner_instructions,
    )
    .await
    .unwrap();

    let mut multisig_data =
        DAppMultisigData::unpack_unchecked(&[0; DAppMultisigData::LEN]).unwrap();
    multisig_data
        .init(
            context.wallet_account.pubkey(),
            context.balance_account_guid_hash.clone(),
            dapp,
            inner_instructions.len().as_u8(),
        )
        .unwrap();

    for (ix, instruction) in inner_instructions.iter().enumerate() {
        multisig_data
            .add_instruction(ix.as_u8(), instruction)
            .unwrap()
    }

    // attempting to finalize before approval should result in a transaction simulation
    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[finalize_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &balance_account,
                    &context.payer.pubkey(),
                    &context.balance_account_guid_hash,
                    &multisig_data.hash().unwrap(),
                    &inner_instructions,
                )],
                Some(&context.payer.pubkey()),
                &[&context.payer, &mint, &mint_authority],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::SimulationFinished as u32)),
    );
}

#[tokio::test]
async fn test_dapp_transaction_without_dapps_enabled() {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(None).await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let multisig_op_account_rent = rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();
    let dapp = DAppBookEntry {
        address: context.program_id.clone(),
        name_hash: DAppBookEntryNameHash::new(&hash_of(b"Strike Wallet")),
    };
    let inner_instructions = inner_instructions(
        &mut context,
        &multisig_op_account.pubkey(),
        &balance_account,
        123,
    )
    .await;
    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &multisig_op_account.pubkey(),
                        multisig_op_account_rent,
                        MultisigOp::LEN as u64,
                        &context.program_id,
                    ),
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &multisig_data_account.pubkey(),
                        multisig_data_account_rent,
                        DAppMultisigData::LEN as u64,
                        &context.program_id,
                    ),
                    init_dapp_transaction(
                        &context.program_id,
                        &context.wallet_account.pubkey(),
                        &multisig_op_account.pubkey(),
                        &multisig_data_account.pubkey(),
                        &context.initiator_account.pubkey(),
                        &context.balance_account_guid_hash,
                        dapp,
                        inner_instructions.len().as_u8(),
                    ),
                ],
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &multisig_op_account,
                    &multisig_data_account,
                    &context.initiator_account,
                ],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(2, Custom(WalletError::DAppsDisabled as u32)),
    );
}

#[tokio::test]
async fn test_dapp_transaction_unwhitelisted() {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(None).await;

    account_settings_update(
        &mut context,
        Some(BooleanSetting::On),
        Some(BooleanSetting::On),
        None,
    )
    .await;

    let rent = context.banks_client.get_rent().await.unwrap();
    let multisig_op_account_rent = rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();
    let dapp = DAppBookEntry {
        address: context.program_id.clone(),
        name_hash: DAppBookEntryNameHash::new(&hash_of(b"Strike Wallet")),
    };
    let inner_instructions = inner_instructions(
        &mut context,
        &multisig_op_account.pubkey(),
        &balance_account,
        123,
    )
    .await;
    assert_eq!(
        context
            .banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &multisig_op_account.pubkey(),
                        multisig_op_account_rent,
                        MultisigOp::LEN as u64,
                        &context.program_id,
                    ),
                    system_instruction::create_account(
                        &context.payer.pubkey(),
                        &multisig_data_account.pubkey(),
                        multisig_data_account_rent,
                        DAppMultisigData::LEN as u64,
                        &context.program_id,
                    ),
                    init_dapp_transaction(
                        &context.program_id,
                        &context.wallet_account.pubkey(),
                        &multisig_op_account.pubkey(),
                        &multisig_data_account.pubkey(),
                        &context.initiator_account.pubkey(),
                        &context.balance_account_guid_hash,
                        dapp,
                        inner_instructions.len().as_u8(),
                    ),
                ],
                Some(&context.payer.pubkey()),
                &[
                    &context.payer,
                    &multisig_op_account,
                    &multisig_data_account,
                    &context.initiator_account,
                ],
                context.recent_blockhash,
            ))
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(2, Custom(WalletError::DAppNotAllowed as u32)),
    );
}

#[tokio::test]
async fn test_dapp_transaction_whitelisted() {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(None).await;

    account_settings_update(
        &mut context,
        Some(BooleanSetting::On),
        Some(BooleanSetting::On),
        None,
    )
    .await;

    let multisig_op_account_rent = context.rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = context.rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();
    let inner_instructions = inner_instructions(
        &mut context,
        &multisig_op_account.pubkey(),
        &balance_account,
        123,
    )
    .await;
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_op_account.pubkey(),
                    multisig_op_account_rent,
                    MultisigOp::LEN as u64,
                    &context.program_id,
                ),
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_data_account.pubkey(),
                    multisig_data_account_rent,
                    DAppMultisigData::LEN as u64,
                    &context.program_id,
                ),
                init_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &context.initiator_account.pubkey(),
                    &context.balance_account_guid_hash,
                    context.allowed_dapp,
                    inner_instructions.len().as_u8(),
                ),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &multisig_op_account,
                &multisig_data_account,
                &context.initiator_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_supply_instruction_errors() {
    let (mut context, balance_account) =
        utils::setup_balance_account_tests_and_finalize(Some(100000)).await;

    account_settings_update(
        &mut context,
        Some(BooleanSetting::Off),
        Some(BooleanSetting::On),
        None,
    )
    .await;

    let multisig_op_account_rent = context.rent.minimum_balance(MultisigOp::LEN);
    let multisig_op_account = Keypair::new();
    let multisig_data_account_rent = context.rent.minimum_balance(DAppMultisigData::LEN);
    let multisig_data_account = Keypair::new();
    let inner_multisig_op_account = Keypair::new();
    let dapp = DAppBookEntry {
        address: context.program_id.clone(),
        name_hash: DAppBookEntryNameHash::new(&hash_of(b"Strike Wallet")),
    };

    let inner_instructions = inner_instructions(
        &mut context,
        &inner_multisig_op_account.pubkey(),
        &balance_account,
        123,
    )
    .await;

    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_op_account.pubkey(),
                    multisig_op_account_rent,
                    MultisigOp::LEN as u64,
                    &context.program_id,
                ),
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &multisig_data_account.pubkey(),
                    multisig_data_account_rent,
                    DAppMultisigData::LEN as u64,
                    &context.program_id,
                ),
                init_dapp_transaction(
                    &context.program_id,
                    &context.wallet_account.pubkey(),
                    &multisig_op_account.pubkey(),
                    &multisig_data_account.pubkey(),
                    &context.initiator_account.pubkey(),
                    &context.balance_account_guid_hash,
                    dapp,
                    inner_instructions.len().as_u8(),
                ),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &multisig_op_account,
                &multisig_data_account,
                &context.initiator_account,
            ],
            context.recent_blockhash,
        ))
        .await
        .unwrap();

    // test that you cannot supply an instruction outside of the range
    assert_eq!(
        supply_instructions(
            &mut context,
            &multisig_op_account,
            &multisig_data_account,
            2,
            &vec![inner_instructions[1].clone()]
        )
        .await
        .unwrap_err()
        .unwrap(),
        TransactionError::InstructionError(0, Custom(WalletError::DAppInstructionOverflow as u32)),
    );

    // test that you cannot supply an instruction more than once
    supply_instructions(
        &mut context,
        &multisig_op_account,
        &multisig_data_account,
        0,
        &vec![inner_instructions[0].clone()],
    )
    .await
    .unwrap();

    assert_eq!(
        supply_instructions(
            &mut context,
            &multisig_op_account,
            &multisig_data_account,
            0,
            &vec![inner_instructions[1].clone()]
        )
        .await
        .unwrap_err()
        .unwrap(),
        TransactionError::InstructionError(
            0,
            Custom(WalletError::DAppInstructionAlreadySupplied as u32)
        ),
    );

    // test that you cannot supply an instruction more than once
    supply_instructions(
        &mut context,
        &multisig_op_account,
        &multisig_data_account,
        0,
        &vec![inner_instructions[0].clone()],
    )
    .await
    .unwrap();
}

async fn supply_instructions(
    context: &mut BalanceAccountTestContext,
    multisig_op_account: &Keypair,
    multisig_data_account: &Keypair,
    starting_index: u8,
    instructions: &Vec<Instruction>,
) -> transport::Result<()> {
    context
        .banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[supply_dapp_transaction_instructions(
                &context.program_id,
                &multisig_op_account.pubkey(),
                &multisig_data_account.pubkey(),
                &context.initiator_account.pubkey(),
                starting_index,
                instructions,
            )],
            Some(&context.payer.pubkey()),
            &[&context.payer, &context.initiator_account],
            context.recent_blockhash,
        ))
        .await
}
