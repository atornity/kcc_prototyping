use bevy::prelude::*;

pub mod camera;
pub mod character;
pub mod input;
pub mod level;
pub mod move_and_slide;
pub mod movement;

#[derive(Component)]
#[relationship(relationship_target = Attachments)]
pub struct AttachedTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = AttachedTo)]
pub struct Attachments(Vec<Entity>); // not sure about generaling this 
