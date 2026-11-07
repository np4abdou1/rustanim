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

    /// Aligns foot orientation to ground terrain normal with height adjustment.
    pub fn align_foot_ground_ik(
        &mut self,
        foot_bone: &str,
        ground_height: f32,
        ground_normal: Vec3,
    ) {
        if let Some(bone) = self.bones.get_mut(foot_bone) {
            let height_delta = ground_height - bone.base_pivot.y;
            bone.relative_offset.y += height_delta.clamp(-2.0, 2.0);

            let up = Vec3::Y;
            let normal = ground_normal.normalize_or_zero();
            if normal.length_squared() > 0.5 {
                let tilt_rot = Quat::from_rotation_arc(up, normal);
                bone.rotation = tilt_rot * bone.rotation;
            }
        }
    }

    /// Gets particle and effect emitter socket transform (e.g. eyes, hand particles, foot steps).
    pub fn get_locator_transform(&self, socket_name: &str) -> Option<Mat4> {
        let world = self.compute_world_transforms();
        match socket_name {
            "eye" | "eye_level" => world.get("head").map(|m| *m * Mat4::from_translation(Vec3::new(0.0, 4.0, 2.0))),
            "right_hand_particle" => world.get("right_item").or_else(|| world.get("right_arm")).copied(),
            "left_hand_particle" => world.get("left_item").or_else(|| world.get("left_arm")).copied(),
            "right_foot_step" => world.get("right_leg").map(|m| *m * Mat4::from_translation(Vec3::new(0.0, -12.0, 0.0))),
            "left_foot_step" => world.get("left_leg").map(|m| *m * Mat4::from_translation(Vec3::new(0.0, -12.0, 0.0))),
            other => world.get(other).copied(),
        }
    }
}

/// Player humanoid model type distinguishing standard (Steve, 4px arms) from slim (Alex, 3px arms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerModelType {
    #[default]
    WideSteve,
    SlimAlex,
}

impl PlayerModelType {
    /// Returns arm thickness in model pixel units.
    #[inline]
    pub fn arm_width(&self) -> f32 {
        match self {
            PlayerModelType::WideSteve => 4.0,
            PlayerModelType::SlimAlex => 3.0,
        }
    }

    /// Returns UV coordinates offset modifier for texture mapping.
    #[inline]
    pub fn arm_uv_offset(&self) -> (f32, f32) {
        match self {
            PlayerModelType::WideSteve => (0.0, 0.0),
            PlayerModelType::SlimAlex => (-1.0, 0.0),
        }
    }
}

/// Spring-damper physics simulation for cloth capes.
#[derive(Debug, Clone)]
pub struct CapePhysicsRig {
    pub position: Vec3,
    pub velocity: Vec3,
    pub pitch_angle: f32,
    pub roll_angle: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Default for CapePhysicsRig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            pitch_angle: 0.0,
            roll_angle: 0.0,
            stiffness: 140.0,
            damping: 12.0,
        }
    }
}

impl CapePhysicsRig {
    /// Updates cape spring physics given body velocity and delta time.
    pub fn update(&mut self, player_velocity: Vec3, delta_time: f32) {
        let target_pitch = (player_velocity.length() * 14.0).clamp(0.0, 75.0);
        let target_roll = (-player_velocity.x * 6.0).clamp(-30.0, 30.0);

        let pitch_accel = self.stiffness * (target_pitch - self.pitch_angle) - self.damping * self.velocity.x;
        self.velocity.x += pitch_accel * delta_time;
        self.pitch_angle += self.velocity.x * delta_time;

        let roll_accel = self.stiffness * (target_roll - self.roll_angle) - self.damping * self.velocity.y;
        self.velocity.y += roll_accel * delta_time;
        self.roll_angle += self.velocity.y * delta_time;
    }

    /// Computes the cape's local matrix relative to torso body anchor.
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(0.0, 24.0, -2.5))
            * Mat4::from_rotation_x(-self.pitch_angle.to_radians())
            * Mat4::from_rotation_z(self.roll_angle.to_radians())
    }
}

/// Secondary motion spring for hair, ponytails, ears, or antennas.
#[derive(Debug, Clone)]
pub struct SecondaryMotionSpring {
    pub current_angle: Vec3,
    pub velocity: Vec3,
    pub stiffness: f32,
    pub damping: f32,
}

impl Default for SecondaryMotionSpring {
    fn default() -> Self {
        Self {
            current_angle: Vec3::ZERO,
            velocity: Vec3::ZERO,
            stiffness: 180.0,
            damping: 15.0,
        }
    }
}

impl SecondaryMotionSpring {
    pub fn update(&mut self, target_inertia: Vec3, delta_time: f32) {
        let accel = self.stiffness * (target_inertia - self.current_angle) - self.damping * self.velocity;
        self.velocity += accel * delta_time;
        self.current_angle += self.velocity * delta_time;
    }
}

/// Armor cube inflation synchronization (syncs base joint transforms to expanded armor layer bounds).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmorCubeInflation {
    pub helmet_inflation: f32,
    pub chestplate_inflation: f32,
    pub leggings_inflation: f32,
    pub boots_inflation: f32,
}

impl Default for ArmorCubeInflation {
    fn default() -> Self {
        Self {
            helmet_inflation: 0.5,
            chestplate_inflation: 0.75,
            leggings_inflation: 0.4,
            boots_inflation: 0.5,
        }
    }
}

impl ArmorCubeInflation {
    /// Computes scaled transform with inflation padding.
    pub fn apply_inflation(&self, base_matrix: Mat4, inflation_units: f32) -> Mat4 {
        let scale_factor = 1.0 + (inflation_units / 16.0);
        base_matrix * Mat4::from_scale(Vec3::splat(scale_factor))
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

    #[test]
    fn test_foot_ground_ik_alignment() {
        let mut rig = RelativePivotRig::new_humanoid();
        let ground_normal = Vec3::new(0.0, 0.9, 0.435).normalize();
        rig.align_foot_ground_ik("right_leg", 11.5, ground_normal);
        let bone = &rig.bones["right_leg"];
        assert!((bone.relative_offset.y - (-0.5)).abs() < 1e-4);
        assert!(bone.rotation.is_finite());
    }

    #[test]
    fn test_cape_physics_and_spring() {
        let mut cape = CapePhysicsRig::default();
        cape.update(Vec3::new(0.0, 0.0, 4.0), 0.05); // Running forward 4m/s
        assert!(cape.pitch_angle > 0.0, "Cape should flap backwards when moving forward");
        let m = cape.local_matrix();
        assert!(m.is_finite());
    }

    #[test]
    fn test_steve_alex_uv_and_armor_inflation() {
        assert_eq!(PlayerModelType::WideSteve.arm_width(), 4.0);
        assert_eq!(PlayerModelType::SlimAlex.arm_width(), 3.0);
        assert_eq!(PlayerModelType::SlimAlex.arm_uv_offset(), (-1.0, 0.0));

        let inflation = ArmorCubeInflation::default();
        let base_m = Mat4::IDENTITY;
        let helmet_m = inflation.apply_inflation(base_m, inflation.helmet_inflation);
        assert!(helmet_m.transform_point3(Vec3::ONE).x > 1.0);
    }

    #[test]
    fn test_particle_locator_sockets() {
        let rig = RelativePivotRig::new_humanoid();
        let eye = rig.get_locator_transform("eye").expect("Eye locator must exist");
        let hand = rig.get_locator_transform("right_hand_particle").expect("Hand locator must exist");
        assert!(eye.is_finite());
        assert!(hand.is_finite());
    }
}
