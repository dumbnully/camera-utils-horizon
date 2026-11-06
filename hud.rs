fn cu_hud(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    crate::game::camera_utils::hud(game, batch);
}
