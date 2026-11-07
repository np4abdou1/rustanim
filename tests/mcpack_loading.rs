//! Integration tests for loading and evaluating Bedrock animations directly from Actions_and_Stuff_112.mcpack.

use rustanim::anim::{BedrockAnimationFile, FramePose, LoopMode};
use rustanim::mcpack::{McpackArchive, McpackError};
use std::path::PathBuf;
use glam::Vec3;

fn get_mcpack_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("MCPACK_PATH") {
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    let candidates = [
        PathBuf::from("Actions_and_Stuff_112.mcpack"),
        PathBuf::from("../Actions_and_Stuff_112.mcpack"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

#[test]
fn test_real_mcpack_manifest_and_file_count() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => {
            eprintln!("Skipping test: Actions_and_Stuff_112.mcpack not found");
            return;
        }
    };

    let mut archive = McpackArchive::open(&path).expect("Failed to open Actions_and_Stuff_112.mcpack");
    assert!(archive.has_file("manifest.json"), "manifest.json must exist in mcpack");
    assert!(!archive.is_empty(), "mcpack archive must not be empty");
    assert!(archive.len() > 19000, "mcpack should contain >19000 files, found {}", archive.len());

    let manifest_text = archive.read_to_string("manifest.json").expect("Read manifest.json");
    let manifest_json: serde_json::Value =
        serde_json::from_str(&manifest_text).expect("Valid manifest JSON");
    assert_eq!(manifest_json["format_version"], 3, "Manifest must be format_version 3");
    assert_eq!(
        manifest_json["header"]["uuid"],
        "2cf066eb-1254-4b7d-affb-80fe3216b18c",
        "Manifest UUID match"
    );

    let anim_files = archive.list_animation_files();
    assert!(
        !anim_files.is_empty(),
        "mcpack should contain animation JSON files"
    );
    assert!(
        anim_files.iter().any(|f| f.ends_with("f2da1071.json")),
        "Core f2da1071.json animation file must be present"
    );
}

#[test]
fn test_real_mcpack_single_clip_file_lwc() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => return,
    };
    let mut archive = McpackArchive::open(&path).unwrap();
    let anim_file: BedrockAnimationFile = archive
        .load_animation_file("animations/lwc.json")
        .expect("Load lwc.json");

    assert_eq!(anim_file.format_version, "1.8.0");
    assert_eq!(anim_file.animations.len(), 1);
    let clip = anim_file
        .animations
        .get("animation.oreville_ans.xghxmf")
        .expect("Clip exists");
    assert_eq!(clip.loop_mode, LoopMode::Loop);
    assert_eq!(
        clip.anim_time_update.as_deref(),
        Some("v.evlypi='jhotdc';return 0;")
    );
    assert!(clip.bones.is_empty());
}

#[test]
fn test_real_mcpack_multi_clip_file_mfl() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => return,
    };
    let mut archive = McpackArchive::open(&path).unwrap();
    let anim_file: BedrockAnimationFile = archive
        .load_animation_file("animations/mfl.json")
        .expect("Load mfl.json");

    assert_eq!(anim_file.animations.len(), 5);
    let expected_clips = [
        "animation.oreville_ans.zeosts",
        "animation.oreville_ans.lduxzk",
        "animation.oreville_ans.zvhsej",
        "animation.oreville_ans.ooxfeb",
        "animation.oreville_ans.jpowga",
    ];
    for clip_name in &expected_clips {
        assert!(
            anim_file.animations.contains_key(*clip_name),
            "Missing clip: {}",
            clip_name
        );
    }

    let zeosts = anim_file.animations.get("animation.oreville_ans.zeosts").unwrap();
    assert_eq!(zeosts.animation_length, 0.25);
    assert!(zeosts.bones.contains_key("head"));
}

#[test]
fn test_real_mcpack_f2da1071_key_clips() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => return,
    };
    let mut archive = McpackArchive::open(&path).unwrap();

    // 6.1 MB core animation file, 2,739 clips
    let anim_file: BedrockAnimationFile = archive
        .load_animation_file("animations/f2da1071.json")
        .expect("Load f2da1071.json (6.1MB)");

    assert_eq!(
        anim_file.animations.len(),
        2739,
        "Total clips in f2da1071.json must match 2,739"
    );

    // 1. Walk clip: animation.oreville_ans.tldiyl
    let walk = anim_file
        .animations
        .get("animation.oreville_ans.tldiyl")
        .expect("Walk clip");
    assert_eq!(walk.loop_mode, LoopMode::Loop);
    assert_eq!(walk.animation_length, 1.0);
    assert_eq!(walk.bones.len(), 15, "Walk clip must contain 15 bones");
    assert!(walk.bones.contains_key("waist"));
    assert!(walk.bones.contains_key("body"));
    assert!(walk.bones.contains_key("head"));
    assert!(walk.bones.contains_key("rightarm"));
    assert!(walk.bones.contains_key("leftarm"));
    assert!(walk.bones.contains_key("leftleg"));
    assert!(walk.bones.contains_key("rightleg"));

    // 2. Bob clip: animation.oreville_ans.tdrcwc
    let bob = anim_file
        .animations
        .get("animation.oreville_ans.tdrcwc")
        .expect("Bob clip");
    assert_eq!(bob.loop_mode, LoopMode::HoldOnLastFrame);

    // 3. Run clip: animation.oreville_ans.syawbu
    let run = anim_file
        .animations
        .get("animation.oreville_ans.syawbu")
        .expect("Run clip");
    assert_eq!(run.loop_mode, LoopMode::HoldOnLastFrame);
    assert_eq!(run.animation_length, 0.9583);
    assert_eq!(run.bones.len(), 18);

    // 4. Attack swing clip: animation.oreville_ans.vvnkii
    let attack = anim_file
        .animations
        .get("animation.oreville_ans.vvnkii")
        .expect("Attack clip");
    assert_eq!(attack.loop_mode, LoopMode::Once);
    assert_eq!(attack.animation_length, 1.25);
    assert_eq!(attack.bones.len(), 15);

    // 5. Sword slash clip: animation.oreville_ans.jwfwgi
    let slash = anim_file
        .animations
        .get("animation.oreville_ans.jwfwgi")
        .expect("Slash clip");
    assert_eq!(slash.loop_mode, LoopMode::Once);
    assert_eq!(slash.animation_length, 1.125);
    assert_eq!(slash.bones.len(), 4);

    // 6. Look at target (idle head): animation.oreville_ans.cndmcx
    let look = anim_file
        .animations
        .get("animation.oreville_ans.cndmcx")
        .expect("Look clip");
    assert_eq!(look.loop_mode, LoopMode::Loop);
    assert!(look.bones.contains_key("head"));

    // 7. Holding (idle stand): animation.oreville_ans.dpjhxl
    let holding = anim_file
        .animations
        .get("animation.oreville_ans.dpjhxl")
        .expect("Holding clip");
    assert_eq!(holding.loop_mode, LoopMode::HoldOnLastFrame);

    // 8. Catmull-Rom spline clip: animation.oreville_ans.uqyrzu
    let spline_clip = anim_file
        .animations
        .get("animation.oreville_ans.uqyrzu")
        .expect("Spline clip");
    assert_eq!(spline_clip.animation_length, 0.6667);
    let bone_62j1 = spline_clip.bones.get("62j1").expect("Bone 62j1 exists");
    assert!(bone_62j1.rotation.is_some(), "Bone 62j1 has rotation channel");
    let rot_channel = bone_62j1.rotation.as_ref().unwrap();
    assert_eq!(rot_channel.keyframe_count(), 5);
    assert!(rot_channel.has_catmullrom());
}

#[test]
fn test_real_clip_frame_pose_evaluation() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => return,
    };
    let mut archive = McpackArchive::open(&path).unwrap();
    let anim_file: BedrockAnimationFile = archive
        .load_animation_file("animations/f2da1071.json")
        .expect("Load f2da1071.json");

    // Test evaluating walk clip at t = 0.25 and t = 0.75
    let walk = anim_file
        .animations
        .get("animation.oreville_ans.tldiyl")
        .expect("Walk clip");

    let pose_t0: FramePose = walk.evaluate_constant(0.0);
    let pose_t025: FramePose = walk.evaluate_constant(0.25);
    let pose_t05: FramePose = walk.evaluate_constant(0.5);

    assert!(pose_t0.bones.contains_key("leftleg"));
    assert!(pose_t0.bones.contains_key("rightleg"));
    assert!(pose_t025.bones.contains_key("leftleg"));
    assert!(pose_t05.bones.contains_key("leftleg"));

    // Left and right leg must produce real numerical rotations
    let leftleg_0 = pose_t0.get_bone("leftleg");
    let leftleg_05 = pose_t05.get_bone("leftleg");
    assert_ne!(leftleg_0.rotation, leftleg_05.rotation);

    // Test evaluating Catmull-Rom spline clip uqyrzu
    let spline_clip = anim_file
        .animations
        .get("animation.oreville_ans.uqyrzu")
        .expect("Spline clip");

    let pose_spline_0 = spline_clip.evaluate_constant(0.0);
    let pose_spline_125 = spline_clip.evaluate_constant(0.125);
    let pose_spline_250 = spline_clip.evaluate_constant(0.250);
    let pose_spline_375 = spline_clip.evaluate_constant(0.375);

    let b_0 = pose_spline_0.get_bone("62j1");
    let b_125 = pose_spline_125.get_bone("62j1");
    let b_250 = pose_spline_250.get_bone("62j1");
    let b_375 = pose_spline_375.get_bone("62j1");

    // Keyframe 0.0 is [0, 0, 0]
    assert_eq!(b_0.rotation, Vec3::ZERO);

    // Keyframe 0.125 is [-1.8, 0, 52]
    assert!((b_125.rotation.x - (-1.8)).abs() < 1e-3);
    assert!((b_125.rotation.z - 52.0).abs() < 1e-3);

    // Keyframe 0.375 is [-10, 0, -5]
    assert!((b_375.rotation.x - (-10.0)).abs() < 1e-3);
    assert!((b_375.rotation.z - (-5.0)).abs() < 1e-3);

    // Midpoint 0.250 must be smoothly interpolated between 0.125 and 0.375
    assert!(b_250.rotation.x < b_125.rotation.x && b_250.rotation.x > b_375.rotation.x);
    assert!(b_250.rotation.z < b_125.rotation.z && b_250.rotation.z > b_375.rotation.z);
}

#[test]
fn test_real_mcpack_error_handling() {
    let path = match get_mcpack_path() {
        Some(p) => p,
        None => return,
    };
    let mut archive = McpackArchive::open(&path).unwrap();

    let res = archive.read_file("animations/file_does_not_exist_at_all.json");
    assert!(res.is_err(), "Non-existent path must return Err");
    match res {
        Err(McpackError::FileNotFound(name)) => {
            assert_eq!(name, "animations/file_does_not_exist_at_all.json");
        }
        other => panic!("Expected FileNotFound, got {:?}", other),
    }
}
