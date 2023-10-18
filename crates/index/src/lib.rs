use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use unit_abi::header::AbiHeader;

use unit_utils::{err::bail, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexEntry {
    pub path: String, // relative to storage location
    pub abi_header: AbiHeader,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexData {
    pub entries: Vec<IndexEntry>,
}

impl IndexData {
    pub fn new() -> IndexData {
        IndexData { entries: vec![] }
    }
}

pub struct Index {
    storage_path: PathBuf,
    data: IndexData,
}

fn decode_index_data(bytes: &[u8]) -> Result<IndexData> {
    let data = bincode::deserialize(bytes)?;
    Ok(data)
}

fn encode_index_data(data: &IndexData) -> Result<Vec<u8>> {
    let bytes = bincode::serialize(data)?;
    Ok(bytes)
}

fn read_and_decode_index_data(path: PathBuf) -> Result<IndexData> {
    if !path.try_exists()? {
        let index = IndexData::new();
        let bytes = encode_index_data(&index)?;
        std::fs::write(&path, &bytes)?;
    }

    let bytes = std::fs::read(path)?;
    return decode_index_data(&bytes);
}

impl Index {
    pub fn load(storage_location: String) -> Result<Index> {
        let storage_location: PathBuf = storage_location.parse()?;

        if !storage_location.is_dir() {
            bail!("Storage location is not a directory");
        }

        let storage_path = storage_location.join("index.unit");

        let index_data = read_and_decode_index_data(storage_path.clone())?;

        Ok(Index {
            storage_path,
            data: index_data,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        self.data = read_and_decode_index_data(self.storage_path.clone())?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let bytes = encode_index_data(&self.data)?;
        std::fs::write(&self.storage_path, &bytes)?;

        Ok(())
    }

    pub fn entries(&self) -> &Vec<IndexEntry> {
        &self.data.entries
    }

    pub fn set_entries(&mut self, entries: Vec<IndexEntry>) -> Result<()> {
        self.data.entries = entries;
        self.save()
    }

    pub fn add_entry(&mut self, entry: IndexEntry) -> Result<()> {
        self.data.entries.push(entry);
        self.save()
    }

    pub fn update_entry(&mut self, new: IndexEntry) -> Result<()> {
        let index = self
            .data
            .entries
            .iter()
            .position(|e| e.abi_header.name == new.abi_header.name);

        let Some(index) = index else {
            bail!("Entry not found");
        };

        self.data.entries[index] = new;
        self.save()
    }

    pub fn add_or_update_entry(&mut self, entry: IndexEntry) -> Result<()> {
        let index = self
            .data
            .entries
            .iter()
            .position(|e| e.abi_header.name == entry.abi_header.name);

        if let Some(index) = index {
            self.data.entries[index] = entry;
        } else {
            self.data.entries.push(entry);
        }

        self.save()
    }
}
