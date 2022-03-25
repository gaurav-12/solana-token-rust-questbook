use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, SetAuthority, Transfer};
use solana_program::entrypoint::ProgramResult;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mytokendapp {
    use super::*;

    pub fn proxy_transfer(ctx: Context<ProxyTransfer>, amount: u64) -> ProgramResult {
        token::transfer(ctx.accounts.into(), amount);
        Ok(())
    }

    pub fn proxy_mint_to(ctx: Context<ProxyMintTo>, amount: u64) -> ProgramResult {
        token::mint_to(ctx.accounts.into(), amount);
        Ok(())
    }

    pub fn proxy_burn(ctx: Context<ProxyBurn>, amount: u64) -> ProgramResult {
        token::burn(ctx.accounts.into(), amount);
        Ok(())
    }

    pub fn proxy_set_authority(
        ctx: Context<ProxySetAuthority>,
        authority_type: AuthorityType,
        new_authority: Option<Pubkey>,
    ) -> ProgramResult {
        token::set_authority(ctx.accounts.into(), authority_type.into(), new_authority);
        Ok(())
    }
}

// 'derive' will (de)serialize/convert this enum to an 'account'
// using the 'AnchorSerialize' and 'AnchorDeserealize' traits
// 'account' in Solana is like 'storage' type of variable in Solidity
// it is used to persist data in between the calls
#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum AuthorityType {
    MintTokens,    // Authority to mint tokens
    FreezeAccount, // Authority to freeze token account
    AccountOwner,  // Owner of the token account
    CloseAccount,  // Authority to close token account
}

// 'derive' is a macro, and 'Accounts' is trait
#[derive(Accounts)]
pub struct ProxyTransfer<'info> {
    // 'account' are attribute macro

    #[account(signer)]
    /// CHECK: This is not probably not dangerous
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub from: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub to: AccountInfo<'info>,
    /// CHECK: This is not probably not dangerous
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ProxyMintTo<'info> {
    #[account(signer)]
    /// CHECK: This is not probably not dangerous
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub to: AccountInfo<'info>,
    /// CHECK: This is not probably not dangerous
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ProxyBurn<'info> {
    #[account(signer)]
    /// CHECK: This is not probably not dangerous
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub to: AccountInfo<'info>,
    /// CHECK: This is not probably not dangerous
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ProxySetAuthority<'info> {
    #[account(signer)]
    /// CHECK: This is not probably not dangerous
    pub current_authority: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is not probably not dangerous
    pub account_or_mint: AccountInfo<'info>,
    /// CHECK: This is not probably not dangerous
    pub token_program: AccountInfo<'info>,
}

// Cross Program Invocations(CPI) implementations

// 'impl' is used to implement types(like 'struct') also to implement 'traits' for some type
// 'traits' are like interfaces

// below we are using 'impl <trait> for <type>'
// trait is 'From' and type is 'CpiContext'
impl<'a, 'b, 'c, 'info> From<&mut ProxyTransfer<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>
{
    fn from(accounts: &mut ProxyTransfer<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: accounts.from.clone(),
            to: accounts.to.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();

        // Observe: no semicolon at the end of last line of all the implementations
        // as in Rust(like Ruby), last line is taken as return statement(if none given)
        // but if we put semicolon at the end, then the function returns '()' called 'Unit'...
        // ...which was not matching the expected type in this case CpiContext
        // error on removing semicolon from last line in this case:
        //      expected struct `anchor_lang::context::CpiContext`, found `()`
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&mut ProxyMintTo<'info>>
    for CpiContext<'a, 'b, 'c, 'info, MintTo<'info>>
{
    fn from(accounts: &mut ProxyMintTo<'info>) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: accounts.mint.clone(),
            to: accounts.to.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&mut ProxyBurn<'info>> for CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
    fn from(accounts: &mut ProxyBurn<'info>) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: accounts.mint.clone(),
            to: accounts.to.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&mut ProxySetAuthority<'info>>
    for CpiContext<'a, 'b, 'c, 'info, SetAuthority<'info>>
{
    fn from(
        accounts: &mut ProxySetAuthority<'info>,
    ) -> CpiContext<'a, 'b, 'c, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: accounts.account_or_mint.clone(),
            current_authority: accounts.current_authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl From<AuthorityType> for spl_token::instruction::AuthorityType {
    fn from(authority_ty: AuthorityType) -> spl_token::instruction::AuthorityType {
        match authority_ty {
            AuthorityType::MintTokens => spl_token::instruction::AuthorityType::MintTokens,
            AuthorityType::FreezeAccount => spl_token::instruction::AuthorityType::FreezeAccount,
            AuthorityType::AccountOwner => spl_token::instruction::AuthorityType::AccountOwner,
            AuthorityType::CloseAccount => spl_token::instruction::AuthorityType::CloseAccount,
        }
    }
}
