use anchor_lang::prelude::*;

declare_id!("9vmHJDhKHeFjSzUbk9G5npqBKxGaGGpsY6B3EFq2kBpD");

#[program]
pub mod gaming_wager {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.player_one = None;
        game.player_two = None;
        game.wager = 0;
        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, player_wallet: Pubkey, wager_amount: u64) -> Result<()> {
        let game = &mut ctx.accounts.game;

        if game.player_one.is_none() {
            // Player one joins
            game.player_one = Some(player_wallet);
        } else if game.player_two.is_none() {
            // Player two joins
            game.player_two = Some(player_wallet);
        } else {
            return Err(ErrorCode::GameFull.into());
        }

        // Update the wager amount (assumes both players wager the same amount)
        game.wager += wager_amount;

        Ok(())
    }

    pub fn resolve_game(ctx: Context<ResolveGame>, winner_wallet: Option<Pubkey>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let pool_account = &mut ctx.accounts.pool_account;

        if game.player_one.is_none() || game.player_two.is_none() {
            return Err(ErrorCode::NotEnoughPlayers.into());
        }

        match winner_wallet {
            Some(winner) => {
                // Transfer the entire wager pool to the winner
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &pool_account.key(),
                    &winner,
                    game.wager,
                );
                anchor_lang::solana_program::program::invoke(
                    &ix,
                    &[pool_account.to_account_info(), ctx.accounts.winner.to_account_info()],
                )?;
            }
            None => {
                // No winner: retain the wager in the pool
            }
        }

        // Reset the game state
        game.player_one = None;
        game.player_two = None;
        game.wager = 0;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(init, payer = user, space = 8 + 64)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(init, payer = user, space = 8 + 64)]
    pub pool_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResolveGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    /// CHECK: Safe because this is only used for transfer
    #[account(mut)]
    pub pool_account: AccountInfo<'info>,
    /// CHECK: Safe because this is only used for transfer
    pub winner: AccountInfo<'info>,
}

#[account]
pub struct Game {
    pub player_one: Option<Pubkey>,
    pub player_two: Option<Pubkey>,
    pub wager: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The game already has two players.")]
    GameFull,
    #[msg("Not enough players have joined the game.")]
    NotEnoughPlayers,
}