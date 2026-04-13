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
        instruction::{ ClaimRewardIx, SetWinnerSideIx },
        ID as ANCHOR_MARKET_ID,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{ Message, VersionedMessage },
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

fn setup_market_and_split(
    svm: &mut LiteSVM,
    payer: &Keypair,
    market_id: u32
) -> (Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Keypair, u64) {
    let program_id = ANCHOR_MARKET_ID;
    let usdc_mint = common::create_usdc_mint(svm, payer);
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
    svm.send_transaction(tx).expect("initialize failed");

    let user = Keypair::new();
    svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();

    let user_collateral = common::create_ata(svm, payer, &user.pubkey(), &usdc_mint);
    let user_outcome_a = common::create_ata(svm, payer, &user.pubkey(), &outcome_a_mint);
    let user_outcome_b = common::create_ata(svm, payer, &user.pubkey(), &outcome_b_mint);

    let split_amount: u64 = 1_000_000_000;
    common::mint_to(svm, payer, &usdc_mint, &user_collateral, split_amount);

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
        &(anchor_market::instruction::SplitTokensIx {
            market_id,
            amount: split_amount,
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    svm.send_transaction(tx).expect("split failed");

    (market, user_collateral, user_outcome_a, user_outcome_b, usdc_mint, user, split_amount)
}

#[test]
fn test_claim_rewards_outcome_a_winner() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let market_id: u32 = 1;
    let (market, user_collateral, user_outcome_a, user_outcome_b, usdc_mint, user, _) =
        setup_market_and_split(&mut svm, &payer, market_id);

    let collateral_vault = common::get_collateral_vault_pda(market_id);
    let outcome_a_mint = common::get_outcome_a_mint_pda(market_id);
    let outcome_b_mint = common::get_outcome_b_mint_pda(market_id);

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
        user: user.pubkey(),
        market,
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
        &(ClaimRewardIx { market_id }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "claim rewards failed: {:?}", res.err());
}

use anchor_market::state::WinningOutcome;
use solana_pubkey::Pubkey;

#[test]
fn test_claim_rewards_outcome_b_winner() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let market_id: u32 = 2;
    let (market, user_collateral, user_outcome_a, user_outcome_b, usdc_mint, user, _) =
        setup_market_and_split(&mut svm, &payer, market_id);

    let collateral_vault = common::get_collateral_vault_pda(market_id);
    let outcome_a_mint = common::get_outcome_a_mint_pda(market_id);
    let outcome_b_mint = common::get_outcome_b_mint_pda(market_id);

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
            winner: WinningOutcome::OutcomeB,
        }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "set winner failed: {:?}", res.err());

    let accounts = anchor_market::accounts::ClaimRewards {
        user: user.pubkey(),
        market,
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
        &(ClaimRewardIx { market_id }).data(),
        accounts.to_account_metas(None)
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&user]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "claim rewards failed: {:?}", res.err());
}
