fn cu_gui_layer(game: &crate::game::Game<'_>, layer: &str, canceled: &mut bool) {
    crate::game::camera_utils::hud_layer(game, layer, canceled);
}
