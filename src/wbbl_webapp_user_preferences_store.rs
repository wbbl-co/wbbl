use serde::{Deserialize, Serialize};
use std::{cell::RefCell, mem, str::FromStr, sync::Arc};
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use yrs::{Map, Transact};

use crate::log;

const GENERAL_SETTINGS_MAP_KEY: &str = "general_settings";
const KEYBOARD_SHORTCUTS_MAP_KEY: &str = "keyboard_shortcuts";
const THEME_MAP_KEY: &str = "theme";

#[wasm_bindgen]
pub struct WbblWebappPreferencesStore {
    next_listener_handle: u32,
    listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>>,
    preferences: Arc<yrs::Doc>,
    general_settings: yrs::MapRef,
    keyboard_shortcuts: yrs::MapRef,
    theme: yrs::MapRef,
    should_emit: bool,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum BaseTheme {
    Light = 0,
    Dark = 1,
    System = 2,
}

#[wasm_bindgen]
pub enum WbblWebappPreferencesStoreError {
    MalformedId,
    FailedToEmit,
}

#[wasm_bindgen]
pub enum KeyboardShortcut {
    Copy,
    Paste,
    Cut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShortcutModifierKey {
    Super,
    Hyper,
}

#[wasm_bindgen]
impl WbblWebappPreferencesStore {
    pub fn empty() -> Result<WbblWebappPreferencesStore, WbblWebappPreferencesStoreError> {
        let preferences = yrs::Doc::new();
        let general_settings = preferences.get_or_insert_map(GENERAL_SETTINGS_MAP_KEY.to_owned());
        let theme = preferences.get_or_insert_map(THEME_MAP_KEY.to_owned());
        let keyboard_shortcuts =
            preferences.get_or_insert_map(KEYBOARD_SHORTCUTS_MAP_KEY.to_owned());

        let listeners = Arc::new(RefCell::new(Vec::<(u32, js_sys::Function)>::new()));

        let mut store = WbblWebappPreferencesStore {
            next_listener_handle: 0,
            listeners: listeners.clone(),
            preferences: Arc::new(preferences),
            general_settings,
            keyboard_shortcuts,
            theme,
            should_emit: false,
        };

        store.should_emit = true;
        store.emit()?;
        Ok(store)
    }

    fn emit(&self) -> Result<(), WbblWebappPreferencesStoreError> {
        for (_, listener) in self.listeners.borrow().iter() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappPreferencesStoreError::FailedToEmit)?;
        }
        Ok(())
    }

    pub fn subscribe(&mut self, subscriber: js_sys::Function) -> u32 {
        let handle = self.next_listener_handle;
        self.listeners.borrow_mut().push((handle, subscriber));
        self.next_listener_handle = self.next_listener_handle + 1;
        handle
    }

    pub fn unsubscribe(&mut self, handle: u32) {
        let mut listeners = self.listeners.borrow_mut();
        if let Some((idx, _)) = listeners
            .iter()
            .enumerate()
            .find(|(_, (k, _))| *k == handle)
        {
            let _ = listeners.remove(idx);
        }
    }

    pub fn set_base_theme(
        &mut self,
        theme: BaseTheme,
    ) -> Result<(), WbblWebappPreferencesStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            self.theme
                .insert(&mut txn, "base", yrs::Any::BigInt(theme as i64));
        }
        self.emit()?;
        Ok(())
    }

    pub fn get_base_theme(&self) -> BaseTheme {
        let txn = self.preferences.transact();
        match self.theme.get(&txn, &"base") {
            Some(yrs::Value::Any(yrs::Any::BigInt(x))) if x == BaseTheme::Dark as i64 => {
                BaseTheme::Dark
            }
            Some(yrs::Value::Any(yrs::Any::BigInt(x))) if x == BaseTheme::Light as i64 => {
                BaseTheme::Light
            }
            _ => BaseTheme::System,
        }
    }
}
