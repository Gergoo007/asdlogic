use crate::canvas::Vec2;

pub const GRID_SPACING: f32 = 16.0;

pub const NODE_RADIUS: f32 = 4.0;
pub const NODE_HITBOX: Vec2 = Vec2::new(1.0, 1.0);

pub const COMPONENT_THICKNESS: f32 = 1.5;
pub const WIRE_THICKNESS: f32 = 2.0;

// How much the mouse is allowed to move (px) between button press and release in order
// for the event to register as clicking the item and not dragging the item
pub const CLICK_ALLOWANCE: f32 = 3.0;
