use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use wasm_bindgen::prelude::*;
use web_sys::js_sys;
use yrs::{Map, Transact};

use crate::{
    graph_transfer_types::{from_type_name, get_type_name, WbblWebappNodeType},
    store_errors::WbblWebappStoreError,
    yrs_utils::{get_atomic_string, get_bool},
};

const GENERAL_SETTINGS_MAP_KEY: &str = "general_settings";
const KEYBOARD_SHORTCUTS_MAP_KEY: &str = "keyboard_shortcuts";
const NODE_KEYBOARD_SHORTCUTS_MAP_KEY: &str = "node_keyboard_shortcuts";

const FAVOURITES_MAP_KEY: &str = "favourites";

const THEME_MAP_KEY: &str = "theme";

#[wasm_bindgen]
pub struct WbblWebappPreferencesStore {
    next_listener_handle: u32,
    listeners: Arc<RefCell<Vec<(u32, js_sys::Function)>>>,
    preferences: Arc<yrs::Doc>,
    general_settings: yrs::MapRef,
    keyboard_shortcuts: yrs::MapRef,
    node_keyboard_shortcuts: yrs::MapRef,
    favourites: yrs::MapRef,
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
pub enum EdgeStyle {
    Default,
    Bezier,
    Metropolis,
}

impl EdgeStyle {
    fn get_string_representation(&self) -> String {
        match self {
            EdgeStyle::Default => "default",
            EdgeStyle::Bezier => "bezier",
            EdgeStyle::Metropolis => "metropolis",
        }
        .to_owned()
    }
    fn from_string_representation(str: &str) -> Option<Self> {
        match str {
            "default" => Some(Self::Default),
            "bezier" => Some(Self::Bezier),
            "metropolis" => Some(Self::Metropolis),
            _ => None,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeyboardShortcut {
    Copy,
    Paste,
    Cut,
    Undo,
    Redo,
    QuickActions,
    Delete,
    Duplicate,
    OpenKeybindings,
    Home,
    Help,
    LinkToPreview,
    Selection,
    SelectAll,
    SelectNone,
    AutoLayout,
    GroupNodes,
    UngroupNodes,
}

impl KeyboardShortcut {
    fn get_string_representation(&self) -> String {
        match self {
            KeyboardShortcut::Copy => "copy",
            KeyboardShortcut::Paste => "paste",
            KeyboardShortcut::Cut => "cut",
            KeyboardShortcut::Undo => "undo",
            KeyboardShortcut::Redo => "redo",
            KeyboardShortcut::QuickActions => "quick_actions",
            KeyboardShortcut::Delete => "delete",
            KeyboardShortcut::Duplicate => "duplicate",
            KeyboardShortcut::OpenKeybindings => "open_keybindings",
            KeyboardShortcut::Home => "home",
            KeyboardShortcut::Help => "help",
            KeyboardShortcut::LinkToPreview => "link_to_preview",
            KeyboardShortcut::Selection => "selection",
            KeyboardShortcut::SelectAll => "select_all",
            KeyboardShortcut::SelectNone => "select_none",
            KeyboardShortcut::AutoLayout => "auto_layout",
            KeyboardShortcut::GroupNodes => "group_nodes",
            KeyboardShortcut::UngroupNodes => "ungroup_nodes",
        }
        .to_owned()
    }

    fn from_string_representation(str: &str) -> Option<Self> {
        match str {
            "copy" => Some(Self::Copy),
            "paste" => Some(Self::Paste),
            "cut" => Some(Self::Cut),
            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "quick_actions" => Some(Self::QuickActions),
            "delete" => Some(Self::Delete),
            "duplicate" => Some(Self::Duplicate),
            "open_keybindings" => Some(Self::OpenKeybindings),
            "home" => Some(Self::Home),
            "help" => Some(Self::Help),
            "link_to_preview" => Some(Self::LinkToPreview),
            "selection" => Some(KeyboardShortcut::Selection),
            "select_all" => Some(KeyboardShortcut::SelectAll),
            "select_none" => Some(KeyboardShortcut::SelectNone),
            "auto_layout" => Some(KeyboardShortcut::AutoLayout),
            "group_nodes" => Some(KeyboardShortcut::GroupNodes),
            "ungroup_nodes" => Some(KeyboardShortcut::UngroupNodes),
            _ => None,
        }
    }
}

fn get_default_keybindings() -> HashMap<KeyboardShortcut, Option<String>> {
    HashMap::from([
        (KeyboardShortcut::Copy, Some("mod+c".to_owned())),
        (KeyboardShortcut::Cut, Some("mod+x".to_owned())),
        (KeyboardShortcut::Paste, Some("mod+v".to_owned())),
        (KeyboardShortcut::Undo, Some("mod+z".to_owned())),
        (KeyboardShortcut::Redo, Some("mod+shift+z".to_owned())),
        (KeyboardShortcut::QuickActions, Some("space".to_owned())),
        (KeyboardShortcut::Delete, Some("mod+backspace".to_owned())),
        (KeyboardShortcut::Duplicate, Some("shift+d".to_owned())),
        (KeyboardShortcut::Home, Some("mod+h".to_owned())),
        (
            KeyboardShortcut::OpenKeybindings,
            Some("mod+shift+k".to_owned()),
        ),
        (KeyboardShortcut::Help, Some("f1".to_owned())),
        (KeyboardShortcut::Selection, Some("shift".to_owned())),
        (KeyboardShortcut::SelectAll, Some("mod+a".to_owned())),
        (KeyboardShortcut::SelectNone, Some("shift+a".to_owned())),
        (KeyboardShortcut::AutoLayout, Some("mod+shift+f".to_owned())),
        (KeyboardShortcut::GroupNodes, Some("mod+g".to_owned())),
        (
            KeyboardShortcut::UngroupNodes,
            Some("mod+shift+g".to_owned()),
        ),
    ])
}

fn get_default_node_keybindings() -> HashMap<String, Option<String>> {
    HashMap::from([
        (get_type_name(WbblWebappNodeType::Add), Some("a".to_owned())),
        (
            get_type_name(WbblWebappNodeType::Subtract),
            Some("s".to_owned()),
        ),
        (
            get_type_name(WbblWebappNodeType::Divide),
            Some("d".to_owned()),
        ),
        (
            get_type_name(WbblWebappNodeType::Multiply),
            Some("m".to_owned()),
        ),
        (
            get_type_name(WbblWebappNodeType::Junction),
            Some("j".to_owned()),
        ),
    ])
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingSnapshot {
    pub keys: HashMap<KeyboardShortcut, Option<String>>,
}

#[wasm_bindgen]
impl WbblWebappPreferencesStore {
    pub fn empty() -> Result<WbblWebappPreferencesStore, WbblWebappStoreError> {
        let preferences = yrs::Doc::new();
        let general_settings = preferences.get_or_insert_map(GENERAL_SETTINGS_MAP_KEY.to_owned());
        let theme = preferences.get_or_insert_map(THEME_MAP_KEY.to_owned());
        let keyboard_shortcuts =
            preferences.get_or_insert_map(KEYBOARD_SHORTCUTS_MAP_KEY.to_owned());
        let favourites = preferences.get_or_insert_map(FAVOURITES_MAP_KEY.to_owned());
        let node_keyboard_shortcuts =
            preferences.get_or_insert_map(NODE_KEYBOARD_SHORTCUTS_MAP_KEY.to_owned());

        let listeners = Arc::new(RefCell::new(Vec::<(u32, js_sys::Function)>::new()));

        let mut store = WbblWebappPreferencesStore {
            next_listener_handle: 0,
            listeners: listeners.clone(),
            preferences: Arc::new(preferences),
            general_settings,
            keyboard_shortcuts,
            node_keyboard_shortcuts,
            favourites,
            theme,
            should_emit: false,
        };

        store.should_emit = true;
        store.emit()?;
        Ok(store)
    }

    fn emit(&self) -> Result<(), WbblWebappStoreError> {
        for (_, listener) in self.listeners.borrow().iter() {
            listener
                .call0(&JsValue::UNDEFINED)
                .map_err(|_| WbblWebappStoreError::FailedToEmit)?;
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

    pub fn set_base_theme(&mut self, theme: BaseTheme) -> Result<(), WbblWebappStoreError> {
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

    pub fn reset_keybinding(
        &mut self,
        command: KeyboardShortcut,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            let command_key = command.get_string_representation();
            self.keyboard_shortcuts.remove(&mut txn, &command_key);
        }
        self.emit()?;
        Ok(())
    }

    pub fn set_keybinding(
        &mut self,
        command: KeyboardShortcut,
        binding: Option<String>,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            let command_key = command.get_string_representation();
            match binding {
                Some(binding) => self
                    .keyboard_shortcuts
                    .insert(&mut txn, command_key, binding),
                None => self.keyboard_shortcuts.insert(&mut txn, command_key, false),
            };
        }
        self.emit()?;
        Ok(())
    }

    pub fn get_favourites(&self) -> Result<Vec<WbblWebappNodeType>, WbblWebappStoreError> {
        let txn = self.preferences.transact();
        let mut result: Vec<WbblWebappNodeType> = Vec::new();
        for (key, _) in self.favourites.iter(&txn) {
            match from_type_name(key) {
                Some(t) => result.push(t),
                None => {}
            };
        }
        Ok(result)
    }

    pub fn set_favourite(
        &mut self,
        node_type: WbblWebappNodeType,
        value: bool,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            if value {
                self.favourites
                    .insert(&mut txn, get_type_name(node_type), true);
            } else {
                self.favourites.remove(&mut txn, &get_type_name(node_type));
            }
        }
        self.emit()?;
        Ok(())
    }

    pub fn is_favourite(&self, node_type: WbblWebappNodeType) -> bool {
        let txn = self.preferences.transact();
        match self.favourites.get(&txn, &get_type_name(node_type)) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_keybindings(&self) -> Result<JsValue, WbblWebappStoreError> {
        let mut bindings = get_default_keybindings();
        let txn = self.preferences.transact();
        for binding in self.keyboard_shortcuts.iter(&txn) {
            match (
                KeyboardShortcut::from_string_representation(binding.0),
                binding.1,
            ) {
                (Some(shortcut), yrs::Value::Any(yrs::Any::String(b))) => {
                    bindings.insert(shortcut, Some(b.to_string()));
                }
                (Some(shortcut), yrs::Value::Any(yrs::Any::Bool(false))) => {
                    bindings.insert(shortcut, None);
                }
                (_, _) => {}
            };
        }
        serde_wasm_bindgen::to_value(&bindings)
            .map_err(|_| WbblWebappStoreError::SerializationFailure)
    }

    pub fn get_keybinding(
        &self,
        shortcut: KeyboardShortcut,
    ) -> Result<Option<String>, WbblWebappStoreError> {
        let bindings = get_default_keybindings();
        let txn = self.preferences.transact();

        match self
            .keyboard_shortcuts
            .get(&txn, &shortcut.get_string_representation())
        {
            Some(yrs::Value::Any(yrs::Any::String(shortcut))) => Ok(Some(shortcut.to_string())),
            None => match bindings.get(&shortcut) {
                Some(Some(shortcut)) => Ok(Some(shortcut.to_owned())),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    pub fn get_node_keybinding(
        &self,
        node_type: WbblWebappNodeType,
    ) -> Result<Option<String>, WbblWebappStoreError> {
        let bindings = get_default_node_keybindings();
        let txn = self.preferences.transact();

        let type_name = get_type_name(node_type);
        match self.node_keyboard_shortcuts.get(&txn, &type_name) {
            Some(yrs::Value::Any(yrs::Any::String(shortcut))) => Ok(Some(shortcut.to_string())),
            None => match bindings.get(&type_name) {
                Some(Some(shortcut)) => Ok(Some(shortcut.to_owned())),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    pub fn get_node_keybindings(&self) -> Result<JsValue, WbblWebappStoreError> {
        let mut bindings = get_default_node_keybindings();
        let txn = self.preferences.transact();
        for binding in self.node_keyboard_shortcuts.iter(&txn) {
            match (from_type_name(binding.0), binding.1) {
                (Some(shortcut), yrs::Value::Any(yrs::Any::String(b))) => {
                    bindings.insert(get_type_name(shortcut), Some(b.to_string()));
                }
                (Some(shortcut), yrs::Value::Any(yrs::Any::Bool(false))) => {
                    bindings.insert(get_type_name(shortcut), None);
                }
                (_, _) => {}
            };
        }
        serde_wasm_bindgen::to_value(&bindings)
            .map_err(|_| WbblWebappStoreError::SerializationFailure)
    }

    pub fn reset_node_keybinding(
        &mut self,
        node_type: WbblWebappNodeType,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();

            self.node_keyboard_shortcuts
                .remove(&mut txn, &get_type_name(node_type));
        }
        self.emit()?;
        Ok(())
    }

    pub fn set_node_keybinding(
        &mut self,
        node_type: WbblWebappNodeType,
        binding: Option<String>,
    ) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            let key = get_type_name(node_type);
            match binding {
                Some(binding) => self.node_keyboard_shortcuts.insert(&mut txn, key, binding),
                None => self.keyboard_shortcuts.insert(&mut txn, key, false),
            };
        }
        self.emit()?;
        Ok(())
    }

    pub fn toggle_wobble(&mut self) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            match get_bool("allow_wobble", &txn, &self.general_settings) {
                Ok(allowed) => {
                    self.general_settings
                        .insert(&mut txn, "allow_wobble", !allowed);
                    Ok(())
                }
                Err(WbblWebappStoreError::NotFound) => {
                    self.general_settings
                        .insert(&mut txn, "allow_wobble", false);
                    Ok(())
                }
                Err(err) => Err(err),
            }?;
        }
        self.emit()?;
        Ok(())
    }

    pub fn get_allow_wobble(&self) -> Result<bool, WbblWebappStoreError> {
        let txn = self.preferences.transact();
        match get_bool("allow_wobble", &txn, &self.general_settings) {
            Ok(allowed) => Ok(allowed),
            Err(WbblWebappStoreError::NotFound) => Ok(true),
            Err(err) => Err(err),
        }
    }

    pub fn get_edge_style(&self) -> Result<EdgeStyle, WbblWebappStoreError> {
        let txn = self.preferences.transact();
        match get_atomic_string("edge_style", &txn, &self.general_settings) {
            Ok(str) => {
                if let Some(style) = EdgeStyle::from_string_representation(&str) {
                    Ok(style)
                } else {
                    Err(WbblWebappStoreError::UnexpectedStructure)
                }
            }
            Err(WbblWebappStoreError::NotFound) => Ok(EdgeStyle::Default),
            Err(err) => Err(err),
        }
    }

    pub fn set_edge_style(&self, style: EdgeStyle) -> Result<(), WbblWebappStoreError> {
        {
            let mut txn = self.preferences.transact_mut();
            if let EdgeStyle::Default = style {
                self.general_settings.remove(&mut txn, "edge_style");
            } else {
                self.general_settings.insert(
                    &mut txn,
                    "edge_style",
                    style.get_string_representation(),
                );
            };
        }
        self.emit()?;
        Ok(())
    }
}
