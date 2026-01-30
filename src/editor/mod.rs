use std::sync::Arc;

use level_meter::PeakMeter;
use nih_plug::{editor::Editor, prelude::AtomicF32, util};
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, Button, RadioButton, Slider, Vec2},
    resizable_window::ResizableWindow,
};
use toggle::toggle_ui;

use crate::{PluginParams, voice::Waveform};

mod level_meter;
mod toggle;

pub(crate) fn create(
    params: Arc<PluginParams>,
    peak_meter: Arc<AtomicF32>,
) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    create_egui_editor(
        egui_state.clone(),
        (),
        |_, _| {},
        move |ctx, setter, _state| {
            ResizableWindow::new("res-wind")
                .min_size(Vec2::new(128.0, 128.0))
                .show(ctx, egui_state.as_ref(), |ui| {
                    // Add padding around the entire UI content
                    ui.add_space(20.0); // Top padding

                    // Add horizontal padding by using columns or spacing
                    ui.horizontal(|ui| {
                        ui.add_space(20.0); // Left padding
                        ui.vertical(|ui| {
                            ui.heading("swarmer");

                            ui.add_space(10.0);

                            // Gain
                            ui.horizontal(|ui| {
                                ui.label("Gain Slider");

                                ui.add(
                                    Slider::from_get_set(
                                        -10.0..=10.0,
                                        |new_value| match new_value {
                                            Some(new_value) => {
                                                setter.begin_set_parameter(&params.gain);
                                                setter
                                                    .set_parameter(&params.gain, new_value as f32);
                                                setter.end_set_parameter(&params.gain);

                                                new_value
                                            }
                                            None => params.gain.value() as f64,
                                        },
                                    )
                                    .show_value(true)
                                    .suffix(" dB"),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Waveform");

                                if ui.add(
                                    RadioButton::new(params.waveform.value() == Waveform::Saw, "Saw")
                                ).clicked() {
                                    setter.begin_set_parameter(&params.waveform);
                                    setter.set_parameter(&params.waveform, Waveform::Saw);
                                    setter.end_set_parameter(&params.waveform);
                                };

                                if ui.add(
                                    RadioButton::new(params.waveform.value() == Waveform::Sine, "Sine")
                                ).clicked() {
                                    setter.begin_set_parameter(&params.waveform);
                                    setter.set_parameter(&params.waveform, Waveform::Sine);
                                    setter.end_set_parameter(&params.waveform);
                                };
                            });

                            ui.add_space(10.0);

                            // Freq slider
                            ui.horizontal(|ui| {
                                ui.label("freq Slider");

                                ui.add(
                                    Slider::from_get_set(
                                        1.0..=20_000.0,
                                        |new_value| match new_value {
                                            Some(new_value) => {
                                                setter.begin_set_parameter(&params.frequency);
                                                setter.set_parameter(
                                                    &params.frequency,
                                                    new_value as f32,
                                                );
                                                setter.end_set_parameter(&params.frequency);

                                                new_value
                                            }
                                            None => params.frequency.value() as f64,
                                        },
                                    )
                                    .show_value(true)
                                    .suffix(" Hz"),
                                );
                            });

                            ui.add_space(10.0);

                            // Peak meter
                            ui.horizontal(|ui| {
                                ui.label("Peak Meter");
                                let peak_meter = util::gain_to_db(
                                    peak_meter.load(std::sync::atomic::Ordering::Relaxed),
                                );
                                ui.add(PeakMeter::new(-60.0..=0.0, peak_meter).show_label(false));
                            });

                            ui.add_space(10.0);

                            // Mute
                            ui.horizontal(|ui| {
                                ui.label("Mute");
                                let mut mute = params.mute.value();
                                if toggle_ui(ui, &mut mute).changed() {
                                    setter.begin_set_parameter(&params.mute);
                                    setter.set_parameter(&params.mute, mute);
                                    setter.end_set_parameter(&params.mute);
                                }
                            });

                            ui.add_space(20.0); // Bottom padding
                        });
                        ui.add_space(20.0); // Right padding
                    });
                });
        },
    )
}
