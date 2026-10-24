use eframe::egui;

use crate::config::ButtonAction;

/// Standalone native vector toggle switch with smooth animated slide transition.
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
    let radius = 0.5 * rect.height();

    // Background track color interpolated between off (slate 203, 213, 225) and on (sky blue 2, 132, 199)
    let off_col = egui::Color32::from_rgb(203, 213, 225);
    let on_col = egui::Color32::from_rgb(2, 132, 199);
    let bg_col = egui::Color32::from_rgb(
        (off_col.r() as f32 + (on_col.r() as f32 - off_col.r() as f32) * how_on) as u8,
        (off_col.g() as f32 + (on_col.g() as f32 - off_col.g() as f32) * how_on) as u8,
        (off_col.b() as f32 + (on_col.b() as f32 - off_col.b() as f32) * how_on) as u8,
    );
    ui.painter().rect_filled(rect, radius, bg_col);

    // Circular white knob with subtle boundary stroke
    let circle_x =
        (rect.left() + radius) + ((rect.right() - radius) - (rect.left() + radius)) * how_on;
    let center = egui::pos2(circle_x, rect.center().y);
    ui.painter()
        .circle_filled(center, radius - 2.5, egui::Color32::WHITE);
    ui.painter().circle_stroke(
        center,
        radius - 2.5,
        egui::Stroke::new(0.5_f32, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 30)),
    );

    response
}

/// Convert HSV values (h: 0.0..1.0, s: 0.0..1.0, v: 0.0..1.0) to RGB Color32.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> egui::Color32 {
    let c = v * s;
    let h_prime = h.rem_euclid(1.0) * 6.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    egui::Color32::from_rgb(
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Return category badge text and distinct accent color for button actions.
pub fn action_badge(action: &ButtonAction) -> (&'static str, egui::Color32) {
    match action {
        ButtonAction::MouseButton(_) => ("CLICK", egui::Color32::from_rgb(2, 132, 199)),
        ButtonAction::DpiSwitch => ("DPI", egui::Color32::from_rgb(6, 182, 212)),
        ButtonAction::FireKey { .. } => ("FIRE", egui::Color32::from_rgb(239, 68, 68)),
        ButtonAction::Media(_) => ("MEDIA", egui::Color32::from_rgb(245, 158, 11)),
        ButtonAction::Keyboard { .. } => ("KEY", egui::Color32::from_rgb(139, 92, 246)),
        ButtonAction::Macro(_) => ("MACRO", egui::Color32::from_rgb(16, 185, 129)),
        ButtonAction::Disabled => ("OFF", egui::Color32::from_rgb(148, 163, 184)),
    }
}

/// Set up custom clean white theme for the application
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    visuals.panel_fill = egui::Color32::from_rgb(255, 255, 255);
    visuals.window_fill = egui::Color32::from_rgb(248, 250, 252);
    visuals.extreme_bg_color = egui::Color32::from_rgb(241, 245, 249);
    visuals.faint_bg_color = egui::Color32::from_rgb(248, 250, 252);

    visuals.override_text_color = Some(egui::Color32::from_rgb(15, 23, 42));

    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(2, 132, 199, 60);
    visuals.selection.stroke = egui::Stroke::NONE;

    visuals.slider_trailing_fill = true;

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240));
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 23, 42));
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

    // Visible light slate rail for sliders and inactive controls
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(226, 232, 240);
    visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(100, 116, 139));
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(219, 234, 254);
    visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
    visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(2, 132, 199);
    visuals.widgets.active.bg_stroke =
        egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(2, 132, 199));
    visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(15, 23, 42));
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);

    visuals.window_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(226, 232, 240));

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.slider_rail_height = 8.0;
    });
}
