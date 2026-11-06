fn cu_distance(
    game: &crate::game::Game<'_>,
    entity: &horizon_client::camera::CameraEntity,
    distance: &mut f32,
) {
    crate::game::camera_utils::distance(game, entity, distance);
}
