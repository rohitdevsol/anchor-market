use anchor_lang::prelude::*;
use anchor_spl::{ token::Token, token_interface::{ Mint, TokenAccount, TokenInterface } };
use crate::{ Market, error::PredictionMarketError };

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market".as_ref(), market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub collateral_mint: InterfaceAccount<'info, Mint>, // USDC one

    #[account(
        init,
        payer = authority,
        token::mint = collateral_mint,
        token::authority = market,
        seeds = [b"vault".as_ref(), market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub collateral_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = market,
        seeds = [b"outcome_a", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub outcome_a_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = market,
        seeds = [b"outcome_b", market_id.to_le_bytes().as_ref()],
        bump
    )]
    pub outcome_b_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitializeMarket<'info> {
    pub fn initilize_market(
        &mut self,
        market_id: u32,
        expiry_ts: i64,
        bumps: InitializeMarketBumps
    ) -> Result<()> {
        // the expiry date should be in future

        let clock = Clock::get()?;

        // checking that the expiry date is of the future .. market should not have an expiry date of the past
        require!(
            expiry_ts > clock.unix_timestamp,
            PredictionMarketError::InvalidSettlementDeadline
        );

        self.market.set_inner(Market {
            authority: self.authority.key(),
            market_id,
            is_settled: false,
            expiry_ts,
            collateral_mint: self.collateral_mint.key(),
            collateral_vault: self.collateral_vault.key(),
            outcome_a_mint: self.outcome_a_mint.key(),
            outcome_b_mint: self.outcome_b_mint.key(),
            winning_outcome: None,
            total_collateral_locked: 0,
            bump: bumps.market,
        });

        Ok(())
    }
}
