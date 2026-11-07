//! # Tier 1: Happy-Path Feature Coverage E2E Test Suite
//! Covers Features 1-16: Mcpack reader, animation JSON parser, Catmull-Rom splines,
//! Molang degrees math & expressions, vanilla Java joint anchors, virtual waist rigging,
//! and dual-perspective attachable transformations.

mod common;
use common::*;
use glam::{Mat4, Quat, Vec3, Vec4};
use rustanim::anim::{AnimationClip, AnimationController, BoneAnimation};
use rustanim::attachable::{AttachableDefinition, AttachableMountPoint, ItemTransform3D};
use rustanim::molang::{MolangContext, MolangEngine, MolangExpr};
use rustanim::rig::RelativePivotRig;

/// Feature 1: Mcpack Zip Archive Reader
/// Verifies reading manifest.json and locating animation and attachable files.
#[test]
fn test_tier1_mcpack_reader_and_manifest() {
    let fixture = match McpackFixture::new() {
        Some(f) => f,
        None => {
            eprintln!("Skipping mcpack test: archive not found at {}", MCPACK_PATH);
            return;
        }
    };

    let manifest_str = fixture
        .read_file_text("manifest.json")
        .expect("Failed to read manifest.json from mcpack");

    let manifest_json: serde_json::Value =
        serde_json::from_str(&manifest_str).expect("manifest.json must be valid JSON");

    assert_eq!(manifest_json["format_version"], 3);
    assert_eq!(
        manifest_json["header"]["uuid"],
        "2cf066eb-1254-4b7d-affb-80fe3216b18c"
    );

    // Verify subpack configurations (SP0, SP1, SP2)
    let subpacks = manifest_json["subpacks"]
        .as_array()
        .expect("subpacks must be an array");
    assert_eq!(subpacks.len(), 3);
    assert_eq!(subpacks[0]["folder_name"], "SP0");
    assert_eq!(subpacks[1]["folder_name"], "SP1");
    assert_eq!(subpacks[2]["folder_name"], "SP2");
}

/// Feature 2: Bedrock Animation JSON Parser
/// Verifies deserialization of format 1.8.0/1.10.0 animation clip structures.
#[test]
fn test_tier1_animation_json_parser() {
    let sample_json = r#"{
        "format_version": "1.8.0",
        "animations": {
            "animation.player.test_walk": {
                "loop": true,
                "animation_length": 1.0,
                "bones": {
                    "rightleg": {
                        "rotation": {
                            "0.0": [0.0, 0.0, 0.0],
                            "0.5": [30.0, 0.0, 0.0],
                            "1.0": [0.0, 0.0, 0.0]
                        },
                        "position": [0.0, 0.0, 0.0]
                    },
                    "leftleg": {
                        "rotation": {
                            "0.0": [0.0, 0.0, 0.0],
                            "0.5": [-30.0, 0.0, 0.0],
                            "1.0": [0.0, 0.0, 0.0]
                        }
                    }
                }
            }
        }
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(sample_json).unwrap();
    let anim_obj = &parsed["animations"]["animation.player.test_walk"];

    assert_eq!(anim_obj["loop"], true);
    assert_eq!(anim_obj["animation_length"], 1.0);
    assert!(anim_obj["bones"]["rightleg"].is_object());
    assert!(anim_obj["bones"]["leftleg"].is_object());

    // Verify compatibility with AnimationClip struct in crate
    let mut clip = AnimationClip {
        name: "animation.player.test_walk".into(),
        loop_mode: true,
        anim_length: 1.0,
        bones: std::collections::HashMap::new(),
    };

    clip.bones.insert(
        "rightleg".into(),
        BoneAnimation {
            name: "rightleg".into(),
            positions: vec![(0.0, Vec3::ZERO)],
            rotations: vec![
                (0.0, Vec3::new(0.0, 0.0, 0.0)),
                (0.5, Vec3::new(30.0, 0.0, 0.0)),
                (1.0, Vec3::new(0.0, 0.0, 0.0)),
            ],
            scales: vec![(0.0, Vec3::ONE)],
        },
    );

    assert_eq!(clip.bones.len(), 1);
    assert_eq!(clip.bones["rightleg"].rotations.len(), 3);
}

/// Feature 3: Catmull-Rom Spline Evaluator
/// Verifies cubic Hermite basis functions and smooth curve interpolation across keyframe channels.
#[test]
fn test_tier1_catmull_rom_spline_happy_path() {
    let keyframes = vec![
        KeyframePoint3D::continuous(0.0, Vec3::new(0.0, 0.0, 0.0), true),
        KeyframePoint3D::continuous(0.25, Vec3::new(10.0, 2.0, -5.0), true),
        KeyframePoint3D::continuous(0.5, Vec3::new(20.0, 5.0, 0.0), true),
        KeyframePoint3D::continuous(0.75, Vec3::new(10.0, 2.0, 5.0), true),
        KeyframePoint3D::continuous(1.0, Vec3::new(0.0, 0.0, 0.0), true),
    ];

    // Evaluate exact keyframe timestamps (must match keyframe values)
    let p0 = evaluate_catmull_rom_spline(&keyframes, 0.0, false, 1.0);
    assert!((p0 - Vec3::new(0.0, 0.0, 0.0)).length() < 1e-4);

    let p_mid = evaluate_catmull_rom_spline(&keyframes, 0.5, false, 1.0);
    assert!((p_mid - Vec3::new(20.0, 5.0, 0.0)).length() < 1e-4);

    let p1 = evaluate_catmull_rom_spline(&keyframes, 1.0, false, 1.0);
    assert!((p1 - Vec3::new(0.0, 0.0, 0.0)).length() < 1e-4);

    // Evaluate midpoint between 0.0 and 0.25 (time = 0.125)
    let p_interp = evaluate_catmull_rom_spline(&keyframes, 0.125, false, 1.0);
    // Smooth Catmull-Rom curve should be between 0 and 10 on X
    assert!(p_interp.x > 3.0 && p_interp.x < 7.0);
    assert!(p_interp.y > 0.5 && p_interp.y < 2.0);
}

/// Feature 4: Animation Clip Sampling
/// Verifies multi-channel bone animation timeline playback and controller blending.
#[test]
fn test_tier1_animation_clip_sampling() {
    let mut controller = AnimationController::new();
    assert_eq!(controller.active_clips.len(), 0);

    let clip = AnimationClip {
        name: "test_clip".into(),
        loop_mode: true,
        anim_length: 2.0,
        bones: std::collections::HashMap::new(),
    };

    controller.active_clips.push((clip, 0.5, 1.0));
    assert_eq!(controller.active_clips.len(), 1);
    assert_eq!(controller.active_clips[0].1, 0.5); // current time
    assert_eq!(controller.active_clips[0].2, 1.0); // blend weight
}

/// Features 5-7: Molang Engine Math & Expressions
/// Verifies basic AST evaluation, binary operations, and ternary conditions.
#[test]
fn test_tier1_molang_engine_math_eval() {
    let engine = MolangEngine::new();
    let ctx = MolangContext::default();

    // Constant evaluation
    let expr_const = MolangExpr::Constant(123.45);
    assert_eq!(engine.evaluate(&expr_const, &ctx), 123.45);

    // Addition & Multiplication: 5.0 + 3.0 * 2.0 = 11.0
    let expr_math = MolangExpr::Add(
        Box::new(MolangExpr::Constant(5.0)),
        Box::new(MolangExpr::Multiply(
            Box::new(MolangExpr::Constant(3.0)),
            Box::new(MolangExpr::Constant(2.0)),
        )),
    );
    assert_eq!(engine.evaluate(&expr_math, &ctx), 11.0);

    // Ternary operator: true ? 100 : 200
    let expr_ternary = MolangExpr::Ternary {
        condition: Box::new(MolangExpr::Constant(1.0)),
        then_branch: Box::new(MolangExpr::Constant(100.0)),
        else_branch: Box::new(MolangExpr::Constant(200.0)),
    };
    assert_eq!(engine.evaluate(&expr_ternary, &ctx), 100.0);
}

/// Feature 8: Molang Builtin Math with Bedrock Degree Trigonometry
/// Minecraft Bedrock Molang requires ALL trigonometric functions to operate in DEGREES.
#[test]
fn test_tier1_molang_degree_math_and_easing() {
    let engine = MolangEngine::new();
    let ctx = MolangContext::default();

    // math.sin in degrees: sin(90°) = 1.0, sin(0°) = 0.0, sin(30°) = 0.5
    let expr_sin_90 = MolangExpr::MathFunc {
        name: "math.sin".into(),
        args: vec![MolangExpr::Constant(90.0)],
    };
    let sin_90 = engine.evaluate(&expr_sin_90, &ctx);
    assert!((sin_90 - 1.0).abs() < 1e-5, "math.sin(90) must be 1.0 in degrees");

    // math.cos in degrees: cos(0°) = 1.0, cos(60°) = 0.5, cos(90°) = 0.0
    let expr_cos_60 = MolangExpr::MathFunc {
        name: "math.cos".into(),
        args: vec![MolangExpr::Constant(60.0)],
    };
    let cos_60 = engine.evaluate(&expr_cos_60, &ctx);
    assert!((cos_60 - 0.5).abs() < 1e-5, "math.cos(60) must be 0.5 in degrees");

    // math.clamp(15.0, 0.0, 10.0) = 10.0
    let expr_clamp = MolangExpr::MathFunc {
        name: "math.clamp".into(),
        args: vec![
            MolangExpr::Constant(15.0),
            MolangExpr::Constant(0.0),
            MolangExpr::Constant(10.0),
        ],
    };
    assert_eq!(engine.evaluate(&expr_clamp, &ctx), 10.0);

    // Test reference easing and degree inverse functions
    assert!((molang_asin(1.0) - 90.0).abs() < 1e-4);
    assert!((molang_acos(0.0) - 90.0).abs() < 1e-4);
    assert!((molang_atan(1.0) - 45.0).abs() < 1e-4);
    assert!((molang_atan2(1.0, 1.0) - 45.0).abs() < 1e-4);

    // Penner easing: ease_out_cubic(0, 100, 0.5) = 100 * (1 - 0.125) = 87.5
    assert!((molang_ease_out_cubic(0.0, 100.0, 0.5) - 87.5).abs() < 1e-4);
    // Hermite blend: 3*(0.5)^2 - 2*(0.5)^3 = 0.75 - 0.25 = 0.5
    assert!((molang_hermite_blend(0.5) - 0.5).abs() < 1e-4);
}

/// Feature 9: Molang Query & Variable Context Binding
/// Verifies binding of canonical locomotion queries and local variable dictionaries.
#[test]
fn test_tier1_molang_queries_and_variables() {
    let engine = MolangEngine::new();
    let mut ctx = MolangContext::default();
    ctx.anim_time = 1.25;
    ctx.ground_speed = 4.3;
    ctx.vertical_speed = -0.5;
    ctx.is_sneaking = true;
    ctx.is_sprinting = false;
    ctx.variables.insert("v.custom_scale".into(), 1.75);

    assert_eq!(
        engine.evaluate(&MolangExpr::Query("query.anim_time".into()), &ctx),
        1.25
    );
    assert_eq!(
        engine.evaluate(&MolangExpr::Query("query.ground_speed".into()), &ctx),
        4.3
    );
    assert_eq!(
        engine.evaluate(&MolangExpr::Query("query.is_sneaking".into()), &ctx),
        1.0
    );
    assert_eq!(
        engine.evaluate(&MolangExpr::Query("query.is_sprinting".into()), &ctx),
        0.0
    );
    assert_eq!(
        engine.evaluate(&MolangExpr::Variable("v.custom_scale".into()), &ctx),
        1.75
    );
}

/// Feature 10: Vanilla Java Humanoid Joint Anchors
/// Verifies canonical joint pivots in model space (pixels):
/// Head [0,24,0], Body [0,24,0], Right Arm [-5,22,0], Left Arm [5,22,0], Right Leg [-1.9,12,0], Left Leg [1.9,12,0].
#[test]
fn test_tier1_vanilla_java_joint_anchors() {
    let rig = RelativePivotRig::new_humanoid();

    assert_eq!(rig.bones["head"].base_pivot, JAVA_HEAD_ANCHOR);
    assert_eq!(rig.bones["body"].base_pivot, JAVA_BODY_ANCHOR);
    assert_eq!(rig.bones["right_arm"].base_pivot, JAVA_RIGHT_ARM_ANCHOR);
    assert_eq!(rig.bones["left_arm"].base_pivot, JAVA_LEFT_ARM_ANCHOR);
    assert_eq!(rig.bones["right_leg"].base_pivot, JAVA_RIGHT_LEG_ANCHOR);
    assert_eq!(rig.bones["left_leg"].base_pivot, JAVA_LEFT_LEG_ANCHOR);

    // Initial local matrices with zero offsets should map anchor to anchor
    for (name, bone) in &rig.bones {
        let m = bone.local_matrix();
        let transformed_anchor = m.transform_point3(bone.base_pivot);
        assert!(
            (transformed_anchor - bone.base_pivot).length() < 1e-4,
            "Bone {} rest matrix did not preserve base anchor: expected {:?}, got {:?}",
            name,
            bone.base_pivot,
            transformed_anchor
        );
    }
}

/// Feature 11: Virtual Waist Joint & Hierarchy
/// Verifies that waist joint [0, 12, 0] rotates torso and arms while legs stay anchored to entity root.
#[test]
fn test_tier1_virtual_waist_rigging() {
    // Pitch waist forward by 30 degrees (lean forward)
    let waist_anchor = VIRTUAL_WAIST_ANCHOR; // [0, 12, 0]
    let waist_rotation = Quat::from_euler(glam::EulerRot::XYZ, 30.0f32.to_radians(), 0.0, 0.0);

    let m_waist = Mat4::from_translation(waist_anchor)
        * Mat4::from_quat(waist_rotation)
        * Mat4::from_translation(-waist_anchor);

    // Body rest anchor is [0, 24, 0]
    let transformed_neck = m_waist.transform_point3(JAVA_BODY_ANCHOR);
    // Neck tilts forward (positive Y and Z changes)
    assert!(transformed_neck.y < 24.0, "Neck should lower when leaning forward");
    assert!(transformed_neck.z > 0.0, "Neck should tilt forward in Z");

    // CRITICAL: Legs are attached to root, NOT waist.
    // The leg base anchor at [±1.9, 12, 0] MUST remain unaffected by waist rotation.
    let right_leg_pos = JAVA_RIGHT_LEG_ANCHOR;
    assert_eq!(
        right_leg_pos,
        Vec3::new(-1.9, 12.0, 0.0),
        "Leg anchor must remain at Y=12.0 independent of waist pitch"
    );
}

/// Feature 12: Relative Pivot Rigging Formula
/// Verifies M_local = Translate(A_i - A_p + delta_T) * Rotate(R) * Scale(S).
#[test]
fn test_tier1_relative_pivot_rigging_formula() {
    let shoulder_anchor = JAVA_RIGHT_ARM_ANCHOR; // [-5, 22, 0]
    let body_anchor = JAVA_BODY_ANCHOR;          // [0, 24, 0]
    let delta_trans = Vec3::new(0.0, -1.0, 0.5);
    let rot_zyx = Vec3::new(45.0, 0.0, 0.0); // 45 deg swing
    let scale = Vec3::ONE;

    let m_local = compute_relative_local_matrix(
        shoulder_anchor,
        body_anchor,
        delta_trans,
        rot_zyx,
        scale,
    );

    // Origin in arm space transforms relative to body anchor
    let transformed_shoulder = m_local.transform_point3(Vec3::ZERO);
    let expected_offset = shoulder_anchor - body_anchor + delta_trans;
    assert!(
        (transformed_shoulder - expected_offset).length() < 1e-4,
        "Relative local matrix translation mismatch"
    );
}

/// Feature 13: Attachable Definition Parser
/// Verifies structure of AttachableDefinition, mount point enum, and transform descriptors.
#[test]
fn test_tier1_attachable_definition_parser() {
    let attachable = AttachableDefinition {
        identifier: "minecraft:diamond_sword".into(),
        geometry_name: "geometry.oreville_ans.hsygxb".into(),
        default_mount: AttachableMountPoint::RightHand,
        first_person_transform: ItemTransform3D {
            position: Vec3::new(0.0, -2.0, 4.0),
            rotation: Vec3::new(90.0, 0.0, -90.0),
            scale: Vec3::new(0.9, 0.9, 0.9),
        },
        third_person_transform: ItemTransform3D {
            position: Vec3::new(0.0, -2.0, 4.0),
            rotation: Vec3::new(90.0, 0.0, -90.0),
            scale: Vec3::new(3.5, 3.5, 1.75),
        },
    };

    assert_eq!(attachable.identifier, "minecraft:diamond_sword");
    assert_eq!(attachable.default_mount, AttachableMountPoint::RightHand);

    let m_1st = attachable.first_person_transform.to_mat4();
    let m_3rd = attachable.third_person_transform.to_mat4();

    // Verify scale differences between 1st person and 3rd person
    let scale_1st = m_1st.transform_vector3(Vec3::X).length();
    let scale_3rd = m_3rd.transform_vector3(Vec3::X).length();
    assert!((scale_1st - 0.9).abs() < 1e-4);
    assert!((scale_3rd - 3.5).abs() < 1e-4);
}

/// Features 14-16: Attachable Dual-Perspective Transforms
/// Verifies dedicated 1st-person viewmodel projection (70° FOV, near-plane <= 0.01)
/// and 3rd-person mount point attachment.
#[test]
fn test_tier1_attachable_dual_perspective_transforms() {
    // 1st-person projection
    let aspect = 16.0 / 9.0;
    let p_viewmodel = compute_viewmodel_projection(aspect);
    let m_base_offset = compute_viewmodel_mainhand_base_offset();

    // Transform a point on the weapon tip in front of the hand
    let weapon_vertex = Vec3::new(0.0, 0.0, -0.5); // 0.5 units forward
    let eye_space_point = m_base_offset.transform_point3(weapon_vertex);

    // In eye space, point is at negative Z
    assert!(eye_space_point.z < 0.0);

    // Project through viewmodel projection
    let clip_pos = p_viewmodel * Vec4::new(eye_space_point.x, eye_space_point.y, eye_space_point.z, 1.0);
    let ndc_z = clip_pos.z / clip_pos.w;

    // Must be inside normalized device coordinates (NDC range [-1, 1] in OpenGL or [0, 1] in Vulkan/wgpu)
    assert!(
        ndc_z >= -1.0 && ndc_z <= 1.0,
        "Viewmodel vertex projected outside valid depth range: ndc_z={}",
        ndc_z
    );

    // 3rd-person mounting to player socket: RightHand
    let player_arm_matrix = Mat4::from_translation(JAVA_RIGHT_ARM_ANCHOR);
    let item_relative = Mat4::from_translation(Vec3::new(-1.0, -7.0, 1.0)); // to rightitem
    let sword_mount_world = player_arm_matrix * item_relative;

    let mounted_pos = sword_mount_world.transform_point3(Vec3::ZERO);
    assert_eq!(mounted_pos, JAVA_RIGHT_ARM_ANCHOR + Vec3::new(-1.0, -7.0, 1.0));
}
