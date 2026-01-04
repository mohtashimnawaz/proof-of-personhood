use anchor_lang::prelude::*;

declare_id!("DAGmRdAFCw3z6uWbrGqA7U2z3Yuhh2aHtBby6StYFxaE");

#[program]
pub mod proof_of_personhood {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
