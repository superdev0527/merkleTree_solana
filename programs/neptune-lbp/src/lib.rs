use anchor_lang::prelude::*;
use merkle::MerkleTree;
use solana_program::hash::Hash;

mod merkle;
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("FVK3JTdRWSDVuqcJAntw6F6oHgG7HWmzsggZMwA4nef1");


#[program]
mod merkle_verify {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, items: Vec<Vec<u8>>) -> Result<()> {
        let merkle_info = &mut ctx.accounts.new_account;
        merkle_info.data = 0;
        merkle_info.owner = ctx.accounts.signer.key();
        merkle_info.items = items;
        Ok(())
    }

    pub fn add_leaf(ctx: Context<AddLeaf>, item: Vec<u8>) -> Result<()> {
        let merkle_info = &mut ctx.accounts.new_account;

        if merkle_info.owner != ctx.accounts.signer.key() {
            return err!(MerkleVerifyError::NotOwner);
        }

        merkle_info.items.push(item);
        Ok(())
    }

    pub fn set_value(ctx: Context<SetValue>, value: u64, index: usize, hash: Hash) -> Result<()> {
        let merkle_info = &mut ctx.accounts.new_account;
        let items_slice: Vec<&[u8]> = merkle_info.items.iter().map(|inner_vec| inner_vec.as_slice()).collect();
        let items = items_slice.as_slice();
        let merkle_tree = MerkleTree::new(items);
        if let Some(proof) = merkle_tree.find_path(index){
            if proof.verify(hash) == false {
                return err!(MerkleVerifyError::InvalidProof);
            }
        } else {
            return err!(MerkleVerifyError::InvalidProof);
        }

        merkle_info.data = value;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // We must specify the space in order to initialize an account.
    // First 8 bytes are default account discriminator,
    // next 8 bytes come from NewAccount.data being type u64.
    // (u64 = 64 bits unsigned integer = 8 bytes)
    #[account(init, payer = signer, seeds = [b"merkle"], bump, space = 10000)]
    pub new_account: Account<'info, MerkleVerifyInfo>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddLeaf<'info> {
    #[account(mut, seeds = [b"merkle"], bump)]
    pub new_account: Account<'info, MerkleVerifyInfo>,

    #[account(mut)]
    pub signer: Signer<'info>
}

#[derive(Accounts)]
pub struct SetValue<'info> {
    #[account(mut, seeds = [b"merkle"], bump)]
    pub new_account: Account<'info, MerkleVerifyInfo>,

    #[account(mut)]
    pub signer: Signer<'info>
}

#[account]
pub struct MerkleVerifyInfo {
    pub owner: Pubkey,
    pub data: u64,
    pub items: Vec<Vec<u8>>
}

#[error_code]
pub enum MerkleVerifyError {
    #[msg("NOT_OWNER")]
    NotOwner,
    #[msg("INVALID_PROOF")]
    InvalidProof
}