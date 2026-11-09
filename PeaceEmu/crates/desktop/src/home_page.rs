//! Journal-only home page for the PeaceEmu Dev desktop frontend.

use super::*;

/// Render the Markdown journal maintained outside the application source.
fn render_dev_journal(ui: &mut egui::Ui, palette: &theme::Palette, markdown: &str) {
    for line in markdown.lines() {
        let line = line.trim();
        let display_line = line.replace("**", "");
        if line.is_empty() {
            ui.add_space(3.0);
        } else if line == "---" {
            ui.separator();
        } else if let Some(heading) = line.strip_prefix("## ") {
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(heading.replace("**", ""))
                    .strong()
                    .size(16.0),
            );
        } else if let Some(heading) = line.strip_prefix("# ") {
            ui.label(
                egui::RichText::new(heading.replace("**", ""))
                    .strong()
                    .size(19.0),
            );
        } else if let Some(heading) = line.strip_prefix("### ") {
            ui.label(
                egui::RichText::new(heading.replace("**", ""))
                    .strong()
                    .size(14.0),
            );
        } else if let Some(item) = line.strip_prefix("- ") {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("•").color(palette.accent));
                ui.label(egui::RichText::new(item.replace("**", "")).color(palette.text));
            });
        } else if line.starts_with("<!--") || line.starts_with("-->") {
            // Ignore Markdown comments.
        } else {
            ui.label(
                egui::RichText::new(display_line)
                    .color(palette.text)
                    .size(13.0),
            );
        }
    }
}

impl DesktopApp {
    pub(super) fn show_home(&mut self, ui: &mut egui::Ui) {
        let palette = theme::Palette::new(self.dark_mode, self.accent_color);
        let french = self.language == "fr";
        let channel = option_env!("PEACEEMU_CHANNEL").unwrap_or("dev");

        if channel != "stable" {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(if french {
                        "JOURNAL DE DÉVELOPPEMENT"
                    } else {
                        "DEVELOPMENT JOURNAL"
                    })
                    .size(theme::FONT_SECTION)
                    .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("DEV")
                            .size(10.0)
                            .strong()
                            .color(palette.text_dim),
                    );
                });
            });
            ui.add_space(10.0);
            theme::card(ui, &palette, |ui| {
                ui.set_width(ui.available_width());
                let journal = super::dev_journal::read_cached();
                if journal.is_empty() {
                    ui.label(
                        egui::RichText::new(if french {
                            "Le journal de développement sera disponible dès qu’une connexion sera établie."
                        } else {
                            "The development journal will appear once a connection is available."
                        })
                        .color(palette.text_dim),
                    );
                } else {
                    render_dev_journal(ui, &palette, &journal);
                }
            });
        }
    }
}
