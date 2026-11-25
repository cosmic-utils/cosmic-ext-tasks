use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
};

use cosmic::widget::{
    self,
    segmented_button::Entity,
};
use uuid::Uuid;

use crate::{
    app::AppModel,
    config::ListSortBy,
    features::{accounts::AccountHeaderMarker, lists::List},
};

impl AppModel {
    pub fn reposition_special_items(&mut self) {
        let mut pos: u16 = 0;
        if self.config.show_favorites {
            self.nav.position_set(self.favorites_entity, pos);
            pos += 1;
        }
        if self.config.show_trash {
            self.nav.position_set(self.trash_entity, pos);
            pos += 1;
        }
        let first_list_pos = pos;

        let list_entities: Vec<Entity> = self
            .nav
            .iter()
            .filter(|e| self.nav.data::<List>(*e).is_some())
            .collect();

        let mut groups: BTreeMap<Option<Uuid>, Vec<Entity>> = BTreeMap::new();
        for account in &self.remote_accounts {
            groups.entry(Some(account.id)).or_default();
        }
        for entity in list_entities {
            let account_id = self
                .nav
                .data::<List>(entity)
                .and_then(|l| l.source.account_id());
            groups.entry(account_id).or_default().push(entity);
        }

        let mut ordered: Vec<(Option<Uuid>, String, Vec<Entity>)> = groups
            .into_iter()
            .map(|(account_id, entities)| {
                let label = account_id
                    .map(|id| crate::features::accounts::account_label(&self.remote_accounts, id))
                    .unwrap_or_default();
                (account_id, label, entities)
            })
            .collect();
        ordered.sort_by(|a, b| match (a.0, b.0) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(_), Some(_)) => a.1.cmp(&b.1),
        });

        let current_account_ids: std::collections::HashSet<Uuid> = ordered
            .iter()
            .filter_map(|(account_id, _, _)| *account_id)
            .collect();
        let stale_headers: Vec<Uuid> = self
            .account_header_entities
            .keys()
            .filter(|id| !current_account_ids.contains(id))
            .copied()
            .collect();
        for account_id in stale_headers {
            if let Some(entity) = self.account_header_entities.remove(&account_id) {
                self.nav.remove(entity);
            }
        }

        let mut pos = first_list_pos;
        for (account_id, label, mut entities) in ordered {
            self.sort_list_entities(&mut entities);

            if let Some(account_id) = account_id {
                let header = match self.account_header_entities.get(&account_id) {
                    Some(&entity) => {
                        self.nav.text_set(entity, label.clone());
                        entity
                    }
                    None => {
                        let entity = self
                            .nav
                            .insert()
                            .text(label.clone())
                            .data(AccountHeaderMarker(account_id))
                            .id();
                        self.account_header_entities.insert(account_id, entity);
                        entity
                    }
                };
                self.nav.position_set(header, pos);
                pos += 1;
            }

            for entity in entities {
                self.nav.position_set(entity, pos);
                pos += 1;
            }
        }

        let entities: Vec<Entity> = self.nav.iter().collect();
        for (i, entity) in entities.iter().enumerate() {
            let is_header = self.nav.data::<AccountHeaderMarker>(*entity).is_some();
            self.nav
                .divider_above_set(*entity, is_header || i == first_list_pos as usize);
        }
    }

    // Keep list IDs and the active list stable when their sidebar rows are hidden.
    pub(crate) fn refresh_sidebar(&mut self) {
        let mut existing: HashMap<Entity, Entity> = self
            .sidebar
            .iter()
            .filter_map(|row| {
                self.sidebar
                    .data::<Entity>(row)
                    .map(|&entity| (entity, row))
            })
            .collect();
        let active = self.nav.active();
        self.sidebar.deactivate();

        for (position, entity) in self
            .nav
            .iter()
            .filter(|&entity| {
                !self
                    .nav
                    .data::<List>(entity)
                    .and_then(|list| list.source.account_id())
                    .is_some_and(|id| self.collapsed_accounts.contains(&id))
            })
            .enumerate()
        {
            let row = existing
                .remove(&entity)
                .unwrap_or_else(|| self.sidebar.insert().data(entity).id());
            let header = self.nav.data::<AccountHeaderMarker>(entity);
            let icon = if let Some(header) = header {
                let name = if self.collapsed_accounts.contains(&header.0) {
                    "pan-end-symbolic"
                } else {
                    "pan-down-symbolic"
                };
                Some(widget::icon::from_name(name).size(16).icon())
            } else {
                self.nav.icon(entity).cloned()
            };
            if let Some(icon) = icon {
                self.sidebar.icon_set(row, icon);
            }
            self.sidebar
                .text_set(row, self.nav.text(entity).unwrap_or_default().to_owned());
            let is_account_list = self
                .nav
                .data::<List>(entity)
                .is_some_and(|list| list.source.account_id().is_some());
            self.sidebar.indent_set(row, u16::from(is_account_list));
            self.sidebar
                .divider_above_set(row, self.nav.divider_above(entity).unwrap_or(false));
            self.sidebar.position_set(row, position as u16);
            if entity == active {
                self.sidebar.activate(row);
            }
        }
        for row in existing.into_values() {
            self.sidebar.remove(row);
        }
    }

    fn sort_list_entities(&self, entities: &mut [Entity]) {
        match self.config.list_sort_by {
            ListSortBy::Manual => {
                entities.sort_by_key(|e| self.nav.data::<List>(*e).map(|l| l.created_at));
            }
            ListSortBy::NameAsc | ListSortBy::NameDesc => {
                entities.sort_by_key(|e| {
                    self.nav
                        .data::<List>(*e)
                        .map(|l| l.name.to_lowercase())
                        .unwrap_or_default()
                });
                if self.config.list_sort_by == ListSortBy::NameDesc {
                    entities.reverse();
                }
            }
        }
    }
}
