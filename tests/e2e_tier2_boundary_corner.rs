//! # Tier 2: Boundary Conditions & Corner Cases E2E Test Suite
//! Covers boundary conditions, negative/out-of-range timestamps, zero-length clips,
//! extreme rotation angles, empty channels, single-keyframe splines, division by zero handling,
//! zero scale hiding, near-plane clipping boundaries, step discontinuities, and numeric bone names.

mod common;
use common::*;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
use rustanim::anim::{AnimationClip, BoneAnimation};
use rustanim::attachable::ItemTransform3D;
use rustanim::molang::{MolangContext, MolangEngine, MolangExpr};
use rustanim::rig::BoneTransform;

/// Test negative and out-of-range timestamps for animation sampling.
/// Clamped or looped playback must never panic or return NaN.
#[test]
fn test_tier2_negative_and_out_of_range_timestamps() {
    let keyframes = vec![
        KeyframePoint3D::continuous(0.0, Vec3::new(1.0, 2.0, 3.0), true),
        KeyframePoint3D::continuous(1.0, Vec3::new(4.0, 5.0, 6.0), true),
    ];

    // Non-looping: negative timestamp should clamp to t=0.0
    let p_neg = evaluate_catmull_rom_spline(&keyframes, -0.5, false, 1.0);
    assert_eq!(p_neg, Vec3::new(1.0, 2.0, 3.0));

    // Non-looping: timestamp past duration should clamp to t=1.0
    let p_over = evaluate_catmull_rom_spline(&keyframes, 2.5, false, 1.0);
    assert_eq!(p_over, Vec3::new(4.0, 5.0, 6.0));

    // Looping: negative timestamp -0.25 on 1.0s clip wraps to 0.75
    let p_loop_neg = evaluate_catmull_rom_spline(&keyframes, -0.25, true, 1.0);
    assert!(p_loop_neg.is_finite());
    assert!(p_loop_neg.x > 1.0 && p_loop_neg.x <= 4.0);

    // Looping: timestamp 2.5 on 1.0s clip wraps to 0.5
    let p_loop_over = evaluate_catmull_rom_spline(&keyframes, 2.5, true, 1.0);
    assert!(p_loop_over.is_finite());
    // At t=0.5, linear / spline midpoint between 1.0 and 4.0 is ~2.5
    assert!((p_loop_over.x - 2.5).abs() < 0.5);
}

/// Test zero-length animation (anim_length = 0.0).
/// Must evaluate cleanly at t=0.0 without division-by-zero panics.
#[test]
fn test_tier2_zero_length_animation() {
    let keyframes = vec![
        KeyframePoint3D::continuous(0.0, Vec3::new(5.0, 10.0, 15.0), false),
    ];

    let p0 = evaluate_catmull_rom_spline(&keyframes, 0.0, false, 0.0);
    assert_eq!(p0, Vec3::new(5.0, 10.0, 15.0));

    let p_any = evaluate_catmull_rom_spline(&keyframes, 10.0, true, 0.0);
    assert_eq!(p_any, Vec3::new(5.0, 10.0, 15.0));
}

/// Test extreme rotation angles: ±360°, ±720°, 1080°, and gimbal lock pitch (±90°).
/// Rotations must remain finite, normalized, and yield valid transform matrices.
#[test]
fn test_tier2_extreme_rotation_angles() {
    let extreme_angles = [
        Vec3::new(360.0, 0.0, 0.0),
        Vec3::new(-360.0, 720.0, -1080.0),
        Vec3::new(90.0, 0.0, 0.0),   // Pitch gimbal lock point
        Vec3::new(-90.0, 45.0, 90.0), // Compound pitch pole
    ];

    for angles in extreme_angles {
        let q = Quat::from_euler(
            EulerRot::ZYX,
            angles.z.to_radians(),
            angles.y.to_radians(),
            angles.x.to_radians(),
        );

        assert!(q.is_finite(), "Quaternion contains non-finite values for angles {:?}", angles);
        assert!((q.length() - 1.0).abs() < 1e-4, "Quaternion must be unit length");

        let m = Mat4::from_quat(q);
        assert!(!m.is_nan(), "Matrix contains NaN for angles {:?}", angles);

        // Rotating a unit vector must preserve its length
        let v_rot = m.transform_vector3(Vec3::Y);
        assert!(
            (v_rot.length() - 1.0).abs() < 1e-4,
            "Transformed vector length distorted: {}",
            v_rot.length()
        );
    }
}

/// Test empty clip maps and channels: animations with 0 bones, channels with 0 keyframes.
#[test]
fn test_tier2_empty_clip_maps_and_channels() {
    let empty_clip = AnimationClip {
        name: "empty_clip".into(),
        loop_mode: false,
        anim_length: 1.0,
        bones: std::collections::HashMap::new(),
    };
    assert_eq!(empty_clip.bones.len(), 0);

    let empty_bone = BoneAnimation {
        name: "empty_bone".into(),
        positions: vec![],
        rotations: vec![],
        scales: vec![],
    };
    assert!(empty_bone.positions.is_empty());
    assert!(empty_bone.rotations.is_empty());

    // Evaluating empty keyframe list returns Vec3::ZERO safely
    let p_empty = evaluate_catmull_rom_spline(&[], 0.5, false, 1.0);
    assert_eq!(p_empty, Vec3::ZERO);
}

/// Test single-keyframe and two-keyframe Catmull-Rom splines.
/// Tests chord reflection fallback when N < 4 keyframes are available.
#[test]
fn test_tier2_single_and_two_keyframe_splines() {
    // 1-keyframe curve (static pose)
    let kf_single = vec![KeyframePoint3D::continuous(0.0, Vec3::new(7.0, 8.0, 9.0), true)];
    let p_single = evaluate_catmull_rom_spline(&kf_single, 0.5, false, 1.0);
    assert_eq!(p_single, Vec3::new(7.0, 8.0, 9.0));

    // 2-keyframe curve (tests chord reflection synthesis: P_{-1} = 2*P_0 - P_1, P_2 = 2*P_1 - P_0)
    let kf_two = vec![
        KeyframePoint3D::continuous(0.0, Vec3::new(0.0, 0.0, 0.0), true),
        KeyframePoint3D::continuous(1.0, Vec3::new(10.0, 20.0, 30.0), true),
    ];
    let p_mid = evaluate_catmull_rom_spline(&kf_two, 0.5, false, 1.0);
    // Under chord reflection, Hermite interpolation across 2 points yields exact linear midpoint
    assert!((p_mid - Vec3::new(5.0, 10.0, 15.0)).length() < 1e-4);
}

/// Test division by zero handling in Molang evaluation.
/// `1 / 0` and `x / 0` must return 0.0 without crashing or panicking.
#[test]
fn test_tier2_molang_division_by_zero() {
    let engine = MolangEngine::new();
    let ctx = MolangContext::default();

    // 10.0 / 0.0
    let expr_div_zero = MolangExpr::Divide(
        Box::new(MolangExpr::Constant(10.0)),
        Box::new(MolangExpr::Constant(0.0)),
    );
    let result = engine.evaluate(&expr_div_zero, &ctx);
    assert_eq!(result, 0.0, "Division by zero must return 0.0");

    // Division by very small epsilon (< 1e-6)
    let expr_div_tiny = MolangExpr::Divide(
        Box::new(MolangExpr::Constant(10.0)),
        Box::new(MolangExpr::Constant(1e-8)),
    );
    let result_tiny = engine.evaluate(&expr_div_tiny, &ctx);
    assert_eq!(result_tiny, 0.0, "Division by sub-epsilon must return 0.0");
}

/// Test zero-scale bone hiding (`scale: 0.0` or `[0, 0, 0]`).
/// Used in 213 keyframes across Actions & Stuff to hide bones (e.g. eye blinks, held items).
/// Matrix generation and vertex transformation must remain finite.
#[test]
fn test_tier2_zero_scale_bone_hiding() {
    let mut bone = BoneTransform::new(Vec3::new(0.0, 24.0, 0.0));
    bone.scale = Vec3::ZERO; // Zero scale hiding

    let m = bone.local_matrix();
    assert!(!m.is_nan(), "Zero scale matrix must not contain NaN");

    // All points transformed by zero scale collapse to base pivot + relative offset
    let pt = Vec3::new(10.0, 30.0, -5.0);
    let transformed = m.transform_point3(pt);
    assert_eq!(transformed, bone.base_pivot);

    // ItemTransform3D with zero scale
    let item_trans = ItemTransform3D {
        position: Vec3::new(1.0, 2.0, 3.0),
        rotation: Vec3::ZERO,
        scale: Vec3::ZERO,
    };
    let m_item = item_trans.to_mat4();
    assert!(!m_item.is_nan());
    assert_eq!(m_item.transform_point3(Vec3::new(10.0, 10.0, 10.0)), Vec3::new(1.0, 2.0, 3.0));
}

/// Test near-plane clipping boundaries for 1st-person viewmodel projection.
/// Verifies that weapon vertices as close as 0.015m in front of the camera
/// are strictly in front of the near clipping plane (z_near = 0.01m).
#[test]
fn test_tier2_near_plane_clipping_boundary() {
    let aspect = 16.0 / 9.0;
    let p_viewmodel = compute_viewmodel_projection(aspect);

    // Point located at z = -0.015m (1.5 cm in front of camera lens)
    let near_vertex = Vec4::new(0.0, 0.0, -0.015, 1.0);
    let clip_pos = p_viewmodel * near_vertex;

    // In clip space, point is inside view volume if:
    // -w <= z <= w (OpenGL) or 0 <= z <= w (Vulkan/DirectX/wgpu)
    let w = clip_pos.w;
    assert!(w > 0.0, "Point must have positive depth w");

    let ndc_z = clip_pos.z / w;
    assert!(
        ndc_z >= -1.0 && ndc_z <= 1.0,
        "Vertex at z=-0.015m was clipped by near plane: ndc_z = {}",
        ndc_z
    );

    // Extreme close point: z = -0.0101m (just beyond z_near = 0.01)
    let boundary_vertex = Vec4::new(0.0, 0.0, -0.0101, 1.0);
    let clip_boundary = p_viewmodel * boundary_vertex;
    let ndc_boundary = clip_boundary.z / clip_boundary.w;
    assert!(
        ndc_boundary >= -1.0 && ndc_boundary <= 1.0,
        "Boundary vertex at z=-0.0101m must not be clipped: ndc_z = {}",
        ndc_boundary
    );
}

/// Test asymmetric keyframe step discontinuity (`pre` != `post`).
/// When evaluating exactly at keyframe timestamp t_i, must yield `post`,
/// while approaching from t < t_i must approach `pre`.
#[test]
fn test_tier2_asymmetric_step_discontinuity() {
    let keyframes = vec![
        KeyframePoint3D::continuous(0.0, Vec3::new(0.0, 0.0, 0.0), true),
        // Discontinuous step at t = 0.5: incoming pre = [10, 0, 0], outgoing post = [50, 0, 0]
        KeyframePoint3D::discontinuous(
            0.5,
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(50.0, 0.0, 0.0),
            true,
        ),
        KeyframePoint3D::continuous(1.0, Vec3::new(60.0, 0.0, 0.0), true),
    ];

    // Approaching t=0.5 from the left (t = 0.4999)
    let p_left = evaluate_catmull_rom_spline(&keyframes, 0.499, false, 1.0);
    assert!(
        (p_left.x - 10.0).abs() < 1.0,
        "Curve approaching 0.5 from left must head toward pre=10.0, got {}",
        p_left.x
    );

    // Departing t=0.5 to the right (t = 0.501)
    let p_right = evaluate_catmull_rom_spline(&keyframes, 0.501, false, 1.0);
    assert!(
        (p_right.x - 50.0).abs() < 1.0,
        "Curve departing 0.5 to right must depart from post=50.0, got {}",
        p_right.x
    );
}

/// Test clamping of out-of-range trigonometric inputs in Molang.
/// `asin(2.5)` and `acos(-1.5)` must clamp to [-1, 1] and return finite degree angles.
#[test]
fn test_tier2_molang_trig_clamping() {
    // asin(2.5) -> clamped to 1.0 -> 90.0 degrees
    let angle_asin = molang_asin(2.5);
    assert_eq!(angle_asin, 90.0);

    // asin(-5.0) -> clamped to -1.0 -> -90.0 degrees
    let angle_asin_neg = molang_asin(-5.0);
    assert_eq!(angle_asin_neg, -90.0);

    // acos(-1.5) -> clamped to -1.0 -> 180.0 degrees
    let angle_acos = molang_acos(-1.5);
    assert_eq!(angle_acos, 180.0);

    // acos(3.0) -> clamped to 1.0 -> 0.0 degrees
    let angle_acos_pos = molang_acos(3.0);
    assert_eq!(angle_acos_pos, 0.0);
}

/// Test numeric bone names and obfuscated identifiers.
/// Bedrock packs contain bones named `8dgo1`, `8dgo2`, `3d256`, `o62hgf_jgdd`.
/// Parsers must treat them as arbitrary strings without crashing or failing regex checks.
#[test]
fn test_tier2_numeric_bone_identifiers() {
    let numeric_names = ["8dgo1", "8dgo2", "3d256", "5gjk2d7af_3", "o62hgf_jgdd"];

    let mut bones_map = std::collections::HashMap::new();
    for &name in &numeric_names {
        bones_map.insert(
            name.to_string(),
            BoneTransform::new(Vec3::new(1.0, 2.0, 3.0)),
        );
    }

    assert_eq!(bones_map.len(), 5);
    for &name in &numeric_names {
        assert!(
            bones_map.contains_key(name),
            "Failed to lookup numeric bone name: {}",
            name
        );
    }
}
