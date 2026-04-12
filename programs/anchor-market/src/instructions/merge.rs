use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Burn, Mint, Token, TokenAccount, Transfer };
use crate::{ Market, error::PredictionMarketError };

#[derive(Accounts)]
#[instruction(market_id:u32)]
pub struct MergeToken<'info> {
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
        constraint = collateral_vault.key() == market.collateral_vault,
        constraint = collateral_vault.owner == market.key(),
        constraint = collateral_vault.mint == market.collateral_mint,
    )]
    pub collateral_vault: Account<'info, TokenAccount>,

    #[account(constraint = outcome_a_mint.key() == market.outcome_a_mint)]
    pub outcome_a_mint: Account<'info, Mint>,
    #[account(constraint = outcome_b_mint.key() == market.outcome_b_mint)]
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

impl<'info> MergeToken<'info> {
    pub fn merge_tokens(&mut self, market_id: u32, amount: u64) -> Result<()> {
        require!(!self.market.is_settled, PredictionMarketError::MarketAlreadySettled);
        require!(amount > 0, PredictionMarketError::InvalidAmount);
        require!(
            Clock::get()?.unix_timestamp < self.market.expiry_ts,
            PredictionMarketError::MarketExpired
        );

        // we need to get the minimum of both
        let payout = std::cmp::min(self.user_outcome_a.amount, self.user_outcome_b.amount);
        require!(payout > 0, PredictionMarketError::InvalidAmount);

        let binding = self.market.market_id.to_be_bytes();

        let seeds = &[b"market".as_ref(), binding.as_ref(), &[self.market.bump]];

        let signer = &[&seeds[..]];

        // transfer from the collateral vault to the user collateral vault
        token::transfer(
            CpiContext::new_with_signer(
                *self.token_program.key,
                Transfer {
                    from: self.collateral_vault.to_account_info(),
                    to: self.user_collateral.to_account_info(),
                    authority: self.market.to_account_info(),
                },
                signer
            ),
            payout
        )?;

        // jitna b payout diya hai utna user me se burn krdo
        // burn the outcome a tokens from user .. payout
        token::burn(
            CpiContext::new(*self.token_program.key, Burn {
                from: self.user_outcome_a.to_account_info(),
                authority: self.user.to_account_info(),
                mint: self.outcome_a_mint.to_account_info(),
            }),
            payout
        )?;

        // burn the outcome b tokens from user .. payout

        token::burn(
            CpiContext::new(*self.token_program.key, Burn {
                mint: self.outcome_b_mint.to_account_info(),
                from: self.user_outcome_b.to_account_info(),
                authority: self.user.to_account_info(),
            }),
            payout
        )?;

        Ok(())
    }
}
