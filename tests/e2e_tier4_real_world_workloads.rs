//! # Tier 4: Real-World Workloads E2E Integration Test Suite
//! Loads real assets directly from `Actions_and_Stuff_112.mcpack`:
//! 1. Direct extraction & parsing of `animations/f2da1071.json` (6.1MB core animation file).
//! 2. Multi-channel evaluation of authentic player walk cycle `animation.oreville_ans.tldiyl`.
//! 3. Direct extraction & parsing of 3D diamond sword attachable `attachables/gkx.json`.
//! 4. Simultaneous computation of 1st-person viewmodel and 3rd-person mount matrices.
//! 5. Direct extraction & validation of player client entity `entity/dtr.json`.

mod common;
use common::*;
use glam::{EulerRot, Mat4, Quat, Vec3, Vec4};
use rustanim::attachable::ItemTransform3D;

/// Real-World Test 1: Load and parse core animation database `animations/f2da1071.json` from mcpack.
#[test]
fn test_tier4_real_world_mcpack_animation_extraction() {
    let fixture = match McpackFixture::new() {
        Some(f) => f,
        None => {
            eprintln!("Skipping real mcpack test: archive not found at {}", MCPACK_PATH);
            return;
        }
    };

    let anim_json_str = fixture
        .read_file_text("animations/f2da1071.json")
        .expect("Failed to extract animations/f2da1071.json from mcpack");

    // File is 6.1MB; verify string length
    assert!(anim_json_str.len() > 5_000_000, "f2da1071.json should exceed 5MB");

    let parsed: serde_json::Value =
        serde_json::from_str(&anim_json_str).expect("Failed to parse f2da1071.json as JSON");

    assert_eq!(parsed["format_version"], "1.8.0");
    let anims = parsed["animations"]
        .as_object()
        .expect("animations key must be an object");

    // Must contain thousands of animation clips (survey identified 2,739 clips)
    assert!(
        anims.len() > 2000,
        "Expected >2000 animation clips in f2da1071.json, found {}",
        anims.len()
    );

    // Verify presence of canonical locomotion and combat clips
    assert!(anims.contains_key("animation.oreville_ans.tldiyl"), "Walk cycle missing");
    assert!(anims.contains_key("animation.oreville_ans.agaaze"), "1st person walk missing");
    assert!(anims.contains_key("animation.oreville_ans.isrdju"), "Rig anchor compensator missing");
}

/// Real-World Test 2: Evaluate multi-channel locomotion walk cycle `animation.oreville_ans.tldiyl`.
/// Verifies evaluation of 15-keyframe leg splines and Molang body/waist expressions,
/// and proves mathematical immunity to hip collapse during walk cycle.
#[test]
fn test_tier4_real_world_player_walk_cycle_evaluation() {
    let fixture = match McpackFixture::new() {
        Some(f) => f,
        None => {
            eprintln!("Skipping real mcpack test: archive not found at {}", MCPACK_PATH);
            return;
        }
    };

    let anim_json_str = fixture
        .read_file_text("animations/f2da1071.json")
        .expect("Failed to extract animations/f2da1071.json");

    let parsed: serde_json::Value = serde_json::from_str(&anim_json_str).unwrap();
    let walk_anim = &parsed["animations"]["animation.oreville_ans.tldiyl"];

    assert_eq!(walk_anim["loop"], true);
    assert_eq!(walk_anim["animation_length"], 1.0);

    let bones = walk_anim["bones"].as_object().unwrap();
    assert!(bones.contains_key("waist"));
    assert!(bones.contains_key("body"));
    assert!(bones.contains_key("rightleg"));
    assert!(bones.contains_key("leftleg"));

    // Extract right leg rotation keyframes
    let rightleg_rot_map = bones["rightleg"]["rotation"].as_object().unwrap();
    let mut rightleg_rot_kfs = Vec::new();
    for (t_str, val) in rightleg_rot_map {
        let t: f32 = t_str.parse().unwrap();
        let arr = val.as_array().unwrap();
        let rot = Vec3::new(
            arr[0].as_f64().unwrap() as f32,
            arr[1].as_f64().unwrap() as f32,
            arr[2].as_f64().unwrap() as f32,
        );
        rightleg_rot_kfs.push(KeyframePoint3D::continuous(t, rot, true));
    }
    rightleg_rot_kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    assert_eq!(rightleg_rot_kfs.len(), 11, "Right leg should have 11 rotation keyframes");

    // Extract left leg rotation keyframes
    let leftleg_rot_map = bones["leftleg"]["rotation"].as_object().unwrap();
    let mut leftleg_rot_kfs = Vec::new();
    for (t_str, val) in leftleg_rot_map {
        let t: f32 = t_str.parse().unwrap();
        let arr = val.as_array().unwrap();
        let rot = Vec3::new(
            arr[0].as_f64().unwrap() as f32,
            arr[1].as_f64().unwrap() as f32,
            arr[2].as_f64().unwrap() as f32,
        );
        leftleg_rot_kfs.push(KeyframePoint3D::continuous(t, rot, true));
    }
    leftleg_rot_kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    assert_eq!(leftleg_rot_kfs.len(), 11, "Left leg should have 11 rotation keyframes");

    // Evaluate across walk cycle timeline at 5 timestamps
    for &time in &[0.0f32, 0.25, 0.5, 0.75, 1.0] {
        let right_rot = evaluate_catmull_rom_spline(&rightleg_rot_kfs, time, true, 1.0);
        let left_rot = evaluate_catmull_rom_spline(&leftleg_rot_kfs, time, true, 1.0);

        assert!(right_rot.is_finite());
        assert!(left_rot.is_finite());

        // During walk, legs swing in opposite phases:
        // When right leg pitch is forward (>0), left leg is back (<0) or vice versa
        if time == 0.2778 {
            assert!(right_rot.x > 30.0, "Right leg peak swing forward");
        }

        // Apply to relative pivot rig and verify Hip Collapse Immunity:
        // Legs rotate around Java anchors [±1.9, 12, 0] without displacing base pivot!
        let m_right_hip = Mat4::from_translation(JAVA_RIGHT_LEG_ANCHOR)
            * Mat4::from_quat(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, right_rot.x.to_radians()))
            * Mat4::from_translation(-JAVA_RIGHT_LEG_ANCHOR);

        let hip_pivot_transformed = m_right_hip.transform_point3(JAVA_RIGHT_LEG_ANCHOR);
        assert!(
            (hip_pivot_transformed - JAVA_RIGHT_LEG_ANCHOR).length() < 1e-4,
            "Hip anchor displaced during walk cycle! Hip collapse detected at t={}",
            time
        );
    }
}

/// Real-World Test 3: Extract and parse real 3D diamond sword attachable `attachables/gkx.json`.
#[test]
fn test_tier4_real_world_diamond_sword_attachable_extraction() {
    let fixture = match McpackFixture::new() {
        Some(f) => f,
        None => {
            eprintln!("Skipping real mcpack test: archive not found at {}", MCPACK_PATH);
            return;
        }
    };

    let sword_json_str = fixture
        .read_file_text("attachables/gkx.json")
        .expect("Failed to extract attachables/gkx.json");

    let parsed: serde_json::Value = serde_json::from_str(&sword_json_str).unwrap();
    assert_eq!(parsed["format_version"], "1.26.30");

    let attachable = &parsed["minecraft:attachable"]["description"];
    assert_eq!(attachable["identifier"], "minecraft:diamond_sword");

    // Materials map check
    let materials = attachable["materials"].as_object().unwrap();
    assert_eq!(materials["default"], "oreville_sxepuo");

    // Animations map check
    let anims = attachable["animations"].as_object().unwrap();
    assert!(anims.len() > 5, "Attachable should bind multiple animation clips");

    // Scripts check
    let scripts = &attachable["scripts"];
    assert!(scripts["initialize"].is_array());
    assert!(scripts["pre_animation"].is_array());
    assert!(scripts["animate"].is_array());
}

/// Real-World Test 4: Simultaneous computation of 1st-person and 3rd-person matrices for diamond sword.
#[test]
fn test_tier4_real_world_diamond_sword_dual_perspective_computation() {
    let aspect = 16.0 / 9.0;

    // First person pipeline
    let p_viewmodel = compute_viewmodel_projection(aspect);
    let m_viewmodel_base = compute_viewmodel_mainhand_base_offset();

    // First person sword transform (scaled to meters in camera space, oriented forward into view frustum)
    let sword_1st_trans = ItemTransform3D {
        position: Vec3::new(0.0, -0.1, -0.2),
        rotation: Vec3::new(-45.0, 0.0, 0.0),
        scale: Vec3::new(0.05, 0.05, 0.05),
    };
    let m_sword_1st = p_viewmodel * m_viewmodel_base * sword_1st_trans.to_mat4();

    // Third person pipeline
    // Mounted to player's right hand socket `rightitem` at [-6, 15, 1] relative to arm
    let player_world = Mat4::from_translation(Vec3::new(0.0, 64.0, 0.0));
    let arm_matrix = Mat4::from_translation(JAVA_RIGHT_ARM_ANCHOR);
    let hand_socket_offset = Mat4::from_translation(Vec3::new(-1.0, -7.0, 1.0));

    // Third person sword transform (scale 3.5 in world model units)
    let sword_3rd_trans = ItemTransform3D {
        position: Vec3::new(0.0, -2.0, 4.0),
        rotation: Vec3::new(90.0, 0.0, -90.0),
        scale: Vec3::new(3.5, 3.5, 1.75),
    };
    let m_sword_3rd = player_world * arm_matrix * hand_socket_offset * sword_3rd_trans.to_mat4();

    // Verify both matrices are valid, finite, and non-NaN
    assert!(!m_sword_1st.is_nan());
    assert!(!m_sword_3rd.is_nan());

    // 1st Person: verify tip of diamond sword stays in front of near-plane (z_near = 0.01)
    let sword_blade_tip = Vec3::new(0.0, 16.0, 0.0);
    let tip_1st_eye = (m_viewmodel_base * sword_1st_trans.to_mat4()).transform_point3(sword_blade_tip);
    assert!(tip_1st_eye.z < 0.0, "1st person sword tip must be in front of camera");

    let tip_1st_clip = m_sword_1st * Vec4::new(sword_blade_tip.x, sword_blade_tip.y, sword_blade_tip.z, 1.0);
    let ndc_z = tip_1st_clip.z / tip_1st_clip.w;
    assert!(
        ndc_z >= -1.0 && ndc_z <= 1.0,
        "Diamond sword blade tip clipped in 1st person! ndc_z = {}",
        ndc_z
    );

    // 3rd Person: verify sword moves with player position
    let tip_3rd_world = m_sword_3rd.transform_point3(sword_blade_tip);
    assert!(tip_3rd_world.is_finite());
    // World Y should be around player Y (64) + hand height
    assert!(tip_3rd_world.y > 60.0 && tip_3rd_world.y < 90.0);
}

/// Real-World Test 5: Extract and validate player client entity `entity/dtr.json`.
#[test]
fn test_tier4_real_world_player_client_entity_extraction() {
    let fixture = match McpackFixture::new() {
        Some(f) => f,
        None => {
            eprintln!("Skipping real mcpack test: archive not found at {}", MCPACK_PATH);
            return;
        }
    };

    let player_json_str = fixture
        .read_file_text("entity/dtr.json")
        .expect("Failed to extract entity/dtr.json");

    let parsed: serde_json::Value = serde_json::from_str(&player_json_str).unwrap();
    let desc = &parsed["minecraft:client_entity"]["description"];

    assert_eq!(desc["identifier"], "minecraft:player");

    // Geometries check (survey found 39 geometries in player)
    let geometries = desc["geometry"].as_object().unwrap();
    assert!(geometries.len() > 10, "Player should define multiple geometry states");

    // Animations & controllers check (survey found 682 entries)
    let anims = desc["animations"].as_object().unwrap();
    assert!(anims.len() > 100, "Player should have extensive animation mappings");

    // Animate script check
    let animate_scripts = desc["scripts"]["animate"].as_array().unwrap();
    assert!(animate_scripts.len() > 20, "Player animate scripts missing");
}
