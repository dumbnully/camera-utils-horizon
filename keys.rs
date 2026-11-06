fn cu_keys(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::camera_utils::key(game, key, pressed);
}
