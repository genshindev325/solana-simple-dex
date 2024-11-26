use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("DqMAcWeDxNWsUFW6FcDCyTPcdMJWKX4ik1HgNTAWtCNQ");

#[program]
pub mod simple_dex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, token_a: Pubkey, token_b: Pubkey) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex_account;
        dex_account.token_a = token_a;
        dex_account.token_b = token_b;
        dex_account.token_a_reserve = 0;
        dex_account.token_b_reserve = 0;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, token_a_amount:u64, token_b_amount: u64) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex_account;

        dex_account.token_a_reserve += token_a_amount;
        dex_account.token_b_reserve += token_b_amount;

        token::transfer(
            ctx.accounts
                .transfer_token_a_context(),
                // .with_signer(&[ctx.accounts.dex_account.to_account_info().key().as_ref()]),
            token_a_amount,
        )?;

        token::transfer(
            ctx.accounts
                .transfer_token_b_context(),
                // .with_signer(&[ctx.accounts.dex_account.to_account_info().key().as_ref()]),
            token_b_amount,
        )?;

        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, input_amount: u64, direction: u8) -> Result<()> {
        let dex_account = &mut ctx.accounts.dex_account;
        if direction == 0 {
            // Swap Token A to Token B
            let amount_out = calculate_swap_out(dex_account.token_a_reserve, dex_account.token_b_reserve, input_amount);
            dex_account.token_a_reserve += input_amount;
            dex_account.token_b_reserve -= amount_out;

            token::transfer(
                ctx.accounts
                    .transfer_output_token_context(),
                    // .with_signer(&[ctx.accounts.dex_account.to_account_info().key().as_ref()]),
                amount_out,
            )?;
        } else {
            // Swap Token B to Token A
            let amount_out = calculate_swap_out(dex_account.token_b_reserve, dex_account.token_a_reserve, input_amount);
            dex_account.token_b_reserve += input_amount;
            dex_account.token_a_reserve -= amount_out;

            token::transfer(
                ctx.accounts
                    .transfer_output_token_context(),
                    // .with_signer(&[ctx.accounts.dex_account.to_account_info().key().as_ref()]),
                amount_out
            )?;
        }

        Ok(())
    }
}

fn calculate_swap_out(input_reserve: u64, output_reserve: u64, input_amount: u64) -> u64 {
    const FEE_NUMERATOR: u64 = 997; // 0.3% fee
    const FEE_DENOMINATOR: u64 = 1000;

    let input_amount_with_fee = input_amount * FEE_NUMERATOR;
    let numerator = input_amount_with_fee * output_reserve;
    let denominator = input_reserve * FEE_DENOMINATOR + input_amount_with_fee;

    numerator / denominator
}


#[account]
pub struct DexAccount {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = initializer, space = 8 + 32 + 32 + 16)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(mut)]
    pub token_a_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    fn transfer_token_a_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_a_account.to_account_info(),
            to: self.dex_account.to_account_info(),
            authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn transfer_token_b_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.token_b_account.to_account_info(),
            to: self.dex_account.to_account_info(),
            authority: self.initializer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub dex_account: Account<'info, DexAccount>,
    #[account(mut)]
    pub input_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub output_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Swap<'info> {
    fn transfer_output_token_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.dex_account.to_account_info(),
            to: self.output_token_account.to_account_info(),
            authority: self.input_token_account.to_account_info(),
        };

        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

