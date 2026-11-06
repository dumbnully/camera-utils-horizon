fn cu_screen_key(game: &crate::game::Game<'_>, key: &str, canceled: &mut bool) {
    crate::game::camera_utils::screen_key(game, key, canceled);
}
