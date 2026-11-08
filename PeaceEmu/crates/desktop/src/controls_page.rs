//! Controls page: one keyboard mapping per console, drawn with the shared
//! theme (page header, card, palette) so it matches Appearance and Application.

use super::*;

/// Groups of actions, as (French title, English title, action indexes).
/// Indexes refer to `key_bindings::ACTIONS`.
/// The NES controller has only A/B: X/Y are not physical NES buttons and
/// must not be offered as remappable actions in this page.
const GROUPS: [(&str, &str, &[usize]); 3] = [
    ("Directions", "Directions", &[0, 1, 2, 3]),
    ("Boutons", "Buttons", &[4, 5]),
    ("Système", "System", &[8, 9]),
];

fn module_display_name(module_id: &str) -> String {
    match module_id {
        "nes" => "Nintendo (NES)".to_owned(),
        "game-boy" => "Game Boy".to_owned(),
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

/// A key shown as a rounded "keycap". While capturing, it switches to the
/// accent colours so the user sees which action is waiting for a key.
fn keycap(
    ui: &mut egui::Ui,
    palette: &theme::Palette,
    text: &str,
    capturing: bool,
) -> egui::Response {
    let (fill, stroke, color) = if capturing {
        (
            palette.accent_subtle,
            egui::Stroke::new(1.5_f32, palette.accent),
            palette.accent_text,
        )
    } else {
        (
            palette.panel,
            egui::Stroke::new(1.0_f32, palette.border),
            palette.text,
        )
    };
    let button = egui::Button::new(
        egui::RichText::new(text)
            .monospace()
            .size(13.0)
            .color(color),
    )
    .fill(fill)
    .stroke(stroke)
    .rounding(egui::Rounding::same(8.0))
    .min_size(egui::vec2(150.0, 32.0));
    ui.add(button)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Approximate centers of the eight physical NES controls in normalized image coordinates.
fn nes_button_anchor(index: usize, rect: egui::Rect) -> egui::Pos2 {
    // Coordinates are measured against the supplied 2400x1340 controller artwork.
    let (x, y) = match index {
        0 => (0.193, 0.455), // Up
        1 => (0.193, 0.735), // Down
        2 => (0.132, 0.595), // Left
        3 => (0.255, 0.595), // Right
        4 => (0.830, 0.690), // A
        5 => (0.700, 0.690), // B
        8 => (0.525, 0.680), // Start
        _ => (0.418, 0.680), // Select
    };
    egui::pos2(
        rect.left() + rect.width() * x,
        rect.top() + rect.height() * y,
    )
}

impl DesktopApp {
    pub(super) fn show_controls(&mut self, ui: &mut egui::Ui) {
        let palette = theme::Palette::new(self.dark_mode, self.accent_color);
        let french = self.language == "fr";
        let installed = self.installed_module_ids();

        theme::page_header(
            ui,
            &palette,
            if french { "Commandes" } else { "Controls" },
            if french {
                "Une configuration clavier par console."
            } else {
                "One keyboard layout per console."
            },
            |_ui| {},
        );

        if installed.is_empty() {
            theme::card(ui, &palette, |ui| {
                ui.set_width(ui.available_width());
                ui.label(if french {
                    "Installez un module depuis Consoles et modules pour configurer ses commandes."
                } else {
                    "Install a module from Systems to configure its controls."
                });
            });
            return;
        }

        // The selected console is kept in egui's memory, so the application
        // state does not need a new field.
        let selection_id = egui::Id::new("controls-selected-module");
        let stored: Option<String> = ui.data_mut(|data| data.get_temp(selection_id));
        let mut selected = match stored {
            Some(id) if installed.contains(&id) => id,
            _ => installed.iter().next().cloned().unwrap_or_default(),
        };

        // ------------------------------------------------------ console tabs
        let mut next_selected: Option<String> = None;
        ui.horizontal_wrapped(|ui| {
            for module_id in &installed {
                let is_selected = *module_id == selected;
                let text = egui::RichText::new(module_display_name(module_id))
                    .size(14.0)
                    .color(if is_selected {
                        palette.text
                    } else {
                        palette.text_dim
                    });
                let button = egui::Button::new(text)
                    .fill(if is_selected {
                        palette.border
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .stroke(egui::Stroke::NONE)
                    .rounding(egui::Rounding::same(8.0))
                    .min_size(egui::vec2(0.0, 32.0));
                if ui
                    .add(button)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                    && !is_selected
                {
                    next_selected = Some(module_id.clone());
                }
            }
        });
        if let Some(next) = next_selected {
            // Leaving a console cancels any pending key capture.
            self.binding_capture = None;
            ui.data_mut(|data| data.insert_temp(selection_id, next.clone()));
            selected = next;
        }

        if !self.key_bindings.contains_key(&selected) {
            let bindings = key_bindings::KeyBindings::load(self.config_path.as_deref(), &selected);
            self.key_bindings.insert(selected.clone(), bindings);
        }
        let labels: Vec<String> = match self.key_bindings.get(&selected) {
            Some(bindings) => (0..key_bindings::ACTIONS.len())
                .map(|index| bindings.display_label(index, french))
                .collect(),
            None => Vec::new(),
        };
        let capturing: Option<usize> = self
            .binding_capture
            .as_ref()
            .and_then(|(id, index)| (id == &selected).then_some(*index));

        // The NES image is bundled from the repository root and cached by egui.
        let controller_texture = if selected == "nes" {
            let texture_id = egui::Id::new("nes-controller-settings-texture");
            let cached: Option<egui::TextureHandle> =
                ui.ctx().data_mut(|data| data.get_temp(texture_id));
            if let Some(texture) = cached {
                Some(texture)
            } else {
                match image::load_from_memory(include_bytes!("../../../../NES-Controller.png")) {
                    Ok(decoded) => {
                        let rgba = decoded.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image =
                            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                        let texture = ui.ctx().load_texture(
                            "nes-controller-settings",
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        ui.ctx()
                            .data_mut(|data| data.insert_temp(texture_id, texture.clone()));
                        Some(texture)
                    }
                    Err(_) => None,
                }
            }
        } else {
            None
        };

        let action_id = egui::Id::new(("controls-selected-action", selected.as_str()));
        let mut selected_action: usize = ui.data_mut(|data| data.get_temp(action_id)).unwrap_or(0);
        if selected == "nes" && ![0, 1, 2, 3, 4, 5, 8, 9].contains(&selected_action) {
            selected_action = 0;
        }

        // ------------------------------------------------------------- card
        let mut clicked: Option<usize> = None;
        let mut reset_clicked = false;
        let mut cancel_clicked = false;
        ui.add_space(8.0);
        theme::card(ui, &palette, |ui| {
            ui.set_width(ui.available_width());
            if selected == "nes" {
                if let Some(texture) = controller_texture.as_ref() {
                    // Keep the supplied controller artwork intact: it is the
                    // visual reference, not a shape to be approximated in egui.
                    let image_size = texture.size_vec2();
                    let mut image_rect = egui::Rect::NOTHING;

                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(if french {
                                "Manette NES — Joueur 1"
                            } else {
                                "NES controller — Player 1"
                            })
                            .strong()
                            .size(16.0)
                            .color(palette.text),
                        );
                        ui.add_space(10.0);

                        let max_width = ui.available_width().min(820.0);
                        let max_height = (max_width / (image_size.x / image_size.y)).min(460.0);
                        let scale = (max_width / image_size.x)
                            .min(max_height / image_size.y)
                            .max(0.01);
                        let display_size = image_size * scale;
                        let response =
                            ui.add(egui::Image::new(&*texture).fit_to_exact_size(display_size));
                        image_rect = response.rect;

                        // The artwork itself is interactive: invisible hit areas sit
                        // over the eight physical controls, so users can select a
                        // button directly instead of having to find its list entry.
                        for index in [0usize, 1, 2, 3, 4, 5, 8, 9] {
                            let anchor = nes_button_anchor(index, image_rect);
                            let hit_size = if index == 4 || index == 5 {
                                egui::vec2(image_rect.width() * 0.075, image_rect.height() * 0.15)
                            } else if index == 8 || index == 9 {
                                egui::vec2(image_rect.width() * 0.065, image_rect.height() * 0.13)
                            } else {
                                egui::vec2(image_rect.width() * 0.075, image_rect.height() * 0.15)
                            };
                            let hit = ui
                                .interact(
                                    egui::Rect::from_center_size(anchor, hit_size),
                                    egui::Id::new(("nes-controller-hit-target", index)),
                                    egui::Sense::click(),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if hit.clicked() {
                                selected_action = index;
                                clicked = Some(index);
                            }
                        }

                        // Highlight the selected physical control directly on the
                        // artwork. A translucent fill and soft outer rings make it
                        // visible without hiding the original controller details.
                        let anchor = nes_button_anchor(selected_action, image_rect);
                        let radius = if selected_action == 4 || selected_action == 5 {
                            image_rect.width() * 0.036
                        } else if selected_action == 8 || selected_action == 9 {
                            image_rect.width() * 0.025
                        } else {
                            image_rect.width() * 0.031
                        };
                        let painter = ui.painter();
                        painter.circle_filled(anchor, radius, palette.accent.linear_multiply(0.28));
                        painter.circle_stroke(
                            anchor,
                            radius,
                            egui::Stroke::new(3.0_f32, palette.accent),
                        );
                        painter.circle_stroke(
                            anchor,
                            radius + 4.0,
                            egui::Stroke::new(2.0_f32, palette.accent.linear_multiply(0.55)),
                        );

                        ui.add_space(14.0);
                        ui.label(
                            egui::RichText::new(if french {
                                "Affectation des touches"
                            } else {
                                "Keyboard mapping"
                            })
                            .strong()
                            .color(palette.text_dim),
                        );
                        ui.add_space(8.0);

                        egui::Grid::new("nes-controller-key-mapping")
                            .num_columns(4)
                            .spacing(egui::vec2(14.0, 10.0))
                            .show(ui, |ui| {
                                for (position, index) in
                                    [0usize, 1, 2, 3, 4, 5, 8, 9].into_iter().enumerate()
                                {
                                    let (fr, en) = key_bindings::ACTIONS[index];
                                    ui.vertical_centered(|ui| {
                                        let selected_row = selected_action == index;
                                        let label = ui.selectable_label(
                                            selected_row,
                                            if french { fr } else { en },
                                        );
                                        if label.clicked() {
                                            selected_action = index;
                                        }

                                        let is_capturing = capturing == Some(index);
                                        let text = if is_capturing {
                                            if french {
                                                "Appuyez…".to_owned()
                                            } else {
                                                "Press…".to_owned()
                                            }
                                        } else {
                                            labels.get(index).cloned().unwrap_or_default()
                                        };
                                        let response = keycap(ui, &palette, &text, is_capturing);
                                        if response.clicked() {
                                            selected_action = index;
                                            clicked = Some(index);
                                        }
                                    });
                                    if position % 4 == 3 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                } else {
                    ui.label(if french {
                        "Impossible de charger NES-Controller.png depuis la racine du projet."
                    } else {
                        "Could not load NES-Controller.png from the project root."
                    });
                }
            } else {
                for (group_index, (fr_title, en_title, indexes)) in GROUPS.iter().enumerate() {
                    if group_index > 0 {
                        ui.add_space(6.0);
                        ui.separator();
                    }
                    ui.label(
                        egui::RichText::new(if french { *fr_title } else { *en_title })
                            .size(12.0)
                            .strong()
                            .color(palette.text_dim),
                    );
                    for &index in indexes.iter() {
                        let (fr, en) = key_bindings::ACTIONS[index];
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(if french { fr } else { en }).size(14.0));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let is_capturing = capturing == Some(index);
                                    let text: String = if is_capturing {
                                        (if french {
                                            "Appuyez sur une touche…"
                                        } else {
                                            "Press a key…"
                                        })
                                        .to_owned()
                                    } else {
                                        labels.get(index).cloned().unwrap_or_default()
                                    };
                                    if keycap(ui, &palette, &text, is_capturing).clicked() {
                                        clicked = Some(index);
                                    }
                                },
                            );
                        });
                    }
                }
            }
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(if french {
                        "Réinitialiser"
                    } else {
                        "Restore defaults"
                    })
                    .clicked()
                {
                    reset_clicked = true;
                }
                if capturing.is_some()
                    && ui
                        .button(if french { "Annuler" } else { "Cancel" })
                        .clicked()
                {
                    cancel_clicked = true;
                }
            });
        });
        if selected == "nes" {
            ui.data_mut(|data| data.insert_temp(action_id, selected_action));
        }
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(if french {
                "Cliquez directement sur un bouton de la manette ou sur une touche, puis appuyez sur la nouvelle. Échap annule. Si la touche est déjà utilisée, les deux actions échangent leurs touches."
            } else {
                "Click a controller button or a key, then press the new one. Esc cancels. If the key is already used, the two actions swap keys."
            })
            .size(13.0)
            .color(palette.text_dim),
        );

        // ------------------------------------------------------ apply input
        if let Some(index) = clicked {
            self.binding_capture = Some((selected.clone(), index));
        }
        if cancel_clicked {
            self.binding_capture = None;
        }
        if reset_clicked {
            if let Some(bindings) = self.key_bindings.get_mut(&selected) {
                bindings.reset();
            }
            self.binding_capture = None;
            self.persist_key_bindings(&selected);
        }

        let mut captured = None;
        if let Some((module_id, index)) = self.binding_capture.clone() {
            for event in ui.input(|input| input.events.clone()) {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } = event
                {
                    captured = Some((module_id.clone(), index, key));
                    break;
                }
            }
        }

        if let Some((module_id, index, key)) = captured {
            self.binding_capture = None;
            if key != egui::Key::Escape {
                let result = self
                    .key_bindings
                    .get_mut(&module_id)
                    .ok_or(usize::MAX)
                    .and_then(|bindings| bindings.assign_or_swap(index, key));
                match result {
                    Ok(swapped) => {
                        self.persist_key_bindings(&module_id);
                        if let Some(other) = swapped {
                            self.message = if french {
                                format!(
                                    "Touches échangées avec « {} ».",
                                    key_bindings::ACTIONS[other].0
                                )
                            } else {
                                format!("Keys swapped with {}.", key_bindings::ACTIONS[other].1)
                            };
                        }
                    }
                    Err(_) => {
                        self.message = if french {
                            "Cette touche n’est pas prise en charge.".to_owned()
                        } else {
                            "That key is not supported.".to_owned()
                        };
                    }
                }
            }
        }
        ui.add_space(16.0);
    }
}
