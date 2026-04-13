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
fn test_multiple_markets_independent() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let usdc_mint1 = common::create_usdc_mint(&mut svm, &payer);
    let expiry_ts: i64 = 1893456000;

    let market_id_1: u32 = 100;
    let market_1 = common::get_market_pda(market_id_1);
    let collateral_vault_1 = common::get_collateral_vault_pda(market_id_1);
    let outcome_a_mint_1 = common::get_outcome_a_mint_pda(market_id_1);
    let outcome_b_mint_1 = common::get_outcome_b_mint_pda(market_id_1);

    let accounts = InitializeMarket {
        market: market_1,
        authority: payer.pubkey(),
        collateral_mint: usdc_mint1,
        collateral_vault: collateral_vault_1,
        outcome_a_mint: outcome_a_mint_1,
        outcome_b_mint: outcome_b_mint_1,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_market::instruction::InitializeIx {
            market_id: market_id_1,
            expiry_ts,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "market 1 init failed: {:?}", res.err());

    let usdc_mint2 = common::create_usdc_mint(&mut svm, &payer);
    let market_id_2: u32 = 200;
    let market_2 = common::get_market_pda(market_id_2);
    let collateral_vault_2 = common::get_collateral_vault_pda(market_id_2);
    let outcome_a_mint_2 = common::get_outcome_a_mint_pda(market_id_2);
    let outcome_b_mint_2 = common::get_outcome_b_mint_pda(market_id_2);

    let accounts = InitializeMarket {
        market: market_2,
        authority: payer.pubkey(),
        collateral_mint: usdc_mint2,
        collateral_vault: collateral_vault_2,
        outcome_a_mint: outcome_a_mint_2,
        outcome_b_mint: outcome_b_mint_2,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        rent: solana_sdk::sysvar::rent::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(anchor_market::instruction::InitializeIx {
            market_id: market_id_2,
            expiry_ts,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "market 2 init failed: {:?}", res.err());

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let user_collateral_1 = common::create_ata(&mut svm, &payer, &user.pubkey(), &usdc_mint1);
    let user_outcome_a_1 = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_a_mint_1);
    let user_outcome_b_1 = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_b_mint_1);

    let user_collateral_2 = common::create_ata(&mut svm, &payer, &user.pubkey(), &usdc_mint2);
    let user_outcome_a_2 = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_a_mint_2);
    let user_outcome_b_2 = common::create_ata(&mut svm, &payer, &user.pubkey(), &outcome_b_mint_2);

    common::mint_to(
        &mut svm,
        &payer,
        &usdc_mint1,
        &user_collateral_1,
        500_000_000,
    );
    common::mint_to(
        &mut svm,
        &payer,
        &usdc_mint2,
        &user_collateral_2,
        500_000_000,
    );

    let accounts = anchor_market::accounts::SplitToken {
        market: market_1,
        user: user.pubkey(),
        collateral_mint: usdc_mint1,
        user_collateral: user_collateral_1,
        collateral_vault: collateral_vault_1,
        outcome_a_mint: outcome_a_mint_1,
        outcome_b_mint: outcome_b_mint_1,
        user_outcome_a: user_outcome_a_1,
        user_outcome_b: user_outcome_b_1,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SplitTokensIx {
            market_id: market_id_1,
            amount: 500_000_000,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "split on market 1 failed: {:?}", res.err());

    let accounts = anchor_market::accounts::SplitToken {
        market: market_2,
        user: user.pubkey(),
        collateral_mint: usdc_mint2,
        user_collateral: user_collateral_2,
        collateral_vault: collateral_vault_2,
        outcome_a_mint: outcome_a_mint_2,
        outcome_b_mint: outcome_b_mint_2,
        user_outcome_a: user_outcome_a_2,
        user_outcome_b: user_outcome_b_2,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SplitTokensIx {
            market_id: market_id_2,
            amount: 500_000_000,
        })
        .data(),
        accounts.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "split on market 2 failed: {:?}", res.err());
}
