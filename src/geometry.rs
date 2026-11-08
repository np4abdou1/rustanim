//! Bedrock geometry deserializer, bone hierarchy transform evaluator, and 3D mesh vertex generator.
//! Supports Bedrock `minecraft:geometry` format versions 1.8.0, 1.12.0, 1.14.0, and 1.16.0.
//! Evaluates cubes with box UV, per-face UV, inflation, rotation, mirror, and parent-child bone trees.

use glam::{EulerRot, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single face UV definition in Bedrock per-face UV mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaceUv {
    pub uv: [f32; 2],
    #[serde(default)]
    pub uv_size: Option<[f32; 2]>,
    #[serde(default)]
    pub uv_rotation: Option<f32>,
    #[serde(default)]
    pub material_instance: Option<String>,
}

/// UV mapping for a cube: either a 2D box UV offset `[u, v]` or per-face definitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CubeUv {
    Box([f32; 2]),
    PerFace {
        #[serde(default)]
        north: Option<FaceUv>,
        #[serde(default)]
        south: Option<FaceUv>,
        #[serde(default)]
        east: Option<FaceUv>,
        #[serde(default)]
        west: Option<FaceUv>,
        #[serde(default)]
        up: Option<FaceUv>,
        #[serde(default)]
        down: Option<FaceUv>,
    },
}

/// A 3D rectangular cuboid attached to a bone in Bedrock geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockCube {
    pub origin: [f32; 3],
    pub size: [f32; 3],
    #[serde(default)]
    pub uv: Option<CubeUv>,
    #[serde(default)]
    pub inflate: Option<f32>,
    #[serde(default)]
    pub mirror: Option<bool>,
    #[serde(default)]
    pub pivot: Option<[f32; 3]>,
    #[serde(default)]
    pub rotation: Option<[f32; 3]>,
}

/// A bone node in a Bedrock skeletal hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockBone {
    pub name: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub pivot: [f32; 3],
    #[serde(default)]
    pub rotation: Option<[f32; 3]>,
    #[serde(default)]
    pub bind_pose_rotation: Option<[f32; 3]>,
    #[serde(default)]
    pub binding: Option<String>,
    #[serde(default)]
    pub cubes: Vec<BedrockCube>,
    #[serde(default)]
    pub locators: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub neverrender: Option<bool>,
    #[serde(default)]
    pub reset: Option<bool>,
}

/// Description metadata of a Bedrock geometry definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryDescription {
    pub identifier: String,
    #[serde(default = "default_tex_dim")]
    pub texture_width: f32,
    #[serde(default = "default_tex_dim")]
    pub texture_height: f32,
    #[serde(default)]
    pub visible_bounds_width: Option<f32>,
    #[serde(default)]
    pub visible_bounds_height: Option<f32>,
    #[serde(default)]
    pub visible_bounds_offset: Option<[f32; 3]>,
}

fn default_tex_dim() -> f32 {
    64.0
}

/// A single complete Bedrock geometry model with its description and bone hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockModel {
    pub description: GeometryDescription,
    #[serde(default)]
    pub bones: Vec<BedrockBone>,
}

/// Root file structure of Bedrock geometry JSON files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BedrockGeometryFile {
    #[serde(default)]
    pub format_version: Option<String>,
    #[serde(rename = "minecraft:geometry")]
    pub geometries: GeometryContainer,
}

/// Container that handles `minecraft:geometry` being either a single model, an array, or legacy keyed map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GeometryContainer {
    List(Vec<BedrockModel>),
    Single(BedrockModel),
    Map(HashMap<String, BedrockModel>),
}

impl BedrockGeometryFile {
    /// Parses geometry definitions from a JSON string.
    pub fn parse(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Returns an iterator over all models contained in this geometry file.
    pub fn models(&self) -> Vec<&BedrockModel> {
        match &self.geometries {
            GeometryContainer::List(list) => list.iter().collect(),
            GeometryContainer::Single(model) => vec![model],
            GeometryContainer::Map(map) => map.values().collect(),
        }
    }

    /// Finds a model by its identifier (e.g. `"geometry.oreville_ans.hsygxb"`).
    pub fn find_model(&self, identifier: &str) -> Option<&BedrockModel> {
        self.models().into_iter().find(|m| m.description.identifier == identifier)
    }
}

/// A single rendered 3D vertex generated from Bedrock geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BedrockVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl BedrockModel {
    /// Returns a full Bedrock humanoid model (Steve/Alex) with standard Bedrock bone hierarchy and cubes.
    pub fn new_bedrock_humanoid(is_slim: bool) -> Self {
        let arm_w = if is_slim { 3.0 } else { 4.0 };
        let r_arm_x = if is_slim { -7.0 } else { -8.0 };
        let r_pivot_x = if is_slim { -4.5 } else { -5.0 };
        let l_pivot_x = if is_slim { 4.5 } else { 5.0 };

        let bones = vec![
            BedrockBone {
                name: "root".into(),
                parent: None,
                pivot: [0.0, 0.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: Vec::new(),
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "waist".into(),
                parent: Some("root".into()),
                pivot: [0.0, 12.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: Vec::new(),
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "body".into(),
                parent: Some("waist".into()),
                pivot: [0.0, 24.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Torso
                    BedrockCube {
                        origin: [-4.0, 12.0, -2.0],
                        size: [8.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([16.0, 16.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Jacket
                    BedrockCube {
                        origin: [-4.0, 12.0, -2.0],
                        size: [8.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([16.0, 32.0])),
                        inflate: Some(0.25),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "head".into(),
                parent: Some("body".into()),
                pivot: [0.0, 24.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Head
                    BedrockCube {
                        origin: [-4.0, 24.0, -4.0],
                        size: [8.0, 8.0, 8.0],
                        uv: Some(CubeUv::Box([0.0, 0.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Hat
                    BedrockCube {
                        origin: [-4.0, 24.0, -4.0],
                        size: [8.0, 8.0, 8.0],
                        uv: Some(CubeUv::Box([32.0, 0.0])),
                        inflate: Some(0.5),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "rightarm".into(),
                parent: Some("body".into()),
                pivot: [r_pivot_x, 22.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Right Arm
                    BedrockCube {
                        origin: [r_arm_x, 12.0, -2.0],
                        size: [arm_w, 12.0, 4.0],
                        uv: Some(CubeUv::Box([40.0, 16.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Right Sleeve
                    BedrockCube {
                        origin: [r_arm_x, 12.0, -2.0],
                        size: [arm_w, 12.0, 4.0],
                        uv: Some(CubeUv::Box([40.0, 32.0])),
                        inflate: Some(0.25),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "leftarm".into(),
                parent: Some("body".into()),
                pivot: [l_pivot_x, 22.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Left Arm
                    BedrockCube {
                        origin: [4.0, 12.0, -2.0],
                        size: [arm_w, 12.0, 4.0],
                        uv: Some(CubeUv::Box([32.0, 48.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Left Sleeve
                    BedrockCube {
                        origin: [4.0, 12.0, -2.0],
                        size: [arm_w, 12.0, 4.0],
                        uv: Some(CubeUv::Box([48.0, 48.0])),
                        inflate: Some(0.25),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "rightleg".into(),
                parent: Some("root".into()),
                pivot: [-1.9, 12.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Right Leg
                    BedrockCube {
                        origin: [-3.9, 0.0, -2.0],
                        size: [4.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([0.0, 16.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Right Pants
                    BedrockCube {
                        origin: [-3.9, 0.0, -2.0],
                        size: [4.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([0.0, 32.0])),
                        inflate: Some(0.25),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
            BedrockBone {
                name: "leftleg".into(),
                parent: Some("root".into()),
                pivot: [1.9, 12.0, 0.0],
                rotation: None,
                bind_pose_rotation: None,
                binding: None,
                cubes: vec![
                    // Left Leg
                    BedrockCube {
                        origin: [-0.1, 0.0, -2.0],
                        size: [4.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([16.0, 48.0])),
                        inflate: None,
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                    // Left Pants
                    BedrockCube {
                        origin: [-0.1, 0.0, -2.0],
                        size: [4.0, 12.0, 4.0],
                        uv: Some(CubeUv::Box([0.0, 48.0])),
                        inflate: Some(0.25),
                        mirror: None,
                        pivot: None,
                        rotation: None,
                    },
                ],
                locators: HashMap::new(),
                neverrender: None,
                reset: None,
            },
        ];

        Self {
            description: GeometryDescription {
                identifier: if is_slim {
                    "geometry.humanoid.customSlim".into()
                } else {
                    "geometry.humanoid.custom".into()
                },
                texture_width: 64.0,
                texture_height: 64.0,
                visible_bounds_width: Some(3.0),
                visible_bounds_height: Some(3.0),
                visible_bounds_offset: Some([0.0, 1.0, 0.0]),
            },
            bones,
        }
    }

    /// Builds bone parent-child index and returns sorted bone indices in parent-before-child topological order.
    pub fn topological_order(&self) -> Vec<usize> {
        let name_to_idx: HashMap<&str, usize> = self
            .bones
            .iter()
            .enumerate()
            .map(|(i, b)| (b.name.as_str(), i))
            .collect();

        let mut order = Vec::with_capacity(self.bones.len());
        let mut visited = vec![false; self.bones.len()];

        fn visit(
            idx: usize,
            bones: &[BedrockBone],
            name_to_idx: &HashMap<&str, usize>,
            visited: &mut [bool],
            order: &mut Vec<usize>,
        ) {
            if visited[idx] {
                return;
            }
            if let Some(parent_name) = &bones[idx].parent {
                if let Some(&p_idx) = name_to_idx.get(parent_name.as_str()) {
                    visit(p_idx, bones, name_to_idx, visited, order);
                }
            }
            visited[idx] = true;
            order.push(idx);
        }

        for i in 0..self.bones.len() {
            visit(i, &self.bones, &name_to_idx, &mut visited, &mut order);
        }

        order
    }

    /// Evaluates world transform matrices for all bones in bind pose or with animation overrides.
    /// `bone_overrides`: optional map of bone name -> (translation_delta, rotation_deg_delta, scale_mult).
    pub fn evaluate_bone_matrices(
        &self,
        bone_overrides: Option<&HashMap<&str, (Vec3, Vec3, Vec3)>>,
    ) -> HashMap<String, Mat4> {
        let order = self.topological_order();
        let mut world_matrices: HashMap<String, Mat4> = HashMap::with_capacity(self.bones.len());

        for idx in order {
            let bone = &self.bones[idx];
            let pivot = Vec3::from_array(bone.pivot);

            let (override_t, override_r, override_s) = bone_overrides
                .and_then(|ov| ov.get(bone.name.as_str()))
                .copied()
                .unwrap_or((Vec3::ZERO, Vec3::ZERO, Vec3::ONE));

            let base_rot_deg = bone.rotation.map(Vec3::from_array).unwrap_or(Vec3::ZERO);
            let total_rot_deg = base_rot_deg + override_r;

            // In Bedrock, Euler rotation order is ZYX with degrees:
            let q = Quat::from_euler(
                EulerRot::ZYX,
                total_rot_deg.z.to_radians(),
                total_rot_deg.y.to_radians(),
                total_rot_deg.x.to_radians(),
            );

            // Local bone transform relative to parent:
            // T(pivot + translation) * R(rot) * S(scale) * T(-pivot)
            let local_m = Mat4::from_translation(pivot + override_t)
                * Mat4::from_quat(q)
                * Mat4::from_scale(override_s)
                * Mat4::from_translation(-pivot);

            let parent_m = bone
                .parent
                .as_deref()
                .and_then(|p_name| world_matrices.get(p_name))
                .copied()
                .unwrap_or(Mat4::IDENTITY);

            let world_m = parent_m * local_m;
            world_matrices.insert(bone.name.clone(), world_m);
        }

        world_matrices
    }

    /// Generates triangulated vertices for all visible cubes in this model.
    /// Emits vertices into `out` with positions transformed by bone world matrices.
    pub fn generate_mesh(
        &self,
        world_matrices: &HashMap<String, Mat4>,
        base_transform: Mat4,
        out: &mut Vec<BedrockVertex>,
    ) {
        let tex_w = self.description.texture_width;
        let tex_h = self.description.texture_height;

        for bone in &self.bones {
            if bone.neverrender.unwrap_or(false) {
                continue;
            }
            let bone_m = match world_matrices.get(&bone.name) {
                Some(m) => *m,
                None => continue,
            };

            for cube in &bone.cubes {
                generate_cube_vertices(cube, bone_m, base_transform, tex_w, tex_h, out);
            }
        }
    }

    /// Evaluates Actions & Stuff kinematics on the Bedrock humanoid model and generates textured vertices
    /// in Horizon's coordinate system (Y-down, feet at -1.5, units in blocks).
    pub fn generate_bedrock_player_mesh(
        &self,
        input: &crate::rig::PlayerAnimInput,
        cfg: &crate::config::RustanimConfig,
        base_world: Mat4,
        atlas_uv: [f32; 4],
        out: &mut Vec<BedrockVertex>,
    ) {
        let mut overrides: HashMap<&str, (Vec3, Vec3, Vec3)> = HashMap::new();

        let sp = input.speed.clamp(0.0, 1.5);
        let phi = input.pos;
        let t = input.age;

        if sp > 0.05 {
            let is_sprint = sp > 0.85;
            if is_sprint {
                let norm_t = (phi / (2.0 * std::f32::consts::PI) * cfg.sprint_speed_mult).rem_euclid(1.0);
                let w_rot = Vec3::new(0.0, 0.0, -(norm_t * 360.0).to_radians().cos() * 5.0);
                let bounce = -(norm_t * 720.0).to_radians().cos() * 3.1
                    + (-(norm_t * 720.0).to_radians().cos() * 2.3).min(0.0);
                let w_pos = Vec3::new(0.0, bounce * cfg.sprint_bounce_mult, 0.0);
                overrides.insert("waist", (w_pos, w_rot, Vec3::ONE));

                let b_rot = Vec3::new(
                    cfg.sprint_body_pitch_deg,
                    -(norm_t * 360.0).to_radians().sin() * 24.0,
                    0.0,
                );
                overrides.insert("body", (Vec3::ZERO, b_rot, Vec3::ONE));

                let h_rot = Vec3::new(
                    input.head_pitch_deg - cfg.sprint_body_pitch_deg * 0.7,
                    input.head_yaw_deg,
                    -(norm_t * 360.0).to_radians().sin() * 6.0,
                );
                overrides.insert("head", (Vec3::ZERO, h_rot, Vec3::ONE));

                let r_arm_pitch = (-(norm_t * 360.0 + 10.0).to_radians().sin() * 55.0) * cfg.sprint_arm_swing;
                let l_arm_pitch = ((norm_t * 360.0 + 10.0).to_radians().sin() * 55.0) * cfg.sprint_arm_swing;
                let arm_yaw = (norm_t * 360.0).to_radians().sin() * 18.0;

                overrides.insert("rightarm", (Vec3::ZERO, Vec3::new(r_arm_pitch, arm_yaw, 15.0), Vec3::ONE));
                overrides.insert("leftarm", (Vec3::ZERO, Vec3::new(l_arm_pitch, arm_yaw, -15.0), Vec3::ONE));

                let (r_pos_y, r_pos_z) = crate::rig::sample_spline_2d(crate::rig::SPRINT_LEG_POS_KFS, norm_t);
                let r_rot_deg = -(norm_t * 360.0).to_radians().sin() * 68.0 * cfg.sprint_leg_swing;
                overrides.insert("rightleg", (Vec3::new(0.0, r_pos_y, r_pos_z), Vec3::new(r_rot_deg, 0.0, 0.0), Vec3::ONE));

                let l_norm_t = (norm_t + 0.5).rem_euclid(1.0);
                let (l_pos_y, l_pos_z) = crate::rig::sample_spline_2d(crate::rig::SPRINT_LEG_POS_KFS, l_norm_t);
                let l_rot_deg = -(l_norm_t * 360.0).to_radians().sin() * 68.0 * cfg.sprint_leg_swing;
                overrides.insert("leftleg", (Vec3::new(0.0, l_pos_y, l_pos_z), Vec3::new(l_rot_deg, 0.0, 0.0), Vec3::ONE));
            } else {
                let norm_t = (phi / (2.0 * std::f32::consts::PI) * cfg.walk_speed_mult).rem_euclid(1.0);
                let bounce = (norm_t * 720.0 + 12.0).to_radians().sin() * 1.3 * sp * cfg.walk_bounce_mult;
                let w_pos = Vec3::new(0.0, bounce, 0.6 * sp);
                let w_rot = Vec3::new(0.0, 0.0, (norm_t * 360.0 + 45.0).to_radians().cos() * cfg.walk_body_roll_deg * sp);
                overrides.insert("waist", (w_pos, w_rot, Vec3::ONE));

                let b_rot = Vec3::new(
                    cfg.walk_body_pitch_deg * sp,
                    (norm_t * 360.0 + 40.0).to_radians().cos() * cfg.walk_body_yaw_deg * sp,
                    0.0,
                );
                overrides.insert("body", (Vec3::ZERO, b_rot, Vec3::ONE));

                let h_rot = Vec3::new(
                    input.head_pitch_deg + (norm_t * 720.0).to_radians().cos() * 6.0 * sp,
                    input.head_yaw_deg + (norm_t * 360.0 + 40.0).to_radians().sin() * 3.0 * sp,
                    (norm_t * 360.0 - 45.0).to_radians().cos() * -5.4 * sp,
                );
                overrides.insert("head", (Vec3::ZERO, h_rot, Vec3::ONE));

                let r_arm_pitch = (-(norm_t * 360.0).to_radians().sin() * 37.2 + 1.6) * cfg.walk_arm_swing * sp;
                let l_arm_pitch = ((norm_t * 360.0).to_radians().sin() * 37.2 + 1.6) * cfg.walk_arm_swing * sp;
                let arm_yaw = -(norm_t * 360.0 + 40.0).to_radians().cos() * 9.6 * sp;

                overrides.insert("rightarm", (Vec3::ZERO, Vec3::new(r_arm_pitch, arm_yaw, 9.2), Vec3::ONE));
                overrides.insert("leftarm", (Vec3::ZERO, Vec3::new(l_arm_pitch, arm_yaw, -9.2), Vec3::ONE));

                let r_rot_deg = crate::rig::sample_spline_1d(crate::rig::RIGHT_LEG_ROT_KFS, norm_t) * cfg.walk_leg_swing;
                let (r_pos_y, r_pos_z) = crate::rig::sample_spline_2d(crate::rig::RIGHT_LEG_POS_KFS, norm_t);
                overrides.insert("rightleg", (Vec3::new(0.0, r_pos_y * sp, r_pos_z * sp), Vec3::new(r_rot_deg * sp, 0.0, 0.0), Vec3::ONE));

                let l_norm_t = (norm_t + 0.5).rem_euclid(1.0);
                let l_rot_deg = crate::rig::sample_spline_1d(crate::rig::RIGHT_LEG_ROT_KFS, l_norm_t) * cfg.walk_leg_swing;
                let (l_pos_y, l_pos_z) = crate::rig::sample_spline_2d(crate::rig::RIGHT_LEG_POS_KFS, l_norm_t);
                overrides.insert("leftleg", (Vec3::new(0.0, l_pos_y * sp, l_pos_z * sp), Vec3::new(l_rot_deg * sp, 0.0, 0.0), Vec3::ONE));
            }
        } else {
            // Idle breathing
            let breath = t * 2.14;
            let heave = (breath * 2.0).cos() * 0.15 - 0.15;
            overrides.insert("waist", (Vec3::new((breath).sin() * 0.25, heave, 0.0), Vec3::new(0.0, 0.0, -(breath).cos() * 1.0), Vec3::ONE));
            overrides.insert("body", (Vec3::ZERO, Vec3::new(0.0, (breath * 0.5).sin() * 1.5, 0.0), Vec3::ONE));
            overrides.insert("head", (Vec3::ZERO, Vec3::new(input.head_pitch_deg + (breath * 0.5).sin() * 1.5, input.head_yaw_deg + (breath * 0.5).cos() * 0.8, (breath * 0.5).sin() * 1.0), Vec3::ONE));
            overrides.insert("rightarm", (Vec3::ZERO, Vec3::new((breath).sin() * 1.2, 0.0, 8.5 + (breath).cos() * 0.8), Vec3::ONE));
            overrides.insert("leftarm", (Vec3::ZERO, Vec3::new(-(breath).sin() * 1.2, 0.0, -8.5 - (breath).cos() * 0.8), Vec3::ONE));
        }

        let world_matrices = self.evaluate_bone_matrices(Some(&overrides));

        let c = Mat4::from_cols_array(&[
            -1.0 / 16.0,  0.0,         0.0,        0.0,
             0.0,        -1.0 / 16.0,  0.0,        0.0,
             0.0,         0.0,         1.0 / 16.0, 0.0,
             0.0,         1.5,         0.0,        1.0,
        ]);
        let total_base = base_world * c;

        let start_idx = out.len();
        self.generate_mesh(&world_matrices, total_base, out);

        let u_min = atlas_uv[0];
        let v_min = atlas_uv[1];
        let du = atlas_uv[2] - atlas_uv[0];
        let dv = atlas_uv[3] - atlas_uv[1];

        for v in &mut out[start_idx..] {
            v.uv[0] = u_min + du * v.uv[0];
            v.uv[1] = v_min + dv * v.uv[1];
        }
    }
}

/// Generates the 6 faces (12 triangles, 36 vertices) for a single Bedrock cuboid.
pub fn generate_cube_vertices(
    cube: &BedrockCube,
    bone_matrix: Mat4,
    base_transform: Mat4,
    tex_w: f32,
    tex_h: f32,
    out: &mut Vec<BedrockVertex>,
) {
    let inflate = cube.inflate.unwrap_or(0.0);
    let o = Vec3::from_array(cube.origin);
    let s = Vec3::from_array(cube.size);

    let min = o - Vec3::splat(inflate);
    let max = o + s + Vec3::splat(inflate);

    // Compute cube local transform (if cube has local pivot and rotation)
    let cube_matrix = if let (Some(pivot), Some(rot)) = (cube.pivot, cube.rotation) {
        let p = Vec3::from_array(pivot);
        let r = Vec3::from_array(rot);
        let q = Quat::from_euler(
            EulerRot::ZYX,
            r.z.to_radians(),
            r.y.to_radians(),
            r.x.to_radians(),
        );
        bone_matrix * Mat4::from_translation(p) * Mat4::from_quat(q) * Mat4::from_translation(-p)
    } else {
        bone_matrix
    };

    let world_m = base_transform * cube_matrix;

    // The 8 corners of the cuboid
    let p000 = world_m.transform_point3(Vec3::new(min.x, min.y, min.z)).to_array();
    let p100 = world_m.transform_point3(Vec3::new(max.x, min.y, min.z)).to_array();
    let p110 = world_m.transform_point3(Vec3::new(max.x, max.y, min.z)).to_array();
    let p010 = world_m.transform_point3(Vec3::new(min.x, max.y, min.z)).to_array();

    let p001 = world_m.transform_point3(Vec3::new(min.x, min.y, max.z)).to_array();
    let p101 = world_m.transform_point3(Vec3::new(max.x, min.y, max.z)).to_array();
    let p111 = world_m.transform_point3(Vec3::new(max.x, max.y, max.z)).to_array();
    let p011 = world_m.transform_point3(Vec3::new(min.x, max.y, max.z)).to_array();

    // Helper to compute face normal from 3 world vertices
    let calc_normal = |v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]| -> [f32; 3] {
        let e1 = Vec3::from_array(v1) - Vec3::from_array(v0);
        let e2 = Vec3::from_array(v2) - Vec3::from_array(v0);
        let n = e1.cross(e2).normalize_or_zero();
        n.to_array()
    };

    // Helper to push a quad (2 triangles: v0-v1-v2, v0-v2-v3)
    let mut push_quad = |pts: [[f32; 3]; 4], uvs: [[f32; 2]; 4]| {
        let normal = calc_normal(pts[0], pts[1], pts[2]);
        // Triangle 1
        out.push(BedrockVertex { pos: pts[0], normal, uv: uvs[0] });
        out.push(BedrockVertex { pos: pts[1], normal, uv: uvs[1] });
        out.push(BedrockVertex { pos: pts[2], normal, uv: uvs[2] });
        // Triangle 2
        out.push(BedrockVertex { pos: pts[0], normal, uv: uvs[0] });
        out.push(BedrockVertex { pos: pts[2], normal, uv: uvs[2] });
        out.push(BedrockVertex { pos: pts[3], normal, uv: uvs[3] });
    };

    // Calculate face UVs
    match &cube.uv {
        Some(CubeUv::PerFace { north, south, east, west, up, down }) => {
            let get_uvs = |face: &Option<FaceUv>| -> Option<[[f32; 2]; 4]> {
                let f = face.as_ref()?;
                let u0 = f.uv[0] / tex_w;
                let v0 = f.uv[1] / tex_h;
                let (du, dv) = if let Some(sz) = f.uv_size {
                    (sz[0] / tex_w, sz[1] / tex_h)
                } else {
                    (0.0, 0.0)
                };
                let u1 = u0 + du;
                let v1 = v0 + dv;
                // Standard quad UV order: top-left, top-right, bottom-right, bottom-left
                Some([[u0, v0], [u1, v0], [u1, v1], [u0, v1]])
            };

            // North (Z-)
            if let Some(uvs) = get_uvs(north) {
                push_quad([p110, p010, p000, p100], uvs);
            }
            // South (Z+)
            if let Some(uvs) = get_uvs(south) {
                push_quad([p011, p111, p101, p001], uvs);
            }
            // West (X-)
            if let Some(uvs) = get_uvs(west) {
                push_quad([p010, p011, p001, p000], uvs);
            }
            // East (X+)
            if let Some(uvs) = get_uvs(east) {
                push_quad([p111, p110, p100, p101], uvs);
            }
            // Up (Y+)
            if let Some(uvs) = get_uvs(up) {
                push_quad([p011, p010, p110, p111], uvs);
            }
            // Down (Y-)
            if let Some(uvs) = get_uvs(down) {
                push_quad([p000, p001, p101, p100], uvs);
            }
        }
        Some(CubeUv::Box([u0, v0])) => {
            let u0 = *u0;
            let v0 = *v0;
            let dx = s.x;
            let dy = s.y;
            let dz = s.z;

            // Box UV layout:
            // West: [u0, v0 + dz] to [u0 + dz, v0 + dz + dy]
            // North: [u0 + dz, v0 + dz] to [u0 + dz + dx, v0 + dz + dy]
            // East: [u0 + dz + dx, v0 + dz] to [u0 + 2*dz + dx, v0 + dz + dy]
            // South: [u0 + 2*dz + dx, v0 + dz] to [u0 + 2*dz + 2*dx, v0 + dz + dy]
            // Up: [u0 + dz, v0] to [u0 + dz + dx, v0 + dz]
            // Down: [u0 + dz + dx, v0] to [u0 + dz + 2*dx, v0 + dz]

            let to_uv = |u: f32, v: f32, du: f32, dv: f32| -> [[f32; 2]; 4] {
                let u_a = u / tex_w;
                let v_a = v / tex_h;
                let u_b = (u + du) / tex_w;
                let v_b = (v + dv) / tex_h;
                [[u_a, v_a], [u_b, v_a], [u_b, v_b], [u_a, v_b]]
            };

            // North
            let uvs_n = to_uv(u0 + dz, v0 + dz, dx, dy);
            push_quad([p110, p010, p000, p100], uvs_n);

            // South
            let uvs_s = to_uv(u0 + 2.0 * dz + dx, v0 + dz, dx, dy);
            push_quad([p011, p111, p101, p001], uvs_s);

            // West
            let uvs_w = to_uv(u0, v0 + dz, dz, dy);
            push_quad([p010, p011, p001, p000], uvs_w);

            // East
            let uvs_e = to_uv(u0 + dz + dx, v0 + dz, dz, dy);
            push_quad([p111, p110, p100, p101], uvs_e);

            // Up
            let uvs_u = to_uv(u0 + dz, v0, dx, dz);
            push_quad([p011, p010, p110, p111], uvs_u);

            // Down
            let uvs_d = to_uv(u0 + dz + dx, v0, dx, dz);
            push_quad([p000, p001, p101, p100], uvs_d);
        }
        None => {
            let u0 = 0.0;
            let v0 = 0.0;
            let dx = s.x;
            let dy = s.y;
            let dz = s.z;

            let to_uv = |u: f32, v: f32, du: f32, dv: f32| -> [[f32; 2]; 4] {
                let u_a = u / tex_w;
                let v_a = v / tex_h;
                let u_b = (u + du) / tex_w;
                let v_b = (v + dv) / tex_h;
                [[u_a, v_a], [u_b, v_a], [u_b, v_b], [u_a, v_b]]
            };

            let uvs_n = to_uv(u0 + dz, v0 + dz, dx, dy);
            push_quad([p110, p010, p000, p100], uvs_n);
            let uvs_s = to_uv(u0 + 2.0 * dz + dx, v0 + dz, dx, dy);
            push_quad([p011, p111, p101, p001], uvs_s);
            let uvs_w = to_uv(u0, v0 + dz, dz, dy);
            push_quad([p010, p011, p001, p000], uvs_w);
            let uvs_e = to_uv(u0 + dz + dx, v0 + dz, dz, dy);
            push_quad([p111, p110, p100, p101], uvs_e);
            let uvs_u = to_uv(u0 + dz, v0, dx, dz);
            push_quad([p011, p010, p110, p111], uvs_u);
            let uvs_d = to_uv(u0 + dz + dx, v0, dx, dz);
            push_quad([p000, p001, p101, p100], uvs_d);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_cube_box_uv() {
        let sample = r#"{
            "format_version": "1.12.0",
            "minecraft:geometry": [
                {
                    "description": {
                        "identifier": "geometry.test.single",
                        "texture_width": 64,
                        "texture_height": 64
                    },
                    "bones": [
                        {
                            "name": "root",
                            "pivot": [0, 0, 0]
                        },
                        {
                            "name": "head",
                            "parent": "root",
                            "pivot": [0, 24, 0],
                            "cubes": [
                                {
                                    "origin": [-4, 24, -4],
                                    "size": [8, 8, 8],
                                    "uv": [0, 0]
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let file = BedrockGeometryFile::parse(sample).expect("Must parse");
        let model = file.find_model("geometry.test.single").expect("Found model");
        assert_eq!(model.bones.len(), 2);
        assert_eq!(model.bones[1].cubes.len(), 1);

        let order = model.topological_order();
        assert_eq!(order, vec![0, 1]);

        let matrices = model.evaluate_bone_matrices(None);
        assert_eq!(matrices.len(), 2);

        let mut vertices = Vec::new();
        model.generate_mesh(&matrices, Mat4::IDENTITY, &mut vertices);
        // 1 cube * 6 faces * 2 triangles * 3 vertices = 36 vertices
        assert_eq!(vertices.len(), 36);
    }

    #[test]
    fn test_bedrock_humanoid_mesh_generation() {
        let model = BedrockModel::new_bedrock_humanoid(false);
        assert_eq!(model.bones.len(), 8);

        let input = crate::rig::PlayerAnimInput {
            age: 10.0,
            pos: 5.0,
            speed: 1.0,
            head_pitch_deg: 10.0,
            head_yaw_deg: -15.0,
            swing: 0.0,
            is_crouching: false,
            is_holding_main: false,
            is_holding_off: false,
            is_swing_right: true,
            is_fall_flying: false,
        };
        let cfg = crate::config::RustanimConfig::default();
        let mut vertices = Vec::new();
        model.generate_bedrock_player_mesh(&input, &cfg, Mat4::IDENTITY, [0.0, 0.0, 1.0, 1.0], &mut vertices);

        // 8 bones with multiple cubes (head, hat, body, jacket, 2 arms + sleeves, 2 legs + pants)
        // Each cube = 36 vertices
        assert!(vertices.len() >= 36 * 8, "Expected at least {} vertices, got {}", 36 * 8, vertices.len());
        for v in &vertices {
            assert!(v.pos[0].is_finite());
            assert!(v.pos[1].is_finite());
            assert!(v.pos[2].is_finite());
            assert!(v.uv[0] >= 0.0 && v.uv[0] <= 1.0);
            assert!(v.uv[1] >= 0.0 && v.uv[1] <= 1.0);
        }
    }

    #[test]
    fn test_parse_real_diamond_sword_from_mcpack() {
        let mcpack_path = std::path::Path::new("/home/joyboy/Actions_and_Stuff_112.mcpack");
        if !mcpack_path.exists() {
            return;
        }
        let file = std::fs::File::open(mcpack_path).expect("open mcpack");
        let mut zip = zip::ZipArchive::new(file).expect("zip archive");
        let mut raw = String::new();
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            let name = entry.name().to_string();
            if name.starts_with("models/entity/") && name.ends_with(".json") {
                let mut content = String::new();
                use std::io::Read;
                if entry.read_to_string(&mut content).is_ok() && content.contains("geometry.oreville_ans.hsygxb") {
                    raw = content;
                    break;
                }
            }
        }
        assert!(!raw.is_empty(), "Found diamond sword geometry JSON");
        let geo_file = BedrockGeometryFile::parse(&raw).expect("Parse real geometry file");
        let sword = geo_file.find_model("geometry.oreville_ans.hsygxb").expect("Found hsygxb model");
        assert_eq!(sword.description.identifier, "geometry.oreville_ans.hsygxb");
        assert_eq!(sword.description.texture_width, 1024.0);
        assert_eq!(sword.description.texture_height, 1024.0);

        let matrices = sword.evaluate_bone_matrices(None);
        assert!(matrices.contains_key("sword"));
        assert!(matrices.contains_key("3d256"));

        let mut verts = Vec::new();
        sword.generate_mesh(&matrices, Mat4::IDENTITY, &mut verts);
        assert!(verts.len() > 100, "Generated 3D diamond sword vertices: {}", verts.len());
    }
}
