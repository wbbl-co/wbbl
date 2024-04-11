use std::str::FromStr;

use yrs::{block::Prelim, Map, MapRef, TransactionMut};

use crate::store_errors::WbblWebappStoreError;

pub(crate) fn get_atomic_string<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<String, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::String(result))) => Ok((*result).to_owned()),
        None => Err(WbblWebappStoreError::NotFound),
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}

pub(crate) fn get_atomic_u128_from_string<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<u128, WbblWebappStoreError> {
    let str = get_atomic_string(key, txn, map)?;
    uuid::Uuid::from_str(&str)
        .map_err(|_| WbblWebappStoreError::MalformedId)
        .map(|uuid| uuid.as_u128())
}

pub(crate) fn get_atomic_bigint<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<i64, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::BigInt(result))) => Ok(result),
        None => Err(WbblWebappStoreError::NotFound),
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}

pub(crate) fn get_float_64<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<f64, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::Number(result))) => Ok(result),
        None => Err(WbblWebappStoreError::NotFound),
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}

pub(crate) fn get_map<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<MapRef, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::YMap(map_ref)) => Ok(map_ref),
        None => Err(WbblWebappStoreError::NotFound),
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}

pub(crate) fn get_or_insert_map<T: Prelim>(
    key: &str,
    txn: &mut TransactionMut,
    map: &yrs::MapRef,
    default_value: T,
) -> Result<MapRef, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::YMap(map_ref)) => Ok(map_ref),
        None => {
            map.insert(txn, key, default_value);
            get_map(key, txn, map)
        }
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}

pub(crate) fn get_bool<Txn: yrs::ReadTxn>(
    key: &str,
    txn: &Txn,
    map: &yrs::MapRef,
) -> Result<bool, WbblWebappStoreError> {
    match map.get(txn, key) {
        Some(yrs::Value::Any(yrs::Any::Bool(result))) => Ok(result),
        None => Err(WbblWebappStoreError::NotFound),
        _ => Err(WbblWebappStoreError::UnexpectedStructure),
    }
}
