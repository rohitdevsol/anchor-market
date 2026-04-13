use crate::common;
use ::{
    anchor_lang::{
        solana_program::instruction::Instruction,
        system_program,
        InstructionData,
        ToAccountMetas,
    },
    anchor_market::{
        accounts::InitializeMarket,
        instruction::{ ClaimRewardIx, SetWinnerSideIx, SplitTokensIx },
        state::WinningOutcome,
        ID as ANCHOR_MARKET_ID,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{ Message, VersionedMessage },
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_multiple_users_split_and_claim() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

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
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "initialize failed: {:?}", res.err());

    let user1 = Keypair::new();
    let user2 = Keypair::new();
    svm.airdrop(&user1.pubkey(), 1_000_000_000).unwrap();
    svm.airdrop(&user2.pubkey(), 1_000_000_000).unwrap();

    let user1_collateral = common::create_ata(&mut svm, &payer, &user1.pubkey(), &usdc_mint);
    let user1_outcome_a = common::create_ata(&mut svm, &payer, &user1.pubkey(), &outcome_a_mint);
    let user1_outcome_b = common::create_ata(&mut svm, &payer, &user1.pubkey(), &outcome_b_mint);

    let user2_collateral = common::create_ata(&mut svm, &payer, &user2.pubkey(), &usdc_mint);
    let user2_outcome_a = common::create_ata(&mut svm, &payer, &user2.pubkey(), &outcome_a_mint);
    let user2_outcome_b = common::create_ata(&mut svm, &payer, &user2.pubkey(), &outcome_b_mint);

    common::mint_to(&mut svm, &payer, &usdc_mint, &user1_collateral, 500_000_000);
    common::mint_to(&mut svm, &payer, &usdc_mint, &user2_collateral, 500_000_000);

    let accounts = anchor_market::accounts::SplitToken {
        market,
        user: user1.pubkey(),
        collateral_mint: usdc_mint,
        user_collateral: user1_collateral,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        user_outcome_a: user1_outcome_a,
        user_outcome_b: user1_outcome_b,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SplitTokensIx {
            market_id,
            amount: 500_000_000,
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user1.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user1]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "user1 split failed: {:?}", res.err());

    let accounts = anchor_market::accounts::SplitToken {
        market,
        user: user2.pubkey(),
        collateral_mint: usdc_mint,
        user_collateral: user2_collateral,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        user_outcome_a: user2_outcome_a,
        user_outcome_b: user2_outcome_b,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SplitTokensIx {
            market_id,
            amount: 500_000_000,
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user2.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user2]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "user2 split failed: {:?}", res.err());

    let accounts = anchor_market::accounts::SetWinner {
        authority: payer.pubkey(),
        market,
        outcome_a_mint,
        outcome_b_mint,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(SetWinnerSideIx {
            market_id,
            winner: WinningOutcome::OutcomeA,
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "set winner failed: {:?}", res.err());

    let accounts = anchor_market::accounts::ClaimRewards {
        user: user1.pubkey(),
        market,
        collateral_mint: usdc_mint,
        user_collateral: user1_collateral,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        user_outcome_a: user1_outcome_a,
        user_outcome_b: user1_outcome_b,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(ClaimRewardIx { market_id }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user1.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user1]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "user1 claim failed: {:?}", res.err());

    let accounts = anchor_market::accounts::ClaimRewards {
        user: user2.pubkey(),
        market,
        collateral_mint: usdc_mint,
        user_collateral: user2_collateral,
        collateral_vault,
        outcome_a_mint,
        outcome_b_mint,
        user_outcome_a: user2_outcome_a,
        user_outcome_b: user2_outcome_b,
        token_program: spl_token::ID,
    };

    let instruction = Instruction::new_with_bytes(
        program_id,
        &(ClaimRewardIx { market_id }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user2.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user2]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "user2 claim failed: {:?}", res.err());
}
