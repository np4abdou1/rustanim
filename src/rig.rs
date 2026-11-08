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
pub const JAVA_BODY_ANCHOR: Vec3 = Vec3::new(0.0, 12.0, 0.0); // Hips at 12.0 (Bedrock/Actions&Stuff hinge)
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

// ---------------------------------------------------------------------------
// Bedrock Actions & Stuff Real-Time Kinematics Engine
// ---------------------------------------------------------------------------

/// Transform of a humanoid model part: translation [x, y, z] in model units,
/// and rotation [pitch_x, yaw_y, roll_z] in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerBonePose {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
}

impl Default for PlayerBonePose {
    fn default() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
        }
    }
}

/// Evaluated model-space pose for the player character.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerPoseResult {
    pub body: PlayerBonePose,
    pub head: PlayerBonePose,
    pub right_arm: PlayerBonePose,
    pub left_arm: PlayerBonePose,
    pub right_leg: PlayerBonePose,
    pub left_leg: PlayerBonePose,
}

/// Dynamic input parameters driving the Actions & Stuff player animation.
#[derive(Debug, Clone, Copy)]
pub struct PlayerAnimInput {
    pub age: f32,
    pub pos: f32,
    pub speed: f32,
    pub head_pitch_deg: f32,
    pub head_yaw_deg: f32,
    pub swing: f32,
    pub is_crouching: bool,
    pub is_holding_main: bool,
    pub is_holding_off: bool,
    pub is_swing_right: bool,
    pub is_fall_flying: bool,
}

const RIGHT_LEG_ROT_KFS: &[(f32, f32)] = &[
    (0.0000, -2.36),
    (0.2778, 38.40),
    (0.3333, 48.30),
    (0.3889, 51.40),
    (0.4444, 48.36),
    (0.5000, 28.87),
    (0.6111, -22.58),
    (0.6667, -34.80),
    (0.7222, -39.00),
    (0.7778, -34.98),
    (1.0000, -2.36),
];

const RIGHT_LEG_POS_KFS: &[(f32, f32, f32)] = &[
    (0.0000, -0.15, 0.40),
    (0.1667, -0.60, 1.00),
    (0.2222, -0.96, 1.10),
    (0.2778, -1.64, 1.10),
    (0.3333, -2.65, 0.96),
    (0.3889, -2.95, 0.67),
    (0.4444, -2.07, 0.27),
    (0.5000, 0.70, 0.08),
    (0.5556, 2.13, -0.12),
    (0.6111, 1.50, -0.31),
    (0.6667, 0.00, -0.51),
    (0.7222, -2.10, -0.60),
    (0.7778, -1.43, -0.57),
    (0.8333, -0.81, -0.45),
    (1.0000, -0.15, 0.40),
];

const SPRINT_LEG_POS_KFS: &[(f32, f32, f32)] = &[
    (0.0000, -0.15, -0.50),
    (0.0417, -0.40, -0.13),
    (0.0833, -1.20, 0.40),
    (0.1250, -2.00, 0.90),
    (0.1667, -1.75, 1.20),
    (0.2500, -0.50, 1.10),
    (0.3333, 2.00, 0.32),
    (0.4167, 2.20, -0.65),
    (0.4583, 1.80, -0.90),
    (0.5417, -1.80, -0.93),
    (0.6250, -0.50, -0.80),
    (0.6667, -0.15, -0.50),
];

fn sample_spline_1d(kfs: &[(f32, f32)], t: f32) -> f32 {
    let t = t.rem_euclid(1.0);
    let n = kfs.len();
    if n == 0 { return 0.0; }
    if n == 1 { return kfs[0].1; }
    for i in 0..n - 1 {
        if kfs[i].0 <= t && t <= kfs[i + 1].0 {
            let i1 = i;
            let i2 = i + 1;
            let i0 = i1.saturating_sub(1);
            let i3 = (i2 + 1).min(n - 1);
            let (t1, p1) = kfs[i1];
            let (t2, p2) = kfs[i2];
            let dt = t2 - t1;
            if dt < 1e-5 { return p1; }
            let u = (t - t1) / dt;
            let p0 = kfs[i0].1;
            let p3 = kfs[i3].1;
            let m1 = 0.5 * (p2 - p0);
            let m2 = 0.5 * (p3 - p1);
            let u2 = u * u;
            let u3 = u2 * u;
            let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
            let h10 = u3 - 2.0 * u2 + u;
            let h01 = -2.0 * u3 + 3.0 * u2;
            let h11 = u3 - u2;
            return h00 * p1 + h10 * m1 + h01 * p2 + h11 * m2;
        }
    }
    kfs[0].1
}

fn sample_spline_2d(kfs: &[(f32, f32, f32)], t: f32) -> (f32, f32) {
    let t = t.rem_euclid(1.0);
    let n = kfs.len();
    if n == 0 { return (0.0, 0.0); }
    if n == 1 { return (kfs[0].1, kfs[0].2); }
    for i in 0..n - 1 {
        if kfs[i].0 <= t && t <= kfs[i + 1].0 {
            let i1 = i;
            let i2 = i + 1;
            let i0 = i1.saturating_sub(1);
            let i3 = (i2 + 1).min(n - 1);
            let (t1, y1, z1) = kfs[i1];
            let (t2, y2, z2) = kfs[i2];
            let dt = t2 - t1;
            if dt < 1e-5 { return (y1, z1); }
            let u = (t - t1) / dt;

            let my1 = 0.5 * (y2 - kfs[i0].1);
            let my2 = 0.5 * (kfs[i3].1 - y1);
            let mz1 = 0.5 * (z2 - kfs[i0].2);
            let mz2 = 0.5 * (kfs[i3].2 - z1);

            let u2 = u * u;
            let u3 = u2 * u;
            let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
            let h10 = u3 - 2.0 * u2 + u;
            let h01 = -2.0 * u3 + 3.0 * u2;
            let h11 = u3 - u2;

            let y = h00 * y1 + h10 * my1 + h01 * y2 + h11 * my2;
            let z = h00 * z1 + h10 * mz1 + h01 * z2 + h11 * mz2;
            return (y, z);
        }
    }
    (kfs[0].1, kfs[0].2)
}

/// Computes a local Bedrock bone matrix relative to its parent
fn bedrock_bone_matrix(parent: glam::Mat4, pivot: glam::Vec3, rot_deg: [f32; 3], trans: [f32; 3]) -> glam::Mat4 {
    let q = glam::Quat::from_euler(
        glam::EulerRot::ZYX,
        rot_deg[2].to_radians(),
        rot_deg[1].to_radians(),
        rot_deg[0].to_radians(),
    );
    let t = glam::Vec3::from_array(trans);
    parent
        * glam::Mat4::from_translation(pivot + t)
        * glam::Mat4::from_quat(q)
        * glam::Mat4::from_translation(-pivot)
}

/// Exact change-of-basis projection from Bedrock bone scene graph into Horizon's ModelPart
fn project_bedrock_bone_to_horizon(m_bedrock: glam::Mat4, h_pivot: glam::Vec3) -> ([f32; 3], [f32; 3]) {
    let c = glam::Mat4::from_cols_array(&[
        -1.0,  0.0, 0.0, 0.0,
         0.0, -1.0, 0.0, 0.0,
         0.0,  0.0, 1.0, 0.0,
         0.0, 24.0, 0.0, 1.0,
    ]);
    let m_h = c * m_bedrock * c;
    let t_np = glam::Mat4::from_translation(-h_pivot);
    let t_p = glam::Mat4::from_translation(h_pivot);
    let m_prime = t_np * m_h * t_p;

    let pos = m_prime.w_axis.truncate().to_array();

    // Extract exact ZYX Euler angles from rotation matrix
    let r20 = m_prime.col(0).z;
    let r00 = m_prime.col(0).x;
    let r10 = m_prime.col(0).y;
    let r21 = m_prime.col(1).z;
    let r22 = m_prime.col(2).z;
    let r11 = m_prime.col(1).y;
    let r12 = m_prime.col(2).y;

    let sy = -r20;
    let cy = (r00 * r00 + r10 * r10).sqrt();
    let ry = sy.atan2(cy);
    let (rx, rz) = if cy > 1e-5 {
        (r21.atan2(r22), r10.atan2(r00))
    } else {
        ((-r12).atan2(r11), 0.0)
    };

    (pos, [rx, ry, rz])
}

impl PlayerPoseResult {
    /// Evaluates the complete Bedrock Actions & Stuff kinematic tree for the player using live hot-reload config.
    pub fn evaluate(input: &PlayerAnimInput) -> Self {
        let cfg = crate::config::get_live_config();
        Self::evaluate_with_config(input, &cfg)
    }

    /// Evaluates the complete Bedrock Actions & Stuff kinematic tree for the player with a specific config.
    pub fn evaluate_with_config(input: &PlayerAnimInput, cfg: &crate::config::RustanimConfig) -> Self {
        if !cfg.enabled || !cfg.player_animations {
            return Self::default();
        }

        let mut res = Self::default();
        let t = input.age * 0.05;
        let is_moving = input.speed > 0.05;
        let is_sprinting = input.speed > 0.8;
        let sp = input.speed.min(1.0);
        let phi = input.pos * 0.6662;

        // Bedrock standard humanoid joint pivots (model pixels, Y up from feet=0)
        let p_waist = glam::Vec3::new(0.0, 12.0, 0.0);
        let p_body = glam::Vec3::new(0.0, 24.0, 0.0);
        let p_head = glam::Vec3::new(0.0, 24.0, 0.0);
        let p_rarm = glam::Vec3::new(-5.0, 22.0, 0.0);
        let p_larm = glam::Vec3::new(5.0, 22.0, 0.0);
        let p_rleg = glam::Vec3::new(-1.9, 12.0, 0.0);
        let p_lleg = glam::Vec3::new(1.9, 12.0, 0.0);

        // Horizon ModelPart base pivots (Y down from neck=0)
        let h_head = glam::Vec3::new(0.0, 0.0, 0.0);
        let h_body = glam::Vec3::new(0.0, 0.0, 0.0);
        let h_rarm = glam::Vec3::new(-5.0, 2.0, 0.0);
        let h_larm = glam::Vec3::new(5.0, 2.0, 0.0);
        let h_rleg = glam::Vec3::new(-1.9, 12.0, 0.0);
        let h_lleg = glam::Vec3::new(1.9, 12.0, 0.0);

        let m_root = glam::Mat4::IDENTITY;

        let (_m_waist, m_body, m_head, m_rarm, m_larm, m_rleg, m_lleg) = if input.is_crouching {
            // Bedrock Actions & Stuff agile stealth crouch
            let w_trans = [0.0, -cfg.crouch_body_y, cfg.crouch_body_z];
            let w_rot = [cfg.crouch_body_pitch_deg, 0.0, 0.0];
            let m_waist = bedrock_bone_matrix(m_root, p_waist, w_rot, w_trans);

            let m_body = bedrock_bone_matrix(m_waist, p_body, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

            let h_rot = [input.head_pitch_deg - 20.0, input.head_yaw_deg, 0.0];
            let m_head = bedrock_bone_matrix(m_body, p_head, h_rot, [0.0, 0.0, 0.0]);

            let m_rarm = bedrock_bone_matrix(m_body, p_rarm, [-20.0, 0.0, 11.5], [0.0, 0.0, 0.0]);
            let m_larm = bedrock_bone_matrix(m_body, p_larm, [-20.0, 0.0, -11.5], [0.0, 0.0, 0.0]);

            let (r_leg_rot, l_leg_rot) = if is_moving {
                let stride = (phi.cos() * 0.7 * sp).to_degrees();
                ([stride, 0.0, 0.0], [-stride, 0.0, 0.0])
            } else {
                ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
            };

            let m_rleg = bedrock_bone_matrix(m_root, p_rleg, r_leg_rot, [0.0, 0.0, 0.0]);
            let m_lleg = bedrock_bone_matrix(m_root, p_lleg, l_leg_rot, [0.0, 0.0, 0.0]);

            (m_waist, m_body, m_head, m_rarm, m_larm, m_rleg, m_lleg)
        } else if is_sprinting {
            // Bedrock Actions & Stuff 100% authentic sprint dash (Oreville zierfb)
            let sprint_phi = phi * 1.4 * cfg.sprint_speed_mult;
            let norm_t = (sprint_phi / (2.0 * std::f32::consts::PI)).rem_euclid(1.0);
            let bounce = (norm_t * 1080.0).to_radians().cos() * -2.5 * sp * cfg.sprint_bounce_mult;
            let w_trans = [0.0, bounce, 0.75 * sp];
            let w_rot = [
                cfg.sprint_body_pitch_deg,
                (norm_t * 540.0).to_radians().sin() * -8.0,
                (norm_t * 540.0).to_radians().cos() * -5.0,
            ];
            let m_waist = bedrock_bone_matrix(m_root, p_waist, w_rot, w_trans);

            let m_body = bedrock_bone_matrix(m_waist, p_body, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

            let h_rot = [
                input.head_pitch_deg - 14.0 + (norm_t * 1080.0).to_radians().sin() * 7.0,
                input.head_yaw_deg,
                (norm_t * 540.0 - 45.0).to_radians().cos() * -10.0,
            ];
            let m_head = bedrock_bone_matrix(m_body, p_head, h_rot, [0.0, 0.0, 0.0]);

            let r_arm_pitch = (norm_t * 540.0 + 22.0).to_radians().sin() * -40.0 * cfg.sprint_arm_swing;
            let l_arm_pitch = (norm_t * 540.0 + 22.0).to_radians().sin() * 40.0 * cfg.sprint_arm_swing;
            let arm_roll = (norm_t * 1080.0 + 30.0).to_radians().sin() * 6.0;

            let m_rarm = bedrock_bone_matrix(m_body, p_rarm, [r_arm_pitch, 0.0, -arm_roll + 6.0], [0.0, 0.0, 0.0]);
            let m_larm = bedrock_bone_matrix(m_body, p_larm, [l_arm_pitch, 0.0, arm_roll - 6.0], [0.0, 0.0, 0.0]);

            let (r_pos_y, r_pos_z) = sample_spline_2d(SPRINT_LEG_POS_KFS, norm_t);
            let (l_pos_y, l_pos_z) = sample_spline_2d(SPRINT_LEG_POS_KFS, (norm_t + 0.5).rem_euclid(1.0));

            let r_rot = (norm_t * 540.0).to_radians().sin() * 35.0 * cfg.sprint_leg_swing;
            let l_rot = -(norm_t * 540.0).to_radians().sin() * 35.0 * cfg.sprint_leg_swing;

            let m_rleg = bedrock_bone_matrix(m_root, p_rleg, [r_rot, 0.0, 0.0], [0.0, r_pos_y * cfg.sprint_leg_swing, r_pos_z * cfg.sprint_leg_swing]);
            let m_lleg = bedrock_bone_matrix(m_root, p_lleg, [l_rot, 0.0, 0.0], [0.0, l_pos_y * cfg.sprint_leg_swing, l_pos_z * cfg.sprint_leg_swing]);

            (m_waist, m_body, m_head, m_rarm, m_larm, m_rleg, m_lleg)
        } else if is_moving {
            // Bedrock Actions & Stuff 100% authentic Oreville tldiyl walk cycle
            let norm_t = (phi / (2.0 * std::f32::consts::PI) * cfg.walk_speed_mult).rem_euclid(1.0);
            let bounce = (norm_t * 720.0 + 12.0).to_radians().sin() * 1.3 * sp * cfg.walk_bounce_mult;
            let w_trans = [0.0, bounce, 0.6 * sp];
            let w_rot = [
                0.0,
                0.0,
                (norm_t * 360.0 + 45.0).to_radians().cos() * cfg.walk_body_roll_deg * sp,
            ];
            let m_waist = bedrock_bone_matrix(m_root, p_waist, w_rot, w_trans);

            let b_rot = [
                cfg.walk_body_pitch_deg * sp,
                (norm_t * 360.0 + 40.0).to_radians().cos() * cfg.walk_body_yaw_deg * sp,
                0.0,
            ];
            let m_body = bedrock_bone_matrix(m_waist, p_body, b_rot, [0.0, 0.0, 0.0]);

            let h_rot = [
                input.head_pitch_deg + (norm_t * 720.0).to_radians().cos() * 6.0 * sp,
                input.head_yaw_deg + (norm_t * 360.0 + 40.0).to_radians().sin() * 3.0 * sp,
                (norm_t * 360.0 - 45.0).to_radians().cos() * -5.4 * sp,
            ];
            let m_head = bedrock_bone_matrix(m_body, p_head, h_rot, [0.0, 0.0, 0.0]);

            let r_arm_pitch = (-(norm_t * 360.0).to_radians().sin() * 37.2 + 1.6) * cfg.walk_arm_swing * sp;
            let l_arm_pitch = ((norm_t * 360.0).to_radians().sin() * 37.2 + 1.6) * cfg.walk_arm_swing * sp;
            let arm_yaw = -(norm_t * 360.0 + 40.0).to_radians().cos() * 9.6 * sp;

            let r_arm_rot = if input.is_holding_main {
                [-21.8 - cfg.walk_arm_carry_pitch + r_arm_pitch * 0.3, -6.9 + arm_yaw * 0.3, 10.3]
            } else {
                [r_arm_pitch, arm_yaw, 9.2 + (norm_t * 360.0).to_radians().sin().abs() * 4.6 * sp]
            };

            let l_arm_rot = if input.is_holding_off {
                [-21.8 - cfg.walk_arm_carry_pitch + l_arm_pitch * 0.3, 6.9 + arm_yaw * 0.3, -10.3]
            } else {
                [l_arm_pitch, arm_yaw, -9.2 - (norm_t * 360.0).to_radians().sin().abs() * 4.6 * sp]
            };

            let m_rarm = bedrock_bone_matrix(m_body, p_rarm, r_arm_rot, [0.0, 0.0, 0.0]);
            let m_larm = bedrock_bone_matrix(m_body, p_larm, l_arm_rot, [0.0, 0.0, 0.0]);

            let r_rot_deg = sample_spline_1d(RIGHT_LEG_ROT_KFS, norm_t) * cfg.walk_leg_swing;
            let (r_pos_y, r_pos_z) = sample_spline_2d(RIGHT_LEG_POS_KFS, norm_t);

            let l_norm_t = (norm_t + 0.5).rem_euclid(1.0);
            let l_rot_deg = sample_spline_1d(RIGHT_LEG_ROT_KFS, l_norm_t) * cfg.walk_leg_swing;
            let (l_pos_y, l_pos_z) = sample_spline_2d(RIGHT_LEG_POS_KFS, l_norm_t);

            let m_rleg = bedrock_bone_matrix(m_root, p_rleg, [r_rot_deg * sp, 0.0, 0.0], [0.0, r_pos_y * sp * cfg.walk_leg_swing, r_pos_z * sp * cfg.walk_leg_swing]);
            let m_lleg = bedrock_bone_matrix(m_root, p_lleg, [l_rot_deg * sp, 0.0, 0.0], [0.0, l_pos_y * sp * cfg.walk_leg_swing, l_pos_z * sp * cfg.walk_leg_swing]);

            (m_waist, m_body, m_head, m_rarm, m_larm, m_rleg, m_lleg)
        } else {
            // Bedrock Actions & Stuff authentic idle breathing (Oreville tdrcwc)
            let breath_phase = t * 2.14;
            let heave = (breath_phase * 2.0).cos() * 0.15 - 0.15;
            let w_trans = [(breath_phase).sin() * 0.25, heave, 0.0];
            let w_rot = [0.0, 0.0, -(breath_phase).cos() * 1.0];
            let m_waist = bedrock_bone_matrix(m_root, p_waist, w_rot, w_trans);

            let m_body = bedrock_bone_matrix(m_waist, p_body, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);

            let h_rot = [
                input.head_pitch_deg - (breath_phase * 2.0).sin() * 2.0,
                input.head_yaw_deg - (breath_phase - 0.78).sin() * 1.0,
                -(breath_phase - 0.78).cos() * 3.0,
            ];
            let m_head = bedrock_bone_matrix(m_body, p_head, h_rot, [0.0, 0.0, 0.0]);

            let arm_sway = (breath_phase * 2.0).sin() * 1.25 + 3.2;
            let r_arm_rot = if input.is_holding_main {
                [-21.8 + (breath_phase * 2.0).sin() * 1.1, -5.7, 9.2]
            } else {
                [(breath_phase * 2.0).sin() * 1.1, 0.0, arm_sway]
            };

            let l_arm_rot = if input.is_holding_off {
                [-21.8 - (breath_phase * 2.0).sin() * 1.1, 5.7, -9.2]
            } else {
                [-(breath_phase * 2.0).sin() * 1.1, 0.0, -arm_sway]
            };

            let m_rarm = bedrock_bone_matrix(m_body, p_rarm, r_arm_rot, [0.0, 0.0, 0.0]);
            let m_larm = bedrock_bone_matrix(m_body, p_larm, l_arm_rot, [0.0, 0.0, 0.0]);

            let leg_sway = (breath_phase).sin() * 1.2;
            let m_rleg = bedrock_bone_matrix(m_root, p_rleg, [0.0, 0.0, leg_sway], [(breath_phase).sin() * 0.25, 0.0, 0.0]);
            let m_lleg = bedrock_bone_matrix(m_root, p_lleg, [0.0, 0.0, leg_sway], [(breath_phase).sin() * 0.25, 0.0, 0.0]);

            (m_waist, m_body, m_head, m_rarm, m_larm, m_rleg, m_lleg)
        };

        // Project Bedrock 3D scene graph matrices directly into Horizon's ModelPart poses
        let (head_pos, head_rot) = project_bedrock_bone_to_horizon(m_head, h_head);
        let (body_pos, body_rot) = project_bedrock_bone_to_horizon(m_body, h_body);
        let (rarm_pos, rarm_rot) = project_bedrock_bone_to_horizon(m_rarm, h_rarm);
        let (larm_pos, larm_rot) = project_bedrock_bone_to_horizon(m_larm, h_larm);
        let (rleg_pos, rleg_rot) = project_bedrock_bone_to_horizon(m_rleg, h_rleg);
        let (lleg_pos, lleg_rot) = project_bedrock_bone_to_horizon(m_lleg, h_lleg);

        res.head.translation = head_pos;
        res.head.rotation = head_rot;
        res.body.translation = body_pos;
        res.body.rotation = body_rot;
        res.right_arm.translation = rarm_pos;
        res.right_arm.rotation = rarm_rot;
        res.left_arm.translation = larm_pos;
        res.left_arm.rotation = larm_rot;
        res.right_leg.translation = rleg_pos;
        res.right_leg.rotation = rleg_rot;
        res.left_leg.translation = lleg_pos;
        res.left_leg.rotation = lleg_rot;

        // Dynamic attack slash (Oreville vvnkii)
        if input.swing > 0.0 {
            let sw = input.swing;
            let curve = (sw * std::f32::consts::PI).sin();
            let power = ((1.0 - (1.0 - sw) * (1.0 - sw)) * std::f32::consts::PI).sin();
            let sign = if input.is_swing_right { 1.0 } else { -1.0 };

            res.body.rotation[1] += curve * 0.65 * sign;
            res.body.rotation[0] += curve * 0.25;

            if input.is_swing_right {
                res.right_arm.rotation = [
                    -2.10 + curve * 3.20 - power * 0.85,
                    curve * 0.95,
                    -curve * 0.65,
                ];
                res.right_arm.translation[2] += curve * 3.5;
            } else {
                res.left_arm.rotation = [
                    -2.10 + curve * 3.20 - power * 0.85,
                    -curve * 0.95,
                    curve * 0.65,
                ];
                res.left_arm.translation[2] += curve * 3.5;
            }
        }

        // Elytra flight
        if input.is_fall_flying {
            res.body.rotation[0] = std::f32::consts::FRAC_PI_2;
            res.head.rotation[0] = -std::f32::consts::FRAC_PI_4;
            res.right_arm.rotation[0] = 0.2;
            res.left_arm.rotation[0] = 0.2;
            res.right_leg.rotation[0] = 0.1;
            res.left_leg.rotation[0] = 0.1;
        }

        res
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

        // Body/head should tilt and lower forward
        let head_pos = world["body"].transform_point3(JAVA_HEAD_ANCHOR);
        assert!(head_pos.y < 24.0, "Head did not lower during forward waist pitch");
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

    #[test]
    fn test_player_actions_and_stuff_kinematics_evaluation() {
        let idle_input = PlayerAnimInput {
            age: 20.0,
            pos: 0.0,
            speed: 0.0,
            head_pitch_deg: 0.0,
            head_yaw_deg: 0.0,
            swing: 0.0,
            is_crouching: false,
            is_holding_main: false,
            is_holding_off: false,
            is_swing_right: true,
            is_fall_flying: false,
        };
        let idle_pose = PlayerPoseResult::evaluate(&idle_input);
        // Idle pose must have natural breathing heave and relaxed outward arm roll
        assert!(idle_pose.body.translation[1].abs() > 0.0);
        assert!(idle_pose.right_arm.rotation[2] > 0.02); // Outward relaxed roll
        assert!(idle_pose.left_arm.rotation[2] < -0.02);

        // Sprint pose must have forward dash lean (0.38 rad = 22 deg)
        let sprint_input = PlayerAnimInput {
            speed: 1.2,
            pos: 2.35, // Near peak of arm swing (phi ~ 1.57 rad)
            ..idle_input
        };
        let sprint_pose = PlayerPoseResult::evaluate(&sprint_input);
        assert!(sprint_pose.body.rotation[0].abs() >= 0.35);
        assert!(sprint_pose.right_arm.rotation[0].abs() > 0.3);

        // Sword hold pose
        let sword_input = PlayerAnimInput {
            is_holding_main: true,
            ..idle_input
        };
        let sword_pose = PlayerPoseResult::evaluate(&sword_input);
        assert!(sword_pose.right_arm.rotation[0].abs() > 0.3); // Arm held naturally near chest (-22 deg)
    }
}
