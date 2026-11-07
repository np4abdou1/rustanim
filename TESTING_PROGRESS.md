# rustanim Manual Testing & In-Game Verification Checklist

> User Testing Progress for Actions & Stuff Bedrock Parity in Horizon Minecraft (Rust).
> Mark any test scenario complete by changing `- [ ]` to `- [x]`, then run `python3 tools/progress.py`
> to update the green squares in `progress-testing.svg` and `badge-testing.svg`!

---

## 1. First-Person In-Game Tests (0/30)

- [ ] `fp_sword_slash_arc`: Diamond/Iron sword swing arc in 1st person
- [ ] `fp_sword_crit_air`: Jump attack sweep in 1st person
- [ ] `fp_bare_hands_punch`: Unarmed punch animation
- [ ] `fp_axe_overhead_chop`: Axe wood chopping animation
- [ ] `fp_pickaxe_mine_loop`: Pickaxe continuous stone mining
- [ ] `fp_shovel_dig_sand`: Shovel digging dirt/sand
- [ ] `fp_hoe_till_grass`: Hoe tilling farmland
- [ ] `fp_bow_pull_stage0`: Bow initial draw string pull
- [ ] `fp_bow_pull_stage1`: Bow half-drawn zoom
- [ ] `fp_bow_full_charge`: Bow fully charged shake
- [ ] `fp_crossbow_crank1`: Crossbow first crank lever
- [ ] `fp_crossbow_crank2`: Crossbow loaded lock
- [ ] `fp_shield_raise_block`: Shield raised to block angle
- [ ] `fp_shield_block_hit`: Shield deflection recoil impact
- [ ] `fp_shield_lower`: Shield lower to standby position
- [ ] `fp_trident_aim_charge`: Trident throw aim posture
- [ ] `fp_eat_food_cycle`: Bread/steak eating hand cycle
- [ ] `fp_drink_potion_tilt`: Potion bottle tipping back
- [ ] `fp_spyglass_eye_pan`: Spyglass panning at eye level
- [ ] `fp_map_two_hands_hold`: Holding map in both hands
- [ ] `fp_book_open_read`: Book held open reading
- [ ] `fp_compass_look_down`: Looking down at compass needle
- [ ] `fp_bundle_peek_inside`: Opening and peeking into bundle
- [ ] `fp_totem_undying_float`: Totem of Undying activation float
- [ ] `fp_fishing_rod_cast`: Fishing rod line whip cast
- [ ] `fp_fishing_rod_strain`: Fishing rod bend under catch
- [ ] `fp_flint_steel_strike`: Flint & steel sparking strike
- [ ] `fp_shears_snip_cut`: Shears snipping wool
- [ ] `fp_viewmodel_fov_70_check`: Verify 70° FOV viewmodel framing
- [ ] `fp_near_plane_clipping_guard`: Weapon never clips behind camera eye

---

## 2. Third-Person Locomotion & Actions (0/30)

- [ ] `tp_walk_forward_bob`: Walk cycle with torso swaying
- [ ] `tp_walk_backwards`: Backwards step leg coordination
- [ ] `tp_strafe_left_lean`: Lateral strafe left with torso tilt
- [ ] `tp_strafe_right_lean`: Lateral strafe right with torso tilt
- [ ] `tp_sprint_forward_lean`: Sprinting forward lean posture
- [ ] `tp_sprint_stop_slide`: Abrupt sprint stop slide braking
- [ ] `tp_jump_squat_pre`: Pre-jump knee compression squat
- [ ] `tp_jump_apex_float`: Mid-air apex flight silhouette
- [ ] `tp_jump_fall_wind`: Downward falling arm flail
- [ ] `tp_jump_land_squat`: Landing impact leg compression
- [ ] `tp_sneak_creep_pose`: Sneaking hunched creeping walk
- [ ] `tp_sneak_idle_standby`: Sneak idle waiting pose
- [ ] `tp_crawl_prone_drag`: 1-block crawling belly drag
- [ ] `tp_ladder_climb_up`: Ladder rungs alternating climbing
- [ ] `tp_ladder_climb_down`: Ladder controlled descent
- [ ] `tp_swim_breaststroke`: Water swimming breaststroke kick
- [ ] `tp_swim_dive_down`: Diving downwards in deep ocean
- [ ] `tp_elytra_glide_level`: Horizontal elytra flight glide
- [ ] `tp_elytra_glide_dive`: High-speed elytra nose dive
- [ ] `tp_boat_row_forward`: Boat oars synchronous rowing
- [ ] `tp_horse_mount_seat`: Riding horse saddle posture
- [ ] `tp_strider_wobble_ride`: Strider riding lava bounce
- [ ] `tp_bed_sleep_pose`: Sleeping in bed horizontal pose
- [ ] `tp_sheathed_sword_back`: Sword holstered on back holster
- [ ] `tp_sheathed_katana_hip`: Katana/dagger mounted on hip
- [ ] `tp_shield_back_holster`: Shield carried on upper back
- [ ] `tp_armor_pauldron_sync`: Armor shoulders move with arms
- [ ] `tp_helmet_look_pitch`: Helmet tracks head pitch/yaw
- [ ] `tp_emotes_wave_friendly`: Friendly hand waving emote
- [ ] `tp_emotes_cheer_fist`: Fist pump celebration emote

---

## 3. Horizon wgpu & Parity Checks (0/30)

- [ ] `hz_no_hip_collapse`: Waist 45° tilt leaves leg anchors at Y=12.0
- [ ] `hz_no_limb_stretching`: Shoulder joints never detach during swing
- [ ] `hz_molang_time_clock`: query.anim_time ticks smoothly at 60 FPS
- [ ] `hz_molang_ground_speed`: query.ground_speed drives walk bobbing
- [ ] `hz_molang_zero_alloc`: Zero heap allocations on render loop
- [ ] `hz_catmull_rom_smooth`: Keyframe curves smooth without jerks
- [ ] `hz_wgpu_vertex_buffer`: Direct upload to Horizon vertex buffer
- [ ] `hz_matrix_simd_speed`: Mat4 transform pipeline executes < 50ns
- [ ] `hz_steve_alex_uv_toggle`: Slim Alex 3px arm UV renders cleanly
- [ ] `hz_cape_cloth_physics`: Cape flaps realistically in wind/velocity
- [ ] `hz_secondary_springs`: Hair and antenna spring damper motion
- [ ] `hz_foot_ground_ik_terrain`: Feet tilt to match hillside slopes
- [ ] `hz_armor_cube_inflation`: Armor layers expand without z-fighting
- [ ] `hz_particle_eye_emitter`: Eye particle locator sits at eye level
- [ ] `hz_particle_hand_emitter`: Hand particle locator tracks held item
- [ ] `hz_orbit_cam_spherical`: 3rd person orbit rotates smoothly
- [ ] `hz_shoulder_cam_lerp`: Over-the-shoulder camera tracks movement
- [ ] `hz_underwater_refraction`: Underwater viewport optical distortion
- [ ] `hz_death_cam_rolling_tilt`: Camera rolls sideways on player death
- [ ] `hz_vr_stereo_dual_eye`: Stereo VR left/right IPD projection
- [ ] `hz_perspective_toggle_live`: F5 toggle between 1st and 3rd person
- [ ] `hz_mcpack_streaming_open`: 33.8MB mcpack loads under 100ms
- [ ] `hz_multi_clip_blend`: Blending walk and sword swing concurrently
- [ ] `hz_offhand_shield_held`: Left-hand shield renders in dual viewport
- [ ] `hz_head_pitch_yaw_sync`: Looking up/down rotates head on torso
- [ ] `hz_server_tick_sync`: Client animations sync with server velocity
- [ ] `hz_frame_rate_60fps`: Renders 60+ FPS without GC spikes
- [ ] `hz_memory_footprint_low`: Engine uses < 15MB resident memory
- [ ] `hz_cargo_check_clean`: cargo check has 0 warnings and 0 errors
- [ ] `hz_cargo_test_all_green`: cargo test passes 100% on VPS and CI
