use egui::{Align2, Rect, Response, Sense, Ui, Vec2, WidgetInfo, WidgetType, emath::GuiRounding};

pub fn close_button_ui(ui: &mut Ui, inner_rect: Rect) -> Response {
    // Compute button geometry
    let button_center = Align2::RIGHT_CENTER
        .align_size_within_rect(Vec2::splat(inner_rect.height()), inner_rect)
        .center();
    let button_size = Vec2::splat(ui.spacing().icon_width);
    let button_rect = Rect::from_center_size(button_center, button_size);
    let button_rect = button_rect.round_to_pixels(ui.pixels_per_point());

    // Create interaction ID and response
    let close_id = ui.auto_id_with("window_close_button");
    let response = ui.interact(button_rect, close_id, Sense::click());
    response
        .widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), "Close window"));

    ui.expand_to_include_rect(response.rect);

    // Draw the "X" icon
    let visuals = ui.style().interact(&response);
    let rect = button_rect.shrink(2.0).expand(visuals.expansion);
    let stroke = visuals.fg_stroke;

    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);

    response
}
