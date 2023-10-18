use serde::{Deserialize, Serialize};
use unit_utils::{err::bail, Result};

use crate::magic::{find_magic, MAGIC};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbiHeader {
    pub name: String,
}

pub fn decode_abi_header(bytes: &[u8]) -> Result<AbiHeader> {
    let Some(magic_index) = find_magic(&bytes) else {
        bail!("Failed to locate magic");
    };

    let len = u16::from_be_bytes([bytes[magic_index + 8], bytes[magic_index + 9]]);
    let data = &bytes[magic_index + 10..magic_index + 10 + len as usize];
    let header = bincode::deserialize(data)?;

    Ok(header)
}

pub fn encode_abi_header(header: &AbiHeader) -> Result<Vec<u8>> {
    let data = bincode::serialize(header)?;

    let mut bytes = Vec::with_capacity(MAGIC.len() + 2 + data.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(data.len() as u16).to_be_bytes());
    bytes.extend_from_slice(&data);

    Ok(bytes)
}
