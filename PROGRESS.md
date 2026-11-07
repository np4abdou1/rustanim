# rustanim Implementation & Parity Progress

> Actions & Stuff (Bedrock) to Native Rust Port for Horizon

**Animations Progress:** 143 / 143 (100.0%)  
**3D Attachables Progress:** 115 / 115 (100.0%)  
**Bedrock Parity Progress:** 67 / 67 (100.0%)  

---

## 1. Actions & Stuff Animation Clips

### Locomotion (32/32 - 100.0%)

- [x] `walk_forward`
- [x] `walk_bob`
- [x] `sprint_straight`
- [x] `idle_stand`
- [x] `sprint_lean`
- [x] `sprint_stop_slide`
- [x] `walk_backwards`
- [x] `strafe_left`
- [x] `strafe_right`
- [x] `jump_squat`
- [x] `jump_air_apex`
- [x] `jump_fall`
- [x] `jump_land_squat`
- [x] `jump_run_land`
- [x] `sneak_creep`
- [x] `sneak_idle`
- [x] `sneak_walk_back`
- [x] `crawl_prone`
- [x] `crawl_move`
- [x] `ladder_climb_up`
- [x] `ladder_climb_down`
- [x] `ladder_idle`
- [x] `scaffolding_move`
- [x] `stumble_bump`
- [x] `air_turn_yaw`
- [x] `slope_tilt`
- [x] `sprint_jump_chain`
- [x] `hard_fall_recoil`
- [x] `turn_inplace_left`
- [x] `turn_inplace_right`
- [x] `step_up_knee`
- [x] `edge_balance`

### Combat (36/36 - 100.0%)

- [x] `sword_basic_slash`
- [x] `bare_hands_punch`
- [x] `sword_combo_1`
- [x] `sword_combo_2`
- [x] `sword_spin_heavy`
- [x] `sword_sprint_dash`
- [x] `sword_critical_air`
- [x] `sword_sweep_cleave`
- [x] `axe_chop_overhead`
- [x] `axe_stun_smash`
- [x] `dagger_quick_thrust`
- [x] `shield_raise`
- [x] `shield_block_hit`
- [x] `shield_parry_spark`
- [x] `shield_lower`
- [x] `bow_draw_charge`
- [x] `bow_hold_aim`
- [x] `bow_release_kick`
- [x] `crossbow_crank_stage1`
- [x] `crossbow_crank_stage2`
- [x] `crossbow_loaded_aim`
- [x] `crossbow_fire_recoil`
- [x] `trident_charge_aim`
- [x] `trident_throw_release`
- [x] `trident_riptide_spin`
- [x] `mace_smash_falling`
- [x] `mace_wind_slam`
- [x] `hurt_flinch_light`
- [x] `hurt_heavy_knockback`
- [x] `hurt_head_recoil`
- [x] `death_collapse`
- [x] `death_spin_fall`
- [x] `pvp_strafe_parry`
- [x] `crit_fall_impact`
- [x] `knockback_recovery`
- [x] `dual_wield_ready`

### Actions (26/26 - 100.0%)

- [x] `mine_pickaxe_swing`
- [x] `mine_axe_chop`
- [x] `mine_shovel_dig`
- [x] `mine_hoe_till`
- [x] `block_place_push`
- [x] `block_interact_press`
- [x] `eat_cycle_hand`
- [x] `eat_bite_recoil`
- [x] `drink_potion_tilt`
- [x] `drink_finish_toss`
- [x] `spyglass_aim_eye`
- [x] `spyglass_pan`
- [x] `goat_horn_raise`
- [x] `goat_horn_blow`
- [x] `brush_archaeology_sweep`
- [x] `flint_and_steel_strike`
- [x] `shears_snip`
- [x] `fishing_cast`
- [x] `fishing_reel_strain`
- [x] `firework_elytra_boost`
- [x] `totem_activate_float`
- [x] `book_read_open`
- [x] `compass_look_down`
- [x] `map_both_hands_hold`
- [x] `bundle_open_peek`
- [x] `chest_open_reach`

### Aquatic & Flight (18/18 - 100.0%)

- [x] `swim_breaststroke`
- [x] `swim_paddle_fast`
- [x] `swim_tread_idle`
- [x] `swim_dive_down`
- [x] `swim_surface_breach`
- [x] `swim_turn_yaw`
- [x] `water_sink_drift`
- [x] `elytra_glide_level`
- [x] `elytra_glide_dive`
- [x] `elytra_glide_climb`
- [x] `elytra_bank_roll_left`
- [x] `elytra_bank_roll_right`
- [x] `elytra_deploy_puff`
- [x] `elytra_stall_flutter`
- [x] `boat_row_forward`
- [x] `boat_row_turn`
- [x] `boat_idle_sit`
- [x] `drown_struggle`

### Mounts & Emotes (16/16 - 100.0%)

- [x] `horse_mount_seat`
- [x] `horse_gallop_sway`
- [x] `horse_jump_brace`
- [x] `pig_saddle_bounce`
- [x] `strider_wobble_ride`
- [x] `camel_seat_duo`
- [x] `bed_sleep_pose`
- [x] `bed_wake_turn`
- [x] `sit_ledge_dangle`
- [x] `wave_hand_friendly`
- [x] `clap_hands`
- [x] `shrug_shoulders`
- [x] `point_finger`
- [x] `nod_agree`
- [x] `shake_head_disagree`
- [x] `cheer_fist_pump`

### Idle & Facial (15/15 - 100.0%)

- [x] `idle_look_head`
- [x] `idle_weight_shift`
- [x] `idle_shoulder_sigh`
- [x] `idle_inspect_held`
- [x] `idle_scratch_head`
- [x] `idle_look_skyward`
- [x] `eye_blink_natural`
- [x] `eye_blink_double`
- [x] `eye_squint_aim`
- [x] `mouth_jaw_open`
- [x] `mouth_pant_exhausted`
- [x] `eyebrow_raise`
- [x] `eyebrow_frown`
- [x] `item_twirl_fidget`
- [x] `foot_tap_bored`

---

## 2. 3D Attachables & Mount Points

### 3D Weapons (38/38 - 100.0%)

- [x] `diamond_sword_3d`
- [x] `iron_sword_3d`
- [x] `netherite_sword_3d`
- [x] `golden_sword_3d`
- [x] `stone_sword_3d`
- [x] `wooden_sword_3d`
- [x] `1st_person_sword_offset_fix`
- [x] `1st_person_sword_swing_arc`
- [x] `1st_person_shield_fov_fix`
- [x] `sheathed_weapon_back_mount`
- [x] `sheathed_weapon_hip_mount`
- [x] `mace_heavy_3d`
- [x] `trident_item_3d`
- [x] `bow_unpulled_3d`
- [x] `bow_pulling_0_3d`
- [x] `bow_pulling_1_3d`
- [x] `bow_pulling_2_3d`
- [x] `crossbow_standby_3d`
- [x] `crossbow_pulling_0_3d`
- [x] `crossbow_pulling_1_3d`
- [x] `crossbow_pulling_2_3d`
- [x] `crossbow_arrow_loaded_3d`
- [x] `crossbow_firework_loaded_3d`
- [x] `custom_dagger_sheath`
- [x] `custom_greatsword_back`
- [x] `custom_scythe_3d`
- [x] `custom_rapier_3d`
- [x] `custom_katana_hip`
- [x] `custom_battleaxe_double`
- [x] `custom_spear_shaft`
- [x] `custom_warhammer_stone`
- [x] `custom_halberd_steel`
- [x] `enchant_glow_sword_overlay`
- [x] `sharpness_spark_emitter`
- [x] `fire_aspect_flame_bones`
- [x] `sweeping_edge_blade_trail`
- [x] `1st_person_bow_aim_view`
- [x] `offhand_sword_parry_pos`

### 3D Tools (29/29 - 100.0%)

- [x] `diamond_pickaxe_3d`
- [x] `netherite_pickaxe_3d`
- [x] `iron_pickaxe_3d`
- [x] `golden_pickaxe_3d`
- [x] `stone_pickaxe_3d`
- [x] `wooden_pickaxe_3d`
- [x] `netherite_axe_3d`
- [x] `diamond_axe_3d`
- [x] `iron_axe_3d`
- [x] `golden_axe_3d`
- [x] `stone_axe_3d`
- [x] `wooden_axe_3d`
- [x] `netherite_shovel_3d`
- [x] `diamond_shovel_3d`
- [x] `iron_shovel_3d`
- [x] `stone_shovel_3d`
- [x] `diamond_hoe_3d`
- [x] `iron_hoe_3d`
- [x] `shears_jointed_3d`
- [x] `fishing_rod_cast_3d`
- [x] `fishing_rod_hook_3d`
- [x] `flint_and_steel_3d`
- [x] `clock_dynamic_face_3d`
- [x] `compass_needle_pivot_3d`
- [x] `recovery_compass_3d`
- [x] `brush_archaeology_bristle_3d`
- [x] `spyglass_brass_body_3d`
- [x] `lead_knot_held_3d`
- [x] `lead_tie_mount`

### 3D Armor & Wear (26/26 - 100.0%)

- [x] `shield_back_holster_3d`
- [x] `turtle_helmet_3d`
- [x] `netherite_helmet_crest_3d`
- [x] `diamond_helmet_visor_3d`
- [x] `iron_helmet_brow_3d`
- [x] `golden_crown_3d`
- [x] `chainmail_coif_3d`
- [x] `leather_cap_feather_3d`
- [x] `netherite_chestplate_pauldron_3d`
- [x] `diamond_chestplate_guard_3d`
- [x] `iron_chestplate_gorget_3d`
- [x] `elytra_folded_back_3d`
- [x] `elytra_spread_wings_3d`
- [x] `quiver_arrow_storage_3d`
- [x] `backpack_traveler_3d`
- [x] `cape_physics_rig_3d`
- [x] `heavy_boots_spurs_3d`
- [x] `greaves_knee_poleyns_3d`
- [x] `crown_gem_shimmer`
- [x] `witch_hat_slouched_3d`
- [x] `pumpkin_carved_mask_3d`
- [x] `dragon_head_jaw_open_3d`
- [x] `piglin_head_ear_sway_3d`
- [x] `creeper_head_shudder_3d`
- [x] `skeleton_skull_eye_socket_3d`
- [x] `wither_skull_aura_3d`

### Held & Offhand (22/22 - 100.0%)

- [x] `shield_offhand_grip_3d`
- [x] `shield_blocking_angle_3d`
- [x] `shield_banner_pattern_3d`
- [x] `totem_undying_held_3d`
- [x] `torch_burning_flame_3d`
- [x] `soul_torch_blue_smoke_3d`
- [x] `redstone_torch_spark_3d`
- [x] `lantern_swing_chain_3d`
- [x] `soul_lantern_swing_3d`
- [x] `potion_cork_flask_3d`
- [x] `splash_potion_round_3d`
- [x] `lingering_potion_cloud_3d`
- [x] `honey_bottle_viscous_3d`
- [x] `milk_bucket_rim_3d`
- [x] `water_bucket_surface_3d`
- [x] `lava_bucket_glow_3d`
- [x] `golden_apple_halo_3d`
- [x] `enchanted_gapple_glint_3d`
- [x] `ender_pearl_swirl_3d`
- [x] `eye_of_ender_pupil_3d`
- [x] `wind_charge_vortex_3d`
- [x] `firework_rocket_tail_3d`

---

## 3. Bedrock Engine Parity Subsystems

### Molang Engine (32/32 - 100.0%)

- [x] `tokenize_symbols`
- [x] `parse_math_sin_cos`
- [x] `parse_query_anim_time`
- [x] `parse_query_is_sneaking`
- [x] `parse_ternary_operator`
- [x] `parse_boolean_logic`
- [x] `eval_binary_ops`
- [x] `variable_local_scope`
- [x] `query_ground_speed`
- [x] `query_vertical_speed`
- [x] `math_clamp_lerp`
- [x] `math_abs_floor_ceil`
- [x] `math_min_max_pow`
- [x] `math_asin_acos_atan`
- [x] `math_atan2`
- [x] `math_dice_roll`
- [x] `query_yaw_speed`
- [x] `query_pitch`
- [x] `query_head_yaw`
- [x] `query_target_dist`
- [x] `query_is_swimming`
- [x] `query_is_gliding`
- [x] `query_is_on_fire`
- [x] `query_is_in_water`
- [x] `query_item_in_use`
- [x] `query_item_max_use_duration`
- [x] `query_main_hand_empty`
- [x] `query_off_hand_empty`
- [x] `query_shield_blocking`
- [x] `context_lifetime_tracker`
- [x] `molang_jit_bytecode`
- [x] `molang_zero_alloc_eval`

### Rig & Pivots (20/20 - 100.0%)

- [x] `relative_pivot_calc`
- [x] `bone_parent_matrix`
- [x] `rotation_order_zyx`
- [x] `preserve_hip_leg_joint`
- [x] `preserve_shoulder_pivot`
- [x] `preserve_head_neck_pivot`
- [x] `waist_spine_chain`
- [x] `knee_elbow_ik_bend`
- [x] `hand_item_mount_socket`
- [x] `offhand_mount_socket`
- [x] `bone_scale_interpolation`
- [x] `custom_bone_extender`
- [x] `matrix_cache_simd`
- [x] `wgpu_vertex_transform_pass`
- [x] `foot_ground_align_ik`
- [x] `cape_cloth_spring_physics`
- [x] `hair_ponytail_spring`
- [x] `armor_cube_inflation_sync`
- [x] `slim_steve_alex_uv_switch`
- [x] `locator_particle_sockets`

### 1st/3rd Viewport (15/15 - 100.0%)

- [x] `third_person_camera_attach`
- [x] `third_person_bone_blend`
- [x] `first_person_fov_clipping_fix`
- [x] `first_person_3d_item_pos`
- [x] `first_person_item_bob_pitch`
- [x] `first_person_mining_swing`
- [x] `first_person_bow_draw_fov`
- [x] `first_person_shield_block_offset`
- [x] `first_person_dual_render`
- [x] `third_person_free_orbit`
- [x] `shoulder_cam_cinematic_lerp`
- [x] `submerged_water_refraction`
- [x] `death_cam_tilt_horizon`
- [x] `vr_stereo_rig_compat`
- [x] `replay_perspective_export`
