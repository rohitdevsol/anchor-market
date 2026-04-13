use crate::common;
use {
    anchor_lang::system_program,
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    anchor_market::{accounts::InitializeMarket, ID as ANCHOR_MARKET_ID},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_initialize() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // create a local mock USDC mint
    let usdc_mint = common::create_usdc_mint(&mut svm, &payer);

    let market_id: u32 = 1;
    let expiry_ts: i64 = 1893456000;

    let market = common::get_market_pda(market_id);
    let collateral_vault = common::get_collateral_vault_pda(market_id);
    let outcome_a_mint = common::get_outcome_a_mint_pda(market_id);
    let outcome_b_mint = common::get_outcome_b_mint_pda(market_id);

    let accounts = InitializeMarket {
        market,
        authority: payer.pubkey(),
        collateral_mint: usdc_mint,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_market::instruction::InitializeIx {
            market_id,
            expiry_ts,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "initialize failed: {:?}", res.err());
}
