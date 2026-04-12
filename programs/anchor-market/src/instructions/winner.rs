use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::instruction::AuthorityType,
    token_interface::{ self, Mint, SetAuthority, TokenInterface },
};
use crate::{ Market, error::PredictionMarketError, state::WinningOutcome };

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
    pub market: Box<Account<'info, Market>>,

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
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> SetWinner<'info> {
    pub fn set_winner(&mut self, market_id: u32, winner: WinningOutcome) -> Result<()> {
        require!(!self.market.is_settled, PredictionMarketError::MarketAlreadySettled);
        require!(
            Clock::get()?.unix_timestamp < self.market.expiry_ts,
            PredictionMarketError::MarketExpired
        );

        require!(
            matches!(winner, WinningOutcome::OutcomeA | WinningOutcome::OutcomeB),
            PredictionMarketError::InvalidWinningOutcome
        );

        // settle the market

        self.market.is_settled = true;
        self.market.winning_outcome = Some(winner);

        // now nobody should be able to mint outcome a and outcome b tokens to anyone
        // not even the market maker or the market itself.

        let binding = self.market.market_id.to_le_bytes();

        let seeds = &[b"market".as_ref(), binding.as_ref(), &[self.market.bump]];
        let signer = &[&seeds[..]];

        token_interface::set_authority(
            CpiContext::new_with_signer(
                self.token_program.key(),
                SetAuthority {
                    current_authority: self.market.to_account_info(),
                    account_or_mint: self.outcome_a_mint.to_account_info(),
                },
                signer
            ),
            AuthorityType::MintTokens,
            None
        )?;

        token_interface::set_authority(
            CpiContext::new_with_signer(
                self.token_program.key(),
                SetAuthority {
                    current_authority: self.market.to_account_info(),
                    account_or_mint: self.outcome_b_mint.to_account_info(),
                },
                signer
            ),
            AuthorityType::MintTokens,
            None
        )?;

        msg!("Winner is settled");
        Ok(())
    }
}
