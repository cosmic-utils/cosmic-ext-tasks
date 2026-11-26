// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeSet;

use cosmic::{
    cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[version = 1]
pub struct AppConfig {
    pub app_theme: AppTheme,
    #[serde(default)]
    pub disabled_accounts: BTreeSet<Uuid>,
    pub hide_completed: bool,
    pub last_list_id: Option<Uuid>,
    pub list_sort_by: ListSortBy,
    pub show_favorites: bool,
    pub show_trash: bool,
    pub sort_by: SortBy,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::default(),
            disabled_accounts: BTreeSet::new(),
            hide_completed: false,
            last_list_id: None,
            list_sort_by: ListSortBy::default(),
            show_favorites: true,
            show_trash: true,
            sort_by: SortBy::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ListSortBy {
    Manual,
    #[default]
    NameAsc,
    NameDesc,
}

impl From<usize> for ListSortBy {
    fn from(value: usize) -> Self {
        match value {
            1 => ListSortBy::NameDesc,
            2 => ListSortBy::Manual,
            _ => ListSortBy::NameAsc,
        }
    }
}

impl From<ListSortBy> for usize {
    fn from(value: ListSortBy) -> Self {
        match value {
            ListSortBy::Manual => 2,
            ListSortBy::NameAsc => 0,
            ListSortBy::NameDesc => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum SortBy {
    #[default]
    DateAsc,
    DateDesc,
    Manual,
    NameAsc,
    NameDesc,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl From<usize> for AppTheme {
    fn from(value: usize) -> Self {
        match value {
            1 => AppTheme::Dark,
            2 => AppTheme::Light,
            _ => AppTheme::System,
        }
    }
}

impl From<AppTheme> for usize {
    fn from(value: AppTheme) -> Self {
        match value {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        }
    }
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::system_dark(),
            Self::Light => theme::system_light(),
            Self::System => theme::system_preference(),
        }
    }
}
