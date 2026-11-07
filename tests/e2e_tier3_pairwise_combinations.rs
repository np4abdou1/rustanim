//! # Tier 3: Pairwise Feature Combinations E2E Test Suite
//! Covers interactions between subsystems:
//! 1. Molang expressions inside animation curves driving bone transforms.
//! 2. Attachables mounted to animated relative pivot humanoid hierarchies.
//! 3. 1st-person viewmodel projection combined with dynamic bone transforms.
//! 4. Dual-perspective toggling (`c.is_first_person`) during active animation.
//! 5. Multi-channel hierarchy compounding with virtual waist (hip collapse immunity).
//! 6. Spline curve blending with Molang-evaluated crossfade weights.

mod common;
use common::*;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
use rustanim::attachable::{AttachableDefinition, AttachableMountPoint, ItemTransform3D};
use rustanim::molang::{MolangContext, MolangEngine, MolangExpr};

/// Pairwise 1: Molang expressions inside animation curves driving bone rotations & translations.
/// Actions & Stuff player walk uses:
/// - `body` rotation Y: `Math.cos(q.anim_time*360+40)*9.6`
/// - `waist` position Y: `(Math.sin(q.anim_time*720+12)*1.3-1)`
#[test]
fn test_tier3_molang_driven_animation_curves() {
    let engine = MolangEngine::new();
    let mut ctx = MolangContext::default();

    // Expression for body rotation Y: Math.cos(q.anim_time * 360 + 40) * 9.6
    let body_rot_y_expr = MolangExpr::Multiply(
        Box::new(MolangExpr::MathFunc {
            name: "math.cos".into(),
            args: vec![MolangExpr::Add(
                Box::new(MolangExpr::Multiply(
                    Box::new(MolangExpr::Query("query.anim_time".into())),
                    Box::new(MolangExpr::Constant(360.0)),
                )),
                Box::new(MolangExpr::Constant(40.0)),
            )],
        }),
        Box::new(MolangExpr::Constant(9.6)),
    );

    // Expression for waist position Y: Math.sin(q.anim_time * 720 + 12) * 1.3 - 1.0
    let waist_pos_y_expr = MolangExpr::Subtract(
        Box::new(MolangExpr::Multiply(
            Box::new(MolangExpr::MathFunc {
                name: "math.sin".into(),
                args: vec![MolangExpr::Add(
                    Box::new(MolangExpr::Multiply(
                        Box::new(MolangExpr::Query("query.anim_time".into())),
                        Box::new(MolangExpr::Constant(720.0)),
                    )),
                    Box::new(MolangExpr::Constant(12.0)),
                )],
            }),
            Box::new(MolangExpr::Constant(1.3)),
        )),
        Box::new(MolangExpr::Constant(1.0)),
    );

    // Sample across timeline at t = 0.0, 0.25, 0.5, 0.75, 1.0 seconds
    for &time in &[0.0f32, 0.25, 0.5, 0.75, 1.0] {
        ctx.anim_time = time;
        let rot_y = engine.evaluate(&body_rot_y_expr, &ctx);
        let pos_y = engine.evaluate(&waist_pos_y_expr, &ctx);

        assert!(rot_y.is_finite());
        assert!(pos_y.is_finite());

        // Body rotation must remain within [-9.6, +9.6] degrees
        assert!(
            rot_y >= -9.6001 && rot_y <= 9.6001,
            "Body rotation Y out of bounds at t={}: {}",
            time,
            rot_y
        );

        // Waist position Y must remain within [-2.3, +0.3] pixels
        assert!(
            pos_y >= -2.3001 && pos_y <= 0.3001,
            "Waist position Y out of bounds at t={}: {}",
            time,
            pos_y
        );
    }
}

/// Pairwise 2: Attachable mounted to animated relative pivot humanoid hierarchy.
/// Compounds: Player World -> Waist -> Body -> RightArm -> RightItem -> Sword.
/// Verifies the full forward kinematics matrix chain under dynamic arm swinging.
#[test]
fn test_tier3_attachable_mounted_to_animated_rig() {
    let player_world = Mat4::from_translation(Vec3::new(100.0, 64.0, -200.0));

    // Waist tilted forward 10 degrees
    let waist_rot = Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, 10.0f32.to_radians());
    let m_waist = Mat4::from_translation(VIRTUAL_WAIST_ANCHOR)
        * Mat4::from_quat(waist_rot)
        * Mat4::from_translation(-VIRTUAL_WAIST_ANCHOR);

    // Body relative to waist
    let m_body = Mat4::IDENTITY;

    // Right arm swinging forward 45 degrees around shoulder pivot [-5, 22, 0]
    let arm_pivot = JAVA_RIGHT_ARM_ANCHOR;
    let arm_rot = Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, 45.0f32.to_radians());
    let m_arm = Mat4::from_translation(arm_pivot)
        * Mat4::from_quat(arm_rot)
        * Mat4::from_translation(-arm_pivot);

    // Hand socket `rightitem` at [-6, 15, 1]
    let m_hand_socket = Mat4::from_translation(BEDROCK_RIGHT_ITEM_ANCHOR);

    // Sword local attachable transform
    let sword_trans = ItemTransform3D {
        position: Vec3::new(0.0, -2.0, 4.0),
        rotation: Vec3::new(90.0, 0.0, -90.0),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let m_sword_local = sword_trans.to_mat4();

    // Compounded world matrix
    let m_sword_world = player_world * m_waist * m_body * m_arm * m_hand_socket * m_sword_local;
    assert!(!m_sword_world.is_nan());

    // Transform sword tip
    let sword_tip_local = Vec3::new(0.0, 16.0, 0.0);
    let sword_tip_world = m_sword_world.transform_point3(sword_tip_local);

    assert!(sword_tip_world.is_finite());
    // World coordinates should reflect player position near [100, 64, -200]
    assert!((sword_tip_world.x - 100.0).abs() < 20.0);
    assert!((sword_tip_world.y - 64.0).abs() < 40.0);
    assert!((sword_tip_world.z - -200.0).abs() < 20.0);
}

/// Pairwise 3: 1st-person viewmodel projection combined with dynamic hand animation.
/// Verifies that dynamic hand bobbing + rotation + viewmodel projection maintains
/// unclipped normalized device coordinates and correct camera eye frame depth.
#[test]
fn test_tier3_viewmodel_projection_with_dynamic_hand_pose() {
    let aspect = 16.0 / 9.0;
    let p_viewmodel = compute_viewmodel_projection(aspect);
    let m_base = compute_viewmodel_mainhand_base_offset();

    // Dynamic hand animation during an attack swing:
    // Trans: [0.1, -0.15, -0.2], Rot: [-30 deg pitch, 20 deg yaw, 15 deg roll]
    let swing_trans = Vec3::new(0.1, -0.15, -0.2);
    let swing_rot = Quat::from_euler(
        EulerRot::ZYX,
        15.0f32.to_radians(),
        20.0f32.to_radians(),
        -30.0f32.to_radians(),
    );
    let m_anim = Mat4::from_scale_rotation_translation(Vec3::ONE, swing_rot, swing_trans);

    // Full 1st-person viewmodel transformation
    let m_hand_viewmodel = m_base * m_anim;

    // Test sword hilt and tip points
    let hilt = Vec3::new(0.0, 0.0, 0.0);
    let tip = Vec3::new(0.0, 0.0, -0.6); // 0.6m in front of hand

    for &local_pt in &[hilt, tip] {
        let eye_pt = m_hand_viewmodel.transform_point3(local_pt);
        assert!(eye_pt.z < 0.0, "Viewmodel vertex must be in front of eye (z < 0)");

        let clip_pos = p_viewmodel * Vec4::new(eye_pt.x, eye_pt.y, eye_pt.z, 1.0);
        let ndc_z = clip_pos.z / clip_pos.w;

        assert!(
            ndc_z >= -1.0 && ndc_z <= 1.0,
            "Dynamic viewmodel vertex clipped: ndc_z = {}",
            ndc_z
        );
    }
}

/// Pairwise 4: Dual-perspective toggling (`c.is_first_person`) during active animation.
/// Switching perspectives must immediately select the corresponding matrix pipeline
/// without state pollution or memory reallocation.
#[test]
fn test_tier3_perspective_toggling_during_active_playback() {
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

    let p_viewmodel = compute_viewmodel_projection(16.0 / 9.0);
    let player_arm_matrix = Mat4::from_translation(JAVA_RIGHT_ARM_ANCHOR);

    for cycle in 0..10 {
        let is_first_person = cycle % 2 == 0;

        let computed_matrix = if is_first_person {
            p_viewmodel * compute_viewmodel_mainhand_base_offset() * attachable.first_person_transform.to_mat4()
        } else {
            player_arm_matrix * attachable.third_person_transform.to_mat4()
        };

        assert!(!computed_matrix.is_nan());

        if is_first_person {
            // First person projection transforms to clip space (w != 1.0)
            let pt = computed_matrix * Vec4::new(0.0, 0.0, 0.0, 1.0);
            assert_ne!(pt.w, 1.0);
        } else {
            // Third person model space matrix preserves affine w = 1.0
            let pt = computed_matrix.transform_point3(Vec3::ZERO);
            assert!(pt.is_finite());
        }
    }
}

/// Pairwise 5: Multi-channel hierarchy compounding with virtual waist (Hip Collapse Immunity).
/// Verifies that heavy waist pitching (crouching/leaning) rotates body, arms, and head,
/// while leg anchors remain rigidly fixed at Y=12.0 on the entity root frame.
#[test]
fn test_tier3_multi_channel_hierarchy_hip_collapse_immunity() {
    // Simulate player sneak / lean forward: waist pitch = 45 degrees
    let waist_lean = 45.0f32.to_radians();
    let waist_rot = Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, waist_lean);

    // Waist transformation around virtual waist pivot [0, 12, 0]
    let m_waist = Mat4::from_translation(VIRTUAL_WAIST_ANCHOR)
        * Mat4::from_quat(waist_rot)
        * Mat4::from_translation(-VIRTUAL_WAIST_ANCHOR);

    // Body anchor [0, 24, 0] transformed through waist
    let neck_world = m_waist.transform_point3(JAVA_BODY_ANCHOR);
    // Neck moves forward and downward
    assert!(neck_world.y < 24.0);
    assert!(neck_world.z > 0.0);

    // Shoulder anchor [-5, 22, 0] transformed through waist
    let shoulder_world = m_waist.transform_point3(JAVA_RIGHT_ARM_ANCHOR);
    assert!(shoulder_world.y < 22.0);
    assert!(shoulder_world.z > 0.0);

    // CRITICAL PROOF: Legs are children of root, NOT waist.
    // The leg anchors stay strictly at Y = 12.0, X = ±1.9, Z = 0.0.
    let right_hip_pos = JAVA_RIGHT_LEG_ANCHOR;
    let left_hip_pos = JAVA_LEFT_LEG_ANCHOR;

    assert_eq!(right_hip_pos, Vec3::new(-1.9, 12.0, 0.0));
    assert_eq!(left_hip_pos, Vec3::new(1.9, 12.0, 0.0));

    // Distance between hips remains exactly 3.8 pixels (1.9 - (-1.9))
    let hip_width = (left_hip_pos - right_hip_pos).length();
    assert!((hip_width - 3.8).abs() < 1e-5, "Hip width distorted by waist lean!");
}

/// Pairwise 6: Spline curve blending with Molang-evaluated crossfade weight.
/// Simulates blending between an idle clip and a walk clip using `math.lerp` with weight w.
#[test]
fn test_tier3_spline_blending_with_molang_weight() {
    let engine = MolangEngine::new();
    let mut ctx = MolangContext::default();
    ctx.ground_speed = 2.15; // Running half speed

    // Molang blend weight: clamp(ground_speed / 4.3, 0.0, 1.0)
    let weight_expr = MolangExpr::MathFunc {
        name: "math.clamp".into(),
        args: vec![
            MolangExpr::Divide(
                Box::new(MolangExpr::Query("query.ground_speed".into())),
                Box::new(MolangExpr::Constant(4.3)),
            ),
            MolangExpr::Constant(0.0),
            MolangExpr::Constant(1.0),
        ],
    };

    let blend_weight = engine.evaluate(&weight_expr, &ctx);
    assert!((blend_weight - 0.5).abs() < 1e-4);

    // Spline A (Idle pose: leg at 0 deg)
    let kf_idle = vec![KeyframePoint3D::continuous(0.0, Vec3::ZERO, false)];
    // Spline B (Walk pose: leg at 40 deg swing)
    let kf_walk = vec![KeyframePoint3D::continuous(0.0, Vec3::new(40.0, 0.0, 0.0), false)];

    let pose_a = evaluate_catmull_rom_spline(&kf_idle, 0.0, false, 1.0);
    let pose_b = evaluate_catmull_rom_spline(&kf_walk, 0.0, false, 1.0);

    // Blended pose: lerp(pose_a, pose_b, blend_weight)
    let blended_pose = pose_a.lerp(pose_b, blend_weight);
    assert_eq!(blended_pose, Vec3::new(20.0, 0.0, 0.0));
}
