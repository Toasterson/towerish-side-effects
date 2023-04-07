use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Bundle)]
pub struct PhysicsBundle {
    flags: ActiveEvents,
    active_collition_types: ActiveCollisionTypes,
    collider: Collider,
    colliding_entities: CollidingEntities,
    rigid_body: RigidBody,
    rotation_contraint: LockedAxes,
    velocity: Velocity,
}

impl PhysicsBundle {
    pub fn moving_entity() -> Self {
        Self {
            flags: ActiveEvents::COLLISION_EVENTS,
            active_collition_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            collider: Collider::ball(0.5),
            colliding_entities: CollidingEntities::default(),
            rigid_body: RigidBody::Dynamic,
            rotation_contraint: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::zero(),
        }
    }
    pub fn from_mesh(
        mesh: &Mesh,
        collider_shape: &ComputedColliderShape,
    ) -> Self {
        let collider = Collider::from_bevy_mesh(mesh, collider_shape).unwrap();
        //collider.set_scale(Vec3::new(2.0, 2.0, 2.0), 1);
        Self {
            flags: ActiveEvents::COLLISION_EVENTS,
            active_collition_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
            collider,
            colliding_entities: CollidingEntities::default(),
            rigid_body: RigidBody::Dynamic,
            rotation_contraint: LockedAxes::ROTATION_LOCKED,
            velocity: Velocity::zero(),
        }
    }

    pub fn make_kinematic(mut self) -> Self {
        self.rigid_body = RigidBody::KinematicPositionBased;
        self
    }

    pub fn make_fixed(mut self) -> Self {
        self.rigid_body = RigidBody::Fixed;
        self
    }

    pub fn set_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }
}

pub fn physics_plugin(_app: &mut App) {}
