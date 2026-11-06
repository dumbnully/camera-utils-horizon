fn cu_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::camera_utils::settings(game, mod_id, buttons);
}
