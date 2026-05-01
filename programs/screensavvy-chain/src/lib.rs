use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

declare_id!("4iPR5eXUbtfvCoVXEJfozZ7KJPYt4y3VtPh6CLLtnCXn");

// ─────────────────────────────────────────────────────────────
// PROGRAM INSTRUCTIONS
// ─────────────────────────────────────────────────────────────
#[program]
pub mod screensavvy_chain {
    use super::*;

    // One-time setup. Called once by admin after deployment.
    // Stores admin wallet, backend signer public key, SST mint address,
    // PDA bump, and default daily/per-claim limits.
    pub fn initialize(
        ctx: Context<Initialize>,
        backend_signer: Pubkey,
        sst_mint: Pubkey,
        mint_authority_bump: u8,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.backend_signer = backend_signer;
        config.sst_mint = sst_mint;
        config.paused = false;
        config.mint_authority_bump = mint_authority_bump;
        config.daily_budget = 1_000_000_000_000;       // 1,000 SST (9 decimals)
        config.remaining_daily_budget = 1_000_000_000_000;
        config.max_claim_amount = 100_000_000_000;     // 100 SST per claim
        Ok(())
    }

    // Admin can update backend signer key and minting limits.
    // Updating daily_budget also resets remaining_daily_budget.
    pub fn set_params(
        ctx: Context<SetParams>,
        backend_signer: Option<Pubkey>,
        daily_budget: Option<u64>,
        max_claim_amount: Option<u64>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if let Some(s) = backend_signer {
            config.backend_signer = s;
        }
        if let Some(b) = daily_budget {
            config.daily_budget = b;
            config.remaining_daily_budget = b;
        }
        if let Some(m) = max_claim_amount {
            config.max_claim_amount = m;
        }
        Ok(())
    }

    // Emergency stop. Admin can pause or unpause all minting instantly.
    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    // Core reward instruction. Called by the app after getting a signed claim
    // from the backend. Verifies the Ed25519 signature using Solana's native
    // precompile, checks all guards, then mints SST to the user.
    //
    // How the signature check works:
    // The CALLER must include a Solana Ed25519 instruction BEFORE this one
    // in the same transaction. This instruction uses Solana's native Ed25519
    // precompile program to verify the signature on-chain. Our program then
    // reads that instruction from the sysvar and confirms it matches.
    pub fn verify_and_mint(
        ctx: Context<VerifyAndMint>,
        payload: ClaimPayload,
        signature: [u8; 64],
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        // Guard 1: Program must not be paused
        require!(!config.paused, ErrorCode::Paused);

        // Guard 2: Payload recipient must match the account passed in
        require!(
            payload.recipient == ctx.accounts.recipient.key(),
            ErrorCode::RecipientMismatch
        );

        // Guard 3: Mint address must match config
        require!(
            ctx.accounts.sst_mint.key() == config.sst_mint,
            ErrorCode::InvalidMint
        );

        // Guard 4: Claim must not be expired
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= payload.expires_at,
            ErrorCode::ClaimExpired
        );

        // Guard 5: Verify Ed25519 signature using instruction sysvar
        // The Ed25519 verify instruction must be the instruction immediately
        // before this one in the transaction (relative index -1).
        let payload_bytes = payload.try_to_vec()?;
        verify_ed25519_ix(
            &ctx.accounts.ix_sysvar,
            &config.backend_signer.to_bytes(),
            &payload_bytes,
            &signature,
        )?;

        // Guard 6: Nonce must not already be used (replay protection)
        require!(
            !ctx.accounts.nonce_account.used,
            ErrorCode::NonceAlreadyUsed
        );

        // Guard 7: Amount must be positive and within caps
        require!(payload.amount > 0, ErrorCode::InvalidAmount);
        require!(
            payload.amount <= config.max_claim_amount,
            ErrorCode::AmountExceedsCap
        );
        require!(
            payload.amount <= config.remaining_daily_budget,
            ErrorCode::DailyBudgetExceeded
        );

        // Guard 8: Verify the mint authority PDA is correct
        let expected_authority = Pubkey::create_program_address(
            &[
                b"mint_authority",
                config.sst_mint.as_ref(),
                &[config.mint_authority_bump],
            ],
            ctx.program_id,
        )
        .map_err(|_| error!(ErrorCode::InvalidMintAuthority))?;
        require!(
            ctx.accounts.mint_authority.key() == expected_authority,
            ErrorCode::InvalidMintAuthority
        );

        // Mint SST to recipient's associated token account
        let sst_mint_key = config.sst_mint;
        let bump = config.mint_authority_bump;
        let seeds = &[
            b"mint_authority".as_ref(),
            sst_mint_key.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.sst_mint.to_account_info(),
                    to: ctx.accounts.recipient_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            payload.amount,
        )?;

        // Mark nonce as used so it can never be replayed
        ctx.accounts.nonce_account.used = true;

        // Decrement remaining daily budget
        config.remaining_daily_budget = config
            .remaining_daily_budget
            .saturating_sub(payload.amount);

        // Emit event so indexers and the frontend can track mints
        emit!(RewardMinted {
            recipient: payload.recipient,
            amount: payload.amount,
            challenge_id: payload.challenge_id,
            nonce: payload.nonce,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    // Reset remaining daily budget back to full.
    // Called by a Cloud Scheduler job every 24 hours.
    pub fn reset_daily_budget(ctx: Context<ResetDailyBudget>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.remaining_daily_budget = config.daily_budget;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// ED25519 SIGNATURE VERIFICATION
// Reads the previous instruction in the transaction and confirms
// it was a valid Ed25519 signature over our payload, signed by
// the backend signer key stored in config.
// ─────────────────────────────────────────────────────────────
fn verify_ed25519_ix(
    ix_sysvar: &AccountInfo,
    pubkey: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<()> {
    use anchor_lang::solana_program::sysvar::instructions::get_instruction_relative;
    use anchor_lang::solana_program::ed25519_program;

    // Get the instruction immediately before verify_and_mint
    let ix = get_instruction_relative(-1, ix_sysvar)
        .map_err(|_| error!(ErrorCode::InvalidSignature))?;

    // Must be Solana's native Ed25519 precompile program
    require!(
        ix.program_id == ed25519_program::ID,
        ErrorCode::InvalidSignature
    );

    // Ed25519 instruction data layout:
    // [0]      num_signatures (u8)
    // [1]      padding (u8)
    // [2..3]   signature_offset (u16 le)
    // [4..5]   signature_instruction_index (u16 le) -- 0xFFFF = this ix
    // [6..7]   public_key_offset (u16 le)
    // [8..9]   public_key_instruction_index (u16 le)
    // [10..11] message_data_offset (u16 le)
    // [12..13] message_data_size (u16 le)
    // [14..15] message_instruction_index (u16 le)
    // followed by: signature(64) + pubkey(32) + message
    let data = &ix.data;
    require!(data.len() >= 16, ErrorCode::InvalidSignature);

    let sig_offset = u16::from_le_bytes([data[2], data[3]]) as usize;
    let key_offset = u16::from_le_bytes([data[6], data[7]]) as usize;
    let msg_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let msg_size   = u16::from_le_bytes([data[12], data[13]]) as usize;

    require!(
        data.len() >= sig_offset + 64
            && data.len() >= key_offset + 32
            && data.len() >= msg_offset + msg_size,
        ErrorCode::InvalidSignature
    );

    // Verify the signature, pubkey, and message all match what we expect
    require!(
        &data[sig_offset..sig_offset + 64] == signature,
        ErrorCode::InvalidSignature
    );
    require!(
        &data[key_offset..key_offset + 32] == pubkey,
        ErrorCode::InvalidSignature
    );
    require!(
        &data[msg_offset..msg_offset + msg_size] == message,
        ErrorCode::InvalidSignature
    );

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// ACCOUNT CONTEXTS
// Each instruction has a context struct that lists all the
// accounts it needs, with validation constraints.
// ─────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct Initialize<'info> {
    // Create the config account, paid for by admin
    #[account(init, payer = admin, space = Config::LEN)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetParams<'info> {
    // has_one = admin means config.admin must equal the admin signer
    #[account(mut, has_one = admin)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(payload: ClaimPayload)]
pub struct VerifyAndMint<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,

    /// CHECK: validated against payload.recipient in instruction logic
    pub recipient: AccountInfo<'info>,

    // Create user's token account if it doesn't exist yet
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = sst_mint,
        associated_token::authority = recipient,
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    // Nonce PDA — created fresh for every claim, prevents replay
    // Seeds: "nonce" + the 32-byte nonce from payload
    #[account(
        init,
        payer = payer,
        space = NonceAccount::LEN,
        seeds = [b"nonce", payload.nonce.as_ref()],
        bump,
    )]
    pub nonce_account: Account<'info, NonceAccount>,

    // SST mint account — must match config.sst_mint
    #[account(mut, address = config.sst_mint)]
    pub sst_mint: Account<'info, Mint>,

    /// CHECK: PDA that has mint authority, validated in instruction logic
    #[account(
        seeds = [b"mint_authority", config.sst_mint.as_ref()],
        bump = config.mint_authority_bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    // Payer for account creation fees (admin/relayer)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Solana instructions sysvar — used to read the Ed25519 instruction
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub ix_sysvar: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResetDailyBudget<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, Config>,
    pub admin: Signer<'info>,
}

// ─────────────────────────────────────────────────────────────
// ON-CHAIN STATE
// ─────────────────────────────────────────────────────────────

// Main program configuration. One per deployment.
#[account]
pub struct Config {
    pub admin: Pubkey,                  // 32 — wallet that controls this program
    pub backend_signer: Pubkey,         // 32 — Firebase backend signer public key
    pub sst_mint: Pubkey,               // 32 — SST token mint address
    pub paused: bool,                   // 1  — emergency stop flag
    pub mint_authority_bump: u8,        // 1  — PDA bump for mint authority
    pub daily_budget: u64,              // 8  — total SST that can be minted per day
    pub remaining_daily_budget: u64,    // 8  — remaining SST for today
    pub max_claim_amount: u64,          // 8  — max SST per single claim
}

impl Config {
    // 8 (discriminator) + 32 + 32 + 32 + 1 + 1 + 8 + 8 + 8
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1 + 1 + 8 + 8 + 8;
}

// One account per claim nonce. Marks the nonce as used.
// Prevents the same signed payload from being submitted twice.
#[account]
pub struct NonceAccount {
    pub used: bool, // 1
}

impl NonceAccount {
    pub const LEN: usize = 8 + 1;
}

// ─────────────────────────────────────────────────────────────
// CLAIM PAYLOAD
// This is what the backend signs. It contains everything needed
// to authorize a specific mint to a specific user.
// ─────────────────────────────────────────────────────────────
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ClaimPayload {
    pub recipient: Pubkey,      // 32 — user wallet that receives SST
    pub amount: u64,            // 8  — how many SST (in smallest unit, 9 decimals)
    pub challenge_id: [u8; 32], // 32 — which challenge was completed
    pub nonce: [u8; 32],        // 32 — random bytes, prevents replay
    pub expires_at: i64,        // 8  — unix timestamp, claim expires after this
}

// ─────────────────────────────────────────────────────────────
// EVENTS
// Emitted after a successful mint. Indexers and the frontend
// can listen for these to track reward history.
// ─────────────────────────────────────────────────────────────
#[event]
pub struct RewardMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub challenge_id: [u8; 32],
    pub nonce: [u8; 32],
    pub timestamp: i64,
}

// ─────────────────────────────────────────────────────────────
// ERROR CODES
// ─────────────────────────────────────────────────────────────
#[error_code]
pub enum ErrorCode {
    #[msg("Program is paused — minting disabled")]
    Paused,
    #[msg("Claim has expired")]
    ClaimExpired,
    #[msg("Invalid Ed25519 signature")]
    InvalidSignature,
    #[msg("Nonce already used — replay attack prevented")]
    NonceAlreadyUsed,
    #[msg("Amount exceeds per-claim cap")]
    AmountExceedsCap,
    #[msg("Daily minting budget exceeded")]
    DailyBudgetExceeded,
    #[msg("Recipient does not match payload")]
    RecipientMismatch,
    #[msg("Invalid SST mint address")]
    InvalidMint,
    #[msg("Invalid mint authority PDA")]
    InvalidMintAuthority,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
}