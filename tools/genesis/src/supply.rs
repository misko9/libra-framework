
use libra_types::legacy_types::{
  legacy_address::LegacyAddress,
  legacy_recovery::LegacyRecovery,
};
use std::path::PathBuf;
use anyhow::Context;

#[derive(Debug, Clone, Default)]
pub struct Supply {
  pub total: u64,
  pub normal: u64,
  pub validator: u64, // will overlap with slow wallet
  pub validator_locked: u64,
  pub slow_total: u64,
  pub slow_locked: u64,
  pub slow_unlocked: u64,
  pub donor_directed: u64,
}


fn inc_supply(mut acc: Supply, r: &LegacyRecovery, dd_wallet_list: &Vec<LegacyAddress>) -> anyhow::Result<Supply> {

    // get balances
    let amount: u64 = match &r.balance {
        Some(b) => {
          b.coin
        },
        None => 0,
    };
    acc.total = acc.total.checked_add(amount).unwrap();

    // get donor directed
    if dd_wallet_list.contains(&r.account.unwrap()) {
      acc.donor_directed = acc.donor_directed.checked_add(amount).unwrap();
    } else if let Some(sl) = &r.slow_wallet {
      acc.slow_total = acc.slow_total.checked_add(amount).unwrap();
      if sl.unlocked > 0 {
        acc.slow_unlocked = acc.slow_unlocked.checked_add(amount).unwrap();
        if amount > sl.unlocked { // Note: the validator may have transferred everything out, and the unlocked may not have changed
          let locked = amount - sl.unlocked;
          acc.slow_locked = acc.slow_locked.checked_add(locked).unwrap();
          // if this is the special case of a validator account with slow locked balance
          if r.val_cfg.is_some() {
            acc.validator = acc.validator.checked_add(amount).unwrap();
            acc.validator_locked = acc.validator_locked.checked_add(locked).unwrap();
          }

        }
      }


    } else {
      acc.normal = acc.normal.checked_add(amount).unwrap();
    }
    Ok(acc)
}

/// iterate over the recovery file and get the sum of all balances.
/// Note: this may not be the "total supply", since there may be coins in other structs beside an account::balance, e.g escrowed in contracts.
pub fn get_supply_struct(rec: &Vec<LegacyRecovery>) -> anyhow::Result<Supply> {
  let zeroth = Supply {
    total: 0,
    normal: 0,
    validator: 0,
    validator_locked: 0,
    slow_total: 0,
    slow_locked: 0,
    slow_unlocked: 0,
    donor_directed: 0,
  };

  let dd_wallets = rec.iter()
    .find(|el| { el.comm_wallet.is_some() })
    .context("could not find 0x0 state")?
    .comm_wallet
    .as_ref()
    .context("could not find list of community wallets")?;

  rec.iter().try_fold(zeroth, |acc, r| {
    inc_supply(acc, r, &dd_wallets.list)
  })
}


#[test]
fn test_get_struct() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_export_recovery.json");
    
    let r = crate::parse_json::parse(p).unwrap();

    let supply = get_supply_struct(&r).unwrap();
    dbg!(&supply);
    let pct_slow = supply.slow_total as f64 / supply.total as f64;
    dbg!(&pct_slow);
    let pct_dd = supply.donor_directed as f64 / supply.total as f64;
    dbg!(&pct_dd);
    let pct_val_locked = supply.validator_locked as f64 / supply.total as f64;
    dbg!(&pct_val_locked);
    assert!(supply.total == 2397436809784621);
}