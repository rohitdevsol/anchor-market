use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token };
use crate::Market;

#[derive(Accounts)]
#[instruction(market_id :u32)]
pub struct SetWinner<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id,
        constraint = market.authority == authority.key()
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        constraint = outcome_a_mint.key() == market.outcome_a_mint
    )]
    pub outcome_a_mint: Account<'info, Mint>,
    #[account(
        mut,
         constraint = outcome_b_mint.key() == market.outcome_b_mint
    )]
    pub outcome_b_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}
