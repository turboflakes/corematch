use subxt::backend::legacy::rpc_methods::StorageKey;
use subxt::config::substrate::AccountId32;

pub fn get_para_id_from_storage_key(key: StorageKey) -> u32 {
    let s = &key[key.len() - 4..];
    let v: [u8; 4] = s.try_into().expect("slice with incorrect length");
    u32::from_le_bytes(v)
}

pub fn get_nft_id_from_storage_key(key: StorageKey) -> u32 {
    let s = &key[key.len() - 4..];
    let v: [u8; 4] = s.try_into().expect("slice with incorrect length");
    u32::from_le_bytes(v)
}

pub fn str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8(bytes).expect("Data not utf-8"))
}

pub fn compact(account: &AccountId32) -> String {
    let a = account.to_string();
    [&a[..4], &a[a.len() - 4..a.len()]].join("...")
}
