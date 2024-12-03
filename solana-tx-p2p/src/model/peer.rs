use libp2p::{identity, PeerId};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::CompiledInstruction,
    message::{Message, MessageHeader},
    pubkey::Pubkey,
    transaction::Transaction,
};
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiMessage, UiTransaction,
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

/// Workaround for `OpenAPI` docs
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

/// Workaround for `OpenAPI` docs
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

/// Workaround for `OpenAPI` docs
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

/// Workaround for `OpenAPI` docs
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

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetail {
    /// base58 encoded string of signature
    signatures: Vec<String>,
    /// base58 encoded string of account key
    account_keys: Vec<String>,
    /// base58 encoded string of signer `PeerId`
    signers: Vec<String>,
    log_messages: Vec<String>,
}

impl From<EncodedConfirmedTransactionWithStatusMeta> for TransactionDetail {
    fn from(
        EncodedConfirmedTransactionWithStatusMeta {  transaction, ..}: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Self {
        match transaction.transaction {
            EncodedTransaction::Json(UiTransaction { signatures, message }) => {
                if let UiMessage::Raw(message) = message {
                    let log_messages = transaction
                        .meta
                        .and_then(|meta| meta.log_messages.into())
                        .unwrap_or_default();
                    Self {
                        signatures,
                        account_keys: message.account_keys.clone(),
                        signers: message.account_keys
                            [0..message.header.num_required_signatures as usize]
                            .iter()
                            .map(|pkey| solana_public_key_str_to_peer_id(pkey).to_base58())
                            .collect(),
                        log_messages,
                    }
                } else {
                    unimplemented!("unsupported `UiMessage` {message:?}")
                }
            }
            _ => unimplemented!("unsupported `EncodedTransaction`"),
        }
    }
}

impl From<TransactionDetail> for proto::TransactionDetail {
    fn from(
        TransactionDetail { signatures, account_keys, signers, log_messages }: TransactionDetail,
    ) -> Self {
        Self { signatures, account_keys, signers, log_messages }
    }
}

fn solana_public_key_str_to_peer_id(solana_pkey_str: &str) -> PeerId {
    let solana_pkey = Pubkey::from_str_const(solana_pkey_str);
    let pkey: identity::PublicKey =
        identity::ed25519::PublicKey::try_from_bytes(&solana_pkey.to_bytes())
            .expect("convert solana public key to libp2p ed25519 public key")
            .into();

    pkey.into()
}
