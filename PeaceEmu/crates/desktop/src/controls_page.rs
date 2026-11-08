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

/// Draws a fully native NES controller with egui primitives (no bitmap texture).
/// Returns the physical action clicked by the user, if any.
fn draw_nes_controller(
    ui: &mut egui::Ui,
    palette: &theme::Palette,
    selected_action: usize,
) -> Option<usize> {
    let available = ui.available_width().min(820.0);
    let size = egui::vec2(available, (available / 2.25).min(360.0));
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let shell = rect.shrink2(egui::vec2(rect.width() * 0.015, rect.height() * 0.07));
    let frame = shell.shrink2(egui::vec2(rect.width() * 0.035, rect.height() * 0.075));
    let face = frame.shrink2(egui::vec2(rect.width() * 0.012, rect.height() * 0.025));

    painter.rect_filled(
        shell.translate(egui::vec2(0.0, 3.0)),
        egui::Rounding::same(24.0),
        egui::Color32::from_rgb(95, 99, 106),
    );
    painter.rect_filled(
        shell,
        egui::Rounding::same(24.0),
        egui::Color32::from_rgb(185, 190, 197),
    );
    painter.rect_stroke(
        shell,
        egui::Rounding::same(24.0),
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(115, 120, 128)),
    );
    painter.rect_filled(
        frame,
        egui::Rounding::same(18.0),
        egui::Color32::from_rgb(27, 29, 34),
    );
    painter.rect_filled(
        face,
        egui::Rounding::same(13.0),
        egui::Color32::from_rgb(211, 214, 218),
    );

    let point = |x: f32, y: f32| {
        egui::pos2(
            rect.left() + rect.width() * x,
            rect.top() + rect.height() * y,
        )
    };
    let control_rect = |x: f32, y: f32, w: f32, h: f32| {
        egui::Rect::from_center_size(point(x, y), egui::vec2(rect.width() * w, rect.height() * h))
    };

    // D-pad: four distinct, clickable directions around a dark central hub.
    let dpad_parts = [
        (0usize, control_rect(0.225, 0.365, 0.075, 0.205)),
        (1usize, control_rect(0.225, 0.635, 0.075, 0.205)),
        (2usize, control_rect(0.155, 0.500, 0.105, 0.115)),
        (3usize, control_rect(0.295, 0.500, 0.105, 0.115)),
    ];
    let dpad_color = egui::Color32::from_rgb(34, 36, 41);
    for (_, part) in dpad_parts {
        painter.rect_filled(part, egui::Rounding::same(5.0), dpad_color);
        painter.rect_stroke(
            part,
            egui::Rounding::same(5.0),
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(12, 13, 16)),
        );
    }
    painter.circle_filled(point(0.225, 0.5), rect.height() * 0.047, dpad_color);
    painter.circle_stroke(
        point(0.225, 0.5),
        rect.height() * 0.047,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(12, 13, 16)),
    );

    // Select and Start are separate dark, slightly rounded oblong switches.
    let select_rect = control_rect(0.405, 0.625, 0.105, 0.075);
    let start_rect = control_rect(0.535, 0.625, 0.105, 0.075);
    for button in [select_rect, start_rect] {
        painter.rect_filled(
            button,
            egui::Rounding::same(8.0),
            egui::Color32::from_rgb(43, 45, 50),
        );
        painter.rect_stroke(
            button,
            egui::Rounding::same(8.0),
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 16, 19)),
        );
    }

    // The two iconic red action buttons.
    let b_center = point(0.690, 0.625);
    let a_center = point(0.815, 0.625);
    let button_radius = rect.height() * 0.105;
    for center in [b_center, a_center] {
        painter.circle_filled(
            center + egui::vec2(0.0, 2.0),
            button_radius + 2.0,
            egui::Color32::from_rgb(80, 25, 28),
        );
        painter.circle_filled(center, button_radius, egui::Color32::from_rgb(190, 38, 48));
        painter.circle_stroke(
            center,
            button_radius,
            egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(105, 24, 30)),
        );
    }

    // Console branding and control labels.
    painter.text(
        point(0.515, 0.285),
        egui::Align2::CENTER_CENTER,
        "NINTENDO",
        egui::FontId::proportional(rect.height() * 0.075),
        egui::Color32::from_rgb(55, 58, 64),
    );
    painter.text(
        point(0.515, 0.365),
        egui::Align2::CENTER_CENTER,
        "ENTERTAINMENT SYSTEM",
        egui::FontId::proportional(rect.height() * 0.027),
        egui::Color32::from_rgb(75, 78, 84),
    );
    for (label, center) in [
        ("SELECT", point(0.405, 0.735)),
        ("START", point(0.535, 0.735)),
    ] {
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(rect.height() * 0.034),
            egui::Color32::from_rgb(55, 57, 62),
        );
    }
    for (label, center) in [("B", b_center), ("A", a_center)] {
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(rect.height() * 0.11),
            egui::Color32::WHITE,
        );
    }

    let mut hit_targets = dpad_parts
        .iter()
        .map(|(index, target)| (*index, *target))
        .collect::<Vec<_>>();
    hit_targets.extend([(9, select_rect), (8, start_rect)]);
    hit_targets.push((
        5,
        egui::Rect::from_center_size(
            b_center,
            egui::vec2(button_radius * 2.4, button_radius * 2.4),
        ),
    ));
    hit_targets.push((
        4,
        egui::Rect::from_center_size(
            a_center,
            egui::vec2(button_radius * 2.4, button_radius * 2.4),
        ),
    ));

    let mut clicked = None;
    for (index, target) in hit_targets {
        let response = ui
            .interact(
                target,
                egui::Id::new(("nes-controller-hit-target", index)),
                egui::Sense::click(),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            clicked = Some(index);
        }
    }

    // Selection glow follows the actual shape of the chosen control.
    let accent_fill = palette.accent.linear_multiply(0.30);
    let accent_stroke = egui::Stroke::new(3.0_f32, palette.accent);
    match selected_action {
        0..=3 => {
            if let Some((_, target)) = dpad_parts
                .iter()
                .find(|(index, _)| *index == selected_action)
            {
                painter.rect_filled(*target, egui::Rounding::same(5.0), accent_fill);
                painter.rect_stroke(*target, egui::Rounding::same(5.0), accent_stroke);
            }
        }
        4 | 5 => {
            let center = if selected_action == 4 {
                a_center
            } else {
                b_center
            };
            painter.circle_filled(center, button_radius, accent_fill);
            painter.circle_stroke(center, button_radius, accent_stroke);
            painter.circle_stroke(
                center,
                button_radius + 5.0,
                egui::Stroke::new(2.0_f32, palette.accent.linear_multiply(0.55)),
            );
        }
        8 | 9 => {
            let target = if selected_action == 8 {
                start_rect
            } else {
                select_rect
            };
            painter.rect_filled(target, egui::Rounding::same(8.0), accent_fill);
            painter.rect_stroke(target, egui::Rounding::same(8.0), accent_stroke);
        }
        _ => {}
    }

    clicked
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

                    if let Some(index) = draw_nes_controller(ui, &palette, selected_action) {
                        selected_action = index;
                        clicked = Some(index);
                    }

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
