fn cu_fov(game: &crate::game::Game<'_>, base: f32, modifier: &mut f32) {
    crate::game::camera_utils::fov(game, base, modifier);
}
