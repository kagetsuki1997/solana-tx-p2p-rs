use solana_sdk::{
    instruction::CompiledInstruction,
    message::{Message, MessageHeader},
    transaction::Transaction,
};
use utoipa::ToSchema;

use crate::proto::peer as proto;

impl From<CompiledInstruction> for proto::CompiledInstruction {
    fn from(CompiledInstruction { program_id_index, accounts, data }: CompiledInstruction) -> Self {
        Self { program_id_index: program_id_index.into(), accounts, data }
    }
}

impl From<MessageHeader> for proto::MessageHeader {
    fn from(
        MessageHeader {
            num_required_signatures,
            num_readonly_signed_accounts,
            num_readonly_unsigned_accounts,
        }: MessageHeader,
    ) -> Self {
        Self {
            num_required_signatures: num_required_signatures.into(),
            num_readonly_signed_accounts: num_readonly_signed_accounts.into(),
            num_readonly_unsigned_accounts: num_readonly_unsigned_accounts.into(),
        }
    }
}

impl From<Message> for proto::Message {
    fn from(Message { header, account_keys, recent_blockhash, instructions }: Message) -> Self {
        Self {
            header: Some(header.into()),
            account_keys: account_keys.into_iter().map(|key| key.to_string()).collect(),
            recent_blockhash: recent_blockhash.to_string(),
            instructions: instructions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Transaction> for proto::Transaction {
    fn from(Transaction { signatures, message }: Transaction) -> Self {
        Self {
            signatures: signatures.into_iter().map(|signature| signature.to_string()).collect(),
            message: Some(message.into()),
        }
    }
}

/// Workaround for OpenAPI docs
#[derive(ToSchema)]
#[schema(as = Transaction)]
pub struct TransactionForUtoipa {
    /// A set of signatures of a serialized [`Message`], signed by the first
    /// keys of the `Message`'s [`account_keys`], where the number of signatures
    /// is equal to [`num_required_signatures`] of the `Message`'s
    /// [`MessageHeader`].
    ///
    /// [`account_keys`]: Message::account_keys
    /// [`MessageHeader`]: crate::message::MessageHeader
    /// [`num_required_signatures`]: crate::message::MessageHeader::num_required_signatures
    // NOTE: Serialization-related changes must be paired with the direct read at sigverify.
    pub signatures: Vec<String>,

    /// The message to sign.
    #[schema(inline)]
    pub message: MessageForUtoipa,
}

/// Workaround for OpenAPI docs
#[derive(ToSchema)]
#[schema(as = Message)]
pub struct MessageForUtoipa {
    /// The message header, identifying signed and read-only `account_keys`.
    // NOTE: Serialization-related changes must be paired with the direct read at sigverify.
    #[schema(inline)]
    pub header: MessageHeaderForUtoipa,

    /// All the account keys used by this transaction.
    pub account_keys: Vec<String>,

    /// The id of a recent ledger entry.
    pub recent_blockhash: String,

    /// Programs that will be executed in sequence and committed in one atomic
    /// transaction if all succeed.
    #[schema(inline)]
    pub instructions: Vec<CompiledInstructionForUtoipa>,
}

/// Workaround for OpenAPI docs
#[derive(ToSchema)]
#[schema(as = MessageHeader)]
pub struct MessageHeaderForUtoipa {
    /// The number of signatures required for this message to be considered
    /// valid. The signers of those signatures must match the first
    /// `num_required_signatures` of [`Message::account_keys`].
    // NOTE: Serialization-related changes must be paired with the direct read at sigverify.
    pub num_required_signatures: u8,

    /// The last `num_readonly_signed_accounts` of the signed keys are read-only
    /// accounts.
    pub num_readonly_signed_accounts: u8,

    /// The last `num_readonly_unsigned_accounts` of the unsigned keys are
    /// read-only accounts.
    pub num_readonly_unsigned_accounts: u8,
}

/// Workaround for OpenAPI docs
#[derive(ToSchema)]
#[schema(as = CompiledInstruction)]
pub struct CompiledInstructionForUtoipa {
    /// Index into the transaction keys array indicating the program account
    /// that executes this instruction.
    pub program_id_index: u8,
    /// Ordered indices into the transaction keys array indicating which
    /// accounts to pass to the program.
    pub accounts: Vec<u8>,
    /// The program input data.
    pub data: Vec<u8>,
}
