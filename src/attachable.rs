//! Bedrock attachables (`attachables/*.json`), 3D item geometries, and dual-perspective mount point transforms.
//! Provides dedicated 1st-person viewmodel projection (70° FOV, near-plane clipping guard)
//! and 3rd-person player socket attachments.

use glam::{EulerRot, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard mount point attachment locations on a humanoid character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttachableMountPoint {
    RightHand,
    LeftHand,
    Head,
    BodyBack,
    HipLeft,
    HipRight,
}

impl AttachableMountPoint {
    /// Returns the canonical rig bone/socket name this mount point attaches to.
    pub fn socket_name(&self) -> &'static str {
        match self {
            AttachableMountPoint::RightHand => "right_item",
            AttachableMountPoint::LeftHand => "left_item",
            AttachableMountPoint::Head => "head",
            AttachableMountPoint::BodyBack => "body",
            AttachableMountPoint::HipLeft => "waist",
            AttachableMountPoint::HipRight => "waist",
        }
    }
}

/// 3D transform descriptor for item positioning, rotation (Euler degrees ZYX), and scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemTransform3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for ItemTransform3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

impl ItemTransform3D {
    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Converts this 3D transform to a standard 4x4 matrix using ZYX Euler angle order.
    #[inline]
    pub fn to_mat4(&self) -> Mat4 {
        let rot = Quat::from_euler(
            EulerRot::ZYX,
            self.rotation.z.to_radians(),
            self.rotation.y.to_radians(),
            self.rotation.x.to_radians(),
        );
        Mat4::from_scale_rotation_translation(self.scale, rot, self.position)
    }
}

/// Full Bedrock attachable definition (`attachables/*.json`) containing geometries,
/// textures, mount locations, and dual-perspective transform configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachableDefinition {
    pub identifier: String,
    pub geometry_name: String,
    pub default_mount: AttachableMountPoint,
    pub first_person_transform: ItemTransform3D,
    pub third_person_transform: ItemTransform3D,
}

impl AttachableDefinition {
    /// Constructs a standard attachable with dual-perspective configurations.
    pub fn new(
        identifier: impl Into<String>,
        geometry_name: impl Into<String>,
        default_mount: AttachableMountPoint,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            geometry_name: geometry_name.into(),
            default_mount,
            first_person_transform: ItemTransform3D::default(),
            third_person_transform: ItemTransform3D::default(),
        }
    }

    /// Selects the appropriate transform descriptor based on viewpoint.
    #[inline]
    pub fn get_transform(&self, is_first_person: bool) -> &ItemTransform3D {
        if is_first_person {
            &self.first_person_transform
        } else {
            &self.third_person_transform
        }
    }

    /// Computes the final world matrix for this attachable mounted to a player rig in 3rd person.
    pub fn compute_third_person_world_matrix(
        &self,
        socket_world_matrix: Mat4,
        mount_override: Option<AttachableMountPoint>,
    ) -> Mat4 {
        let mount = mount_override.unwrap_or(self.default_mount);
        let mount_offset = match mount {
            AttachableMountPoint::RightHand => Mat4::IDENTITY,
            AttachableMountPoint::LeftHand => Mat4::IDENTITY,
            AttachableMountPoint::Head => Mat4::from_translation(Vec3::new(0.0, 4.0, 0.0)),
            AttachableMountPoint::BodyBack => {
                // Holstered on back: rotated flat against torso
                Mat4::from_translation(Vec3::new(0.0, -2.0, -2.5))
                    * Mat4::from_rotation_y(180.0f32.to_radians())
                    * Mat4::from_rotation_z(45.0f32.to_radians())
            }
            AttachableMountPoint::HipLeft => {
                Mat4::from_translation(Vec3::new(3.0, -1.0, 0.0))
                    * Mat4::from_rotation_z(-30.0f32.to_radians())
            }
            AttachableMountPoint::HipRight => {
                Mat4::from_translation(Vec3::new(-3.0, -1.0, 0.0))
                    * Mat4::from_rotation_z(30.0f32.to_radians())
            }
        };

        socket_world_matrix * mount_offset * self.third_person_transform.to_mat4()
    }
}

// ---------------------------------------------------------------------------
// 1st-Person Viewmodel Matrix Math & Camera Anti-Clipping
// ---------------------------------------------------------------------------

/// Dedicated 1st-person viewmodel projection matrix:
/// Fixed 70° vertical FOV, z_near = 0.01 (10mm) to prevent near-plane weapon clipping during swings.
#[inline]
pub fn compute_viewmodel_projection(aspect: f32) -> Mat4 {
    Mat4::perspective_rh(70.0f32.to_radians(), aspect, 0.01, 100.0)
}

/// Standard 1st-person camera eye base offset matrix for the main hand item:
/// Placed [+0.56, -0.52, -0.72] in front and to the right of the camera eye.
#[inline]
pub fn compute_viewmodel_mainhand_base_offset() -> Mat4 {
    Mat4::from_translation(Vec3::new(0.56, -0.52, -0.72))
}

/// Standard 1st-person camera eye base offset matrix for the offhand item:
/// Placed [-0.56, -0.52, -0.72] in front and to the left of the camera eye.
#[inline]
pub fn compute_viewmodel_offhand_base_offset() -> Mat4 {
    Mat4::from_translation(Vec3::new(-0.56, -0.52, -0.72))
}

/// Computes the complete 1st-person viewmodel model-view matrix for an item:
/// M_viewmodel = EyeBaseOffset * HandSwayPose * ItemTransform
pub fn compute_viewmodel_matrix(
    is_main_hand: bool,
    item_transform: &ItemTransform3D,
    hand_sway_and_swing: Mat4,
) -> Mat4 {
    let base_eye = if is_main_hand {
        compute_viewmodel_mainhand_base_offset()
    } else {
        compute_viewmodel_offhand_base_offset()
    };

    base_eye * hand_sway_and_swing * item_transform.to_mat4()
}

/// Checks whether a vertex in eye space lies within the visible viewmodel clipping bounds
/// to prevent geometry penetrating behind the camera near plane.
#[inline]
pub fn is_viewmodel_vertex_safe(eye_vertex: Vec3, min_near_z: f32) -> bool {
    // In right-handed OpenGL/Vulkan eye space, camera looks down -Z.
    // Safe vertices must have Z < -min_near_z.
    eye_vertex.z <= -min_near_z
}

/// Third-person free orbit camera defined by spherical coordinates.
#[derive(Debug, Clone)]
pub struct ThirdPersonCamera {
    pub target: Vec3,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub distance: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 24.0, 0.0),
            yaw_deg: 0.0,
            pitch_deg: 15.0,
            distance: 4.0,
        }
    }
}

impl ThirdPersonCamera {
    /// Computes the camera eye position in world space.
    pub fn eye_position(&self) -> Vec3 {
        let yaw_rad = self.yaw_deg.to_radians();
        let pitch_rad = self.pitch_deg.to_radians();
        let x = self.distance * pitch_rad.cos() * yaw_rad.sin();
        let y = self.distance * pitch_rad.sin();
        let z = self.distance * pitch_rad.cos() * yaw_rad.cos();
        self.target + Vec3::new(x, y, z)
    }

    /// Computes the right-handed view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }
}

/// Over-the-shoulder cinematic camera with smooth exponential lerp.
#[derive(Debug, Clone)]
pub struct ShoulderCamera {
    pub current_eye: Vec3,
    pub shoulder_offset: Vec3, // e.g. [+0.6, 0.2, 2.5]
    pub lerp_speed: f32,
}

impl Default for ShoulderCamera {
    fn default() -> Self {
        Self {
            current_eye: Vec3::ZERO,
            shoulder_offset: Vec3::new(0.65, 0.25, 2.8),
            lerp_speed: 10.0,
        }
    }
}

impl ShoulderCamera {
    pub fn update(&mut self, player_pos: Vec3, player_yaw_deg: f32, dt: f32) {
        let yaw_rot = Quat::from_rotation_y(player_yaw_deg.to_radians());
        let target_eye = player_pos + yaw_rot * self.shoulder_offset;
        let t = (self.lerp_speed * dt).clamp(0.0, 1.0);
        self.current_eye = self.current_eye.lerp(target_eye, t);
    }
}

/// Optical refraction offset for underwater viewports.
pub fn compute_underwater_refraction_matrix(time_seconds: f32) -> Mat4 {
    let wobble_x = (time_seconds * 2.0).sin() * 0.015;
    let wobble_y = (time_seconds * 2.5).cos() * 0.012;
    Mat4::from_translation(Vec3::new(wobble_x, wobble_y, 0.0))
}

/// Death camera horizon tilt matrix (rolls camera sideways as character collapses).
pub fn compute_death_camera_matrix(death_progress_0_to_1: f32) -> Mat4 {
    let progress = death_progress_0_to_1.clamp(0.0, 1.0);
    let roll_angle = progress * 65.0; // Roll sideways up to 65 degrees
    let drop_y = -progress * 1.5;     // Drop toward ground
    Mat4::from_translation(Vec3::new(0.0, drop_y, 0.0))
        * Mat4::from_rotation_z(roll_angle.to_radians())
}

/// Stereoscopic VR viewport rig supporting dual eye projections with configurable IPD.
#[derive(Debug, Clone)]
pub struct StereoRigVr {
    pub ipd_meters: f32, // Default 0.064m (64mm)
    pub fov_deg: f32,
}

impl Default for StereoRigVr {
    fn default() -> Self {
        Self {
            ipd_meters: 0.064,
            fov_deg: 90.0,
        }
    }
}

impl StereoRigVr {
    /// Computes view matrix for the left eye (-IPD/2 offset) or right eye (+IPD/2 offset).
    pub fn eye_view_matrix(&self, head_matrix: Mat4, is_left_eye: bool) -> Mat4 {
        let half_ipd = self.ipd_meters * 0.5;
        let eye_offset = if is_left_eye { -half_ipd } else { half_ipd };
        let offset_matrix = Mat4::from_translation(Vec3::new(eye_offset, 0.0, 0.0));
        (head_matrix * offset_matrix).inverse()
    }

    /// Computes symmetric VR eye perspective projection matrix.
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_deg.to_radians(), aspect, 0.05, 1000.0)
    }
}

/// Replay cinematic camera keyframe path interpolator.
#[derive(Debug, Clone)]
pub struct ReplayCameraPath {
    pub keyframes: Vec<(f32, Vec3, Vec3)>, // (timestamp, position, target_look_at)
}

impl ReplayCameraPath {
    pub fn new() -> Self {
        Self { keyframes: Vec::new() }
    }

    pub fn add_keyframe(&mut self, time: f32, position: Vec3, target: Vec3) {
        self.keyframes.push((time, position, target));
        self.keyframes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    pub fn sample(&self, time: f32) -> (Vec3, Vec3) {
        if self.keyframes.is_empty() {
            return (Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        }
        if self.keyframes.len() == 1 || time <= self.keyframes[0].0 {
            return (self.keyframes[0].1, self.keyframes[0].2);
        }
        if time >= self.keyframes.last().unwrap().0 {
            let last = self.keyframes.last().unwrap();
            return (last.1, last.2);
        }
        let idx = self.keyframes.partition_point(|k| k.0 <= time).saturating_sub(1);
        let k0 = &self.keyframes[idx];
        let k1 = &self.keyframes[(idx + 1).min(self.keyframes.len() - 1)];
        let dt = k1.0 - k0.0;
        let u = if dt > 1e-6 { (time - k0.0) / dt } else { 0.0 };
        (k0.1.lerp(k1.1, u), k0.2.lerp(k1.2, u))
    }
}

// ---------------------------------------------------------------------------
// Bedrock Attachable JSON Schema Serialization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockAttachableFile {
    pub format_version: String,
    #[serde(rename = "minecraft:attachable")]
    pub attachable: BedrockAttachableInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockAttachableInner {
    pub description: BedrockAttachableDescription,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockAttachableDescription {
    pub identifier: String,
    #[serde(default)]
    pub item: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub materials: HashMap<String, String>,
    #[serde(default)]
    pub textures: HashMap<String, String>,
    #[serde(default)]
    pub geometry: HashMap<String, String>,
    #[serde(default)]
    pub animations: HashMap<String, String>,
    #[serde(default)]
    pub scripts: Option<serde_json::Value>,
    #[serde(default)]
    pub render_controllers: Vec<serde_json::Value>,
}

impl BedrockAttachableFile {
    /// Deserializes a Bedrock attachable JSON string into a structured definition.
    pub fn parse(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Converts this raw Bedrock attachable file into a high-level `AttachableDefinition`.
    pub fn to_definition(&self) -> AttachableDefinition {
        let desc = &self.attachable.description;
        let geometry_name = desc
            .geometry
            .get("default")
            .cloned()
            .unwrap_or_else(|| "geometry.default".into());

        // Infer default mount point from identifier or item tags
        let default_mount = if desc.identifier.contains("helmet") || desc.identifier.contains("hat") {
            AttachableMountPoint::Head
        } else if desc.identifier.contains("shield") {
            AttachableMountPoint::LeftHand
        } else {
            AttachableMountPoint::RightHand
        };

        AttachableDefinition {
            identifier: desc.identifier.clone(),
            geometry_name,
            default_mount,
            first_person_transform: ItemTransform3D::default(),
            third_person_transform: ItemTransform3D::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    #[test]
    fn test_attachable_mount_point_sockets() {
        assert_eq!(AttachableMountPoint::RightHand.socket_name(), "right_item");
        assert_eq!(AttachableMountPoint::LeftHand.socket_name(), "left_item");
        assert_eq!(AttachableMountPoint::Head.socket_name(), "head");
        assert_eq!(AttachableMountPoint::BodyBack.socket_name(), "body");
        assert_eq!(AttachableMountPoint::HipLeft.socket_name(), "waist");
        assert_eq!(AttachableMountPoint::HipRight.socket_name(), "waist");
    }

    #[test]
    fn test_viewmodel_projection_aspect_and_clipping() {
        let aspect = 1920.0 / 1080.0;
        let proj = compute_viewmodel_projection(aspect);
        let base_offset = compute_viewmodel_mainhand_base_offset();

        let weapon_point = Vec3::new(0.0, 0.0, -0.5); // 0.5m in front of hand
        let eye_pt = base_offset.transform_point3(weapon_point);
        assert!(eye_pt.z < 0.0);
        assert!(is_viewmodel_vertex_safe(eye_pt, 0.01));

        let clip_pos = proj * Vec4::new(eye_pt.x, eye_pt.y, eye_pt.z, 1.0);
        let ndc_z = clip_pos.z / clip_pos.w;
        assert!(ndc_z >= -1.0 && ndc_z <= 1.0);
    }

    #[test]
    fn test_attachable_json_deserialization() {
        let sample_json = r#"{
            "format_version": "1.10.0",
            "minecraft:attachable": {
                "description": {
                    "identifier": "minecraft:diamond_sword",
                    "materials": {
                        "default": "entity_alphatest"
                    },
                    "textures": {
                        "default": "textures/items/diamond_sword"
                    },
                    "geometry": {
                        "default": "geometry.oreville_ans.diamond_sword"
                    }
                }
            }
        }"#;

        let parsed = BedrockAttachableFile::parse(sample_json).expect("Attachable JSON must parse");
        assert_eq!(parsed.attachable.description.identifier, "minecraft:diamond_sword");
        let def = parsed.to_definition();
        assert_eq!(def.geometry_name, "geometry.oreville_ans.diamond_sword");
        assert_eq!(def.default_mount, AttachableMountPoint::RightHand);
    }

    #[test]
    fn test_third_person_body_back_mount_offset() {
        let def = AttachableDefinition::new(
            "minecraft:diamond_sword",
            "geometry.oreville_ans.diamond_sword",
            AttachableMountPoint::BodyBack,
        );
        let body_world = Mat4::from_translation(Vec3::new(0.0, 24.0, 0.0));
        let back_world = def.compute_third_person_world_matrix(body_world, None);
        let pos = back_world.transform_point3(Vec3::ZERO);
        assert!(pos.z < 0.0, "BodyBack mount should position item behind torso (Z < 0)");
    }

    #[test]
    fn test_viewport_cameras_and_vr() {
        // Orbit camera
        let orbit = ThirdPersonCamera::default();
        let view_m = orbit.view_matrix();
        assert!(view_m.is_finite());

        // Shoulder camera
        let mut shoulder = ShoulderCamera::default();
        shoulder.update(Vec3::new(10.0, 64.0, 10.0), 45.0, 0.1);
        assert!(shoulder.current_eye.length() > 0.0);

        // Underwater & Death cameras
        let water = compute_underwater_refraction_matrix(1.5);
        let death = compute_death_camera_matrix(0.8);
        assert!(water.is_finite());
        assert!(death.is_finite());

        // VR Stereo
        let vr = StereoRigVr::default();
        let left_eye = vr.eye_view_matrix(Mat4::IDENTITY, true);
        let right_eye = vr.eye_view_matrix(Mat4::IDENTITY, false);
        assert!(left_eye.is_finite());
        assert!(right_eye.is_finite());

        // Replay camera path
        let mut path = ReplayCameraPath::new();
        path.add_keyframe(0.0, Vec3::ZERO, Vec3::new(0.0, 0.0, 10.0));
        path.add_keyframe(2.0, Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 0.0, 10.0));
        let (sampled_pos, _) = path.sample(1.0);
        assert!((sampled_pos.y - 2.5).abs() < 1e-4);
    }
}
