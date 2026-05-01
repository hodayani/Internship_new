use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("BweiTb9neZgixwdb7bAiy66BtoA9sPALrBYjTnDp6JQ3");

const DEFAULT_APR_BPS: u64 = 1000; // 10%
const EARLY_UNSTAKE_PENALTY_BPS: u64 = 500; // 5%
const MIN_LOCK_SECONDS: i64 = 7 * 24 * 60 * 60; // 7 days
const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
//tells Anchor that this module contains on-chain instructions.

#[program]
pub mod staking_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault_config;

        vault.admin = ctx.accounts.admin.key();
        vault.sst_mint = ctx.accounts.sst_mint.key();
        vault.vault_authority_bump = ctx.bumps.vault_authority;
        vault.apr_bps = DEFAULT_APR_BPS;
        vault.total_staked = 0;
        vault.paused = false;

        Ok(())
    }

    pub fn set_vault_params(
        ctx: Context<SetVaultParams>,
        apr_bps: Option<u64>,
        paused: Option<bool>,
    ) -> Result<()> {
        
        let vault = &mut ctx.accounts.vault_config;
//If the caller provided a new APR value, enter this block.Reject APR above 100%.Save the new APR.
        if let Some(apr) = apr_bps {
            require!(apr <= 10_000, StakingError::InvalidApr); // max 100%
            vault.apr_bps = apr;
        }

        if let Some(p) = paused {
            vault.paused = p;
        }

        Ok(())
    }
//admin add SST into the reward pool.
    pub fn fund_reward_pool(ctx: Context<FundRewardPool>, amount: u64) -> Result<()> {
        require!(amount > 0, StakingError::InvalidAmount);
//Create a CPI context. CPI means Cross Program Invocation. It allows this program to call another program (in this case, the SPL Token program) to perform actions like token transfers. The context includes the accounts involved in the transfer and the authority that has permission to move the tokens.
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.admin_sst_account.to_account_info(),
                    to: ctx.accounts.reward_pool_sst_account.to_account_info(),
                    authority: ctx.accounts.admin.to_account_info(),
                },
            ),
            amount,
        )?;
//Emit an event after success.Log who funded, how much, and when.
        emit!(RewardPoolFunded {
            admin: ctx.accounts.admin.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
        lock_duration_seconds: i64,
    ) -> Result<()> {

        let vault = &mut ctx.accounts.vault_config;
require!(!vault.paused, StakingError::VaultPaused);
        require!(amount > 0, StakingError::InvalidAmount);
        //The lock duration must be at least 7 days. This encourages longer-term staking and prevents abuse of the early unstake penalty mechanism.
        require!(
            lock_duration_seconds >= MIN_LOCK_SECONDS,
            StakingError::LockTooShort
        );

        let now = Clock::get()?.unix_timestamp;

        token::transfer(
            //Use the token program.Transfer the staked amount from the user's token account to the vault's token account. The user must have approved this program to spend their tokens beforehand.
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_sst_account.to_account_info(),
                    to: ctx.accounts.vault_sst_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        let position = &mut ctx.accounts.stake_position;
        position.owner = ctx.accounts.user.key();
        position.amount_staked = amount;
        position.staked_at = now;
        position.lock_until = now
            .checked_add(lock_duration_seconds)
            .ok_or(StakingError::Overflow)?;
        position.last_yield_claimed_at = now;
        position.total_yield_claimed = 0;
        position.apr_bps_at_stake = vault.apr_bps;
        //This position is active. It will be marked closed when the user unstakes.
        position.is_closed = false;
//Increase the vault’s total staked value. This is used for analytics and could be used in future features like dynamic APR based on total staked.
        vault.total_staked = vault
            .total_staked
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;

        emit!(Staked {
            user: ctx.accounts.user.key(),
            amount,
            lock_until: position.lock_until,
            timestamp: now,
        });

        Ok(())
    }
//lets the user claim only rewards. It does not affect the staked amount or lock period. Users can call this multiple times to claim rewards as they accrue, without needing to unstake.
    pub fn claim_yield(ctx: Context<ClaimYield>) -> Result<()> {
        let vault = &ctx.accounts.vault_config;
        require!(!vault.paused, StakingError::VaultPaused);

        let now = Clock::get()?.unix_timestamp;
        let position = &mut ctx.accounts.stake_position;

        require!(!position.is_closed, StakingError::PositionClosed);
        require!(
            position.owner == ctx.accounts.user.key(),
            StakingError::Unauthorized
        );

        let seconds_elapsed_i64 = now
            .checked_sub(position.last_yield_claimed_at)
            .ok_or(StakingError::InvalidTime)?;
        let seconds_elapsed = seconds_elapsed_i64 as u64;

        let yield_amount = calculate_yield(
            position.amount_staked,
            position.apr_bps_at_stake,
            seconds_elapsed,
        )?;

        require!(yield_amount > 0, StakingError::NoYieldAvailable);
        require!(
            ctx.accounts.reward_pool_sst_account.amount >= yield_amount,
            StakingError::InsufficientRewardPool
        );

        let vault_config_key = ctx.accounts.vault_config.key();
        let bump = vault.vault_authority_bump;
        let seeds: &[&[u8]] = &[
            b"vault_authority",
            vault_config_key.as_ref(),
            &[bump],
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.reward_pool_sst_account.to_account_info(),
                    to: ctx.accounts.user_sst_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[seeds],
            ),
            yield_amount,
        )?;

        position.last_yield_claimed_at = now;
        position.total_yield_claimed = position
            .total_yield_claimed
            .checked_add(yield_amount)
            .ok_or(StakingError::Overflow)?;

        emit!(YieldClaimed {
            user: ctx.accounts.user.key(),
            yield_amount,
            timestamp: now,
        });

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let vault_config_key = ctx.accounts.vault_config.key();
        let vault = &mut ctx.accounts.vault_config;
        require!(!vault.paused, StakingError::VaultPaused);

        let now = Clock::get()?.unix_timestamp;
        let position = &mut ctx.accounts.stake_position;

        require!(!position.is_closed, StakingError::PositionClosed);
        require!(
            position.owner == ctx.accounts.user.key(),
            StakingError::Unauthorized
        );

        let is_early = now < position.lock_until;
        let staked_amount = position.amount_staked;
//If unstaking early, reward is zero. If not, calculate pending yield since last claim. This ensures users get all the rewards they earned up until they unstake, even if they didn't call claim_yield recently.
        let pending_yield = if is_early {
            0
        } else {
            let seconds_elapsed_i64 = now
                .checked_sub(position.last_yield_claimed_at)
                .ok_or(StakingError::InvalidTime)?;
            let seconds_elapsed = seconds_elapsed_i64 as u64;

            calculate_yield(
                staked_amount,
                position.apr_bps_at_stake,
                seconds_elapsed,
            )?
        };

        let (principal_to_return, penalty_amount) = if is_early {
            let penalty = staked_amount
                .checked_mul(EARLY_UNSTAKE_PENALTY_BPS)
                .ok_or(StakingError::Overflow)?
                .checked_div(10_000)
                .ok_or(StakingError::Overflow)?;
            let principal_after_penalty = staked_amount
                .checked_sub(penalty)
                .ok_or(StakingError::Overflow)?;
            (principal_after_penalty, penalty)
        } else {
            (staked_amount, 0)
        };

        let bump = vault.vault_authority_bump;
        let seeds: &[&[u8]] = &[
            b"vault_authority",
            vault_config_key.as_ref(),
            &[bump],
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_sst_account.to_account_info(),
                    to: ctx.accounts.user_sst_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                &[seeds],
            ),
            principal_to_return,
        )?;

        if pending_yield > 0 {
            require!(
                ctx.accounts.reward_pool_sst_account.amount >= pending_yield,
                StakingError::InsufficientRewardPool
            );

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.reward_pool_sst_account.to_account_info(),
                        to: ctx.accounts.user_sst_account.to_account_info(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    &[seeds],
                ),
                pending_yield,
            )?;
        }

        vault.total_staked = vault
            .total_staked
            .checked_sub(staked_amount)
            .ok_or(StakingError::Overflow)?;

        position.is_closed = true;
        position.last_yield_claimed_at = now;
        position.total_yield_claimed = position
            .total_yield_claimed
            .checked_add(pending_yield)
            .ok_or(StakingError::Overflow)?;

        emit!(Unstaked {
            user: ctx.accounts.user.key(),
            principal_returned: principal_to_return,
            penalty: penalty_amount,
            yield_paid: pending_yield,
            early: is_early,
            timestamp: now,
        });

        Ok(())
    }
}

fn calculate_yield(amount: u64, apr_bps: u64, seconds_elapsed: u64) -> Result<u64> {
    //Convert to u128 to avoid overflow during multiplication.
    let yield_amount = (amount as u128)
        .checked_mul(apr_bps as u128)
        .ok_or(StakingError::Overflow)?
        .checked_mul(seconds_elapsed as u128)
        .ok_or(StakingError::Overflow)?
        .checked_div(10_000u128)
        .ok_or(StakingError::Overflow)?
        .checked_div(SECONDS_PER_YEAR as u128)
        .ok_or(StakingError::Overflow)? as u64;

    Ok(yield_amount)
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = admin,
        space = VaultConfig::LEN
    )]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: PDA authority used to control vault token accounts
    #[account(
        seeds = [b"vault_authority", vault_config.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"vault_tokens", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub vault_sst_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        seeds = [b"reward_pool", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub reward_pool_sst_account: Account<'info, TokenAccount>,

    pub sst_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetVaultParams<'info> {
    #[account(mut, has_one = admin)]
    pub vault_config: Account<'info, VaultConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct FundRewardPool<'info> {
    #[account(has_one = admin)]
    pub vault_config: Account<'info, VaultConfig>,

    /// CHECK: PDA authority used to own reward pool
    #[account(
        seeds = [b"vault_authority", vault_config.key().as_ref()],
        bump = vault_config.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"reward_pool", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub reward_pool_sst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = admin_sst_account.mint == sst_mint.key() @ StakingError::InvalidMint,
        constraint = admin_sst_account.owner == admin.key() @ StakingError::InvalidTokenOwner,
    )]
    pub admin_sst_account: Account<'info, TokenAccount>,

    pub sst_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        init,
        payer = user,
        space = StakePosition::LEN,
        seeds = [b"stake_position", user.key().as_ref()],
        bump
    )]
    pub stake_position: Account<'info, StakePosition>,

    /// CHECK: PDA authority controlling vault token accounts
    #[account(
        seeds = [b"vault_authority", vault_config.key().as_ref()],
        bump = vault_config.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault_tokens", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub vault_sst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_sst_account.mint == sst_mint.key() @ StakingError::InvalidMint,
        constraint = user_sst_account.owner == user.key() @ StakingError::InvalidTokenOwner,
    )]
    pub user_sst_account: Account<'info, TokenAccount>,

    #[account(address = vault_config.sst_mint)]
    pub sst_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        seeds = [b"stake_position", user.key().as_ref()],
        bump,
        constraint = stake_position.owner == user.key() @ StakingError::Unauthorized
    )]
    pub stake_position: Account<'info, StakePosition>,

    /// CHECK: PDA authority controlling vault token accounts
    #[account(
        seeds = [b"vault_authority", vault_config.key().as_ref()],
        bump = vault_config.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"reward_pool", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub reward_pool_sst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_sst_account.mint == sst_mint.key() @ StakingError::InvalidMint,
        constraint = user_sst_account.owner == user.key() @ StakingError::InvalidTokenOwner,
    )]
    pub user_sst_account: Account<'info, TokenAccount>,

    #[account(address = vault_config.sst_mint)]
    pub sst_mint: Account<'info, Mint>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub vault_config: Account<'info, VaultConfig>,

    #[account(
        mut,
        close = user,
        seeds = [b"stake_position", user.key().as_ref()],
        bump,
        constraint = stake_position.owner == user.key() @ StakingError::Unauthorized
    )]
    pub stake_position: Account<'info, StakePosition>,

    /// CHECK: PDA authority controlling vault token accounts
    #[account(
        seeds = [b"vault_authority", vault_config.key().as_ref()],
        bump = vault_config.vault_authority_bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"vault_tokens", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub vault_sst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool", vault_config.key().as_ref()],
        bump,
        token::mint = sst_mint,
        token::authority = vault_authority,
    )]
    pub reward_pool_sst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_sst_account.mint == sst_mint.key() @ StakingError::InvalidMint,
        constraint = user_sst_account.owner == user.key() @ StakingError::InvalidTokenOwner,
    )]
    pub user_sst_account: Account<'info, TokenAccount>,

    #[account(address = vault_config.sst_mint)]
    pub sst_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct VaultConfig {
    pub admin: Pubkey,
    pub sst_mint: Pubkey,
    pub vault_authority_bump: u8,
    pub apr_bps: u64,
    pub total_staked: u64,
    pub paused: bool,
}

impl VaultConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 1;
}

#[account]
pub struct StakePosition {
    pub owner: Pubkey,
    pub amount_staked: u64,
    pub staked_at: i64,
    pub lock_until: i64,
    pub last_yield_claimed_at: i64,
    pub total_yield_claimed: u64,
    pub apr_bps_at_stake: u64,
    pub is_closed: bool,
}

impl StakePosition {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}

#[event]
pub struct RewardPoolFunded {
    pub admin: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct Staked {
    pub user: Pubkey,
    pub amount: u64,
    pub lock_until: i64,
    pub timestamp: i64,
}

#[event]
pub struct YieldClaimed {
    pub user: Pubkey,
    pub yield_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct Unstaked {
    pub user: Pubkey,
    pub principal_returned: u64,
    pub penalty: u64,
    pub yield_paid: u64,
    pub early: bool,
    pub timestamp: i64,
}

#[error_code]
pub enum StakingError {
    #[msg("Vault is paused")]
    VaultPaused,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Lock duration must be at least 7 days")]
    LockTooShort,
    #[msg("No yield available to claim")]
    NoYieldAvailable,
    #[msg("Stake position is already closed")]
    PositionClosed,
    #[msg("Not authorized")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("APR cannot exceed 100%")]
    InvalidApr,
    #[msg("Invalid token mint")]
    InvalidMint,
    #[msg("Invalid token account owner")]
    InvalidTokenOwner,
    #[msg("Reward pool does not have enough tokens")]
    InsufficientRewardPool,
    #[msg("Invalid time calculation")]
    InvalidTime,
}