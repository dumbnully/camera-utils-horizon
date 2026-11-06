fn cu_camera(
    game: &mut crate::game::Game<'_>,
    partial: f32,
) -> std::ops::ControlFlow<Option<(glam::DVec3, f32, f32)>> {
    fn original(game: &mut crate::game::Game<'_>, partial: f32) -> Option<(glam::DVec3, f32, f32)> {
        let (entity, descending) = match game.camera_mob() {
            Some(m) => (super::remote_camera::mob_camera_entity(m), false),
            None => {
                let mut entity = game.player_camera_entity()?;
                if let Some(position) = game.remote_minecart_rider_position(partial) {
                    entity.position = position;
                    entity.previous = position;
                }
                (entity, game.player.as_ref()?.shift_key_down)
            }
        };
        let mut view = game.view;
        let kind = match if crate::game::camera_utils::special(game) {
            CameraType::FirstPerson
        } else {
            game.camera_type
        } {
            CameraType::FirstPerson => horizon_client::camera::CameraType::FirstPerson,
            CameraType::ThirdPersonBack => horizon_client::camera::CameraType::ThirdPersonBack,
            CameraType::ThirdPersonFront => horizon_client::camera::CameraType::ThirdPersonFront,
        };
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(0),
                y: entity.position[1],
                descending,
                main_hand: Some(CollisionItem::EMPTY),
            },
            false,
        );
        let mut rays = VisualRays {
            scene: SceneCollisions {
                world: &game.world,
                binding: game.collision,
                subject: None,
            },
            facts: game.res.facts(),
            context,
        };
        view.align_with_entity_distance(
            &entity,
            partial,
            kind,
            |distance| {
                let mut distance = distance;
                game.native_camera_distance(partial, kind.is_mirrored(), &entity, &mut distance);
                distance
            },
            |from, to| {
                let hit = clip_world(
                    &mut rays,
                    Vec3::new(from[0], from[1], from[2]),
                    Vec3::new(to[0], to[1], to[2]),
                    RayBlockMode::Visual,
                    RayFluidMode::None,
                    4096,
                )
                .ok()?;
                (!hit.miss).then_some([hit.hit.location.x, hit.hit.location.y, hit.hit.location.z])
            },
        );
        crate::game::camera_utils::pose(game, &mut view);
        game.view = view;
        Some((
            glam::DVec3::from_array(view.position),
            view.y_rot,
            view.x_rot,
        ))
    }
    if !crate::game::camera_utils::special(game) {
        return std::ops::ControlFlow::Continue(());
    }
    std::ops::ControlFlow::Break(original(game, partial))
}
