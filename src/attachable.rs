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
}
