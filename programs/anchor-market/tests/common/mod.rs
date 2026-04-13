use spl_associated_token_account::get_associated_token_address;
use ::{
    anchor_market::{ ID as ANCHOR_MARKET_ID },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{ Message, VersionedMessage },
    solana_program::program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_system_interface::instruction::create_account,
    solana_transaction::versioned::VersionedTransaction,
};

pub fn get_market_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"market", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

pub fn get_collateral_vault_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

pub fn get_outcome_a_mint_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"outcome_a", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

pub fn get_outcome_b_mint_pda(market_id: u32) -> Pubkey {
    Pubkey::find_program_address(&[b"outcome_b", &market_id.to_le_bytes()], &ANCHOR_MARKET_ID).0
}

pub fn create_usdc_mint(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
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

pub fn get_user_token_account(mint: &Pubkey, owner: &Pubkey) -> Pubkey {
    get_associated_token_address(owner, mint)
}

pub fn create_ata(svm: &mut LiteSVM, payer: &Keypair, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let ata = get_associated_token_address(owner, mint);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        owner,
        mint,
        &spl_token::ID
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).expect("failed to create ATA");
    ata
}

pub fn mint_to(svm: &mut LiteSVM, payer: &Keypair, mint: &Pubkey, dest: &Pubkey, amount: u64) {
    let ix = spl_token::instruction
        ::mint_to(
            &spl_token::ID,
            mint,
            dest,
            &payer.pubkey(), // mint authority
            &[],
            amount
        )
        .unwrap();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).expect("failed to mint tokens");
}
