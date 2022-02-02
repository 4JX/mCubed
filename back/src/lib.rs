use std::sync::mpsc::{Receiver, Sender};

use messages::{CheckProgress, ToBackend, ToFrontend};
use modrinth::Modrinth;
use tokio::runtime::Runtime;
mod modrinth;

pub mod messages;
pub mod mod_entry;

pub struct Back {
    rt: Runtime,
    modrinth: Modrinth,
    back_tx: Sender<ToFrontend>,
    front_rx: Receiver<ToBackend>,
}

impl Back {
    pub fn new(back_tx: Sender<ToFrontend>, front_rx: Receiver<ToBackend>) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();

        Self {
            rt,
            modrinth: Default::default(),
            back_tx,
            front_rx,
        }
    }

    pub fn init(&self) {
        self.rt.block_on(async {
            loop {
                match self.front_rx.recv() {
                    Ok(message) => match message {
                        ToBackend::CheckForUpdates { mut mod_list } => {
                            let total_len = mod_list.len();
                            for (position, mod_entry) in mod_list.iter_mut().enumerate() {
                                // Update the frontend on whats happening
                                self.back_tx
                                    .send(ToFrontend::CheckForUpdatesProgress {
                                        progress: CheckProgress {
                                            name: mod_entry.display_name.clone(),
                                            position,
                                            total_len,
                                        },
                                    })
                                    .unwrap();

                                self.modrinth.check_for_updates(mod_entry).await;
                            }

                            self.back_tx
                                .send(ToFrontend::UpdateModList { mod_list })
                                .unwrap();
                        }
                    },
                    Err(err) => {
                        let _ = err;
                    }
                };
            }
        });
    }
}
