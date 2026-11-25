// SPDX-License-Identifier: GPL-3.0-only

use accounts::{Event, Service};
use cosmic::iced::{futures::stream, Subscription};
use tokio::sync::broadcast::error::RecvError;

use crate::app::Message;

use super::{manager, AccountsAction};

/// Reloads Tasks' account list when its embedded manager adds, removes,
/// reconnects, or changes a Tasks account. The periodic retry covers startup
/// before the secure store has been opened.
pub fn subscription() -> Subscription<Message> {
    Subscription::run_with((), |()| {
        stream::unfold((), |()| async move {
            match manager::manager().await {
                Ok(accounts) => {
                    let mut events = accounts.subscribe();
                    loop {
                        match events.recv().await {
                            Ok(Event::AccountsChanged(Service::Tasks))
                            | Err(RecvError::Lagged(_)) => break,
                            Ok(_) => continue,
                            Err(RecvError::Closed) => {
                                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                                break;
                            }
                        }
                    }
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(30)).await,
            }
            Some((Message::Accounts(AccountsAction::Changed), ()))
        })
    })
}
