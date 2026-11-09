//! Unified module inventory and lifecycle overview.

use super::*;
use emulation_core::module_management::ModuleId;
use emulation_core::module_state_store::{default_user_state_path, ModuleStateStore};
use std::collections::BTreeSet;

fn module_tile(ui: &mut egui::Ui, palette: &theme::Palette, name: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::Rounding::same(9.0), palette.accent_subtle);
    let initials: String = name
        .chars()
        .filter(|character| character.is_alphanumeric())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initials,
        egui::FontId::proportional(15.0),
        palette.text,
    );
}

fn status_badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    theme::badge(ui, text, color);
}

fn module_brand_index(id: &str) -> usize {
    let normalized = id.to_ascii_lowercase();
    if normalized.starts_with("xbox") {
        1
    } else if normalized.starts_with("playstation") || normalized.starts_with("ps") {
        2
    } else {
        0
    }
}

fn module_display_name(id: &str) -> String {
    match id {
        "nes" => "Nintendo (NES)".to_owned(),
        other => other
            .split(['-', '_'])
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn module_inventory(
    registered: &BTreeSet<String>,
    available: &BTreeSet<String>,
    installed: &BTreeSet<String>,
) -> BTreeSet<String> {
    registered
        .union(available)
        .cloned()
        .chain(installed.iter().cloned())
        .collect()
}

fn persist_nes_install_state(version: Option<&str>) -> Result<(), String> {
    let path = default_user_state_path().map_err(|error| error.to_string())?;
    let store = ModuleStateStore::new(path);
    let mut state = store.load().map_err(|error| error.to_string())?;
    let id = ModuleId::new("nes").map_err(|error| error.to_string())?;

    if let Some(version) = version {
        state
            .record_installed(id, version.to_owned())
            .map_err(|error| error.to_string())?;
    } else {
        state.record_uninstalled(&id);
    }
    store.save(&state).map_err(|error| error.to_string())
}

fn version_from_installed_path(path: &std::path::Path) -> Option<String> {
    path.parent()?
        .file_name()?
        .to_str()?
        .strip_prefix("nes-module-")
        .map(str::to_owned)
}

impl DesktopApp {
    pub(super) fn show_systems(&mut self, ui: &mut egui::Ui) {
        if let Some(receiver) = self.nes_install_progress_receiver.take() {
            let mut keep_receiver = true;
            loop {
                match receiver.try_recv() {
                    Ok(progress) => self.nes_install_progress = Some(progress),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        keep_receiver = false;
                        break;
                    }
                }
            }
            if keep_receiver {
                self.nes_install_progress_receiver = Some(receiver);
            }
        }

        if let Some(receiver) = self.nes_install_receiver.take() {
            match receiver.try_recv() {
                Ok(Ok(path)) => {
                    self.nes_installing = false;
                    self.nes_install_progress_receiver = None;
                    let persisted = version_from_installed_path(&path)
                        .ok_or_else(|| "version du paquet installé introuvable".to_owned())
                        .and_then(|version| persist_nes_install_state(Some(&version)));
                    self.message = match persisted {
                        Ok(()) if self.language == "fr" => {
                            format!("Nintendo (NES) installé et vérifié : {}", path.display())
                        }
                        Ok(()) => format!("Nintendo (NES) installed and verified: {}", path.display()),
                        Err(error) if self.language == "fr" => format!(
                            "Module NES installé, mais son état n’a pas pu être enregistré : {error}"
                        ),
                        Err(error) => format!(
                            "NES module installed, but its state could not be saved: {error}"
                        ),
                    };
                }
                Ok(Err(error)) => {
                    self.nes_installing = false;
                    self.nes_install_progress_receiver = None;
                    self.nes_install_progress = None;
                    self.message = if self.language == "fr" {
                        format!("Échec de l’installation de Nintendo (NES) : {error}")
                    } else {
                        format!("Nintendo (NES) installation failed: {error}")
                    };
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.nes_install_receiver = Some(receiver);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.nes_installing = false;
                    self.nes_install_progress_receiver = None;
                    self.nes_install_progress = None;
                    self.message = if self.language == "fr" {
                        "Le téléchargement du module Nintendo (NES) s’est interrompu.".to_owned()
                    } else {
                        "Nintendo (NES) module download stopped unexpectedly.".to_owned()
                    };
                }
            }
        }

        if let Some(receiver) = self.nes_uninstall_receiver.take() {
            match receiver.try_recv() {
                Ok(Ok(removed)) => {
                    self.nes_uninstalling = false;
                    let persisted = persist_nes_install_state(None);
                    self.message = match (persisted, self.language == "fr") {
                        (Ok(()), true) => format!(
                            "Nintendo (NES) désinstallé. {removed} version(s) supprimée(s). Vos jeux et sauvegardes sont conservés."
                        ),
                        (Ok(()), false) => format!(
                            "Nintendo (NES) uninstalled. Removed {removed} version(s). Your games and saves were kept."
                        ),
                        (Err(error), true) => format!(
                            "Module désinstallé, mais l’état local n’a pas pu être mis à jour : {error}"
                        ),
                        (Err(error), false) => format!(
                            "Module uninstalled, but local state could not be updated: {error}"
                        ),
                    };
                }
                Ok(Err(error)) => {
                    self.nes_uninstalling = false;
                    self.message = if self.language == "fr" {
                        format!("Échec de la désinstallation de Nintendo (NES) : {error}")
                    } else {
                        format!("Nintendo (NES) uninstall failed: {error}")
                    };
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.nes_uninstall_receiver = Some(receiver);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.nes_uninstalling = false;
                    self.message = if self.language == "fr" {
                        "La désinstallation du module Nintendo (NES) s’est interrompue.".to_owned()
                    } else {
                        "Nintendo (NES) uninstall stopped unexpectedly.".to_owned()
                    };
                }
            }
        }

        let palette = theme::Palette::new(self.dark_mode, self.accent_color);
        let french = self.language == "fr";
        let registered: BTreeSet<String> = self
            .registry
            .system_ids()
            .map(|id| id.as_str().to_owned())
            .collect();

        let state_path = match default_user_state_path() {
            Ok(path) => path,
            Err(error) => {
                theme::card(ui, &palette, |ui| {
                    ui.label(
                        egui::RichText::new(if french {
                            format!("État des modules indisponible : {error}")
                        } else {
                            format!("Module state unavailable: {error}")
                        })
                        .color(palette.danger),
                    );
                });
                return;
            }
        };
        let store = ModuleStateStore::new(state_path);
        let state = match store.load() {
            Ok(state) => state,
            Err(error) => {
                theme::card(ui, &palette, |ui| {
                    ui.label(
                        egui::RichText::new(if french {
                            format!("Impossible de lire l’état des modules : {error}")
                        } else {
                            format!("Could not load module state: {error}")
                        })
                        .strong()
                        .color(palette.danger),
                    );
                    ui.label(self.language_pack.text("ui.systems_page.003"));
                });
                return;
            }
        };

        // Availability is based on the embedded catalogue and this host target,
        // not on how many modules happen to be compiled into the desktop shell.
        let catalogue_available: BTreeSet<String> = nes_module_manager::available_module_ids()
            .into_iter()
            .collect();
        let installed_external: BTreeSet<String> =
            if nes_module_manager::installed_executable_path().is_some() {
                ["nes".to_owned()].into_iter().collect()
            } else {
                BTreeSet::new()
            };
        // A persisted record alone is not proof that package files still exist.
        // The external NES executable is verified by its SHA-256 marker above;
        // built-in cores are supplied by this running application.
        let installed_ids: BTreeSet<String> =
            registered.union(&installed_external).cloned().collect();
        let available_ids: BTreeSet<String> =
            registered.union(&catalogue_available).cloned().collect();
        let module_ids = module_inventory(&registered, &catalogue_available, &installed_ids);

        theme::page_header(
            ui,
            &palette,
            self.language_pack.text("ui.systems_page.001"),
            if french {
                "Installez une console : elle sera ajoutée directement à PeaceEmu et prête à être utilisée."
            } else {
                "Install a console to add it directly to PeaceEmu, ready to use."
            },
            |_| {},
        );

        if let Some(progress) = self.nes_install_progress.as_ref() {
            let stage = match (french, progress.stage) {
                (true, "prepare") => "Préparation",
                (true, "catalogue") => "Récupération des informations",
                (true, "download") => "Téléchargement du module",
                (true, "download_checksum") => "Téléchargement du fichier de contrôle",
                (true, "verify") => "Vérification de l’intégrité",
                (true, "extract_skipped") => "Décompression : non nécessaire",
                (true, "install") => "Installation des fichiers",
                (true, "finalize") => "Finalisation",
                (true, "done") => "Installation terminée",
                (false, "prepare") => "Preparing",
                (false, "catalogue") => "Fetching release information",
                (false, "download") => "Downloading module",
                (false, "download_checksum") => "Downloading checksum",
                (false, "verify") => "Integrity verification",
                (false, "extract_skipped") => "Extraction: not required",
                (false, "install") => "Installing files",
                (false, "finalize") => "Finalizing",
                (false, "done") => "Installation complete",
                (true, _) => "En cours",
                (false, _) => "In progress",
            };
            theme::card(ui, &palette, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(if french {
                            "Installation de Nintendo (NES)"
                        } else {
                            "Nintendo (NES) installation"
                        })
                        .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("{}%", progress.percent)).strong());
                    });
                });
                ui.add(egui::ProgressBar::new(progress.percent as f32 / 100.0));
                ui.label(egui::RichText::new(stage).color(palette.text_dim));
                if !progress.detail.is_empty() {
                    ui.label(
                        egui::RichText::new(&progress.detail)
                            .size(11.0)
                            .color(palette.text_dim),
                    );
                }
            });
        }

        let columns = if ui.available_width() < 620.0 { 1 } else { 3 };
        ui.columns(columns, |cols| {
            let stats = [
                (
                    if french { "DISPONIBLES" } else { "AVAILABLE" },
                    available_ids.len().to_string(),
                    if french {
                        "catalogue compatible + cœurs intégrés"
                    } else {
                        "compatible catalogue + built-in cores"
                    },
                ),
                (
                    if french { "INSTALLÉS" } else { "INSTALLED" },
                    installed_ids.len().to_string(),
                    if french {
                        "prêts à être utilisés"
                    } else {
                        "ready to use"
                    },
                ),
                (
                    if french { "INTÉGRÉS" } else { "BUILT IN" },
                    registered.len().to_string(),
                    if french {
                        "fournis avec l’application"
                    } else {
                        "shipped with the app"
                    },
                ),
            ];
            for (index, (label, value, detail)) in stats.into_iter().enumerate() {
                theme::card(&mut cols[index % columns], &palette, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(label)
                            .size(10.0)
                            .strong()
                            .color(palette.text_dim),
                    );
                    ui.label(egui::RichText::new(value).size(25.0).strong());
                    ui.label(
                        egui::RichText::new(detail)
                            .size(11.0)
                            .color(palette.text_dim),
                    );
                });
            }
        });

        ui.add_space(14.0);
        ui.label(egui::RichText::new("CONSOLES").strong().size(13.0));
        ui.separator();

        let mut grouped_modules: [Vec<String>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for id in module_ids {
            grouped_modules[module_brand_index(&id)].push(id);
        }

        ui.columns(3, |brand_columns| {
            for (brand_index, brand_name) in ["Nintendo", "Xbox", "PlayStation"].iter().enumerate()
            {
                let ui = &mut brand_columns[brand_index];
                ui.label(egui::RichText::new(*brand_name).strong().size(16.0));
                ui.separator();
                if grouped_modules[brand_index].is_empty() {
                    theme::card(ui, &palette, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new(if french {
                                "Aucune console disponible pour le moment."
                            } else {
                                "No console available yet."
                            })
                            .color(palette.text_dim),
                        );
                    });
                }
                for id in grouped_modules[brand_index].iter().cloned() {
                    let Ok(module_id) = ModuleId::new(id.clone()) else {
                        continue;
                    };
                    let is_bundled = registered.contains(&id);
                    let nes_executable = if id == "nes" {
                        nes_module_manager::installed_executable_path()
                    } else {
                        None
                    };
                    let installed_version = if id == "nes" {
                        nes_executable
                            .as_ref()
                            .and_then(|path| version_from_installed_path(path))
                    } else {
                        state.installed_version(&module_id).map(str::to_owned)
                    };
                    let is_installed = is_bundled || installed_version.is_some();
                    let is_available = available_ids.contains(&id);
                    let display_name = module_display_name(&id);

                    theme::card(ui, &palette, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            module_tile(ui, &palette, &display_name);
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&display_name).size(16.0).strong());
                                ui.label(
                                    egui::RichText::new(if id == "nes" {
                                        "Nintendo Entertainment System"
                                    } else {
                                        id.as_str()
                                    })
                                    .size(12.0)
                                    .color(palette.text_dim),
                                );
                                ui.horizontal_wrapped(|ui| {
                                    if is_installed {
                                        status_badge(
                                            ui,
                                            if french { "Installé" } else { "Installed" },
                                            palette.success,
                                        );
                                    } else if is_available {
                                        status_badge(
                                            ui,
                                            if french { "Disponible" } else { "Available" },
                                            palette.accent,
                                        );
                                    } else {
                                        status_badge(
                                            ui,
                                            if french {
                                                "Indisponible"
                                            } else {
                                                "Unavailable"
                                            },
                                            palette.warning,
                                        );
                                    }
                                    if is_bundled {
                                        status_badge(
                                            ui,
                                            if french { "Inclus" } else { "Built in" },
                                            palette.accent,
                                        );
                                    }
                                    if let Some(version) = installed_version.as_deref() {
                                        status_badge(ui, &format!("v{version}"), palette.success);
                                    }
                                });
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if id == "nes" {
                                        if is_installed
                                            && ui
                                                .add_enabled(
                                                    !self.nes_installing && !self.nes_uninstalling,
                                                    egui::Button::new(if french {
                                                        "Désinstaller"
                                                    } else {
                                                        "Uninstall"
                                                    }),
                                                )
                                                .clicked()
                                        {
                                            self.nes_uninstall_confirm_open = true;
                                        }
                                        let label = if self.nes_installing {
                                            if french {
                                                "Installation…"
                                            } else {
                                                "Installing…"
                                            }
                                        } else if self.nes_uninstalling {
                                            if french {
                                                "Désinstallation…"
                                            } else {
                                                "Uninstalling…"
                                            }
                                        } else if is_installed {
                                            if french {
                                                "Mettre à jour"
                                            } else {
                                                "Update"
                                            }
                                        } else if is_available {
                                            if french {
                                                "Installer"
                                            } else {
                                                "Install"
                                            }
                                        } else if french {
                                            "Non disponible"
                                        } else {
                                            "Unavailable"
                                        };
                                        let can_install =
                                            is_available && !self.nes_installing && !self.nes_uninstalling;
                                        if ui
                                            .add_enabled(can_install, egui::Button::new(label))
                                            .clicked()
                                        {
                                            self.start_nes_module_install();
                                        }
                                    } else if is_bundled {
                                        status_badge(
                                            ui,
                                            if french {
                                                "Prêt à l’emploi"
                                            } else {
                                                "Ready to use"
                                            },
                                            palette.success,
                                        );
                                    }
                                },
                            );
                        });

                        if let Some(descriptor) = self
                            .registry
                            .system_ids()
                            .find(|system_id| system_id.as_str() == id)
                            .and_then(|system_id| self.registry.registered_descriptor(system_id))
                        {
                            let caps = descriptor.capabilities;
                            ui.add_space(6.0);
                            ui.horizontal_wrapped(|ui| {
                                for (label, enabled) in [
                                    (
                                        if french { "Contenu" } else { "Content" },
                                        caps.content_loading,
                                    ),
                                    (if french { "Entrées" } else { "Input" }, caps.input),
                                    (if french { "Vidéo" } else { "Video" }, caps.video_output),
                                    ("Audio", caps.audio_output),
                                    (
                                        if french { "États" } else { "Save states" },
                                        caps.save_states,
                                    ),
                                    (
                                        if french { "Sauvegardes" } else { "Saves" },
                                        caps.persistent_saves,
                                    ),
                                ] {
                                    status_badge(
                                        ui,
                                        &format!("{} {label}", if enabled { "●" } else { "○" }),
                                        if enabled {
                                            palette.success
                                        } else {
                                            palette.text_dim
                                        },
                                    );
                                }
                            });
                        }
                    });
                }
            }
        });

        let mut confirm_uninstall = false;
        let mut close_confirmation = false;
        if self.nes_uninstall_confirm_open {
            let mut open = true;
            egui::Window::new(if french {
                "Désinstaller Nintendo (NES) ?"
            } else {
                "Uninstall Nintendo (NES)?"
            })
            .id(egui::Id::new("confirm_nes_module_uninstall"))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(320.0);
                ui.label(if french {
                    "Seuls les fichiers du module seront supprimés. Vos ROM, sauvegardes et paramètres resteront intacts."
                } else {
                    "Only module files will be removed. Your ROMs, saves and settings will remain untouched."
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(if french { "Annuler" } else { "Cancel" }).clicked() {
                        close_confirmation = true;
                    }
                    if ui
                        .add_enabled(
                            !self.nes_uninstalling,
                            egui::Button::new(if self.nes_uninstalling {
                                if french { "Désinstallation…" } else { "Uninstalling…" }
                            } else if french {
                                "Confirmer la désinstallation"
                            } else {
                                "Confirm uninstall"
                            }),
                        )
                        .clicked()
                    {
                        confirm_uninstall = true;
                        close_confirmation = true;
                    }
                });
            });
            if !open {
                close_confirmation = true;
            }
        }
        if close_confirmation {
            self.nes_uninstall_confirm_open = false;
        }
        if confirm_uninstall {
            self.start_nes_module_uninstall();
        }

        ui.add_space(16.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nes_module_has_a_human_friendly_nintendo_system_name() {
        assert_eq!(module_display_name("nes"), "Nintendo (NES)");
    }

    #[test]
    fn module_inventory_deduplicates_built_in_catalogue_and_installed_entries() {
        let registered = ["game-boy".to_owned(), "nes".to_owned()]
            .into_iter()
            .collect();
        let available = ["nes".to_owned(), "nintendo-switch".to_owned()]
            .into_iter()
            .collect();
        let installed = ["nes".to_owned()].into_iter().collect();

        let inventory = module_inventory(&registered, &available, &installed);
        assert_eq!(inventory.len(), 3);
        assert!(inventory.contains("nes"));
        assert!(inventory.contains("game-boy"));
        assert!(inventory.contains("nintendo-switch"));
    }

    #[test]
    fn module_display_names_are_readable_for_hyphenated_ids() {
        assert_eq!(module_display_name("game-boy"), "Game Boy");
        assert_eq!(module_display_name("nintendo-switch"), "Nintendo Switch");
    }

    #[test]
    fn consoles_are_grouped_by_manufacturer() {
        assert_eq!(module_brand_index("nes"), 0);
        assert_eq!(module_brand_index("nintendo-switch"), 0);
        assert_eq!(module_brand_index("xbox-series-x"), 1);
        assert_eq!(module_brand_index("playstation-5"), 2);
        assert_eq!(module_brand_index("ps2"), 2);
    }

    #[test]
    fn installed_path_version_uses_the_release_version_not_the_tag_prefix() {
        assert_eq!(
            version_from_installed_path(std::path::Path::new(
                "modules/nes/nes-module-0.2.3/peaceemu-nes-desktop.exe"
            )),
            Some("0.2.3".to_owned())
        );
    }
}
