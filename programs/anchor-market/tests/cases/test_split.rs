use crate::common;
use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program, InstructionData, ToAccountMetas,
    },
    anchor_market::{
        accounts::InitializeMarket, instruction::SplitTokensIx, ID as ANCHOR_MARKET_ID,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_split() {
    //initialize first
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

    // now it is time to split

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let user_collateral = common::create_ata(&mut svm, &payer, &user.pubkey(), &usdc_mint);
    let user_outcome_a = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_a_mint);
    let user_outcome_b = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_b_mint);

    let split_amount: u64 = 1_000_000_000;
    common::mint_to(&mut svm, &payer, &usdc_mint, &user_collateral, split_amount);

    let accounts = anchor_market::accounts::SplitToken {
        market,
        user: user.pubkey(),
        collateral_mint: usdc_mint,
        user_collateral,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        user_outcome_a,
        user_outcome_b,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SplitTokensIx {
            market_id,
            amount: split_amount,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    // user must sign — they are the signer in SplitToken
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "split failed: {:?}", res.err());
}
