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
    let width = ui.available_width().min(900.0);
    let size = egui::vec2(width, width * 0.430);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let p = |x: f32, y: f32| {
        egui::pos2(
            rect.left() + rect.width() * x,
            rect.top() + rect.height() * y,
        )
    };
    let r = |x: f32, y: f32, w: f32, h: f32| {
        egui::Rect::from_center_size(p(x, y), egui::vec2(rect.width() * w, rect.height() * h))
    };
    let light = egui::Color32::from_rgb(246, 247, 250);
    let logo = egui::Color32::from_rgb(190, 62, 72);

    // Moulded plastic shell: soft outer shadow, dark lower edge and stepped
    // highlights create a physical bevel without using a controller bitmap.
    let shell = r(0.5, 0.595, 0.992, 0.805);
    painter.rect_filled(
        shell.translate(egui::vec2(0.0, 5.0)),
        egui::Rounding::same(18.0),
        egui::Color32::from_rgb(76, 78, 83),
    );
    painter.rect_filled(
        shell.translate(egui::vec2(0.0, 2.5)),
        egui::Rounding::same(17.0),
        egui::Color32::from_rgb(151, 153, 159),
    );
    painter.rect_filled(
        shell,
        egui::Rounding::same(17.0),
        egui::Color32::from_rgb(220, 222, 226),
    );
    painter.rect_stroke(
        shell,
        egui::Rounding::same(17.0),
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(250, 250, 251)),
    );
    let inner_shell = shell.shrink2(egui::vec2(3.0, 3.0));
    painter.rect_stroke(
        inner_shell,
        egui::Rounding::same(14.0),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(187, 189, 194)),
    );

    // Fine mould lines on the top and bottom edges of the casing.
    painter.line_segment(
        [p(0.035, 0.245), p(0.965, 0.245)],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(194, 196, 201)),
    );
    painter.line_segment(
        [p(0.035, 0.945), p(0.965, 0.945)],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(174, 176, 182)),
    );
    let face = r(0.5, 0.655, 0.936, 0.575);
    painter.rect_filled(
        face.translate(egui::vec2(0.0, 2.5)),
        egui::Rounding::same(7.0),
        egui::Color32::from_rgb(8, 9, 11),
    );
    painter.rect_filled(
        face,
        egui::Rounding::same(7.0),
        egui::Color32::from_rgb(24, 25, 28),
    );
    painter.rect_stroke(
        face,
        egui::Rounding::same(7.0),
        egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(91, 92, 96)),
    );
    painter.line_segment(
        [
            face.left_top() + egui::vec2(8.0, 4.0),
            face.right_top() + egui::vec2(-8.0, 4.0),
        ],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(57, 58, 62)),
    );

    // Silver inset panel, with the characteristic upper and lower centre tabs.
    let panel = r(0.5, 0.655, 0.90, 0.515);
    painter.rect_filled(
        panel.translate(egui::vec2(0.0, 2.0)),
        egui::Rounding::same(12.0),
        egui::Color32::from_rgb(91, 93, 98),
    );
    painter.rect_filled(
        panel,
        egui::Rounding::same(12.0),
        egui::Color32::from_rgb(205, 207, 208),
    );
    painter.rect_stroke(
        panel,
        egui::Rounding::same(12.0),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 227, 228)),
    );
    for (y, h) in [(0.355, 0.15), (0.905, 0.105)] {
        let tab = r(0.49, y, 0.275, h);
        painter.rect_filled(
            tab.translate(egui::vec2(0.0, 2.0)),
            egui::Rounding::same(8.0),
            egui::Color32::from_rgb(107, 108, 109),
        );
        painter.rect_filled(
            tab,
            egui::Rounding::same(8.0),
            egui::Color32::from_rgb(176, 176, 172),
        );
    }

    // Three red graphic bands on the right: two broad bars and one thin stripe.
    for (y, h) in [(0.455, 0.085), (0.605, 0.055), (0.775, 0.022)] {
        let band = r(0.765, y, 0.335, h);
        painter.rect_filled(band, egui::Rounding::ZERO, egui::Color32::from_rgb(232, 25, 37));
    }

    // D-pad with a bright silver raised surround, dark recess and sculpted cross.
    let dc = p(0.175, 0.655);
    let unit = rect.height() * 0.130;
    let dpad_well = egui::Rect::from_center_size(dc, egui::vec2(unit * 3.55, unit * 3.55));
    painter.rect_filled(
        dpad_well.translate(egui::vec2(0.0, 2.0)),
        egui::Rounding::same(7.0),
        egui::Color32::from_rgb(8, 9, 11),
    );
    painter.rect_filled(
        dpad_well,
        egui::Rounding::same(7.0),
        egui::Color32::from_rgb(33, 34, 38),
    );
    painter.rect_stroke(
        dpad_well,
        egui::Rounding::same(7.0),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(74, 76, 81)),
    );
    let cross = |scale: f32| {
        let u = unit * scale;
        vec![
            egui::pos2(dc.x - u * 0.39, dc.y - u * 1.40),
            egui::pos2(dc.x + u * 0.39, dc.y - u * 1.40),
            egui::pos2(dc.x + u * 0.39, dc.y - u * 0.39),
            egui::pos2(dc.x + u * 1.40, dc.y - u * 0.39),
            egui::pos2(dc.x + u * 1.40, dc.y + u * 0.39),
            egui::pos2(dc.x + u * 0.39, dc.y + u * 0.39),
            egui::pos2(dc.x + u * 0.39, dc.y + u * 1.40),
            egui::pos2(dc.x - u * 0.39, dc.y + u * 1.40),
            egui::pos2(dc.x - u * 0.39, dc.y + u * 0.39),
            egui::pos2(dc.x - u * 1.40, dc.y + u * 0.39),
            egui::pos2(dc.x - u * 1.40, dc.y - u * 0.39),
            egui::pos2(dc.x - u * 0.39, dc.y - u * 0.39),
        ]
    };
    painter.add(egui::Shape::convex_polygon(
        cross(1.22),
        egui::Color32::from_rgb(185, 187, 192),
        egui::Stroke::new(2.5_f32, light),
    ));
    painter.add(egui::Shape::convex_polygon(
        cross(1.06),
        egui::Color32::from_rgb(15, 16, 18),
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(92, 94, 99)),
    ));
    painter.add(egui::Shape::convex_polygon(
        cross(0.91),
        egui::Color32::from_rgb(48, 50, 54),
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(9, 10, 12)),
    ));
    let dpad = [
        (0, r(0.175, 0.525, 0.075, 0.155)),
        (1, r(0.175, 0.785, 0.075, 0.155)),
        (2, r(0.105, 0.655, 0.105, 0.105)),
        (3, r(0.245, 0.655, 0.105, 0.105)),
    ];
    let arrows = [
        (p(0.175, 0.525), [0.0, -1.0]),
        (p(0.175, 0.785), [0.0, 1.0]),
        (p(0.105, 0.655), [-1.0, 0.0]),
        (p(0.245, 0.655), [1.0, 0.0]),
    ];
    for (center, direction) in arrows {
        let (dx, dy) = (direction[0], direction[1]);
        let side = unit * 0.28;
        let vertices = if dx == 0.0 {
            vec![
                center + egui::vec2(0.0, dy * side),
                center + egui::vec2(-side * 0.62, -dy * side * 0.42),
                center + egui::vec2(side * 0.62, -dy * side * 0.42),
            ]
        } else {
            vec![
                center + egui::vec2(dx * side, 0.0),
                center + egui::vec2(-dx * side * 0.42, -side * 0.62),
                center + egui::vec2(-dx * side * 0.42, side * 0.62),
            ]
        };
        painter.add(egui::Shape::convex_polygon(
            vertices,
            egui::Color32::from_rgb(132, 135, 141),
            egui::Stroke::new(0.9_f32, egui::Color32::from_rgb(5, 6, 8)),
        ));
    }
    painter.circle_filled(dc, unit * 0.22, egui::Color32::from_rgb(7, 8, 10));
    painter.circle_filled(
        dc - egui::vec2(unit * 0.035, unit * 0.045),
        unit * 0.105,
        egui::Color32::from_rgb(151, 154, 160),
    );
    painter.circle_stroke(
        dc,
        unit * 0.22,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(165, 168, 174)),
    );

    // Red SELECT / START legends sit above two narrow, pill-shaped rubber keys.
    for (label, center) in [("SELECT", p(0.405, 0.895)), ("START", p(0.535, 0.895))] {
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(rect.height() * 0.060),
            logo,
        );
    }
    let plate = r(0.465, 0.905, 0.275, 0.105);
    painter.rect_filled(
        plate.translate(egui::vec2(0.0, 2.0)),
        egui::Rounding::same(7.0),
        egui::Color32::from_rgb(139, 142, 149),
    );
    painter.rect_filled(plate, egui::Rounding::same(7.0), light);
    painter.rect_stroke(
        plate,
        egui::Rounding::same(7.0),
        egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(224, 226, 230)),
    );
    let select = r(0.405, 0.705, 0.090, 0.070);
    let start = r(0.535, 0.705, 0.090, 0.070);
    for button in [select, start] {
        let rounding = egui::Rounding::same(button.height() * 0.5);
        // Recessed socket, then the raised dark-grey rubber cap.
        painter.rect_filled(
            button.expand(3.0).translate(egui::vec2(0.0, 1.5)),
            egui::Rounding::same(button.height() * 0.65),
            egui::Color32::from_rgb(125, 127, 133),
        );
        painter.rect_filled(
            button.expand(1.5),
            egui::Rounding::same(button.height() * 0.62),
            egui::Color32::from_rgb(31, 32, 35),
        );
        painter.rect_filled(
            button.translate(egui::vec2(0.0, 1.5)),
            rounding,
            egui::Color32::from_rgb(12, 13, 15),
        );
        painter.rect_filled(button, rounding, egui::Color32::from_rgb(57, 59, 64));
        painter.rect_stroke(
            button,
            rounding,
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(29, 30, 33)),
        );
        painter.line_segment(
            [
                button.left_center() + egui::vec2(button.height() * 0.55, -1.0),
                button.right_center() - egui::vec2(button.height() * 0.55, 1.0),
            ],
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(112, 114, 119)),
        );
    }

    painter.text(
        p(0.465, 0.455),
        egui::Align2::CENTER_CENTER,
        "Nintendo",
        egui::FontId::proportional(rect.height() * 0.087),
        logo,
    );
    painter.text(
        p(0.465, 0.515),
        egui::Align2::CENTER_CENTER,
        "ENTERTAINMENT SYSTEM",
        egui::FontId::monospace(rect.height() * 0.038),
        egui::Color32::from_rgb(18, 19, 21),
    );

    // Separate square silver wells and glossy domed red A/B caps.
    let b = p(0.700, 0.705);
    let a = p(0.830, 0.705);
    let radius = rect.height() * 0.087;
    let mut action_targets = Vec::new();
    for (label, center) in [("B", b), ("A", a)] {
        let bezel = egui::Rect::from_center_size(center, egui::vec2(radius * 2.65, radius * 2.65));
        painter.rect_filled(
            bezel.translate(egui::vec2(0.0, 2.0)),
            egui::Rounding::same(5.0),
            egui::Color32::from_rgb(139, 142, 149),
        );
        painter.rect_filled(bezel, egui::Rounding::same(5.0), light);
        painter.rect_stroke(
            bezel,
            egui::Rounding::same(5.0),
            egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(223, 225, 229)),
        );
        // Deep square socket, dark rubber shadow and a domed red cap with
        // several restrained specular layers to suggest curved plastic.
        painter.rect_filled(
            bezel.shrink(5.0).translate(egui::vec2(0.0, 2.5)),
            egui::Rounding::same(4.0),
            egui::Color32::from_rgb(112, 114, 119),
        );
        painter.circle_filled(
            center + egui::vec2(0.0, 3.0),
            radius * 1.05,
            egui::Color32::from_rgb(83, 10, 18),
        );
        painter.circle_filled(center, radius * 0.99, egui::Color32::from_rgb(139, 17, 29));
        painter.circle_filled(
            center - egui::vec2(0.0, radius * 0.07),
            radius * 0.91,
            egui::Color32::from_rgb(190, 27, 39),
        );
        painter.circle_filled(
            center - egui::vec2(radius * 0.20, radius * 0.30),
            radius * 0.58,
            egui::Color32::from_rgb(215, 48, 57),
        );
        painter.circle_filled(
            center - egui::vec2(radius * 0.29, radius * 0.39),
            radius * 0.20,
            egui::Color32::from_rgb(244, 111, 115),
        );
        painter.circle_stroke(
            center,
            radius * 0.98,
            egui::Stroke::new(1.2_f32, egui::Color32::from_rgb(116, 12, 23)),
        );
        painter.text(
            center + egui::vec2(0.0, radius * 0.06),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(radius * 0.70),
            egui::Color32::from_rgb(255, 238, 231),
        );
        action_targets.push(egui::Rect::from_center_size(
            center,
            egui::vec2(radius * 2.4, radius * 2.4),
        ));
    }
    let mut targets = dpad
        .iter()
        .map(|(i, target)| (*i, *target))
        .collect::<Vec<_>>();
    targets.extend([
        (9, select),
        (8, start),
        (5, action_targets[0]),
        (4, action_targets[1]),
    ]);
    let mut clicked = None;
    for (index, target) in targets {
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

    // Selection tint overlays only the corresponding physical key.
    let fill = palette.accent.linear_multiply(0.32);
    let stroke = egui::Stroke::new(2.5_f32, palette.accent);
    match selected_action {
        0..=3 => {
            if let Some((_, target)) = dpad.iter().find(|(i, _)| *i == selected_action) {
                painter.rect_filled(*target, egui::Rounding::same(4.0), fill);
                painter.rect_stroke(*target, egui::Rounding::same(4.0), stroke);
            }
        }
        4 | 5 => {
            let center = if selected_action == 4 { a } else { b };
            painter.circle_filled(center, radius * 0.94, fill);
            painter.circle_stroke(center, radius * 0.94, stroke);
        }
        8 | 9 => {
            let target = if selected_action == 8 { start } else { select };
            painter.rect_filled(target, egui::Rounding::same(11.0), fill);
            painter.rect_stroke(target, egui::Rounding::same(11.0), stroke);
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
