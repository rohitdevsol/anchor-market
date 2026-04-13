use ::{
    anchor_lang::system_program,
    anchor_lang::{ solana_program::instruction::Instruction, InstructionData, ToAccountMetas },
    anchor_market::{ accounts::InitializeMarket, ID as ANCHOR_MARKET_ID },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{ Message, VersionedMessage },
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    solana_program::program_pack::Pack,
    solana_system_interface::instruction::create_account,
};

fn get_market_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"market", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

fn get_collateral_vault_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

fn get_outcome_a_mint_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"outcome_a", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

fn get_outcome_b_mint_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"outcome_b", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

fn create_usdc_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    let usdc_mint_keypair = Keypair::new();
    let usdc_mint = usdc_mint_keypair.pubkey();

    let mint_rent = svm.minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN);

    let create_account_ix = create_account(
        &payer.pubkey(),
        &usdc_mint,
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::ID
    );

    let init_mint_ix = spl_token::instruction
        ::initialize_mint(&spl_token::ID, &usdc_mint, &payer.pubkey(), None, 6)
        .unwrap();

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[create_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &blockhash
    );
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[payer, &usdc_mint_keypair]
    ).unwrap();

    svm.send_transaction(tx).expect("failed to create usdc mint");

    usdc_mint
}

#[test]
fn test_initialize() {
    let program_id = ANCHOR_MARKET_ID;
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_market.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // create a local mock USDC mint
    let usdc_mint = create_usdc_mint(&mut svm, &payer);

    let market_id: u32 = 1;
    let expiry_ts: i64 = 1893456000;

    let market = get_market_pda(market_id);
    let collateral_vault = get_collateral_vault_pda(market_id);
    let outcome_a_mint = get_outcome_a_mint_pda(market_id);
    let outcome_b_mint = get_outcome_b_mint_pda(market_id);

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
}
