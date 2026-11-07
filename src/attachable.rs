//! Bedrock attachables (`attachables/*.json`), 3D item geometries, and mount point transforms.

use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachableMountPoint {
    RightHand,
    LeftHand,
    Head,
    BodyBack,
    HipLeft,
    HipRight,
}

#[derive(Debug, Clone)]
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
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_euler(
                glam::EulerRot::ZYX,
                self.rotation.z.to_radians(),
                self.rotation.y.to_radians(),
                self.rotation.x.to_radians(),
            ),
            self.position,
        )
    }
}

pub struct AttachableDefinition {
    pub identifier: String,
    pub geometry_name: String,
    pub default_mount: AttachableMountPoint,
    pub first_person_transform: ItemTransform3D,
    pub third_person_transform: ItemTransform3D,
}
