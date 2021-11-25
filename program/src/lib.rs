// First we include what we are going to need in our program. 
// This is the Rust style of importing things.
// Remember we added the dependencies in cargo.toml
// And from the `solana_program` crate we are including all the required things.
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use borsh::{BorshDeserialize, BorshSerialize};

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

    if instruction_data.len() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Now we just check and call the function for each of them.
    if instruction_data[0] == 0 {
        return initialize_bet(
            program_id,
            accounts,
            // Notice we pass program_id and accounts as they where 
            // but we pass a reference to slice of [instruction_data]. 
            // we do not want the first element in any of our functions.
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 1 {
        return place_bet_amount(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 2 {
        return set_bet_outcome(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 3 {
        return release_bet_winnings(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }

    // If instruction_data doesn't match we give an error.
    // Note I have used msg!() macro and passed a string here. 
    // It is good to do this as this would 
    // also get printed in the console window if a program fails.
    msg!("Didn't find the entrypoint required");
    Err(ProgramError::InvalidInstructionData)

}

entrypoint!(process_instruction);

fn initialize_bet(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();

    // The account running this instruction, created by the Solana program
    let writing_account_pda = next_account_info(accounts_iter)?;

    // The account thats calling to initialize the bet
    let creator_account = next_account_info(accounts_iter)?;

    // We want to write in this account, so we want to make sure its owner is the program itself.
    if writing_account_pda.owner != program_id {
        msg!("writing_account_pda isn't owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check to see if this transaction was not signed by the creator_accounts public key
    if !creator_account.is_signer {
        msg!("The creator_account should be the signer of this instruction");
        return Err(ProgramError::IncorrectProgramId);
    }

    // We try to deserialize the instruction data into our BetState struct to work with
    let mut bet_state = BetState::try_from_slice(&instruction_data).expect("Instruction data serialization did not work");

    // Make sure that the creator of the bet state is the one who initialized the bet
    if bet_state.admin != *creator_account.key {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // get the minimum balance we need in our program account by using the length of our writing program derived account/address
    let rent_exemption = Rent::get()?.minimum_balance(writing_account_pda.data_len());
    
    // And we make sure our program account (`writing_account`) has that much lamports(balance).
    if **writing_account_pda.lamports.borrow() < rent_exemption {
        msg!("The balance of writing_account should be more than rent_exemption");
        return Err(ProgramError::InsufficientFunds);
    }

    // Then we can set the initial pool to be zero.
    bet_state.total_pool=0;

    bet_state.serialize(&mut &mut writing_account_pda.data.borrow_mut()[..])?;

    Ok(())
}

fn place_bet_amount(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

fn set_bet_outcome(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

fn release_bet_winnings(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct BetState {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub image_link: String,
    pub total_pool: u64,
    pub party1_pool: u64,
    pub party1_bettors: Vec<Bettor>,
    pub party2_pool: u64,
    pub party2_bettors: Vec<Bettor>,
    pub outcome: BetOutcome
    
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct Bettor {
    pub address: Pubkey,
    pub value: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct BetOutcome {
    pub party1_result: bool,
    pub party2_result: bool,
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
