use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::KCCMarker;

#[derive(Bundle)]
pub struct KCCBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    kcc_marker: KCCMarker,
}

impl Default for KCCBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(0.35, 1.0),
            rigid_body: RigidBody::Kinematic,
            kcc_marker: KCCMarker,
        }
    }
}
