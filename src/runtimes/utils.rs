use subxt::backend::legacy::rpc_methods::StorageKey;

pub fn get_para_id_from_storage_key(key: Vec<u8>) -> u32 {
    let s = &key[key.len() - 4..];
    let v: [u8; 4] = s.try_into().expect("slice with incorrect length");
    u32::from_le_bytes(v)
}

pub fn get_nft_id_from_storage_key(key: StorageKey) -> u32 {
    let s = &key[key.len() - 4..];
    let v: [u8; 4] = s.try_into().expect("slice with incorrect length");
    u32::from_le_bytes(v)
}
