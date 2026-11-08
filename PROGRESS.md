# rustanim Implementation & Parity Progress

> Actions & Stuff (Bedrock) to Native Rust Port for Horizon

**Animations Progress:** 2 / 143 (1.4%)  
**3D Attachables Progress:** 0 / 115 (0.0%)  
**Bedrock Parity Progress:** 12 / 67 (17.91%)  

---

## 1. Actions & Stuff Animation Clips

### Locomotion (2/32 - 6.2%)

- [x] `walk_forward`
- [x] `idle_stand`
- [ ] `walk_bob`
- [ ] `sprint_straight`
- [ ] `sprint_lean`
- [ ] `sprint_stop_slide`
- [ ] `walk_backwards`
- [ ] `strafe_left`
- [ ] `strafe_right`
- [ ] `jump_squat`
- [ ] `jump_air_apex`
- [ ] `jump_fall`
- [ ] `jump_land_squat`
- [ ] `jump_run_land`
- [ ] `sneak_creep`
- [ ] `sneak_idle`
- [ ] `sneak_walk_back`
- [ ] `crawl_prone`
- [ ] `crawl_move`
- [ ] `ladder_climb_up`
- [ ] `ladder_climb_down`
- [ ] `ladder_idle`
- [ ] `scaffolding_move`
- [ ] `stumble_bump`
- [ ] `air_turn_yaw`
- [ ] `slope_tilt`
- [ ] `sprint_jump_chain`
- [ ] `hard_fall_recoil`
- [ ] `turn_inplace_left`
- [ ] `turn_inplace_right`
- [ ] `step_up_knee`
- [ ] `edge_balance`

### Combat (0/36 - 0.0%)

- [ ] `sword_basic_slash`
- [ ] `bare_hands_punch`
- [ ] `sword_combo_1`
- [ ] `sword_combo_2`
- [ ] `sword_spin_heavy`
- [ ] `sword_sprint_dash`
- [ ] `sword_critical_air`
- [ ] `sword_sweep_cleave`
- [ ] `axe_chop_overhead`
- [ ] `axe_stun_smash`
- [ ] `dagger_quick_thrust`
- [ ] `shield_raise`
- [ ] `shield_block_hit`
- [ ] `shield_parry_spark`
- [ ] `shield_lower`
- [ ] `bow_draw_charge`
- [ ] `bow_hold_aim`
- [ ] `bow_release_kick`
- [ ] `crossbow_crank_stage1`
- [ ] `crossbow_crank_stage2`
- [ ] `crossbow_loaded_aim`
- [ ] `crossbow_fire_recoil`
- [ ] `trident_charge_aim`
- [ ] `trident_throw_release`
- [ ] `trident_riptide_spin`
- [ ] `mace_smash_falling`
- [ ] `mace_wind_slam`
- [ ] `hurt_flinch_light`
- [ ] `hurt_heavy_knockback`
- [ ] `hurt_head_recoil`
- [ ] `death_collapse`
- [ ] `death_spin_fall`
- [ ] `pvp_strafe_parry`
- [ ] `crit_fall_impact`
- [ ] `knockback_recovery`
- [ ] `dual_wield_ready`

### Actions (0/26 - 0.0%)

- [ ] `mine_pickaxe_swing`
- [ ] `mine_axe_chop`
- [ ] `mine_shovel_dig`
- [ ] `mine_hoe_till`
- [ ] `block_place_push`
- [ ] `block_interact_press`
- [ ] `eat_cycle_hand`
- [ ] `eat_bite_recoil`
- [ ] `drink_potion_tilt`
- [ ] `drink_finish_toss`
- [ ] `spyglass_aim_eye`
- [ ] `spyglass_pan`
- [ ] `goat_horn_raise`
- [ ] `goat_horn_blow`
- [ ] `brush_archaeology_sweep`
- [ ] `flint_and_steel_strike`
- [ ] `shears_snip`
- [ ] `fishing_cast`
- [ ] `fishing_reel_strain`
- [ ] `firework_elytra_boost`
- [ ] `totem_activate_float`
- [ ] `book_read_open`
- [ ] `compass_look_down`
- [ ] `map_both_hands_hold`
- [ ] `bundle_open_peek`
- [ ] `chest_open_reach`

### Aquatic & Flight (0/18 - 0.0%)

- [ ] `swim_breaststroke`
- [ ] `swim_paddle_fast`
- [ ] `swim_tread_idle`
- [ ] `swim_dive_down`
- [ ] `swim_surface_breach`
- [ ] `swim_turn_yaw`
- [ ] `water_sink_drift`
- [ ] `elytra_glide_level`
- [ ] `elytra_glide_dive`
- [ ] `elytra_glide_climb`
- [ ] `elytra_bank_roll_left`
- [ ] `elytra_bank_roll_right`
- [ ] `elytra_deploy_puff`
- [ ] `elytra_stall_flutter`
- [ ] `boat_row_forward`
- [ ] `boat_row_turn`
- [ ] `boat_idle_sit`
- [ ] `drown_struggle`

### Mounts & Emotes (0/16 - 0.0%)

- [ ] `horse_mount_seat`
- [ ] `horse_gallop_sway`
- [ ] `horse_jump_brace`
- [ ] `pig_saddle_bounce`
- [ ] `strider_wobble_ride`
- [ ] `camel_seat_duo`
- [ ] `bed_sleep_pose`
- [ ] `bed_wake_turn`
- [ ] `sit_ledge_dangle`
- [ ] `wave_hand_friendly`
- [ ] `clap_hands`
- [ ] `shrug_shoulders`
- [ ] `point_finger`
- [ ] `nod_agree`
- [ ] `shake_head_disagree`
- [ ] `cheer_fist_pump`

### Idle & Facial (0/15 - 0.0%)

- [ ] `idle_look_head`
- [ ] `idle_weight_shift`
- [ ] `idle_shoulder_sigh`
- [ ] `idle_inspect_held`
- [ ] `idle_scratch_head`
- [ ] `idle_look_skyward`
- [ ] `eye_blink_natural`
- [ ] `eye_blink_double`
- [ ] `eye_squint_aim`
- [ ] `mouth_jaw_open`
- [ ] `mouth_pant_exhausted`
- [ ] `eyebrow_raise`
- [ ] `eyebrow_frown`
- [ ] `item_twirl_fidget`
- [ ] `foot_tap_bored`

---

## 2. 3D Attachables & Mount Points

### 3D Weapons (0/38 - 0.0%)

- [ ] `diamond_sword_3d`
- [ ] `iron_sword_3d`
- [ ] `netherite_sword_3d`
- [ ] `golden_sword_3d`
- [ ] `stone_sword_3d`
- [ ] `wooden_sword_3d`
- [ ] `1st_person_sword_offset_fix`
- [ ] `1st_person_sword_swing_arc`
- [ ] `1st_person_shield_fov_fix`
- [ ] `sheathed_weapon_back_mount`
- [ ] `sheathed_weapon_hip_mount`
- [ ] `mace_heavy_3d`
- [ ] `trident_item_3d`
- [ ] `bow_unpulled_3d`
- [ ] `bow_pulling_0_3d`
- [ ] `bow_pulling_1_3d`
- [ ] `bow_pulling_2_3d`
- [ ] `crossbow_standby_3d`
- [ ] `crossbow_pulling_0_3d`
- [ ] `crossbow_pulling_1_3d`
- [ ] `crossbow_pulling_2_3d`
- [ ] `crossbow_arrow_loaded_3d`
- [ ] `crossbow_firework_loaded_3d`
- [ ] `custom_dagger_sheath`
- [ ] `custom_greatsword_back`
- [ ] `custom_scythe_3d`
- [ ] `custom_rapier_3d`
- [ ] `custom_katana_hip`
- [ ] `custom_battleaxe_double`
- [ ] `custom_spear_shaft`
- [ ] `custom_warhammer_stone`
- [ ] `custom_halberd_steel`
- [ ] `enchant_glow_sword_overlay`
- [ ] `sharpness_spark_emitter`
- [ ] `fire_aspect_flame_bones`
- [ ] `sweeping_edge_blade_trail`
- [ ] `1st_person_bow_aim_view`
- [ ] `offhand_sword_parry_pos`

### 3D Tools (0/29 - 0.0%)

- [ ] `diamond_pickaxe_3d`
- [ ] `netherite_pickaxe_3d`
- [ ] `iron_pickaxe_3d`
- [ ] `golden_pickaxe_3d`
- [ ] `stone_pickaxe_3d`
- [ ] `wooden_pickaxe_3d`
- [ ] `netherite_axe_3d`
- [ ] `diamond_axe_3d`
- [ ] `iron_axe_3d`
- [ ] `golden_axe_3d`
- [ ] `stone_axe_3d`
- [ ] `wooden_axe_3d`
- [ ] `netherite_shovel_3d`
- [ ] `diamond_shovel_3d`
- [ ] `iron_shovel_3d`
- [ ] `stone_shovel_3d`
- [ ] `diamond_hoe_3d`
- [ ] `iron_hoe_3d`
- [ ] `shears_jointed_3d`
- [ ] `fishing_rod_cast_3d`
- [ ] `fishing_rod_hook_3d`
- [ ] `flint_and_steel_3d`
- [ ] `clock_dynamic_face_3d`
- [ ] `compass_needle_pivot_3d`
- [ ] `recovery_compass_3d`
- [ ] `brush_archaeology_bristle_3d`
- [ ] `spyglass_brass_body_3d`
- [ ] `lead_knot_held_3d`
- [ ] `lead_tie_mount`

### 3D Armor & Wear (0/26 - 0.0%)

- [ ] `shield_back_holster_3d`
- [ ] `turtle_helmet_3d`
- [ ] `netherite_helmet_crest_3d`
- [ ] `diamond_helmet_visor_3d`
- [ ] `iron_helmet_brow_3d`
- [ ] `golden_crown_3d`
- [ ] `chainmail_coif_3d`
- [ ] `leather_cap_feather_3d`
- [ ] `netherite_chestplate_pauldron_3d`
- [ ] `diamond_chestplate_guard_3d`
- [ ] `iron_chestplate_gorget_3d`
- [ ] `elytra_folded_back_3d`
- [ ] `elytra_spread_wings_3d`
- [ ] `quiver_arrow_storage_3d`
- [ ] `backpack_traveler_3d`
- [ ] `cape_physics_rig_3d`
- [ ] `heavy_boots_spurs_3d`
- [ ] `greaves_knee_poleyns_3d`
- [ ] `crown_gem_shimmer`
- [ ] `witch_hat_slouched_3d`
- [ ] `pumpkin_carved_mask_3d`
- [ ] `dragon_head_jaw_open_3d`
- [ ] `piglin_head_ear_sway_3d`
- [ ] `creeper_head_shudder_3d`
- [ ] `skeleton_skull_eye_socket_3d`
- [ ] `wither_skull_aura_3d`

### Held & Offhand (0/22 - 0.0%)

- [ ] `shield_offhand_grip_3d`
- [ ] `shield_blocking_angle_3d`
- [ ] `shield_banner_pattern_3d`
- [ ] `totem_undying_held_3d`
- [ ] `torch_burning_flame_3d`
- [ ] `soul_torch_blue_smoke_3d`
- [ ] `redstone_torch_spark_3d`
- [ ] `lantern_swing_chain_3d`
- [ ] `soul_lantern_swing_3d`
- [ ] `potion_cork_flask_3d`
- [ ] `splash_potion_round_3d`
- [ ] `lingering_potion_cloud_3d`
- [ ] `honey_bottle_viscous_3d`
- [ ] `milk_bucket_rim_3d`
- [ ] `water_bucket_surface_3d`
- [ ] `lava_bucket_glow_3d`
- [ ] `golden_apple_halo_3d`
- [ ] `enchanted_gapple_glint_3d`
- [ ] `ender_pearl_swirl_3d`
- [ ] `eye_of_ender_pupil_3d`
- [ ] `wind_charge_vortex_3d`
- [ ] `firework_rocket_tail_3d`

---

## 3. Bedrock Engine Parity Subsystems

### Molang Engine (9/32 - 28.1%)

- [x] `tokenize_symbols`
- [x] `parse_math_sin_cos`
- [x] `parse_ternary_operator`
- [x] `eval_binary_ops`
- [x] `math_clamp_lerp`
- [x] `math_abs_floor_ceil`
- [x] `math_min_max_pow`
- [x] `math_asin_acos_atan`
- [x] `math_atan2`
- [ ] `parse_query_anim_time`
- [ ] `parse_query_is_sneaking`
- [ ] `parse_boolean_logic`
- [ ] `variable_local_scope`
- [ ] `query_ground_speed`
- [ ] `query_vertical_speed`
- [ ] `math_dice_roll`
- [ ] `query_yaw_speed`
- [ ] `query_pitch`
- [ ] `query_head_yaw`
- [ ] `query_target_dist`
- [ ] `query_is_swimming`
- [ ] `query_is_gliding`
- [ ] `query_is_on_fire`
- [ ] `query_is_in_water`
- [ ] `query_item_in_use`
- [ ] `query_item_max_use_duration`
- [ ] `query_main_hand_empty`
- [ ] `query_off_hand_empty`
- [ ] `query_shield_blocking`
- [ ] `context_lifetime_tracker`
- [ ] `molang_jit_bytecode`
- [ ] `molang_zero_alloc_eval`

### Rig & Pivots (3/20 - 15.0%)

- [x] `relative_pivot_calc`
- [x] `bone_parent_matrix`
- [x] `rotation_order_zyx`
- [ ] `preserve_hip_leg_joint`
- [ ] `preserve_shoulder_pivot`
- [ ] `preserve_head_neck_pivot`
- [ ] `waist_spine_chain`
- [ ] `knee_elbow_ik_bend`
- [ ] `hand_item_mount_socket`
- [ ] `offhand_mount_socket`
- [ ] `bone_scale_interpolation`
- [ ] `custom_bone_extender`
- [ ] `matrix_cache_simd`
- [ ] `wgpu_vertex_transform_pass`
- [ ] `foot_ground_align_ik`
- [ ] `cape_cloth_spring_physics`
- [ ] `hair_ponytail_spring`
- [ ] `armor_cube_inflation_sync`
- [ ] `slim_steve_alex_uv_switch`
- [ ] `locator_particle_sockets`

### 1st/3rd Viewport (0/15 - 0.0%)

- [ ] `third_person_camera_attach`
- [ ] `third_person_bone_blend`
- [ ] `first_person_fov_clipping_fix`
- [ ] `first_person_3d_item_pos`
- [ ] `first_person_item_bob_pitch`
- [ ] `first_person_mining_swing`
- [ ] `first_person_bow_draw_fov`
- [ ] `first_person_shield_block_offset`
- [ ] `first_person_dual_render`
- [ ] `third_person_free_orbit`
- [ ] `shoulder_cam_cinematic_lerp`
- [ ] `submerged_water_refraction`
- [ ] `death_cam_tilt_horizon`
- [ ] `vr_stereo_rig_compat`
- [ ] `replay_perspective_export`
