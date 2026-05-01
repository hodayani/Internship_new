use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};

declare_id!("CdnUHa1Kcy9ZizpocA3oCj3vrB2Jd9WJnTVVo3ypiNK2");

// ─────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────

// Default fee split in basis points (must always sum to 10000)
const DEFAULT_MERCHANT_BPS: u64 = 8000; // 80% to merchant
const DEFAULT_BURN_BPS: u64 = 1500;     // 15% burned forever
const DEFAULT_TREASURY_BPS: u64 = 500;  // 5% to protocol treasury

// Max daily spend per user: 100,000 SST
const MAX_DAILY_SPEND: u64 = 100_000_000_000_000;

// Seconds in a day
const SECONDS_PER_DAY: i64 = 86400;

// ─────────────────────────────────────────────────────────────
// PROGRAM
// ─────────────────────────────────────────────────────────────
#[program]
pub mod spend_router {
    use super::*;

    // ── ADMIN ────────────────────────────────────────────────

    // One-time setup. Creates the router config.
    // treasury_wallet: the protocol wallet that receives the 5% cut.
    pub fn initialize_router(
        ctx: Context<InitializeRouter>,
        sst_mint: Pubkey,
        treasury_wallet: Pubkey,
    ) -> Result<()> {
        let router = &mut ctx.accounts.router_config;
        router.admin = ctx.accounts.admin.key();
        router.sst_mint = sst_mint;
        router.treasury_wallet = treasury_wallet;
        router.merchant_bps = DEFAULT_MERCHANT_BPS;
        router.burn_bps = DEFAULT_BURN_BPS;
        router.treasury_bps = DEFAULT_TREASURY_BPS;
        router.total_volume = 0;
        router.total_burned = 0;
        router.total_treasury = 0;
        router.spend_count = 0;
        router.paused = false;
        Ok(())
    }

    // Admin updates global fee splits or pauses the router.
    // All three must be provided and must sum to exactly 10000.
    pub fn set_router_params(
        ctx: Context<SetRouterParams>,
        merchant_bps: u64,
        burn_bps: u64,
        treasury_bps: u64,
        paused: Option<bool>,
    ) -> Result<()> {
        let router = &mut ctx.accounts.router_config;

        // Validate sum = 100%
        let total = merchant_bps
            .checked_add(burn_bps)
            .and_then(|s| s.checked_add(treasury_bps))
            .ok_or(RouterError::Overflow)?;
        require!(total == 10000, RouterError::InvalidFeeSplit);

        // No single party can take more than 95%
        require!(merchant_bps <= 9500, RouterError::InvalidFeeSplit);
        require!(burn_bps <= 9500, RouterError::InvalidFeeSplit);
        require!(treasury_bps <= 9500, RouterError::InvalidFeeSplit);

        router.merchant_bps = merchant_bps;
        router.burn_bps = burn_bps;
        router.treasury_bps = treasury_bps;

        if let Some(p) = paused {
            router.paused = p;
        }

        emit!(RouterParamsUpdated {
            merchant_bps,
            burn_bps,
            treasury_bps,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // Admin registers a new merchant.
    // merchant PDA now includes router_config key
    // so multiple routers never share merchant accounts.
    pub fn register_merchant(
        ctx: Context<RegisterMerchant>,
        merchant_id: [u8; 32],
        name: [u8; 64],
        fee_override_bps: Option<u64>,
    ) -> Result<()> {
        let router = &ctx.accounts.router_config;

        // Validate override at registration time.
        // merchant_bps + burn_bps must not exceed 10000.
        // Treasury always gets the remainder.
        if let Some(fee) = fee_override_bps {
            require!(fee >= 500, RouterError::InvalidFeeSplit); // min 5%
            require!(fee <= 9500, RouterError::InvalidFeeSplit); // max 95%
            let total_without_treasury = fee
                .checked_add(router.burn_bps)
                .ok_or(RouterError::Overflow)?;
            require!(
                total_without_treasury <= 10000,
                RouterError::InvalidFeeSplit
            );
        }

        let merchant = &mut ctx.accounts.merchant_account;
        merchant.authority = ctx.accounts.merchant_authority.key();
        merchant.merchant_id = merchant_id;
        merchant.name = name;
        merchant.total_received = 0;
        merchant.total_transactions = 0;
        merchant.is_active = true;
        merchant.fee_override_bps = fee_override_bps;
        merchant.registered_at = Clock::get()?.unix_timestamp;

        emit!(MerchantRegistered {
            merchant_id,
            authority: ctx.accounts.merchant_authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // Admin deactivates a fraudulent or non-compliant merchant.
    pub fn deactivate_merchant(ctx: Context<AdminMerchantAction>) -> Result<()> {
        ctx.accounts.merchant_account.is_active = false;
        emit!(MerchantDeactivated {
            merchant_id: ctx.accounts.merchant_account.merchant_id,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    // Admin reactivates a previously deactivated merchant.
    pub fn reactivate_merchant(ctx: Context<AdminMerchantAction>) -> Result<()> {
        ctx.accounts.merchant_account.is_active = true;
        Ok(())
    }

    // ── USER ─────────────────────────────────────────────────

    // Core spend instruction.
    //
    // 1: merchant_bps is validated against burn_bps INSIDE spend too,
    //        so even if params changed after registration, split is safe.
    // 2: receipt_nonce passed from app/backend — no duplicate receipts.
    //
    // receipt_nonce: 32 random bytes from  backend.
    //               Used to derive a unique receipt address.
    //               Backend should store this to look up the receipt later.
    pub fn spend(
        ctx: Context<Spend>,
        item_id: [u8; 32],
        expected_amount: u64,
        receipt_nonce: [u8; 32],
    ) -> Result<()> {
        let router = &ctx.accounts.router_config;

        // Guard 1: Router not paused
        require!(!router.paused, RouterError::RouterPaused);

        // Guard 2: Amount positive
        require!(expected_amount > 0, RouterError::InvalidAmount);

        // Guard 3: Merchant active
        require!(
            ctx.accounts.merchant_account.is_active,
            RouterError::MerchantInactive
        );

        // Guard 4: Treasury account belongs to the configured treasury wallet
        require!(
            ctx.accounts.treasury_sst_account.owner == router.treasury_wallet,
            RouterError::InvalidTreasury
        );

        // Guard 5: User has enough balance
        require!(
            ctx.accounts.user_sst_account.amount >= expected_amount,
            RouterError::InsufficientBalance
        );

        let clock = Clock::get()?;
        let now = clock.unix_timestamp;

        // Guard 6: Daily spend limit
        let tracker = &mut ctx.accounts.user_spend_tracker;
        let day_start = (now / SECONDS_PER_DAY) * SECONDS_PER_DAY;
        if tracker.last_reset_day < day_start {
            tracker.spent_today = 0;
            tracker.last_reset_day = day_start;
        }
        let new_daily_total = tracker
            .spent_today
            .checked_add(expected_amount)
            .ok_or(RouterError::Overflow)?;
        require!(new_daily_total <= MAX_DAILY_SPEND, RouterError::DailyLimitExceeded);

        //  Determine merchant_bps and validate against burn_bps
        // Use override if set, otherwise use global default.
        let merchant_bps = ctx
            .accounts
            .merchant_account
            .fee_override_bps
            .unwrap_or(router.merchant_bps);

        // Validate: merchant_bps + burn_bps must not exceed 10000
        // This catches any edge case where burn_bps changed after merchant registration
        let merchant_plus_burn = merchant_bps
            .checked_add(router.burn_bps)
            .ok_or(RouterError::Overflow)?;
        require!(merchant_plus_burn <= 10000, RouterError::InvalidFeeSplit);

        // Calculate amounts
        //  treasury is always the exact remainder
        // This means treasury_bps is advisory when override is used,
        // but total always equals expected_amount exactly (no dust).
        let merchant_amount = expected_amount
            .checked_mul(merchant_bps)
            .ok_or(RouterError::Overflow)?
            .checked_div(10000)
            .ok_or(RouterError::Overflow)?;

        let burn_amount = expected_amount
            .checked_mul(router.burn_bps)
            .ok_or(RouterError::Overflow)?
            .checked_div(10000)
            .ok_or(RouterError::Overflow)?;

        // Treasury gets the exact remainder — no dust, no rounding errors
        let treasury_amount = expected_amount
            .checked_sub(merchant_amount)
            .and_then(|r| r.checked_sub(burn_amount))
            .ok_or(RouterError::Overflow)?;

        // 1. Transfer to merchant
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_sst_account.to_account_info(),
                    to: ctx.accounts.merchant_sst_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            merchant_amount,
        )?;

        // 2. Burn — permanently removes SST from supply (deflationary)
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.sst_mint.to_account_info(),
                    from: ctx.accounts.user_sst_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            burn_amount,
        )?;

        // 3. Transfer to treasury
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_sst_account.to_account_info(),
                    to: ctx.accounts.treasury_sst_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            treasury_amount,
        )?;

        // Update router stats
        let router_config = &mut ctx.accounts.router_config;
        router_config.total_volume = router_config
            .total_volume
            .checked_add(expected_amount)
            .ok_or(RouterError::Overflow)?;
        router_config.total_burned = router_config
            .total_burned
            .checked_add(burn_amount)
            .ok_or(RouterError::Overflow)?;
        router_config.total_treasury = router_config
            .total_treasury
            .checked_add(treasury_amount)
            .ok_or(RouterError::Overflow)?;
        router_config.spend_count = router_config
            .spend_count
            .checked_add(1)
            .ok_or(RouterError::Overflow)?;

        // Update merchant stats
        let merchant = &mut ctx.accounts.merchant_account;
        merchant.total_received = merchant
            .total_received
            .checked_add(merchant_amount)
            .ok_or(RouterError::Overflow)?;
        merchant.total_transactions = merchant
            .total_transactions
            .checked_add(1)
            .ok_or(RouterError::Overflow)?;

        // Update daily tracker
        tracker.owner = ctx.accounts.user.key();
        tracker.spent_today = new_daily_total;

        // Write receipt — Flutter app reads this to confirm purchase
        let receipt = &mut ctx.accounts.redemption_receipt;
        receipt.buyer = ctx.accounts.user.key();
        receipt.merchant_id = ctx.accounts.merchant_account.merchant_id;
        receipt.item_id = item_id;
        receipt.receipt_nonce = receipt_nonce;
        receipt.total_amount = expected_amount;
        receipt.merchant_amount = merchant_amount;
        receipt.burn_amount = burn_amount;
        receipt.treasury_amount = treasury_amount;
        receipt.timestamp = now;
        receipt.status = RedemptionStatus::Fulfilled;

        emit!(SpendProcessed {
            receipt_id: ctx.accounts.redemption_receipt.key(),
            user: ctx.accounts.user.key(),
            merchant_id: ctx.accounts.merchant_account.merchant_id,
            item_id,
            receipt_nonce,
            total_amount: expected_amount,
            merchant_amount,
            burn_amount,
            treasury_amount,
            timestamp: now,
        });

        Ok(())
    }

    // Buyer or admin raises a dispute on a fulfilled receipt.
    // Firebase listens for the RedemptionDisputed event and
    // triggers the merchant webhook to initiate the refund flow.
    pub fn dispute_redemption(
        ctx: Context<DisputeRedemption>,
        reason: [u8; 128],
    ) -> Result<()> {
        let receipt_key = ctx.accounts.redemption_receipt.key();

let receipt = &mut ctx.accounts.redemption_receipt;

require!(
    receipt.status == RedemptionStatus::Fulfilled,
    RouterError::InvalidReceiptStatus
);

require!(
    ctx.accounts.authority.key() == receipt.buyer
        || ctx.accounts.authority.key() == ctx.accounts.router_config.admin,
    RouterError::Unauthorized
);

receipt.status = RedemptionStatus::Disputed;

emit!(RedemptionDisputed {
    receipt_id: receipt_key,
    buyer: receipt.buyer,
    merchant_id: receipt.merchant_id,
    reason,
    timestamp: Clock::get()?.unix_timestamp,
});
        Ok(())
    }

    // Admin resolves a dispute.
    // If refunded=true: Firebase Cloud Function transfers tokens back.
    // If refunded=false: dispute rejected, no refund.
    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        refunded: bool,
    ) -> Result<()> {
        let receipt = &mut ctx.accounts.redemption_receipt;
        require!(
            receipt.status == RedemptionStatus::Disputed,
            RouterError::InvalidReceiptStatus
        );
        receipt.status = if refunded {
            RedemptionStatus::Refunded
        } else {
            RedemptionStatus::DisputeRejected
        };
        emit!(DisputeResolved {
            receipt_id: ctx.accounts.redemption_receipt.key(),
            refunded,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// ACCOUNT CONTEXTS
// ─────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeRouter<'info> {
    #[account(init, payer = admin, space = RouterConfig::LEN)]
    pub router_config: Account<'info, RouterConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetRouterParams<'info> {
    #[account(mut, has_one = admin)]
    pub router_config: Account<'info, RouterConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(merchant_id: [u8; 32])]
pub struct RegisterMerchant<'info> {
    #[account(has_one = admin)]
    pub router_config: Account<'info, RouterConfig>,

    //  Seeds include router_config.key() so merchants are scoped
    // to this specific router deployment. Multiple routers never share accounts.
    #[account(
        init,
        payer = admin,
        space = MerchantAccount::LEN,
        seeds = [
            b"merchant",
            router_config.key().as_ref(),
            merchant_id.as_ref()
        ],
        bump,
    )]
    pub merchant_account: Account<'info, MerchantAccount>,

    /// CHECK: merchant wallet stored as authority, no signing needed at registration
    pub merchant_authority: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminMerchantAction<'info> {
    #[account(has_one = admin)]
    pub router_config: Account<'info, RouterConfig>,
    #[account(mut)]
    pub merchant_account: Account<'info, MerchantAccount>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(item_id: [u8; 32], expected_amount: u64, receipt_nonce: [u8; 32])]
pub struct Spend<'info> {
    #[account(mut)]
    pub router_config: Account<'info, RouterConfig>,

    #[account(mut)]
    pub merchant_account: Account<'info, MerchantAccount>,

    // User's SST account — source of payment
    // Validated: must be owned by user and use correct mint
    #[account(
        mut,
        constraint = user_sst_account.owner == user.key() @ RouterError::Unauthorized,
        constraint = user_sst_account.mint == router_config.sst_mint @ RouterError::InvalidMint,
    )]
    pub user_sst_account: Account<'info, TokenAccount>,

    // Merchant's SST account — receives merchant cut
    // Validated: must be owned by the registered merchant authority
    #[account(
        mut,
        constraint = merchant_sst_account.owner == merchant_account.authority @ RouterError::InvalidMerchantAccount,
        constraint = merchant_sst_account.mint == router_config.sst_mint @ RouterError::InvalidMint,
    )]
    pub merchant_sst_account: Account<'info, TokenAccount>,

    // Treasury SST account — receives treasury cut
    // Validated: owner checked against config inside instruction
    #[account(
        mut,
        constraint = treasury_sst_account.mint == router_config.sst_mint @ RouterError::InvalidMint,
    )]
    pub treasury_sst_account: Account<'info, TokenAccount>,

    // SST mint — needed for burn instruction
    #[account(
        mut,
        address = router_config.sst_mint @ RouterError::InvalidMint,
    )]
    pub sst_mint: Account<'info, Mint>,

    // Per-user daily spend tracker
    // Resets automatically every 24 hours inside the instruction
    #[account(
        init_if_needed,
        payer = user,
        space = UserSpendTracker::LEN,
        seeds = [b"spend_tracker", user.key().as_ref()],
        bump,
    )]
    pub user_spend_tracker: Account<'info, UserSpendTracker>,

    //  Receipt seeded with receipt_nonce from backend
    // Backend generates a unique nonce per purchase and stores it
    // alongside the receipt address for later lookup
    #[account(
        init,
        payer = user,
        space = RedemptionReceipt::LEN,
        seeds = [
            b"receipt",
            user.key().as_ref(),
            receipt_nonce.as_ref(),
        ],
        bump,
    )]
    pub redemption_receipt: Account<'info, RedemptionReceipt>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DisputeRedemption<'info> {
    pub router_config: Account<'info, RouterConfig>,
    #[account(mut)]
    pub redemption_receipt: Account<'info, RedemptionReceipt>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(has_one = admin)]
    pub router_config: Account<'info, RouterConfig>,
    #[account(mut)]
    pub redemption_receipt: Account<'info, RedemptionReceipt>,
    pub admin: Signer<'info>,
}

// ─────────────────────────────────────────────────────────────
// ON-CHAIN STATE
// ─────────────────────────────────────────────────────────────

// Global router config. One per deployment.
#[account]
pub struct RouterConfig {
    pub admin: Pubkey,           // 32
    pub sst_mint: Pubkey,        // 32
    pub treasury_wallet: Pubkey, // 32
    pub merchant_bps: u64,       // 8
    pub burn_bps: u64,           // 8
    pub treasury_bps: u64,       // 8
    pub total_volume: u64,       // 8
    pub total_burned: u64,       // 8
    pub total_treasury: u64,     // 8
    pub spend_count: u64,        // 8
    pub paused: bool,            // 1
}

impl RouterConfig {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}

// One account per registered merchant.
#[account]
pub struct MerchantAccount {
    pub authority: Pubkey,             // 32
    pub merchant_id: [u8; 32],         // 32
    pub name: [u8; 64],                // 64
    pub total_received: u64,           // 8
    pub total_transactions: u64,       // 8
    pub is_active: bool,               // 1
    pub fee_override_bps: Option<u64>, // 9  (1 discriminant + 8 value)
    pub registered_at: i64,            // 8
}

impl MerchantAccount {
    pub const LEN: usize = 8 + 32 + 32 + 64 + 8 + 8 + 1 + 9 + 8;
}

// Per-user daily spend tracker.
#[account]
pub struct UserSpendTracker {
    pub owner: Pubkey,       // 32
    pub spent_today: u64,    // 8
    pub last_reset_day: i64, // 8
}

impl UserSpendTracker {
    pub const LEN: usize = 8 + 32 + 8 + 8;
}

// On-chain receipt for every purchase.
// Flutter app reads this to confirm transaction.
// Firebase listens for SpendProcessed events to trigger webhooks.
// Firebase listens for RedemptionDisputed events to trigger webhooks.
// receipt_nonce stored on-chain for backend lookup.
#[account]
pub struct RedemptionReceipt {
    pub buyer: Pubkey,            // 32
    pub merchant_id: [u8; 32],    // 32
    pub item_id: [u8; 32],        // 32
    pub receipt_nonce: [u8; 32],  // 32
    pub total_amount: u64,        // 8
    pub merchant_amount: u64,     // 8
    pub burn_amount: u64,         // 8
    pub treasury_amount: u64,     // 8
    pub timestamp: i64,           // 8
    pub status: RedemptionStatus, // 1
}

impl RedemptionReceipt {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum RedemptionStatus {
    Fulfilled,
    Disputed,
    Refunded,
    DisputeRejected,
}

// ─────────────────────────────────────────────────────────────
// EVENTS
// ─────────────────────────────────────────────────────────────

#[event]
pub struct SpendProcessed {
    pub receipt_id: Pubkey,
    pub user: Pubkey,
    pub merchant_id: [u8; 32],
    pub item_id: [u8; 32],
    pub receipt_nonce: [u8; 32],
    pub total_amount: u64,
    pub merchant_amount: u64,
    pub burn_amount: u64,
    pub treasury_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct MerchantRegistered {
    pub merchant_id: [u8; 32],
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MerchantDeactivated {
    pub merchant_id: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct RouterParamsUpdated {
    pub merchant_bps: u64,
    pub burn_bps: u64,
    pub treasury_bps: u64,
    pub timestamp: i64,
}

#[event]
pub struct RedemptionDisputed {
    pub receipt_id: Pubkey,
    pub buyer: Pubkey,
    pub merchant_id: [u8; 32],
    pub reason: [u8; 128],
    pub timestamp: i64,
}

#[event]
pub struct DisputeResolved {
    pub receipt_id: Pubkey,
    pub refunded: bool,
    pub timestamp: i64,
}

// ─────────────────────────────────────────────────────────────
// ERRORS
// ─────────────────────────────────────────────────────────────
#[error_code]
pub enum RouterError {
    #[msg("Router is paused")]
    RouterPaused,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Merchant is not active")]
    MerchantInactive,
    #[msg("Fee splits must sum to 10000 bps and each be between 0-9500")]
    InvalidFeeSplit,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Daily spend limit exceeded (max 100,000 SST per day)")]
    DailyLimitExceeded,
    #[msg("Insufficient SST balance")]
    InsufficientBalance,
    #[msg("Treasury account does not match config")]
    InvalidTreasury,
    #[msg("Invalid SST mint")]
    InvalidMint,
    #[msg("Merchant token account does not belong to merchant")]
    InvalidMerchantAccount,
    #[msg("Not authorized")]
    Unauthorized,
    #[msg("Receipt is not in the correct status for this operation")]
    InvalidReceiptStatus,
}