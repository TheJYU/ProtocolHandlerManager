use eframe::egui;
use crate::config::Config;
use crate::registry::{register_protocol, unregister_protocol};

pub struct ProtocolApp {
    config: Config,
    new_protocol: String,
    new_target: String,
    error_message: Option<String>,
}

impl Default for ProtocolApp {
    fn default() -> Self {
        Self {
            config: Config::load(),
            new_protocol: String::new(),
            new_target: String::new(),
            error_message: None,
        }
    }
}

impl eframe::App for ProtocolApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_max_width(550.0);
                ui.add_space(20.0);

                ui.heading(egui::RichText::new("Protocol Handler Manager").size(24.0));
                ui.add_space(15.0);

                if let Some(msg) = &self.error_message {
                    ui.group(|ui| {
                        ui.colored_label(egui::Color32::RED, msg);
                    });
                    ui.add_space(10.0);
                }

                // --- ADD NEW SECTION ---
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Add or Update Mapping").strong());
                    });
                    ui.add_space(10.0);
                    
                    let mut trigger_add = false;

                    egui::Grid::new("add_grid")
                        .num_columns(2)
                        .spacing([15.0, 15.0])
                        .min_col_width(120.0)
                        .show(ui, |ui| {
                            ui.label("Protocol Name:");
                            ui.horizontal(|ui| {
                                let resp = ui.text_edit_singleline(&mut self.new_protocol);
                                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    trigger_add = true;
                                }
                                ui.label("://");
                            });
                            ui.end_row();

                            ui.label("Target Executable:");
                            ui.horizontal(|ui| {
                                let resp = ui.text_edit_singleline(&mut self.new_target);
                                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    trigger_add = true;
                                }
                                if ui.button("📁 Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_title("Select Target Executable")
                                        .pick_file() 
                                    {
                                        self.new_target = path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();
                        });

                    ui.add_space(15.0);
                    ui.vertical_centered(|ui| {
                        if ui.button(egui::RichText::new("💾 Add / Update Mapping").strong()).clicked() || trigger_add {
                            if self.new_protocol.is_empty() || self.new_target.is_empty() {
                                self.error_message = Some("Protocol and target cannot be empty".into());
                            } else if !self.new_protocol.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.') {
                                self.error_message = Some("Protocol can only contain letters, numbers, hyphens (-), and dots (.)".into());
                            } else {
                                if let Err(e) = self.config.add_mapping(self.new_protocol.clone(), self.new_target.clone()) {
                                    self.error_message = Some(e);
                                } else if let Err(e) = register_protocol(&self.new_protocol) {
                                    self.error_message = Some(e);
                                } else {
                                    self.error_message = None;
                                    self.new_protocol.clear();
                                    self.new_target.clear();
                                }
                            }
                            }
                            });
                            ui.add_space(5.0);
                            });

                            ui.add_space(20.0);

                            // --- REGISTERED LIST SECTION ---
                            ui.group(|ui| {
                            ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Registered Protocols").strong());
                            });
                            ui.add_space(10.0);

                            let mut to_remove = Vec::new();

                            let mut sorted_mappings: Vec<_> = self.config.mappings.iter().collect();
                            sorted_mappings.sort_by(|a, b| a.0.cmp(b.0));

                            egui::ScrollArea::vertical()
                            .min_scrolled_height(150.0)
                            .max_height(300.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                            let grid_width = ui.available_width() - 30.0; // Account for scrollbar and padding
                            egui::Grid::new("protocols_grid")
                            .num_columns(3)
                            .spacing([10.0, 10.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (protocol, target) in sorted_mappings {
                                    // Column 1: Protocol (Clickable)
                                    let protocol_text = format!("{}://", protocol);
                                    let resp = ui.add_sized([100.0, 20.0], egui::Label::new(
                                        egui::RichText::new(&protocol_text)
                                    ).truncate().sense(egui::Sense::click()));

                                    if resp.clicked() {
                                        self.new_protocol = protocol.clone();
                                        self.new_target = target.clone();
                                    }

                                    // Column 2: Target Path (Clickable)
                                    let path_width = grid_width - 100.0 - 90.0;
                                    let resp = ui.add_sized([path_width, 20.0], egui::Label::new(target)
                                        .truncate()
                                        .sense(egui::Sense::click()));

                                    if resp.clicked() {
                                        self.new_protocol = protocol.clone();
                                        self.new_target = target.clone();
                                    }
                                    // Column 3: Remove Button (Fixed width at the edge)
                                    if ui.add_sized([80.0, 20.0], egui::Button::new("🗑 Remove")).clicked() {
                                        to_remove.push(protocol.clone());
                                    }
                                    ui.end_row();
                                }
                            });
                            });

                            for p in to_remove {
                            let _ = self.config.remove_mapping(&p);
                            let _ = unregister_protocol(&p);
                            }
                            });
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.weak(format!("Protocol Handler Manager v{}", env!("CARGO_PKG_VERSION")));
                    ui.weak(egui::RichText::new("Created by TheJYU").italics())
                        .on_hover_text("https://github.com/TheJYU");
                });
            });
        });
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([650.0, 670.0])
            .with_min_inner_size([600.0, 450.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Protocol Handler Manager",
        options,
        Box::new(|_cc| Ok(Box::new(ProtocolApp::default()))),
    ).unwrap();
}
