#![allow(unexpected_cfgs)]

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo<'_>],
    instruction_data: &[u8],
) -> ProgramResult {
    let data = String::from_utf8(instruction_data.to_vec());
    msg!("Hello, world! {:?}", data);
    Ok(())
}

#[cfg(test)]
mod test {
    use solana_program_test::*;
    use solana_sdk::{signature::Signer, transaction::Transaction};

    use super::*;

    #[tokio::test]
    async fn test_hello_world() {
        let program_id = Pubkey::new_unique();
        let (banks_client, payer, recent_blockhash) =
            ProgramTest::new("solana_hello_world", program_id, processor!(process_instruction))
                .start()
                .await;

        // Create the instruction to invoke the program
        let instruction =
            solana_program::instruction::Instruction::new_with_borsh(program_id, b"doge", vec![]);

        // Add the instruction to a new transaction
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // Process the transaction
        let transaction_result = banks_client.process_transaction(transaction).await;
        assert!(transaction_result.is_ok());
    }
}
