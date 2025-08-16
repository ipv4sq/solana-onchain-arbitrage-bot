use crate::arb::chain::data::Transaction;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    EncodedTransaction, UiInnerInstructions, UiInstruction, UiMessage, UiParsedInstruction,
    UiPartiallyDecodedInstruction,
};

pub fn extract_ix_and_inners(
    tx: &Transaction,
    mut interested_in: impl FnMut(&UiPartiallyDecodedInstruction) -> bool,
) -> Option<(&UiPartiallyDecodedInstruction, &UiInnerInstructions)> {
    let message = match &tx.transaction.transaction {
        EncodedTransaction::Json(t) => match &t.message {
            UiMessage::Parsed(msg) => msg,
            _ => return None,
        },
        _ => return None,
    };

    let (ix, ix_index) = message
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(idx, x)| match x {
            UiInstruction::Compiled(_) => None,
            UiInstruction::Parsed(it) => match it {
                UiParsedInstruction::Parsed(_) => None,
                UiParsedInstruction::PartiallyDecoded(i) => Some((i, idx)),
            },
        })
        .find(|(ix, _)| interested_in(*ix))?;

    // Get the inner instructions for this specific instruction index
    let inner_ixs = tx
        .transaction
        .meta
        .as_ref()
        .and_then(|meta| match &meta.inner_instructions {
            OptionSerializer::Some(inner) => Some(inner),
            _ => None,
        })
        .and_then(|inner| inner.iter().find(|i| i.index == ix_index as u8));

    Some((ix, inner_ixs?))
}
