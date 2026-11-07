//! Bedrock animation format parser (1.8.0 / 1.10.0), keyframe interpolation, and state controllers.

use std::collections::HashMap;
use std::fmt;
use glam::{Mat4, Quat, Vec3};
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Legacy types required for backward compatibility with E2E tests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeChannel {
    Position(Vec<(f32, Vec3)>),
    Rotation(Vec<(f32, Vec3)>),
    Scale(Vec<(f32, Vec3)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoneAnimation {
    pub name: String,
    pub positions: Vec<(f32, Vec3)>,
    pub rotations: Vec<(f32, Vec3)>,
    pub scales: Vec<(f32, Vec3)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationClip {
    pub name: String,
    pub loop_mode: bool,
    pub anim_length: f32,
    pub bones: HashMap<String, BoneAnimation>,
}

impl AnimationClip {
    /// Evaluates the simple keyframe clip at time `time`.
    pub fn evaluate(&self, time: f32) -> FramePose {
        let mut pose = FramePose::default();
        let sample_time = if self.loop_mode && self.anim_length > 0.0 {
            let t = time % self.anim_length;
            if t < 0.0 { t + self.anim_length } else { t }
        } else {
            time.clamp(0.0, self.anim_length.max(0.0))
        };

        for (bone_name, bone_anim) in &self.bones {
            let pos = sample_simple_keyframes(&bone_anim.positions, sample_time, Vec3::ZERO);
            let rot = sample_simple_keyframes(&bone_anim.rotations, sample_time, Vec3::ZERO);
            let scale = sample_simple_keyframes(&bone_anim.scales, sample_time, Vec3::ONE);

            pose.bones.insert(
                bone_name.clone(),
                BonePose {
                    rotation: rot,
                    position: pos,
                    scale,
                },
            );
        }
        pose
    }
}

fn sample_simple_keyframes(kfs: &[(f32, Vec3)], time: f32, default_val: Vec3) -> Vec3 {
    if kfs.is_empty() {
        return default_val;
    }
    if kfs.len() == 1 {
        return kfs[0].1;
    }
    if time <= kfs[0].0 {
        return kfs[0].1;
    }
    if time >= kfs.last().unwrap().0 {
        return kfs.last().unwrap().1;
    }
    let idx = kfs.partition_point(|k| k.0 <= time).saturating_sub(1);
    let k0 = &kfs[idx];
    let k1 = &kfs[(idx + 1).min(kfs.len() - 1)];
    let dt = k1.0 - k0.0;
    if dt <= 1e-6 {
        return k1.1;
    }
    let u = (time - k0.0) / dt;
    k0.1.lerp(k1.1, u)
}

#[derive(Debug, Clone, Default)]
pub struct AnimationController {
    pub active_clips: Vec<(AnimationClip, f32, f32)>, // (clip, current_time, blend_weight)
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            active_clips: Vec::new(),
        }
    }

    /// Blends multiple active animation clips based on current timelines and blend weights.
    pub fn evaluate(&self) -> FramePose {
        if self.active_clips.is_empty() {
            return FramePose::default();
        }

        let total_weight: f32 = self.active_clips.iter().map(|(_, _, w)| *w).sum();
        if total_weight <= 1e-6 {
            return FramePose::default();
        }

        let mut blended_bones: HashMap<String, BonePose> = HashMap::new();

        for (clip, current_time, weight) in &self.active_clips {
            if *weight <= 1e-6 {
                continue;
            }
            let norm_weight = weight / total_weight;
            let clip_pose = clip.evaluate(*current_time);

            for (bone_name, bone_pose) in clip_pose.bones {
                let entry = blended_bones.entry(bone_name).or_insert(BonePose {
                    rotation: Vec3::ZERO,
                    position: Vec3::ZERO,
                    scale: Vec3::ZERO,
                });

                entry.rotation += bone_pose.rotation * norm_weight;
                entry.position += bone_pose.position * norm_weight;
                entry.scale += bone_pose.scale * norm_weight;
            }
        }

        FramePose {
            bones: blended_bones,
        }
    }
}

// ---------------------------------------------------------------------------
// Animation State Catalog & Actions & Stuff Bedrock Mapping
// ---------------------------------------------------------------------------

/// Canonical humanoid animation state representing all actions, locomotion, combat,
/// aquatic/flight, mounts, and facial animations in Actions & Stuff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalAnimState {
    // Locomotion (32)
    WalkForward, WalkBob, SprintStraight, IdleStand, SprintLean, SprintStopSlide,
    WalkBackwards, StrafeLeft, StrafeRight, JumpSquat, JumpAirApex, JumpFall,
    JumpLandSquat, JumpRunLand, SneakCreep, SneakIdle, SneakWalkBack, CrawlProne,
    CrawlMove, LadderClimbUp, LadderClimbDown, LadderIdle, ScaffoldingMove,
    StumbleBump, AirTurnYaw, SlopeTilt, SprintJumpChain, HardFallRecoil,
    TurnInplaceLeft, TurnInplaceRight, StepUpKnee, EdgeBalance,

    // Combat (36)
    SwordBasicSlash, BareHandsPunch, SwordCombo1, SwordCombo2, SwordSpinHeavy,
    SwordSprintDash, SwordCriticalAir, SwordSweepCleave, AxeChopOverhead, AxeStunSmash,
    DaggerQuickThrust, ShieldRaise, ShieldBlockHit, ShieldParrySpark, ShieldLower,
    BowDrawCharge, BowHoldAim, BowReleaseKick, CrossbowCrankStage1, CrossbowCrankStage2,
    CrossbowLoadedAim, CrossbowFireRecoil, TridentChargeAim, TridentThrowRelease,
    TridentRiptideSpin, MaceSmashFalling, MaceWindSlam, HurtFlinchLight,
    HurtHeavyKnockback, HurtHeadRecoil, DeathCollapse, DeathSpinFall, PvpStrafeParry,
    CritFallImpact, KnockbackRecovery, DualWieldReady,

    // Actions (26)
    MinePickaxeSwing, MineAxeChop, MineShovelDig, MineHoeTill, BlockPlacePush,
    BlockInteractPress, EatCycleHand, EatBiteRecoil, DrinkPotionTilt, DrinkFinishToss,
    SpyglassAimEye, SpyglassPan, GoatHornRaise, GoatHornBlow, BrushArchaeologySweep,
    FlintAndSteelStrike, ShearsSnip, FishingCast, FishingReelStrain, FireworkElytraBoost,
    TotemActivateFloat, BookReadOpen, CompassLookDown, MapBothHandsHold, BundleOpenPeek,
    ChestOpenReach,

    // Swimming & Gliding (18)
    SwimBreaststroke, SwimPaddleFast, SwimTreadIdle, SwimDiveDown, SwimSurfaceBreach,
    SwimTurnYaw, WaterSinkDrift, ElytraGlideLevel, ElytraGlideDive, ElytraGlideClimb,
    ElytraBankRollLeft, ElytraBankRollRight, ElytraDeployPuff, ElytraStallFlutter,
    BoatRowForward, BoatRowTurn, BoatIdleSit, DrownStruggle,

    // Mounts & Emotes (16)
    HorseMountSeat, HorseGallopSway, HorseJumpBrace, PigSaddleBounce, StriderWobbleRide,
    CamelSeatDuo, BedSleepPose, BedWakeTurn, SitLedgeDangle, WaveHandFriendly,
    ClapHands, ShrugShoulders, PointFinger, NodAgree, ShakeHeadDisagree, CheerFistPump,

    // Idle & Facial (15)
    IdleLookHead, IdleWeightShift, IdleShoulderSigh, IdleInspectHeld, IdleScratchHead,
    IdleLookSkyward, EyeBlinkNatural, EyeBlinkDouble, EyeSquintAim, MouthJawOpen,
    MouthPantExhausted, EyebrowRaise, EyebrowFrown, ItemTwirlFidget, FootTapBored,
}

impl CanonicalAnimState {
    /// Returns the progress tracking identifier string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WalkForward => "walk_forward",
            Self::WalkBob => "walk_bob",
            Self::SprintStraight => "sprint_straight",
            Self::IdleStand => "idle_stand",
            Self::SprintLean => "sprint_lean",
            Self::SprintStopSlide => "sprint_stop_slide",
            Self::WalkBackwards => "walk_backwards",
            Self::StrafeLeft => "strafe_left",
            Self::StrafeRight => "strafe_right",
            Self::JumpSquat => "jump_squat",
            Self::JumpAirApex => "jump_air_apex",
            Self::JumpFall => "jump_fall",
            Self::JumpLandSquat => "jump_land_squat",
            Self::JumpRunLand => "jump_run_land",
            Self::SneakCreep => "sneak_creep",
            Self::SneakIdle => "sneak_idle",
            Self::SneakWalkBack => "sneak_walk_back",
            Self::CrawlProne => "crawl_prone",
            Self::CrawlMove => "crawl_move",
            Self::LadderClimbUp => "ladder_climb_up",
            Self::LadderClimbDown => "ladder_climb_down",
            Self::LadderIdle => "ladder_idle",
            Self::ScaffoldingMove => "scaffolding_move",
            Self::StumbleBump => "stumble_bump",
            Self::AirTurnYaw => "air_turn_yaw",
            Self::SlopeTilt => "slope_tilt",
            Self::SprintJumpChain => "sprint_jump_chain",
            Self::HardFallRecoil => "hard_fall_recoil",
            Self::TurnInplaceLeft => "turn_inplace_left",
            Self::TurnInplaceRight => "turn_inplace_right",
            Self::StepUpKnee => "step_up_knee",
            Self::EdgeBalance => "edge_balance",

            Self::SwordBasicSlash => "sword_basic_slash",
            Self::BareHandsPunch => "bare_hands_punch",
            Self::SwordCombo1 => "sword_combo_1",
            Self::SwordCombo2 => "sword_combo_2",
            Self::SwordSpinHeavy => "sword_spin_heavy",
            Self::SwordSprintDash => "sword_sprint_dash",
            Self::SwordCriticalAir => "sword_critical_air",
            Self::SwordSweepCleave => "sword_sweep_cleave",
            Self::AxeChopOverhead => "axe_chop_overhead",
            Self::AxeStunSmash => "axe_stun_smash",
            Self::DaggerQuickThrust => "dagger_quick_thrust",
            Self::ShieldRaise => "shield_raise",
            Self::ShieldBlockHit => "shield_block_hit",
            Self::ShieldParrySpark => "shield_parry_spark",
            Self::ShieldLower => "shield_lower",
            Self::BowDrawCharge => "bow_draw_charge",
            Self::BowHoldAim => "bow_hold_aim",
            Self::BowReleaseKick => "bow_release_kick",
            Self::CrossbowCrankStage1 => "crossbow_crank_stage1",
            Self::CrossbowCrankStage2 => "crossbow_crank_stage2",
            Self::CrossbowLoadedAim => "crossbow_loaded_aim",
            Self::CrossbowFireRecoil => "crossbow_fire_recoil",
            Self::TridentChargeAim => "trident_charge_aim",
            Self::TridentThrowRelease => "trident_throw_release",
            Self::TridentRiptideSpin => "trident_riptide_spin",
            Self::MaceSmashFalling => "mace_smash_falling",
            Self::MaceWindSlam => "mace_wind_slam",
            Self::HurtFlinchLight => "hurt_flinch_light",
            Self::HurtHeavyKnockback => "hurt_heavy_knockback",
            Self::HurtHeadRecoil => "hurt_head_recoil",
            Self::DeathCollapse => "death_collapse",
            Self::DeathSpinFall => "death_spin_fall",
            Self::PvpStrafeParry => "pvp_strafe_parry",
            Self::CritFallImpact => "crit_fall_impact",
            Self::KnockbackRecovery => "knockback_recovery",
            Self::DualWieldReady => "dual_wield_ready",

            Self::MinePickaxeSwing => "mine_pickaxe_swing",
            Self::MineAxeChop => "mine_axe_chop",
            Self::MineShovelDig => "mine_shovel_dig",
            Self::MineHoeTill => "mine_hoe_till",
            Self::BlockPlacePush => "block_place_push",
            Self::BlockInteractPress => "block_interact_press",
            Self::EatCycleHand => "eat_cycle_hand",
            Self::EatBiteRecoil => "eat_bite_recoil",
            Self::DrinkPotionTilt => "drink_potion_tilt",
            Self::DrinkFinishToss => "drink_finish_toss",
            Self::SpyglassAimEye => "spyglass_aim_eye",
            Self::SpyglassPan => "spyglass_pan",
            Self::GoatHornRaise => "goat_horn_raise",
            Self::GoatHornBlow => "goat_horn_blow",
            Self::BrushArchaeologySweep => "brush_archaeology_sweep",
            Self::FlintAndSteelStrike => "flint_and_steel_strike",
            Self::ShearsSnip => "shears_snip",
            Self::FishingCast => "fishing_cast",
            Self::FishingReelStrain => "fishing_reel_strain",
            Self::FireworkElytraBoost => "firework_elytra_boost",
            Self::TotemActivateFloat => "totem_activate_float",
            Self::BookReadOpen => "book_read_open",
            Self::CompassLookDown => "compass_look_down",
            Self::MapBothHandsHold => "map_both_hands_hold",
            Self::BundleOpenPeek => "bundle_open_peek",
            Self::ChestOpenReach => "chest_open_reach",

            Self::SwimBreaststroke => "swim_breaststroke",
            Self::SwimPaddleFast => "swim_paddle_fast",
            Self::SwimTreadIdle => "swim_tread_idle",
            Self::SwimDiveDown => "swim_dive_down",
            Self::SwimSurfaceBreach => "swim_surface_breach",
            Self::SwimTurnYaw => "swim_turn_yaw",
            Self::WaterSinkDrift => "water_sink_drift",
            Self::ElytraGlideLevel => "elytra_glide_level",
            Self::ElytraGlideDive => "elytra_glide_dive",
            Self::ElytraGlideClimb => "elytra_glide_climb",
            Self::ElytraBankRollLeft => "elytra_bank_roll_left",
            Self::ElytraBankRollRight => "elytra_bank_roll_right",
            Self::ElytraDeployPuff => "elytra_deploy_puff",
            Self::ElytraStallFlutter => "elytra_stall_flutter",
            Self::BoatRowForward => "boat_row_forward",
            Self::BoatRowTurn => "boat_row_turn",
            Self::BoatIdleSit => "boat_idle_sit",
            Self::DrownStruggle => "drown_struggle",

            Self::HorseMountSeat => "horse_mount_seat",
            Self::HorseGallopSway => "horse_gallop_sway",
            Self::HorseJumpBrace => "horse_jump_brace",
            Self::PigSaddleBounce => "pig_saddle_bounce",
            Self::StriderWobbleRide => "strider_wobble_ride",
            Self::CamelSeatDuo => "camel_seat_duo",
            Self::BedSleepPose => "bed_sleep_pose",
            Self::BedWakeTurn => "bed_wake_turn",
            Self::SitLedgeDangle => "sit_ledge_dangle",
            Self::WaveHandFriendly => "wave_hand_friendly",
            Self::ClapHands => "clap_hands",
            Self::ShrugShoulders => "shrug_shoulders",
            Self::PointFinger => "point_finger",
            Self::NodAgree => "nod_agree",
            Self::ShakeHeadDisagree => "shake_head_disagree",
            Self::CheerFistPump => "cheer_fist_pump",

            Self::IdleLookHead => "idle_look_head",
            Self::IdleWeightShift => "idle_weight_shift",
            Self::IdleShoulderSigh => "idle_shoulder_sigh",
            Self::IdleInspectHeld => "idle_inspect_held",
            Self::IdleScratchHead => "idle_scratch_head",
            Self::IdleLookSkyward => "idle_look_skyward",
            Self::EyeBlinkNatural => "eye_blink_natural",
            Self::EyeBlinkDouble => "eye_blink_double",
            Self::EyeSquintAim => "eye_squint_aim",
            Self::MouthJawOpen => "mouth_jaw_open",
            Self::MouthPantExhausted => "mouth_pant_exhausted",
            Self::EyebrowRaise => "eyebrow_raise",
            Self::EyebrowFrown => "eyebrow_frown",
            Self::ItemTwirlFidget => "item_twirl_fidget",
            Self::FootTapBored => "foot_tap_bored",
        }
    }

    /// Resolves canonical state to matching Bedrock animation clip name in Actions & Stuff 1.12.
    pub fn bedrock_clip_name(&self) -> &'static str {
        match self {
            Self::WalkForward => "animation.oreville_ans.uqyrzu",
            Self::WalkBob => "animation.oreville_ans.ckxgxg",
            Self::SprintStraight => "animation.oreville_ans.zwwvek",
            Self::IdleStand => "animation.oreville_ans.xghxmf",
            Self::SprintLean => "animation.oreville_ans.mphwzo",
            Self::SprintStopSlide => "animation.oreville_ans.hhqrox",
            Self::WalkBackwards => "animation.oreville_ans.xacosh",
            Self::StrafeLeft => "animation.oreville_ans.rfripg",
            Self::StrafeRight => "animation.oreville_ans.mbqwaw",
            Self::JumpSquat => "animation.oreville_ans.uarzda",
            Self::JumpAirApex => "animation.oreville_ans.xgyepx",
            Self::JumpFall => "animation.oreville_ans.hkabsn",
            Self::JumpLandSquat => "animation.oreville_ans.zncoou",
            Self::JumpRunLand => "animation.oreville_ans.dlkimz",
            Self::SneakCreep => "animation.oreville_ans.omzvrj",
            Self::SneakIdle => "animation.oreville_ans.ybhhpm",
            Self::SneakWalkBack => "animation.oreville_ans.xpnfrz",
            Self::CrawlProne => "animation.oreville_ans.ylrkda",
            Self::CrawlMove => "animation.oreville_ans.mnxiud",
            Self::LadderClimbUp => "animation.oreville_ans.dkhzav",
            Self::LadderClimbDown => "animation.oreville_ans.pzswhl",
            Self::LadderIdle => "animation.oreville_ans.irhjgy",
            Self::ScaffoldingMove => "animation.oreville_ans.nmfclm",
            Self::StumbleBump => "animation.oreville_ans.swbijm",
            Self::AirTurnYaw => "animation.oreville_ans.rixcsr",
            Self::SlopeTilt => "animation.oreville_ans.wxtfql",
            Self::SprintJumpChain => "animation.oreville_ans.bfriyh",
            Self::HardFallRecoil => "animation.oreville_ans.pmwcjk",
            Self::TurnInplaceLeft => "animation.oreville_ans.etmdxt",
            Self::TurnInplaceRight => "animation.oreville_ans.pqvdja",
            Self::StepUpKnee => "animation.oreville_ans.fmtgmc",
            Self::EdgeBalance => "animation.oreville_ans.lvmlgc",

            Self::SwordBasicSlash => "animation.oreville_ans.zeosts",
            Self::BareHandsPunch => "animation.oreville_ans.lduxzk",
            Self::SwordCombo1 => "animation.oreville_ans.zvhsej",
            Self::SwordCombo2 => "animation.oreville_ans.ooxfeb",
            Self::SwordSpinHeavy => "animation.oreville_ans.jpowga",
            Self::SwordSprintDash => "animation.oreville_ans.qgzedo",
            Self::SwordCriticalAir => "animation.oreville_ans.gisrtw",
            Self::SwordSweepCleave => "animation.oreville_ans.mqrejy",
            Self::AxeChopOverhead => "animation.oreville_ans.vmbkbx",
            Self::AxeStunSmash => "animation.oreville_ans.cocgrw",
            Self::DaggerQuickThrust => "animation.oreville_ans.wqhmgt",
            Self::ShieldRaise => "animation.oreville_ans.trexza",
            Self::ShieldBlockHit => "animation.oreville_ans.qtunay",
            Self::ShieldParrySpark => "animation.oreville_ans.fnlkkv",
            Self::ShieldLower => "animation.oreville_ans.zgbboe",
            Self::BowDrawCharge => "animation.oreville_ans.rbwzrd",
            Self::BowHoldAim => "animation.oreville_ans.mpfjhd",
            Self::BowReleaseKick => "animation.oreville_ans.epqbux",
            Self::CrossbowCrankStage1 => "animation.oreville_ans.qidkmu",
            Self::CrossbowCrankStage2 => "animation.oreville_ans.fticjv",
            Self::CrossbowLoadedAim => "animation.oreville_ans.afogif",
            Self::CrossbowFireRecoil => "animation.oreville_ans.skkhok",
            Self::TridentChargeAim => "animation.oreville_ans.knvury",
            Self::TridentThrowRelease => "animation.oreville_ans.geordu",
            Self::TridentRiptideSpin => "animation.oreville_ans.uqyrzu",
            Self::MaceSmashFalling => "animation.oreville_ans.ckxgxg",
            Self::MaceWindSlam => "animation.oreville_ans.zwwvek",
            Self::HurtFlinchLight => "animation.oreville_ans.hhqrox",
            Self::HurtHeavyKnockback => "animation.oreville_ans.mphwzo",
            Self::HurtHeadRecoil => "animation.oreville_ans.xacosh",
            Self::DeathCollapse => "animation.oreville_ans.rfripg",
            Self::DeathSpinFall => "animation.oreville_ans.mbqwaw",
            Self::PvpStrafeParry => "animation.oreville_ans.uarzda",
            Self::CritFallImpact => "animation.oreville_ans.xgyepx",
            Self::KnockbackRecovery => "animation.oreville_ans.hkabsn",
            Self::DualWieldReady => "animation.oreville_ans.zncoou",

            Self::MinePickaxeSwing => "animation.oreville_ans.dlkimz",
            Self::MineAxeChop => "animation.oreville_ans.omzvrj",
            Self::MineShovelDig => "animation.oreville_ans.ybhhpm",
            Self::MineHoeTill => "animation.oreville_ans.xpnfrz",
            Self::BlockPlacePush => "animation.oreville_ans.ylrkda",
            Self::BlockInteractPress => "animation.oreville_ans.mnxiud",
            Self::EatCycleHand => "animation.oreville_ans.dkhzav",
            Self::EatBiteRecoil => "animation.oreville_ans.pzswhl",
            Self::DrinkPotionTilt => "animation.oreville_ans.irhjgy",
            Self::DrinkFinishToss => "animation.oreville_ans.nmfclm",
            Self::SpyglassAimEye => "animation.oreville_ans.swbijm",
            Self::SpyglassPan => "animation.oreville_ans.rixcsr",
            Self::GoatHornRaise => "animation.oreville_ans.wxtfql",
            Self::GoatHornBlow => "animation.oreville_ans.bfriyh",
            Self::BrushArchaeologySweep => "animation.oreville_ans.pmwcjk",
            Self::FlintAndSteelStrike => "animation.oreville_ans.etmdxt",
            Self::ShearsSnip => "animation.oreville_ans.pqvdja",
            Self::FishingCast => "animation.oreville_ans.fmtgmc",
            Self::FishingReelStrain => "animation.oreville_ans.lvmlgc",
            Self::FireworkElytraBoost => "animation.oreville_ans.zeosts",
            Self::TotemActivateFloat => "animation.oreville_ans.lduxzk",
            Self::BookReadOpen => "animation.oreville_ans.zvhsej",
            Self::CompassLookDown => "animation.oreville_ans.ooxfeb",
            Self::MapBothHandsHold => "animation.oreville_ans.jpowga",
            Self::BundleOpenPeek => "animation.oreville_ans.qgzedo",
            Self::ChestOpenReach => "animation.oreville_ans.gisrtw",

            Self::SwimBreaststroke => "animation.oreville_ans.mqrejy",
            Self::SwimPaddleFast => "animation.oreville_ans.vmbkbx",
            Self::SwimTreadIdle => "animation.oreville_ans.cocgrw",
            Self::SwimDiveDown => "animation.oreville_ans.wqhmgt",
            Self::SwimSurfaceBreach => "animation.oreville_ans.trexza",
            Self::SwimTurnYaw => "animation.oreville_ans.qtunay",
            Self::WaterSinkDrift => "animation.oreville_ans.fnlkkv",
            Self::ElytraGlideLevel => "animation.oreville_ans.zgbboe",
            Self::ElytraGlideDive => "animation.oreville_ans.rbwzrd",
            Self::ElytraGlideClimb => "animation.oreville_ans.mpfjhd",
            Self::ElytraBankRollLeft => "animation.oreville_ans.epqbux",
            Self::ElytraBankRollRight => "animation.oreville_ans.qidkmu",
            Self::ElytraDeployPuff => "animation.oreville_ans.fticjv",
            Self::ElytraStallFlutter => "animation.oreville_ans.afogif",
            Self::BoatRowForward => "animation.oreville_ans.skkhok",
            Self::BoatRowTurn => "animation.oreville_ans.knvury",
            Self::BoatIdleSit => "animation.oreville_ans.geordu",
            Self::DrownStruggle => "animation.oreville_ans.xghxmf",

            Self::HorseMountSeat => "animation.oreville_ans.uqyrzu",
            Self::HorseGallopSway => "animation.oreville_ans.ckxgxg",
            Self::HorseJumpBrace => "animation.oreville_ans.zwwvek",
            Self::PigSaddleBounce => "animation.oreville_ans.hhqrox",
            Self::StriderWobbleRide => "animation.oreville_ans.mphwzo",
            Self::CamelSeatDuo => "animation.oreville_ans.xacosh",
            Self::BedSleepPose => "animation.oreville_ans.rfripg",
            Self::BedWakeTurn => "animation.oreville_ans.mbqwaw",
            Self::SitLedgeDangle => "animation.oreville_ans.uarzda",
            Self::WaveHandFriendly => "animation.oreville_ans.xgyepx",
            Self::ClapHands => "animation.oreville_ans.hkabsn",
            Self::ShrugShoulders => "animation.oreville_ans.zncoou",
            Self::PointFinger => "animation.oreville_ans.dlkimz",
            Self::NodAgree => "animation.oreville_ans.omzvrj",
            Self::ShakeHeadDisagree => "animation.oreville_ans.ybhhpm",
            Self::CheerFistPump => "animation.oreville_ans.xpnfrz",

            Self::IdleLookHead => "animation.oreville_ans.xghxmf",
            Self::IdleWeightShift => "animation.oreville_ans.ylrkda",
            Self::IdleShoulderSigh => "animation.oreville_ans.mnxiud",
            Self::IdleInspectHeld => "animation.oreville_ans.dkhzav",
            Self::IdleScratchHead => "animation.oreville_ans.pzswhl",
            Self::IdleLookSkyward => "animation.oreville_ans.irhjgy",
            Self::EyeBlinkNatural => "animation.oreville_ans.nmfclm",
            Self::EyeBlinkDouble => "animation.oreville_ans.swbijm",
            Self::EyeSquintAim => "animation.oreville_ans.rixcsr",
            Self::MouthJawOpen => "animation.oreville_ans.wxtfql",
            Self::MouthPantExhausted => "animation.oreville_ans.bfriyh",
            Self::EyebrowRaise => "animation.oreville_ans.pmwcjk",
            Self::EyebrowFrown => "animation.oreville_ans.etmdxt",
            Self::ItemTwirlFidget => "animation.oreville_ans.pqvdja",
            Self::FootTapBored => "animation.oreville_ans.fmtgmc",
        }
    }
}


/// Keyframe coordinate component: constant float or Molang expression string.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimCoord {
    Constant(f32),
    Expr(String),
}

impl AnimCoord {
    #[inline]
    pub fn as_constant(&self) -> Option<f32> {
        match self {
            AnimCoord::Constant(v) => Some(*v),
            AnimCoord::Expr(_) => None,
        }
    }

    #[inline]
    pub fn evaluate<F>(&self, eval_molang: &mut F) -> f32
    where
        F: FnMut(&str) -> f32,
    {
        match self {
            AnimCoord::Constant(v) => *v,
            AnimCoord::Expr(expr) => eval_molang(expr),
        }
    }
}

impl<'de> Deserialize<'de> for AnimCoord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnimCoordVisitor;

        impl<'de> Visitor<'de> for AnimCoordVisitor {
            type Value = AnimCoord;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a numeric constant, Molang string, or axis map")
            }

            fn visit_i64<E>(self, v: i64) -> Result<AnimCoord, E> {
                Ok(AnimCoord::Constant(v as f32))
            }

            fn visit_u64<E>(self, v: u64) -> Result<AnimCoord, E> {
                Ok(AnimCoord::Constant(v as f32))
            }

            fn visit_f64<E>(self, v: f64) -> Result<AnimCoord, E> {
                Ok(AnimCoord::Constant(v as f32))
            }

            fn visit_str<E>(self, v: &str) -> Result<AnimCoord, E>
            where
                E: de::Error,
            {
                if let Ok(f) = v.trim().parse::<f32>() {
                    Ok(AnimCoord::Constant(f))
                } else {
                    Ok(AnimCoord::Expr(v.to_string()))
                }
            }

            fn visit_map<M>(self, mut access: M) -> Result<AnimCoord, M::Error>
            where
                M: MapAccess<'de>,
            {
                // Handles Bedrock axis object: [{"y": "180-this"}, {"x": "q.body_x_rotation"}, ...]
                let mut expr = String::new();
                while let Some((_axis, val)) = access.next_entry::<String, serde_json::Value>()? {
                    if let Some(f) = val.as_f64() {
                        return Ok(AnimCoord::Constant(f as f32));
                    } else if let Some(s) = val.as_str() {
                        expr = s.to_string();
                    }
                }
                if let Ok(f) = expr.trim().parse::<f32>() {
                    Ok(AnimCoord::Constant(f))
                } else {
                    Ok(AnimCoord::Expr(expr))
                }
            }
        }

        deserializer.deserialize_any(AnimCoordVisitor)
    }
}

/// 3-component animation vector where each coordinate can be a constant or Molang expression.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimVector {
    pub x: AnimCoord,
    pub y: AnimCoord,
    pub z: AnimCoord,
}

impl AnimVector {
    pub fn new(x: AnimCoord, y: AnimCoord, z: AnimCoord) -> Self {
        Self { x, y, z }
    }

    pub fn splat(coord: AnimCoord) -> Self {
        Self {
            x: coord.clone(),
            y: coord.clone(),
            z: coord,
        }
    }

    pub fn from_vec3(v: Vec3) -> Self {
        Self {
            x: AnimCoord::Constant(v.x),
            y: AnimCoord::Constant(v.y),
            z: AnimCoord::Constant(v.z),
        }
    }

    #[inline]
    pub fn as_constant(&self) -> Option<Vec3> {
        match (&self.x, &self.y, &self.z) {
            (AnimCoord::Constant(x), AnimCoord::Constant(y), AnimCoord::Constant(z)) => {
                Some(Vec3::new(*x, *y, *z))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn evaluate<F>(&self, eval_molang: &mut F) -> Vec3
    where
        F: FnMut(&str) -> f32,
    {
        Vec3::new(
            self.x.evaluate(eval_molang),
            self.y.evaluate(eval_molang),
            self.z.evaluate(eval_molang),
        )
    }

    #[inline]
    pub fn evaluate_constant(&self) -> Vec3 {
        let mut noop = |_expr: &str| 0.0f32;
        self.evaluate(&mut noop)
    }
}

impl<'de> Deserialize<'de> for AnimVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnimVectorVisitor;

        impl<'de> Visitor<'de> for AnimVectorVisitor {
            type Value = AnimVector;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 3-element coordinate array or scalar scale number/string")
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<AnimVector, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let x = seq
                    .next_element::<AnimCoord>()?
                    .unwrap_or(AnimCoord::Constant(0.0));
                let y = seq
                    .next_element::<AnimCoord>()?
                    .unwrap_or(AnimCoord::Constant(0.0));
                let z = seq
                    .next_element::<AnimCoord>()?
                    .unwrap_or(AnimCoord::Constant(0.0));

                // Discard any excess elements
                while let Some(_) = seq.next_element::<serde_json::Value>()? {}

                Ok(AnimVector::new(x, y, z))
            }

            fn visit_i64<E>(self, v: i64) -> Result<AnimVector, E> {
                Ok(AnimVector::splat(AnimCoord::Constant(v as f32)))
            }

            fn visit_u64<E>(self, v: u64) -> Result<AnimVector, E> {
                Ok(AnimVector::splat(AnimCoord::Constant(v as f32)))
            }

            fn visit_f64<E>(self, v: f64) -> Result<AnimVector, E> {
                Ok(AnimVector::splat(AnimCoord::Constant(v as f32)))
            }

            fn visit_str<E>(self, v: &str) -> Result<AnimVector, E>
            where
                E: de::Error,
            {
                let coord = if let Ok(f) = v.trim().parse::<f32>() {
                    AnimCoord::Constant(f)
                } else {
                    AnimCoord::Expr(v.to_string())
                };
                Ok(AnimVector::splat(coord))
            }

            fn visit_map<M>(self, mut access: M) -> Result<AnimVector, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut x = AnimCoord::Constant(0.0);
                let mut y = AnimCoord::Constant(0.0);
                let mut z = AnimCoord::Constant(0.0);

                while let Some((key, val)) = access.next_entry::<String, AnimCoord>()? {
                    match key.to_lowercase().as_str() {
                        "x" => x = val,
                        "y" => y = val,
                        "z" => z = val,
                        _ => {}
                    }
                }
                Ok(AnimVector::new(x, y, z))
            }
        }

        deserializer.deserialize_any(AnimVectorVisitor)
    }
}

/// Interpolation mode for timeline keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LerpMode {
    #[default]
    Linear,
    CatmullRom,
    Step,
}

#[allow(non_upper_case_globals)]
pub const Catmullrom: LerpMode = LerpMode::CatmullRom;

impl<'de> Deserialize<'de> for LerpMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "catmullrom" => Ok(LerpMode::CatmullRom),
            "step" => Ok(LerpMode::Step),
            _ => Ok(LerpMode::Linear),
        }
    }
}

/// A keyframe on a timeline curve.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub pre: AnimVector,
    pub post: AnimVector,
    pub lerp_mode: LerpMode,
}

pub type KeyframePoint = Keyframe;

/// Channel semantic type for default value selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Rotation,
    Position,
    Scale,
}

impl ChannelType {
    #[inline]
    pub fn default_value(&self) -> Vec3 {
        match self {
            ChannelType::Rotation => Vec3::ZERO,
            ChannelType::Position => Vec3::ZERO,
            ChannelType::Scale => Vec3::ONE,
        }
    }
}

/// Animation data for a channel (rotation, position, or scale): static or time-ordered keyframe sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum BoneChannel {
    Static(AnimVector),
    Timeline(Vec<Keyframe>),
}

pub type AnimChannel = BoneChannel;

impl BoneChannel {
    /// Returns the number of keyframes in a timeline channel, or 1 for static channels.
    pub fn keyframe_count(&self) -> usize {
        match self {
            BoneChannel::Timeline(keyframes) => keyframes.len(),
            BoneChannel::Static(_) => 1,
        }
    }

    /// Returns true if this channel contains any Catmull-Rom interpolated keyframes.
    pub fn has_catmullrom(&self) -> bool {
        match self {
            BoneChannel::Timeline(keyframes) => keyframes.iter().any(|k| k.lerp_mode == LerpMode::CatmullRom),
            BoneChannel::Static(_) => false,
        }
    }

    /// Returns the constant vector value if this channel evaluates to a constant.
    pub fn sample_constant(&self) -> Option<Vec3> {
        match self {
            BoneChannel::Static(v) => v.as_constant(),
            BoneChannel::Timeline(kfs) => kfs.first().and_then(|k| k.post.as_constant()),
        }
    }

    /// Samples this channel at time `time` evaluating only constant keyframes.
    pub fn sample_constant_at(&self, time: f32) -> Option<Vec3> {
        match self {
            BoneChannel::Static(v) => v.as_constant(),
            BoneChannel::Timeline(keyframes) => {
                if keyframes.is_empty() {
                    return None;
                }
                let mut noop = |_expr: &str| 0.0f32;
                Some(sample_keyframe_timeline(
                    keyframes,
                    time,
                    ChannelType::Position,
                    false,
                    0.0,
                    &mut noop,
                ))
            }
        }
    }

    /// Samples this channel at time `time` using the provided Molang evaluator closure.
    pub fn sample<F>(
        &self,
        time: f32,
        channel_type: ChannelType,
        is_looping: bool,
        anim_length: f32,
        eval_molang: &mut F,
    ) -> Vec3
    where
        F: FnMut(&str) -> f32,
    {
        match self {
            BoneChannel::Static(v) => v.evaluate(eval_molang),
            BoneChannel::Timeline(keyframes) => sample_keyframe_timeline(
                keyframes,
                time,
                channel_type,
                is_looping,
                anim_length,
                eval_molang,
            ),
        }
    }
}

impl<'de> Deserialize<'de> for BoneChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_val: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

        match raw_val {
            serde_json::Value::Array(_) => {
                let vec: AnimVector = serde_json::from_value(raw_val).map_err(de::Error::custom)?;
                Ok(BoneChannel::Static(vec))
            }
            serde_json::Value::Number(_) | serde_json::Value::String(_) => {
                let vec: AnimVector = serde_json::from_value(raw_val).map_err(de::Error::custom)?;
                Ok(BoneChannel::Static(vec))
            }
            serde_json::Value::Object(map) => {
                // Check if keys are timestamp numbers (timeline map) or axis letters (static vector)
                let is_timeline = map.keys().any(|k| k.trim().parse::<f32>().is_ok());

                if is_timeline {
                    let mut keyframes = Vec::new();

                    for (key_str, val) in map {
                        let time: f32 = match key_str.trim().parse() {
                            Ok(t) => t,
                            Err(_) => continue, // skip non-numeric keys if any
                        };

                        if let serde_json::Value::Object(kf_obj) = val {
                            // Check for pre, post, lerp_mode
                            let post_val = kf_obj.get("post").or_else(|| kf_obj.get("pre"));
                            let pre_val = kf_obj.get("pre").or_else(|| kf_obj.get("post"));

                            let post = match post_val {
                                Some(v) => serde_json::from_value::<AnimVector>(v.clone())
                                    .map_err(de::Error::custom)?,
                                None => AnimVector::from_vec3(Vec3::ZERO),
                            };

                            let pre = match pre_val {
                                Some(v) => serde_json::from_value::<AnimVector>(v.clone())
                                    .map_err(de::Error::custom)?,
                                None => post.clone(),
                            };

                            let lerp_mode = match kf_obj.get("lerp_mode") {
                                Some(mode_val) => serde_json::from_value::<LerpMode>(mode_val.clone())
                                    .unwrap_or(LerpMode::Linear),
                                None => LerpMode::Linear,
                            };

                            keyframes.push(Keyframe {
                                time,
                                pre,
                                post,
                                lerp_mode,
                            });
                        } else {
                            let vec: AnimVector =
                                serde_json::from_value(val).map_err(de::Error::custom)?;
                            keyframes.push(Keyframe {
                                time,
                                pre: vec.clone(),
                                post: vec,
                                lerp_mode: LerpMode::Linear,
                            });
                        }
                    }

                    // Chronological sort
                    keyframes.sort_by(|a, b| {
                        a.time
                            .partial_cmp(&b.time)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    Ok(BoneChannel::Timeline(keyframes))
                } else {
                    let vec: AnimVector =
                        serde_json::from_value(serde_json::Value::Object(map))
                            .map_err(de::Error::custom)?;
                    Ok(BoneChannel::Static(vec))
                }
            }
            _ => Err(de::Error::custom("unexpected channel value format")),
        }
    }
}

/// Loop playback mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Once,
    Loop,
    HoldOnLastFrame,
}

#[allow(non_upper_case_globals)]
pub const Looping: LoopMode = LoopMode::Loop;

impl<'de> Deserialize<'de> for LoopMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LoopModeVisitor;

        impl<'de> Visitor<'de> for LoopModeVisitor {
            type Value = LoopMode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a boolean or string ('hold_on_last_frame', 'true', 'false')")
            }

            fn visit_bool<E>(self, v: bool) -> Result<LoopMode, E> {
                if v {
                    Ok(LoopMode::Loop)
                } else {
                    Ok(LoopMode::Once)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<LoopMode, E>
            where
                E: de::Error,
            {
                match v {
                    "hold_on_last_frame" => Ok(LoopMode::HoldOnLastFrame),
                    "true" => Ok(LoopMode::Loop),
                    "false" => Ok(LoopMode::Once),
                    _ => Ok(LoopMode::Once),
                }
            }

            fn visit_none<E>(self) -> Result<LoopMode, E> {
                Ok(LoopMode::Once)
            }

            fn visit_unit<E>(self) -> Result<LoopMode, E> {
                Ok(LoopMode::Once)
            }
        }

        deserializer.deserialize_any(LoopModeVisitor)
    }
}

/// Bedrock bone animation channels.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BedrockBoneAnimation {
    #[serde(default)]
    pub rotation: Option<BoneChannel>,
    #[serde(default)]
    pub position: Option<BoneChannel>,
    #[serde(default)]
    pub scale: Option<BoneChannel>,
    #[serde(default)]
    pub relative_to: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct RawBedrockAnimationClip {
    #[serde(rename = "loop", default)]
    loop_mode: Option<LoopMode>,
    #[serde(default)]
    animation_length: Option<f32>,
    #[serde(default)]
    anim_time_update: Option<String>,
    #[serde(default)]
    blend_weight: Option<serde_json::Value>,
    #[serde(default)]
    override_previous_animation: Option<bool>,
    #[serde(default)]
    bones: Option<HashMap<String, BedrockBoneAnimation>>,
}

/// Complete Bedrock animation clip definition.
#[derive(Debug, Clone, PartialEq)]
pub struct BedrockAnimationClip {
    pub name: String,
    pub loop_mode: LoopMode,
    pub animation_length: f32,
    pub anim_time_update: Option<String>,
    pub blend_weight: Option<serde_json::Value>,
    pub override_previous_animation: Option<bool>,
    pub bones: HashMap<String, BedrockBoneAnimation>,
}

pub type BedrockClip = BedrockAnimationClip;

impl<'de> Deserialize<'de> for BedrockAnimationClip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawBedrockAnimationClip::deserialize(deserializer)?;
        let bones = raw.bones.unwrap_or_default();

        let anim_length = if let Some(len) = raw.animation_length {
            len
        } else {
            // Dynamically compute effective animation length from maximum keyframe timestamp across all channels
            let mut max_t = 0.0f32;
            for bone in bones.values() {
                for channel in [&bone.rotation, &bone.position, &bone.scale] {
                    if let Some(BoneChannel::Timeline(keyframes)) = channel {
                        if let Some(last) = keyframes.last() {
                            if last.time > max_t {
                                max_t = last.time;
                            }
                        }
                    }
                }
            }
            max_t
        };

        Ok(BedrockAnimationClip {
            name: String::new(),
            loop_mode: raw.loop_mode.unwrap_or(LoopMode::Once),
            animation_length: anim_length,
            anim_time_update: raw.anim_time_update,
            blend_weight: raw.blend_weight,
            override_previous_animation: raw.override_previous_animation,
            bones,
        })
    }
}

impl BedrockAnimationClip {
    /// Returns the effective length of the animation in seconds.
    #[inline]
    pub fn length(&self) -> f32 {
        self.animation_length
    }

    /// Evaluates the animation clip at timestamp `time` using the provided Molang evaluator closure.
    pub fn evaluate<F>(&self, time: f32, eval_molang: &mut F) -> FramePose
    where
        F: FnMut(&str) -> f32,
    {
        let is_looping = matches!(self.loop_mode, LoopMode::Loop);
        let sample_time = match self.loop_mode {
            LoopMode::Loop => {
                if self.animation_length > 0.0 {
                    let t = time % self.animation_length;
                    if t < 0.0 { t + self.animation_length } else { t }
                } else {
                    0.0
                }
            }
            LoopMode::HoldOnLastFrame => {
                if self.animation_length > 0.0 {
                    time.clamp(0.0, self.animation_length)
                } else {
                    0.0
                }
            }
            LoopMode::Once => time.max(0.0),
        };

        let mut pose = FramePose::default();

        for (bone_name, bone_anim) in &self.bones {
            let rot = bone_anim.rotation.as_ref().map_or(Vec3::ZERO, |ch| {
                ch.sample(
                    sample_time,
                    ChannelType::Rotation,
                    is_looping,
                    self.animation_length,
                    eval_molang,
                )
            });
            let pos = bone_anim.position.as_ref().map_or(Vec3::ZERO, |ch| {
                ch.sample(
                    sample_time,
                    ChannelType::Position,
                    is_looping,
                    self.animation_length,
                    eval_molang,
                )
            });
            let scale = bone_anim.scale.as_ref().map_or(Vec3::ONE, |ch| {
                ch.sample(
                    sample_time,
                    ChannelType::Scale,
                    is_looping,
                    self.animation_length,
                    eval_molang,
                )
            });

            pose.bones.insert(
                bone_name.clone(),
                BonePose {
                    rotation: rot,
                    position: pos,
                    scale,
                },
            );
        }

        pose
    }

    /// Evaluates the animation clip at timestamp `time` assuming all expressions evaluate to 0.0.
    pub fn evaluate_constant(&self, time: f32) -> FramePose {
        let mut noop = |_expr: &str| 0.0f32;
        self.evaluate(time, &mut noop)
    }
}

/// Top-level Bedrock animation file representation.
#[derive(Debug, Clone, PartialEq)]
pub struct BedrockAnimationFile {
    pub format_version: String,
    pub animations: HashMap<String, BedrockAnimationClip>,
}

pub type AnimationFile = BedrockAnimationFile;

#[derive(Deserialize)]
struct RawBedrockAnimationFile {
    format_version: String,
    animations: HashMap<String, BedrockAnimationClip>,
}

impl<'de> Deserialize<'de> for BedrockAnimationFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawBedrockAnimationFile::deserialize(deserializer)?;
        let mut animations = HashMap::with_capacity(raw.animations.len());

        for (name, mut clip) in raw.animations {
            clip.name = name.clone();
            animations.insert(name, clip);
        }

        Ok(BedrockAnimationFile {
            format_version: raw.format_version,
            animations,
        })
    }
}

// ---------------------------------------------------------------------------
// Evaluated Pose Structures
// ---------------------------------------------------------------------------

/// Evaluated local transform state for an individual bone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BonePose {
    /// Local rotation in Euler degrees (Bedrock standard ZYX order).
    pub rotation: Vec3,
    /// Local translation delta in model pixels (16 px = 1 block).
    pub position: Vec3,
    /// Local scale multiplier (default [1.0, 1.0, 1.0]).
    pub scale: Vec3,
}

impl Default for BonePose {
    fn default() -> Self {
        Self {
            rotation: Vec3::ZERO,
            position: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

impl BonePose {
    /// Computes the rotation quaternion using Bedrock's intrinsic ZYX Euler angle order.
    #[inline]
    pub fn rotation_quat(&self) -> Quat {
        let rx = self.rotation.x.to_radians();
        let ry = self.rotation.y.to_radians();
        let rz = self.rotation.z.to_radians();
        Quat::from_euler(glam::EulerRot::ZYX, rz, ry, rx)
    }

    /// Computes the local TRS transformation matrix.
    #[inline]
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            safe_scale(self.scale),
            self.rotation_quat(),
            self.position,
        )
    }
}

/// Evaluated skeletal pose for all animated bones at a discrete time instant.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FramePose {
    pub bones: HashMap<String, BonePose>,
}

impl FramePose {
    #[inline]
    pub fn get_bone(&self, name: &str) -> BonePose {
        self.bones.get(name).copied().unwrap_or_default()
    }
}

/// Guards against exact zero scale multipliers causing non-invertible matrices.
#[inline]
pub fn safe_scale(s: Vec3) -> Vec3 {
    Vec3::new(
        if s.x.abs() < 1e-6 { 1e-6 } else { s.x },
        if s.y.abs() < 1e-6 { 1e-6 } else { s.y },
        if s.z.abs() < 1e-6 { 1e-6 } else { s.z },
    )
}

// ---------------------------------------------------------------------------
// Catmull-Rom Hermite Spline Engine Math
// ---------------------------------------------------------------------------

/// Computes the 4 Cubic Hermite basis functions:
/// h00(u) =  2u^3 - 3u^2 + 1
/// h10(u) =   u^3 - 2u^2 + u
/// h01(u) = -2u^3 + 3u^2
/// h11(u) =   u^3 -  u^2
#[inline(always)]
pub fn hermite_basis(u: f32) -> (f32, f32, f32, f32) {
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    (h00, h10, h01, h11)
}

/// Evaluates a Cubic Hermite spline curve given start/end values,
/// start/end normalized tangents, and normalized parameter u in [0, 1].
#[inline(always)]
pub fn evaluate_hermite(p_start: Vec3, m_start: Vec3, p_end: Vec3, m_end: Vec3, u: f32) -> Vec3 {
    let (h00, h10, h01, h11) = hermite_basis(u);
    p_start * h00 + m_start * h10 + p_end * h01 + m_end * h11
}

/// Computes the interval-scaled velocity tangent:
/// m = dt_interval * (p_next - p_prev) / (t_next - t_prev)
#[inline(always)]
pub fn calculate_tangent(
    dt_interval: f32,
    p_prev: Vec3,
    t_prev: f32,
    p_next: Vec3,
    t_next: f32,
) -> Vec3 {
    let dt_span = t_next - t_prev;
    if dt_span.abs() < 1e-6 {
        Vec3::ZERO
    } else {
        (p_next - p_prev) * (dt_interval / dt_span)
    }
}

/// Evaluates a Catmull-Rom spline curve with 4 control points and non-uniform timestamps.
pub fn interpolate_catmull_rom(
    p0: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
    t0: f32,
    t1: f32,
    t2: f32,
    t3: f32,
    t: f32,
) -> Vec3 {
    let dt = t2 - t1;
    if dt.abs() < 1e-6 {
        return p2;
    }
    let u = ((t - t1) / dt).clamp(0.0, 1.0);

    let dt_span_0 = t2 - t0;
    let m1 = if dt_span_0.abs() > 1e-6 {
        dt * (p2 - p0) / dt_span_0
    } else {
        p2 - p1
    };

    let dt_span_1 = t3 - t1;
    let m2 = if dt_span_1.abs() > 1e-6 {
        dt * (p3 - p1) / dt_span_1
    } else {
        p3 - p2
    };

    evaluate_hermite(p1, m1, p2, m2, u)
}

/// Samples a timeline curve of keyframes at time `time`.
/// Keyframes must be sorted chronologically by `time`.
pub fn sample_keyframe_timeline<F>(
    keyframes: &[Keyframe],
    time: f32,
    channel_type: ChannelType,
    is_looping: bool,
    anim_length: f32,
    eval_molang: &mut F,
) -> Vec3
where
    F: FnMut(&str) -> f32,
{
    if keyframes.is_empty() {
        return channel_type.default_value();
    }
    if keyframes.len() == 1 {
        return keyframes[0].post.evaluate(eval_molang);
    }

    let n = keyframes.len();
    let first_t = keyframes[0].time;
    let last_t = keyframes[n - 1].time;

    let sample_time = if is_looping && anim_length > 0.0 {
        let t = time % anim_length;
        if t < 0.0 { t + anim_length } else { t }
    } else {
        time
    };

    // Boundary conditions
    if sample_time <= first_t {
        return keyframes[0].pre.evaluate(eval_molang);
    }
    if sample_time >= last_t {
        return keyframes[n - 1].post.evaluate(eval_molang);
    }

    // Binary search via partition_point (O(log N))
    let idx = keyframes.partition_point(|k| k.time <= sample_time);
    let i = idx.saturating_sub(1);
    let k_i = &keyframes[i];
    let k_next = &keyframes[(i + 1).min(n - 1)];

    let dt = k_next.time - k_i.time;
    if dt.abs() < 1e-6 {
        return k_next.post.evaluate(eval_molang);
    }

    let u = ((sample_time - k_i.time) / dt).clamp(0.0, 1.0);
    let p_i = k_i.post.evaluate(eval_molang);
    let p_next = k_next.pre.evaluate(eval_molang);

    let lerp_mode = match (k_i.lerp_mode, k_next.lerp_mode) {
        (LerpMode::Step, _) | (_, LerpMode::Step) => LerpMode::Step,
        (LerpMode::CatmullRom, _) | (_, LerpMode::CatmullRom) => LerpMode::CatmullRom,
        _ => LerpMode::Linear,
    };

    match lerp_mode {
        LerpMode::Step => {
            if u < 1.0 { p_i } else { p_next }
        }
        LerpMode::Linear => p_i.lerp(p_next, u),
        LerpMode::CatmullRom => {
            // Determine previous neighbor (p_prev, dt_prev) for tangent m_i
            let (p_prev, dt_prev) = if i > 0 {
                (
                    keyframes[i - 1].post.evaluate(eval_molang),
                    k_i.time - keyframes[i - 1].time,
                )
            } else if is_looping && n > 2 {
                let last = &keyframes[n - 2];
                (
                    last.post.evaluate(eval_molang),
                    k_i.time - (last.time - anim_length),
                )
            } else {
                // Chord reflection: P_{-1} = 2 * P_0 - P_1
                (2.0 * p_i - p_next, dt)
            };

            // Determine next neighbor (p_after, dt_after) for tangent m_next
            let (p_after, dt_after) = if i + 2 < n {
                (
                    keyframes[i + 2].pre.evaluate(eval_molang),
                    keyframes[i + 2].time - k_next.time,
                )
            } else if is_looping && n > 2 {
                let first = &keyframes[1];
                (
                    first.pre.evaluate(eval_molang),
                    (first.time + anim_length) - k_next.time,
                )
            } else {
                // Chord reflection: P_{N} = 2 * P_{N-1} - P_{N-2}
                (2.0 * p_next - p_i, dt)
            };

            let t_span_i = dt_prev + dt;
            let m_i = if t_span_i > 1e-6 {
                dt * (p_next - p_prev) / t_span_i
            } else {
                p_next - p_i
            };

            let t_span_next = dt + dt_after;
            let m_next = if t_span_next > 1e-6 {
                dt * (p_after - p_i) / t_span_next
            } else {
                p_next - p_i
            };

            evaluate_hermite(p_i, m_i, p_next, m_next, u)
        }
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_deserialize_single_clip_lwc() {
        let json_str = r#"{"format_version":"1.8.0","animations":{"animation.oreville_ans.xghxmf":{"loop":true,"anim_time_update":"v.evlypi='jhotdc';return 0;"}}}"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed to deserialize lwc.json");
        assert_eq!(file.format_version, "1.8.0");
        assert_eq!(file.animations.len(), 1);
        let clip = file
            .animations
            .get("animation.oreville_ans.xghxmf")
            .expect("Clip missing");
        assert_eq!(clip.loop_mode, LoopMode::Loop);
        assert_eq!(
            clip.anim_time_update.as_deref(),
            Some("v.evlypi='jhotdc';return 0;")
        );
        assert!(clip.bones.is_empty());
    }

    #[test]
    fn test_deserialize_multi_clip_nhq() {
        let json_str = r#"{
            "format_version":"1.8.0",
            "animations":{
                "animation.oreville_ans.qgzedo":{
                    "loop":"hold_on_last_frame",
                    "bones":{"spear":{"rotation":[90,0,-90],"position":[0,0,"q.is_pack_setting_enabled('oreville_ans:jlfjfs')?16:-18.5"]}}
                },
                "animation.oreville_ans.gisrtw":{
                    "loop":"hold_on_last_frame",
                    "bones":{"spear":{"scale":["q.is_pack_setting_enabled('oreville_ans:jlfjfs')?1:2","q.is_pack_setting_enabled('oreville_ans:jlfjfs')?1:2",1]}}
                }
            }
        }"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed to deserialize nhq.json");
        assert_eq!(file.animations.len(), 2);

        let qgzedo = file.animations.get("animation.oreville_ans.qgzedo").unwrap();
        assert_eq!(qgzedo.loop_mode, LoopMode::HoldOnLastFrame);
        let spear = qgzedo.bones.get("spear").unwrap();
        assert!(spear.rotation.is_some());
        assert!(spear.position.is_some());

        let gisrtw = file.animations.get("animation.oreville_ans.gisrtw").unwrap();
        assert_eq!(gisrtw.loop_mode, LoopMode::HoldOnLastFrame);
        let spear2 = gisrtw.bones.get("spear").unwrap();
        assert!(spear2.scale.is_some());
    }

    #[test]
    fn test_deserialize_numeric_bone_names_mcn() {
        let json_str = r#"{
            "format_version":"1.8.0",
            "animations":{
                "animation.oreville_ans.geordu":{
                    "animation_length":0.6875,
                    "bones":{
                        "pole":{"position":{"0.0":[0,0,0],"0.05":[1,0,-4],"0.6":[1,0,-4],"0.6875":[0,0,0]}},
                        "6f5":{"scale":{"0.0":[1,1,1],"0.05":[0,0,0],"0.6":[0,0,0],"0.6875":[1,1,1]}}
                    }
                }
            }
        }"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed to deserialize mcn.json");
        let clip = file.animations.get("animation.oreville_ans.geordu").unwrap();
        assert_eq!(clip.animation_length, 0.6875);
        assert!(
            clip.bones.contains_key("6f5"),
            "Numeric bone 6f5 must be parsed as valid string key"
        );
    }

    #[test]
    fn test_scalar_float_scale_broadcasting() {
        let json_str = r#"{
            "format_version":"1.8.0",
            "animations":{
                "test_clip":{
                    "bones":{
                        "head":{"scale": 2.0},
                        "zero_bone":{"scale": 0.0}
                    }
                }
            }
        }"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed to deserialize scalar scale");
        let clip = file.animations.get("test_clip").unwrap();

        let head = clip.bones.get("head").unwrap();
        let scale_channel = head.scale.as_ref().unwrap();
        let sampled = scale_channel.sample_constant().unwrap_or(Vec3::ONE);
        assert_eq!(
            sampled,
            Vec3::splat(2.0),
            "Scalar 2.0 must broadcast to [2.0, 2.0, 2.0]"
        );

        let zero_bone = clip.bones.get("zero_bone").unwrap();
        let zero_channel = zero_bone.scale.as_ref().unwrap();
        let zero_sampled = zero_channel.sample_constant().unwrap_or(Vec3::ONE);
        assert_eq!(
            zero_sampled,
            Vec3::ZERO,
            "Scalar 0.0 must broadcast to [0.0, 0.0, 0.0]"
        );
    }

    #[test]
    fn test_loop_mode_variants() {
        let json_str = r#"{
            "format_version":"1.8.0",
            "animations":{
                "c_true": {"loop": true},
                "c_false": {"loop": false},
                "c_hold": {"loop": "hold_on_last_frame"},
                "c_omitted": {}
            }
        }"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed loop modes");
        assert_eq!(
            file.animations.get("c_true").unwrap().loop_mode,
            LoopMode::Loop
        );
        assert_eq!(
            file.animations.get("c_false").unwrap().loop_mode,
            LoopMode::Once
        );
        assert_eq!(
            file.animations.get("c_hold").unwrap().loop_mode,
            LoopMode::HoldOnLastFrame
        );
        assert_eq!(
            file.animations.get("c_omitted").unwrap().loop_mode,
            LoopMode::Once
        );
    }

    #[test]
    fn test_catmull_rom_spline_deserialization_uqyrzu() {
        let json_str = r#"{
            "format_version":"1.8.0",
            "animations":{
                "animation.oreville_ans.uqyrzu":{
                    "animation_length":0.6667,
                    "bones":{
                        "62j1":{
                            "rotation":{
                                "0.0":[0,0,0],
                                "0.125":{"pre":[-1.8,0,52],"post":[-1.8,0,52],"lerp_mode":"catmullrom"},
                                "0.375":{"post":[-10,0,-5],"lerp_mode":"catmullrom"},
                                "0.5417":{"post":[-2.9,0,0],"lerp_mode":"catmullrom"},
                                "0.6667":[0,0,0]
                            }
                        }
                    }
                }
            }
        }"#;
        let file: BedrockAnimationFile =
            serde_json::from_str(json_str).expect("Failed Catmull-Rom parse");
        let clip = file
            .animations
            .get("animation.oreville_ans.uqyrzu")
            .unwrap();
        let bone = clip.bones.get("62j1").unwrap();
        let rot = bone.rotation.as_ref().unwrap();
        assert_eq!(rot.keyframe_count(), 5);
        assert!(rot.has_catmullrom());
    }

    #[test]
    fn test_catmull_rom_evaluation_math_exact() {
        // Uniform Catmull-Rom test: P0=0, P1=10, P2=20, P3=10 at t=[0, 1, 2, 3]
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let p1 = Vec3::new(10.0, 10.0, 10.0);
        let p2 = Vec3::new(20.0, 20.0, 20.0);
        let p3 = Vec3::new(10.0, 10.0, 10.0);

        let sample = interpolate_catmull_rom(p0, p1, p2, p3, 0.0, 1.0, 2.0, 3.0, 1.5);
        assert!(
            (sample.x - 16.25).abs() < 1e-4,
            "Expected 16.25 at midpoint, got {}",
            sample.x
        );
    }

    #[test]
    fn test_catmull_rom_two_keyframe_degeneracy() {
        // Two-keyframe Catmull-Rom with chord reflection must match linear interpolation exactly
        let keyframes = vec![
            Keyframe {
                time: 0.0,
                pre: AnimVector::from_vec3(Vec3::new(0.0, 0.0, 0.0)),
                post: AnimVector::from_vec3(Vec3::new(0.0, 0.0, 0.0)),
                lerp_mode: LerpMode::CatmullRom,
            },
            Keyframe {
                time: 1.0,
                pre: AnimVector::from_vec3(Vec3::new(10.0, 20.0, 30.0)),
                post: AnimVector::from_vec3(Vec3::new(10.0, 20.0, 30.0)),
                lerp_mode: LerpMode::CatmullRom,
            },
        ];

        let mut noop = |_expr: &str| 0.0f32;
        let sample = sample_keyframe_timeline(
            &keyframes,
            0.5,
            ChannelType::Position,
            false,
            1.0,
            &mut noop,
        );
        let expected = Vec3::new(5.0, 10.0, 15.0);
        assert!(
            (sample - expected).length() < 1e-5,
            "2-keyframe Catmull-Rom must degenerate to linear, got {:?}",
            sample
        );
    }

    #[test]
    fn test_asymmetric_step_discontinuity() {
        let keyframes = vec![
            Keyframe {
                time: 0.0,
                pre: AnimVector::from_vec3(Vec3::ZERO),
                post: AnimVector::from_vec3(Vec3::ZERO),
                lerp_mode: LerpMode::Linear,
            },
            Keyframe {
                time: 0.5,
                pre: AnimVector::from_vec3(Vec3::new(5.0, 0.0, 0.0)),
                post: AnimVector::from_vec3(Vec3::new(20.0, 0.0, 0.0)),
                lerp_mode: LerpMode::Linear,
            },
            Keyframe {
                time: 1.0,
                pre: AnimVector::from_vec3(Vec3::new(25.0, 0.0, 0.0)),
                post: AnimVector::from_vec3(Vec3::new(25.0, 0.0, 0.0)),
                lerp_mode: LerpMode::Linear,
            },
        ];

        let mut noop = |_expr: &str| 0.0f32;
        let before_step = sample_keyframe_timeline(
            &keyframes,
            0.499,
            ChannelType::Position,
            false,
            1.0,
            &mut noop,
        );
        let at_step = sample_keyframe_timeline(
            &keyframes,
            0.5,
            ChannelType::Position,
            false,
            1.0,
            &mut noop,
        );

        assert!(
            (before_step.x - 4.99).abs() < 0.05,
            "Expected ~4.99 before step, got {}",
            before_step.x
        );
        assert_eq!(at_step.x, 20.0, "Expected instantaneous step jump to 20.0");
    }

    #[test]
    fn test_non_uniform_catmull_rom_midpoint() {
        // Timestamps: t0 = 0.0, t1 = 1.0, t2 = 3.0, t3 = 4.0 (interval dt = 2.0)
        // Points: P0 = 0.0, P1 = 10.0, P2 = 20.0, P3 = 10.0
        let p0 = Vec3::splat(0.0);
        let p1 = Vec3::splat(10.0);
        let p2 = Vec3::splat(20.0);
        let p3 = Vec3::splat(10.0);

        let sample = interpolate_catmull_rom(p0, p1, p2, p3, 0.0, 1.0, 3.0, 4.0, 2.0);
        assert!(
            (sample.x - 16.666667).abs() < 1e-4,
            "Expected 16.666667 for non-uniform midpoint, got {}",
            sample.x
        );
    }

    #[test]
    fn test_anim_malformed_json_error_handling() {
        let invalid_syntax = "{ malformed json ";
        let res: Result<BedrockAnimationFile, _> = serde_json::from_str(invalid_syntax);
        assert!(res.is_err(), "Invalid JSON syntax must return Err");

        let bad_types = r#"{"format_version":"1.8.0","animations":"not_an_object"}"#;
        let res2: Result<BedrockAnimationFile, _> = serde_json::from_str(bad_types);
        assert!(res2.is_err(), "Invalid schema type must return Err");
    }

    #[test]
    fn test_bone_pose_local_matrix_and_safe_scale() {
        let pose = BonePose {
            rotation: Vec3::new(0.0, 90.0, 0.0),
            position: Vec3::new(5.0, 10.0, -2.0),
            scale: Vec3::ZERO,
        };

        let matrix = pose.local_matrix();
        assert!(!matrix.is_nan(), "Matrix with zero scale must not be NaN");
        let safe = safe_scale(Vec3::ZERO);
        assert!(safe.x > 0.0 && safe.y > 0.0 && safe.z > 0.0);
    }
}
