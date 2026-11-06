fn cu_release(game: &crate::game::Game<'_>, canceled: &mut bool) {
    crate::game::camera_utils::release(game, canceled);
}
