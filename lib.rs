#[allow(unused)]
mod pb;

use pb::transfers::v1 as transfers;
use substreams_solana::b58;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

// System Program: the program that moves native SOL.
const SYSTEM_PROGRAM_ID: [u8; 32] = b58!("11111111111111111111111111111111");

// SystemInstruction::Transfer is variant #2 (a little-endian u32 at the start of the data).
const SYSTEM_TRANSFER: u32 = 2;

#[substreams::handlers::map]
fn map_transfers(blk: Block) -> transfers::Transfers {
    let mut out = transfers::Transfers::default();

    // walk_instructions() yields top-level AND inner (CPI) instructions, each exactly once.
    for instruction in blk.walk_instructions() {
        if instruction.program_id() != SYSTEM_PROGRAM_ID {
            continue;
        }

        // Transfer data layout: 4-byte instruction index + 8-byte lamports = 12 bytes.
        let data: &[u8] = instruction.data();
        if data.len() != 12 {
            continue;
        }
        let kind = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if kind != SYSTEM_TRANSFER {
            continue;
        }
        let amount = u64::from_le_bytes(data[4..12].try_into().unwrap());

        // Accounts of a Transfer: [0] = sender, [1] = recipient.
        let accounts: Vec<Vec<u8>> = instruction
            .accounts()
            .into_iter()
            .map(|a| a.0.clone())
            .collect();
        if accounts.len() < 2 {
            continue;
        }

        let mut t = transfers::Transfer::default();
        t.from = bs58::encode(&accounts[0]).into_string();
        t.to = bs58::encode(&accounts[1]).into_string();
        t.amount = amount;
        out.transfers.push(t);
    }

    out
}
