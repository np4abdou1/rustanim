//! Relative pivot bone rig.
//! Prevents rig collapse and limb stretching by preserving vanilla Java base pivots
//! and applying Bedrock animation deltas purely in relative local space.

use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BoneTransform {
    pub base_pivot: Vec3,
    pub relative_offset: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl BoneTransform {
    pub fn new(base_pivot: Vec3) -> Self {
        Self {
            base_pivot,
            relative_offset: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.base_pivot + self.relative_offset)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
            * Mat4::from_translation(-self.base_pivot)
    }
}

pub struct BoneHierarchy {
    pub parent_indices: Vec<Option<usize>>,
    pub bone_names: Vec<String>,
}

pub struct RelativePivotRig {
    pub bones: HashMap<String, BoneTransform>,
}

impl RelativePivotRig {
    pub fn new_humanoid() -> Self {
        let mut bones = HashMap::new();
        // Vanilla humanoid base joint pivots
        bones.insert("head".into(), BoneTransform::new(Vec3::new(0.0, 24.0, 0.0)));
        bones.insert("body".into(), BoneTransform::new(Vec3::new(0.0, 24.0, 0.0)));
        bones.insert("right_arm".into(), BoneTransform::new(Vec3::new(-5.0, 22.0, 0.0)));
        bones.insert("left_arm".into(), BoneTransform::new(Vec3::new(5.0, 22.0, 0.0)));
        bones.insert("right_leg".into(), BoneTransform::new(Vec3::new(-1.9, 12.0, 0.0)));
        bones.insert("left_leg".into(), BoneTransform::new(Vec3::new(1.9, 12.0, 0.0)));
        Self { bones }
    }
}
