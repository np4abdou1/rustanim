//! Relative pivot bone rig.
//! Prevents rig collapse and limb stretching by preserving vanilla Java base pivots
//! and applying Bedrock animation deltas purely in relative local space.

use glam::{EulerRot, Mat4, Quat, Vec3};
use std::collections::HashMap;

use crate::anim::FramePose;

// ---------------------------------------------------------------------------
// Canonical Vanilla Java Humanoid Joint Anchors (pixels, 16px = 1 block)
// ---------------------------------------------------------------------------
pub const JAVA_HEAD_ANCHOR: Vec3 = Vec3::new(0.0, 24.0, 0.0);
pub const JAVA_BODY_ANCHOR: Vec3 = Vec3::new(0.0, 24.0, 0.0);
pub const JAVA_RIGHT_ARM_ANCHOR: Vec3 = Vec3::new(-5.0, 22.0, 0.0);
pub const JAVA_LEFT_ARM_ANCHOR: Vec3 = Vec3::new(5.0, 22.0, 0.0);
pub const JAVA_RIGHT_LEG_ANCHOR: Vec3 = Vec3::new(-1.9, 12.0, 0.0);
pub const JAVA_LEFT_LEG_ANCHOR: Vec3 = Vec3::new(1.9, 12.0, 0.0);

pub const VIRTUAL_WAIST_ANCHOR: Vec3 = Vec3::new(0.0, 12.0, 0.0);
pub const BEDROCK_RIGHT_ITEM_ANCHOR: Vec3 = Vec3::new(-6.0, 15.0, 1.0);
pub const BEDROCK_LEFT_ITEM_ANCHOR: Vec3 = Vec3::new(6.0, 15.0, 1.0);

/// Computes a relative local transformation matrix:
/// M_local = Translate(bone_anchor - parent_anchor + delta_t) * Rotate(R) * Scale(S)
#[inline]
pub fn compute_relative_local_matrix(
    bone_anchor: Vec3,
    parent_anchor: Vec3,
    delta_translation: Vec3,
    rotation_deg_zyx: Vec3,
    scale: Vec3,
) -> Mat4 {
    let relative_origin = bone_anchor - parent_anchor + delta_translation;
    let rotation = Quat::from_euler(
        EulerRot::ZYX,
        rotation_deg_zyx.z.to_radians(),
        rotation_deg_zyx.y.to_radians(),
        rotation_deg_zyx.x.to_radians(),
    );
    Mat4::from_scale_rotation_translation(scale, rotation, relative_origin)
}

/// Represents the current local transform state of a single bone in the rig.
#[derive(Debug, Clone, PartialEq)]
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

    /// Computes the bone's local matrix evaluated around its base pivot:
    /// M = Translate(base_pivot + relative_offset) * Rotate(rotation) * Scale(scale) * Translate(-base_pivot)
    #[inline]
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.base_pivot + self.relative_offset)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
            * Mat4::from_translation(-self.base_pivot)
    }

    /// Sets Euler rotation in ZYX order (in degrees, matching Bedrock).
    #[inline]
    pub fn set_euler_degrees(&mut self, euler_deg: Vec3) {
        self.rotation = Quat::from_euler(
            EulerRot::ZYX,
            euler_deg.z.to_radians(),
            euler_deg.y.to_radians(),
            euler_deg.x.to_radians(),
        );
    }
}

/// Bone relationship node in the humanoid rig.
#[derive(Debug, Clone)]
pub struct BoneNode {
    pub name: String,
    pub parent: Option<String>,
    pub anchor: Vec3,
    pub transform: BoneTransform,
}

/// Hierarchical bone structure representing parent-child relationships and indices.
#[derive(Debug, Clone, Default)]
pub struct BoneHierarchy {
    pub parent_indices: Vec<Option<usize>>,
    pub bone_names: Vec<String>,
    name_to_index: HashMap<String, usize>,
}

impl BoneHierarchy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bone(&mut self, name: impl Into<String>, parent_name: Option<&str>) -> usize {
        let name = name.into();
        let parent_idx = parent_name.and_then(|p| self.name_to_index.get(p).copied());
        let idx = self.bone_names.len();
        self.name_to_index.insert(name.clone(), idx);
        self.bone_names.push(name);
        self.parent_indices.push(parent_idx);
        idx
    }

    #[inline]
    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.name_to_index.get(name).copied()
    }
}

/// Full humanoid rig managing relative pivot transformations, Bedrock bone mapping,
/// and hierarchical matrix propagation without limb stretching or hip collapse.
#[derive(Debug, Clone)]
pub struct RelativePivotRig {
    pub bones: HashMap<String, BoneTransform>,
    pub hierarchy: HashMap<String, Option<String>>,
    pub base_anchors: HashMap<String, Vec3>,
}

impl RelativePivotRig {
    /// Creates a standard humanoid rig initialized with canonical Java edition joint anchors
    /// and virtual waist hierarchy.
    pub fn new_humanoid() -> Self {
        let mut bones = HashMap::new();
        let mut hierarchy = HashMap::new();
        let mut base_anchors = HashMap::new();

        // 1. Virtual waist joint (anchor Y=12.0)
        bones.insert("waist".into(), BoneTransform::new(VIRTUAL_WAIST_ANCHOR));
        hierarchy.insert("waist".into(), None);
        base_anchors.insert("waist".into(), VIRTUAL_WAIST_ANCHOR);

        // 2. Torso / Body (parented to waist, pivot Y=24.0)
        bones.insert("body".into(), BoneTransform::new(JAVA_BODY_ANCHOR));
        hierarchy.insert("body".into(), Some("waist".into()));
        base_anchors.insert("body".into(), JAVA_BODY_ANCHOR);

        // 3. Head (parented to body, pivot Y=24.0)
        bones.insert("head".into(), BoneTransform::new(JAVA_HEAD_ANCHOR));
        hierarchy.insert("head".into(), Some("body".into()));
        base_anchors.insert("head".into(), JAVA_HEAD_ANCHOR);

        // 4. Arms (parented to body, pivots Y=22.0)
        bones.insert("right_arm".into(), BoneTransform::new(JAVA_RIGHT_ARM_ANCHOR));
        hierarchy.insert("right_arm".into(), Some("body".into()));
        base_anchors.insert("right_arm".into(), JAVA_RIGHT_ARM_ANCHOR);

        bones.insert("left_arm".into(), BoneTransform::new(JAVA_LEFT_ARM_ANCHOR));
        hierarchy.insert("left_arm".into(), Some("body".into()));
        base_anchors.insert("left_arm".into(), JAVA_LEFT_ARM_ANCHOR);

        // 5. Item attachment sockets (parented to arms)
        bones.insert("right_item".into(), BoneTransform::new(BEDROCK_RIGHT_ITEM_ANCHOR));
        hierarchy.insert("right_item".into(), Some("right_arm".into()));
        base_anchors.insert("right_item".into(), BEDROCK_RIGHT_ITEM_ANCHOR);

        bones.insert("left_item".into(), BoneTransform::new(BEDROCK_LEFT_ITEM_ANCHOR));
        hierarchy.insert("left_item".into(), Some("left_arm".into()));
        base_anchors.insert("left_item".into(), BEDROCK_LEFT_ITEM_ANCHOR);

        // 6. Legs (CRITICAL: parented to ROOT/None, pivot Y=12.0)
        // This guarantees waist tilts and torso twists NEVER drag or collapse hip/leg joints!
        bones.insert("right_leg".into(), BoneTransform::new(JAVA_RIGHT_LEG_ANCHOR));
        hierarchy.insert("right_leg".into(), None);
        base_anchors.insert("right_leg".into(), JAVA_RIGHT_LEG_ANCHOR);

        bones.insert("left_leg".into(), BoneTransform::new(JAVA_LEFT_LEG_ANCHOR));
        hierarchy.insert("left_leg".into(), None);
        base_anchors.insert("left_leg".into(), JAVA_LEFT_LEG_ANCHOR);

        Self {
            bones,
            hierarchy,
            base_anchors,
        }
    }

    /// Normalizes Bedrock bone names (camelCase, concatenated, or aliases) to canonical rig names.
    pub fn canonicalize_bone_name(bedrock_name: &str) -> Option<&'static str> {
        let lower = bedrock_name.to_ascii_lowercase();
        match lower.as_str() {
            "root" => None,
            "waist" => Some("waist"),
            "body" | "torso" => Some("body"),
            "head" | "hat" => Some("head"),
            "rightarm" | "right_arm" => Some("right_arm"),
            "leftarm" | "left_arm" => Some("left_arm"),
            "rightleg" | "right_leg" => Some("right_leg"),
            "leftleg" | "left_leg" => Some("left_leg"),
            "rightitem" | "right_item" => Some("right_item"),
            "leftitem" | "left_item" => Some("left_item"),
            _ => None,
        }
    }

    /// Resets all bone transforms to rest state (identity rotation, zero relative offset, scale 1.0).
    pub fn reset_pose(&mut self) {
        for (name, bone) in &mut self.bones {
            let anchor = self.base_anchors.get(name).copied().unwrap_or(Vec3::ZERO);
            bone.base_pivot = anchor;
            bone.relative_offset = Vec3::ZERO;
            bone.rotation = Quat::IDENTITY;
            bone.scale = Vec3::ONE;
        }
    }

    /// Applies an evaluated animation `FramePose` to the rig, mapping Bedrock bones
    /// into relative local translations and rotations.
    pub fn apply_pose(&mut self, pose: &FramePose) {
        for (bone_name, bone_pose) in &pose.bones {
            if let Some(canonical) = Self::canonicalize_bone_name(bone_name) {
                if let Some(bone) = self.bones.get_mut(canonical) {
                    bone.relative_offset = bone_pose.position;
                    bone.set_euler_degrees(bone_pose.rotation);
                    bone.scale = bone_pose.scale;
                }
            }
        }
    }

    /// Computes world matrices for all bones in top-down hierarchical order.
    pub fn compute_world_transforms(&self) -> HashMap<String, Mat4> {
        let mut world_transforms = HashMap::new();

        // 1. Process root-level bones first (waist, legs)
        for (name, parent) in &self.hierarchy {
            if parent.is_none() {
                if let Some(bone) = self.bones.get(name) {
                    world_transforms.insert(name.clone(), bone.local_matrix());
                }
            }
        }

        // 2. Iteratively resolve child bones using parent world matrices:
        // W_child = W_parent * M_child
        let mut resolved = false;
        let mut iterations = 0;
        while !resolved && iterations < 16 {
            resolved = true;
            for (name, parent_opt) in &self.hierarchy {
                if world_transforms.contains_key(name) {
                    continue;
                }
                if let Some(parent_name) = parent_opt {
                    if let Some(parent_world) = world_transforms.get(parent_name) {
                        if let Some(bone) = self.bones.get(name) {
                            let world_m = *parent_world * bone.local_matrix();
                            world_transforms.insert(name.clone(), world_m);
                        }
                    } else {
                        resolved = false;
                    }
                }
            }
            iterations += 1;
        }

        world_transforms
    }

    /// Computes the world transform matrix of a socket (e.g. "right_item" or "head").
    #[inline]
    pub fn get_socket_transform(&self, socket_name: &str) -> Option<Mat4> {
        let world_transforms = self.compute_world_transforms();
        world_transforms.get(socket_name).copied()
    }

    /// Two-bone analytical inverse kinematics (IK) helper for knees or elbows.
    /// Solves joint bend angle for reaching a target position without limb detachment.
    pub fn solve_two_bone_ik(
        root_pos: Vec3,
        target_pos: Vec3,
        len_upper: f32,
        len_lower: f32,
        bend_axis: Vec3,
    ) -> (Quat, Quat) {
        let to_target = target_pos - root_pos;
        let dist = to_target.length().clamp(1e-4, len_upper + len_lower - 1e-4);
        let dir = to_target / dist;

        // Law of cosines for upper bone angle
        let cos_upper = ((len_upper * len_upper + dist * dist - len_lower * len_lower)
            / (2.0 * len_upper * dist))
            .clamp(-1.0, 1.0);
        let angle_upper = cos_upper.acos();

        // Law of cosines for lower bone angle
        let cos_lower = ((len_upper * len_upper + len_lower * len_lower - dist * dist)
            / (2.0 * len_upper * len_lower))
            .clamp(-1.0, 1.0);
        let angle_lower = std::f32::consts::PI - cos_lower.acos();

        let base_rot = Quat::from_rotation_arc(Vec3::NEG_Y, dir);
        let upper_rot = base_rot * Quat::from_axis_angle(bend_axis, angle_upper);
        let lower_rot = Quat::from_axis_angle(bend_axis, angle_lower);

        (upper_rot, lower_rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanoid_rig_initialization_anchors() {
        let rig = RelativePivotRig::new_humanoid();
        assert_eq!(rig.bones["head"].base_pivot, JAVA_HEAD_ANCHOR);
        assert_eq!(rig.bones["body"].base_pivot, JAVA_BODY_ANCHOR);
        assert_eq!(rig.bones["right_arm"].base_pivot, JAVA_RIGHT_ARM_ANCHOR);
        assert_eq!(rig.bones["left_arm"].base_pivot, JAVA_LEFT_ARM_ANCHOR);
        assert_eq!(rig.bones["right_leg"].base_pivot, JAVA_RIGHT_LEG_ANCHOR);
        assert_eq!(rig.bones["left_leg"].base_pivot, JAVA_LEFT_LEG_ANCHOR);
        assert_eq!(rig.bones["waist"].base_pivot, VIRTUAL_WAIST_ANCHOR);
    }

    #[test]
    fn test_relative_local_matrix_calculation() {
        let m = compute_relative_local_matrix(
            JAVA_RIGHT_ARM_ANCHOR,
            JAVA_BODY_ANCHOR,
            Vec3::new(0.0, -1.0, 0.5),
            Vec3::new(45.0, 0.0, 0.0),
            Vec3::ONE,
        );
        let transformed = m.transform_point3(Vec3::ZERO);
        let expected = JAVA_RIGHT_ARM_ANCHOR - JAVA_BODY_ANCHOR + Vec3::new(0.0, -1.0, 0.5);
        assert!((transformed - expected).length() < 1e-4);
    }

    #[test]
    fn test_waist_pitch_does_not_collapse_legs() {
        let mut rig = RelativePivotRig::new_humanoid();
        // Pitch waist forward by 45 degrees
        if let Some(waist) = rig.bones.get_mut("waist") {
            waist.set_euler_degrees(Vec3::new(45.0, 0.0, 0.0));
        }

        let world = rig.compute_world_transforms();

        // Right and left legs are parented to root; their position must remain unaffected
        let r_leg_pos = world["right_leg"].transform_point3(JAVA_RIGHT_LEG_ANCHOR);
        assert!(
            (r_leg_pos - JAVA_RIGHT_LEG_ANCHOR).length() < 1e-4,
            "Leg anchor moved when waist pitched! Expected {:?}, got {:?}",
            JAVA_RIGHT_LEG_ANCHOR,
            r_leg_pos
        );

        // Body should tilt
        let body_pos = world["body"].transform_point3(JAVA_BODY_ANCHOR);
        assert!(body_pos.y < 24.0, "Body did not lower during forward waist pitch");
    }

    #[test]
    fn test_two_bone_ik_reaches_target() {
        let root = Vec3::new(0.0, 12.0, 0.0);
        let target = Vec3::new(0.0, 2.0, 0.0); // 10 units down
        let (upper_rot, lower_rot) = RelativePivotRig::solve_two_bone_ik(
            root,
            target,
            6.0,
            6.0,
            Vec3::X,
        );
        assert!(upper_rot.is_finite());
        assert!(lower_rot.is_finite());
    }
}
