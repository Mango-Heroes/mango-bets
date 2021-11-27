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
        return place_wager(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 2 {
        return settle_bet_outcome(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 3 {
        return claim_winnings(
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

/// Functionality to create a bet
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
    // returns the bet state
    let mut bet_state = BetState::try_from_slice(&instruction_data).expect("Instruction data serialization did not work");

    // Make sure that the creator of the bet state is the one who initialized the bet
    if bet_state.creator != *creator_account.key {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    if bet_state.name.len() < 5 {
        msg!("Name of the bet needs to be longer than 5 characters");
        return Err(ProgramError::InvalidInstructionData);
    }

    if bet_state.description.len() < 10 {
        msg!("Description of the bet needs to be longer than 10 characters");
        return Err(ProgramError::InvalidAccountData);
    }

    // Get the minimum balance we need in our program account by using the length of our writing program derived account/address
    let rent_exemption = Rent::get()?.minimum_balance(writing_account_pda.data_len());
    
    // And we make sure our program account (`writing_account`) has that much lamports(balance).
    if **writing_account_pda.lamports.borrow() < rent_exemption {
        msg!("The balance of writing_account should be more than rent_exemption");
        return Err(ProgramError::InsufficientFunds);
    }

    // Then we can set the initial bet state
    bet_state.total_pool=0; // Initialize an empty total pool to keep track of total funds
    bet_state.party1_pool=0; // Initialize an empty pool for party 1
    bet_state.party2_pool=0; // Initialize an empty pool for party 2
    bet_state.outcome = BetOutcome::new(); // Initialize a fresh unsettled outcome

    // Serialize the bet state struct into a binary format using serialize 
    //to write that data thats in our writing account
    bet_state.serialize(&mut &mut writing_account_pda.data.borrow_mut()[..])?;

    // Return OK
    Ok(())
}

/// Functionality to place a wager on a bet
fn place_wager(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();

    let writing_account_pda = next_account_info(accounts_iter)?;

    let wager_amount_pda = next_account_info(accounts_iter)?;

    let bettor_account_pda = next_account_info(accounts_iter)?;

    let creator_account = next_account_info(accounts_iter)?;

     // We want to write in this account, so we want to make sure its owner is the program itself.
     if writing_account_pda.owner != program_id {
        msg!("writing_account_pda isn't owned by the program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check to see if this transaction was not signed by the bettor_account public key
    if !creator_account.is_signer {
        msg!("The creator_account should be the signer of this instruction");
        return Err(ProgramError::IncorrectProgramId);
    }

    //grab the data to create the Bettor struct
    let mut bettor_details = BettorDetails::try_from_slice(&instruction_data).expect("Error deserializing bettor details data");

    // grab the BetState struct out of the writing account's
    let mut bet_state = BetState::try_from_slice(*writing_account_pda.data.borrow()).expect("Error deserializing the bet state data");

    // get the number of lamports from the bet_aount_pda
    let bet_amount_in_lamports = wager_amount_pda.lamports();

    // get the minimum balance we need in our program account.
    // We need this rent exemption to make sure our bettor accounts that get created for each bettor doesnt get dropped by Solana
    let rent_exemption = Rent::get()?.minimum_balance(bettor_account_pda.data_len());

    // And we make sure our program account (`writing_account`) has that much lamports(balance).
    if **bettor_account_pda.lamports.borrow() < rent_exemption {
        msg!("The balance of bettor_account should be more than the rent_exemption");
        return Err(ProgramError::InsufficientFunds);
    }


    // Set the bettor_details info
    bettor_details.value = bet_amount_in_lamports;
    // **** party 1 and 2 status will be set by the front end solana api code
    // **** bet placer and assoc bet address will be set by the front end solana api code

    // serialize the bettor_details
    bettor_details.serialize(&mut &mut bettor_account_pda.data.borrow_mut()[..])?;

    // set the bet_state info
    bet_state.total_pool += bet_amount_in_lamports;
    if bettor_details.party1 { bet_state.party1_pool += bet_amount_in_lamports; } 
    if bettor_details.party2 { bet_state.party2_pool += bet_amount_in_lamports; }

    // move the lamports from bet_amount_account_pda to writing_account_pda BetState
    **writing_account_pda.try_borrow_mut_lamports()? += **wager_amount_pda.lamports.borrow();
    **wager_amount_pda.try_borrow_mut_lamports()? = 0;

    bet_state.serialize(&mut &mut writing_account_pda.data.borrow_mut()[..])?;

    Ok(())
}

/// Functionality to settle a bet with the outcome
fn settle_bet_outcome(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

/// Functionality for a Bettor to claim their winnings for a particular Bet
fn claim_winnings(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

// BetState struct representing the base structure of a bet within the app
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct BetState {
    pub creator: Pubkey,
    pub name: String,
    pub description: String,
    pub total_pool: u64,
    pub party1_pool: u64,
    pub party2_pool: u64,
    pub outcome: BetOutcome 
}

// Bettor struct representing a single bettor
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct BettorDetails {
    pub bet_placer_address: Pubkey,
    pub value: u64,
    pub assoc_bet_address: Pubkey,
    pub party1: bool,
    pub party2: bool,
}

// BetOutcome struct representing the outcome of a bet
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct BetOutcome {
    pub party1_result: bool,
    pub party2_result: bool,
}

// BetOutcome struct method implementation
impl BetOutcome {
    pub fn new() -> Self {
        Self {
            party1_result: false,
            party2_result: false
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
