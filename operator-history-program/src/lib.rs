use borsh::BorshDeserialize;
use initialize_config::process_initialize_config;
use operator_history_sdk::instruction::OperatorHistoryInstruction;
// use solana_program::{declare_id, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::Pubkey, account_info::AccountInfo};
// #[cfg(not(feature = "no-entrypoint"))]
// use solana_program::entrypoint;
use solana_account_info::AccountInfo;
use solana_msg::msg;
use solana_program_error::{ProgramError, ProgramResult};
use solana_pubkey::Pubkey;

mod initialize_config;

solana_pubkey::declare_id!(env!("OPERATOR_HISTORY_PROGRAM_ID"));

#[cfg(not(feature = "no-entrypoint"))]
solana_program_entrypoint::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if *program_id != id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction = OperatorHistoryInstruction::try_from_slice(instruction_data)?;

    match instruction {
        OperatorHistoryInstruction::InitializeConfig => {
            msg!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)
        }
    }
}
