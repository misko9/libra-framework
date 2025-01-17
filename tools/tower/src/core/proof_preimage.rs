//! genesis preimage formatting.

use byteorder::{LittleEndian, WriteBytesExt};
use hex::decode;
use libra_types::legacy_types::{
    app_cfg::AppCfg,
    block::{GENESIS_VDF_ITERATIONS, GENESIS_VDF_SECURITY_PARAM},
};

/// Format the config file data into a fixed byte structure for easy parsing in Move/other languages
pub fn genesis_preimage(cfg: &AppCfg) -> anyhow::Result<Vec<u8>> {
    const AUTH_KEY_BYTES: usize = 32;
    const CHAIN_ID_BYTES: usize = 16;
    const DIFFICULTY_BYTES: usize = 8;
    const SECURITY_BYTES: usize = 8;
    const PIETRZAK: usize = 1; // PIETRZAK = 1, WESOLOWSKI = 2
    const LINK_TO_TOWER: usize = 64; // optional, hash of the last proof of an existing tower.
    const STATEMENT_BYTES: usize = 895; // remainder

    let mut preimage: Vec<u8> = vec![];

    // assume user has set default_profile_nickname
    let profile = cfg.get_profile(None)?;

    // AUTH_KEY_BYTES
    let mut padded_key_bytes = match decode(profile.auth_key.clone().to_string()) {
        Err(x) => panic!("Invalid 0L Auth Key: {}", x),
        Ok(key_bytes) => padding(key_bytes, AUTH_KEY_BYTES),
    };

    preimage.append(&mut padded_key_bytes);

    // CHAIN_ID_BYTES

    let mut padded_chain_id_bytes = padding(
        cfg.workspace
            .default_chain_id
            .to_string()
            .as_bytes()
            .to_vec(),
        CHAIN_ID_BYTES,
    );

    preimage.append(&mut padded_chain_id_bytes);

    // DIFFICULTY_BYTES
    preimage
        .write_u64::<LittleEndian>(*GENESIS_VDF_ITERATIONS)
        .unwrap();

    // SECURITY_BYTES
    preimage
        .write_u64::<LittleEndian>(*GENESIS_VDF_SECURITY_PARAM)
        .unwrap();

    // PIETRZAK
    preimage.write_u8(1).unwrap();

    // LINK_TO_TOWER
    // Note: V7: Deprecated
    let mut padded_tower_link_bytes = padding("".to_string().as_bytes().to_vec(), LINK_TO_TOWER);
    preimage.append(&mut padded_tower_link_bytes);

    // STATEMENT
    let mut padded_statements_bytes =
        padding(profile.statement.clone().into_bytes(), STATEMENT_BYTES);
    preimage.append(&mut padded_statements_bytes);

    assert_eq!(
        preimage.len(),
        (
            AUTH_KEY_BYTES // 0L Auth_Key
            + CHAIN_ID_BYTES // chain_id
            + DIFFICULTY_BYTES // iterations/difficulty
            + SECURITY_BYTES
            + PIETRZAK
            + LINK_TO_TOWER
            + STATEMENT_BYTES
            // = 1024
        ),
        "Preimage is the incorrect byte length"
    );

    assert_eq!(
        preimage.len(),
        1024,
        "Preimage is the incorrect byte length"
    );

    Ok(preimage)
}

fn padding(mut statement_bytes: Vec<u8>, limit: usize) -> Vec<u8> {
    match statement_bytes.len() {
        d if d > limit => panic!(
            "Message is longer than {} bytes. Got {} bytes",
            limit,
            statement_bytes.len()
        ),
        d if d < limit => {
            let padding_length = limit - statement_bytes.len();
            let mut padding_bytes: Vec<u8> = vec![0; padding_length];
            padding_bytes.append(&mut statement_bytes);
            padding_bytes
        }
        d if d == limit => statement_bytes,
        _ => unreachable!(),
    }
}
