//! Bedrock animation format parser (1.8.0 / 1.10.0), keyframe interpolation, and state controllers.

use std::collections::HashMap;
use glam::Vec3;

#[derive(Debug, Clone)]
pub enum KeyframeChannel {
    Position(Vec<(f32, Vec3)>),
    Rotation(Vec<(f32, Vec3)>),
    Scale(Vec<(f32, Vec3)>),
}

#[derive(Debug, Clone)]
pub struct BoneAnimation {
    pub name: String,
    pub positions: Vec<(f32, Vec3)>,
    pub rotations: Vec<(f32, Vec3)>,
    pub scales: Vec<(f32, Vec3)>,
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub loop_mode: bool,
    pub anim_length: f32,
    pub bones: HashMap<String, BoneAnimation>,
}

pub struct AnimationController {
    pub active_clips: Vec<(AnimationClip, f32, f32)>, // (clip, current_time, blend_weight)
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            active_clips: Vec::new(),
        }
    }
}
