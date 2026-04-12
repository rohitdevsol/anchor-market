use anchor_lang::prelude::*;
use anchor_spl::{ token::Token, token_interface::{ Mint, TokenAccount } };
use crate::Market;

#[derive(Accounts)]
#[instruction(market_id:u32)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        mut,
        constraint = user_collateral.mint == market.collateral_mint,
        constraint = user_collateral.owner == user.key()
    )]
    pub user_collateral: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault
    )]
    pub collateral_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = outcome_a_mint.key() == market.outcome_a_mint
    )]
    pub outcome_a_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = outcome_b_mint.key() == market.outcome_b_mint
    )]
    pub outcome_b_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        constraint = user_outcome_a.mint == market.outcome_a_mint,
        constraint = user_outcome_a.owner == user.key()
    )]
    pub user_outcome_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_outcome_b.mint == market.outcome_b_mint,
        constraint = user_outcome_b.owner == user.key()
    )]
    pub user_outcome_b: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
