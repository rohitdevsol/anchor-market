use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, MintTo, Token, TokenAccount, Transfer };
use crate::{ Market, error::PredictionMarketError };

#[derive(Accounts)]
#[instruction(market_id: u32)]
pub struct SplitToken<'info> {
    #[account(
        mut,
        seeds = [b"market", market.market_id.to_le_bytes().as_ref()],
        bump = market.bump,
        constraint = market.market_id == market_id
    )]
    pub market: Account<'info, Market>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = user_collateral.mint == market.collateral_mint,
        constraint = user_collateral.owner == user.key()
    )]
    pub user_collateral: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = collateral_vault.key() == market.collateral_vault
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

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

    #[account(
        mut,
        constraint = user_outcome_a.mint == market.outcome_a_mint,
        constraint = user_outcome_a.owner == user.key()
    )]
    pub user_outcome_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_outcome_b.mint == market.outcome_b_mint,
        constraint = user_outcome_b.owner == user.key()
    )]
    pub user_outcome_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> SplitToken<'info> {
    pub fn split_token(&mut self, market_id: u32, amount: u64) -> Result<()> {
        // we will get some amount from user and we need something

        // check if the market is settled
        require!(!self.market.is_settled, PredictionMarketError::MarketAlreadySettled);
        require!(amount > 0, PredictionMarketError::InvalidAmount);
        require!(
            Clock::get()?.unix_timestamp < self.market.expiry_ts,
            PredictionMarketError::MarketExpired
        );

        // user to the collateral vault
        token::transfer(
            CpiContext::new(*self.token_program.key, Transfer {
                from: self.user_collateral.to_account_info(),
                to: self.collateral_vault.to_account_info(),
                authority: self.user.to_account_info(),
            }),
            amount
        )?;

        // mint to user outcome accounts
        let binding = self.market.market_id.to_le_bytes();
        let signer_seeds = &[b"market".as_ref(), binding.as_ref(), &[self.market.bump]];
        let signer = &[&signer_seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                *self.token_program.key,
                MintTo {
                    authority: self.market.to_account_info(),
                    mint: self.outcome_a_mint.to_account_info(),
                    to: self.user_outcome_a.to_account_info(),
                },
                signer
            ),
            amount
        )?;

        token::mint_to(
            CpiContext::new_with_signer(
                *self.token_program.key,
                MintTo {
                    mint: self.outcome_b_mint.to_account_info(),
                    to: self.user_outcome_b.to_account_info(),
                    authority: self.market.to_account_info(),
                },
                signer
            ),
            amount
        )?;

        self.market.total_collateral_locked = self.market.total_collateral_locked
            .checked_add(amount)
            .ok_or(PredictionMarketError::MathOverflow)?;

        msg!("Minted {} outcome tokens for user", amount);

        Ok(())
    }
}
