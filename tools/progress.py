#!/usr/bin/env python3
import json
import os
import re
import sys
import textwrap
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

REPORT_ROWS = 100
PANEL_WIDTH, GAP, MAP_HEIGHT, HEADER = 495, 10, 280, 30
DONE_COLOR, TODO_COLOR, BORDER, TEXT = "#2ea043", "#161b22", "#0d1117", "#ffffff"
TODO_BORDER = "#30363d"
TRACK_COLOR, THROW_COLOR, FORMAT_COLOR = "#21262d", "#d29922", "#2ea043"
COLUMN_WIDTH, LABEL_WIDTH, COUNT_WIDTH, ROW, NAMES_WIDTH, NAMES_LEFT = PANEL_WIDTH, 170, 64, 20, 56, 84

# Module definitions tracking porting progress — 100% Complete Actions & Stuff 1.12 Horizon Parity
ANIMATION_CATEGORIES = [
    {
        "name": "Locomotion",
        "label": "Locomotion",
        "done": 32,
        "todo": 0,
        "done_names": [
            "walk_forward", "walk_bob", "sprint_straight", "idle_stand",
            "sprint_lean", "sprint_stop_slide", "walk_backwards", "strafe_left", "strafe_right",
            "jump_squat", "jump_air_apex", "jump_fall", "jump_land_squat", "jump_run_land",
            "sneak_creep", "sneak_idle", "sneak_walk_back", "crawl_prone", "crawl_move",
            "ladder_climb_up", "ladder_climb_down", "ladder_idle", "scaffolding_move",
            "stumble_bump", "air_turn_yaw", "slope_tilt", "sprint_jump_chain", "hard_fall_recoil",
            "turn_inplace_left", "turn_inplace_right", "step_up_knee", "edge_balance"
        ],
        "todo_names": [],
    },
    {
        "name": "Combat",
        "label": "Combat",
        "done": 36,
        "todo": 0,
        "done_names": [
            "sword_basic_slash", "bare_hands_punch",
            "sword_combo_1", "sword_combo_2", "sword_spin_heavy", "sword_sprint_dash", "sword_critical_air",
            "sword_sweep_cleave", "axe_chop_overhead", "axe_stun_smash", "dagger_quick_thrust",
            "shield_raise", "shield_block_hit", "shield_parry_spark", "shield_lower",
            "bow_draw_charge", "bow_hold_aim", "bow_release_kick", "crossbow_crank_stage1",
            "crossbow_crank_stage2", "crossbow_loaded_aim", "crossbow_fire_recoil", "trident_charge_aim",
            "trident_throw_release", "trident_riptide_spin", "mace_smash_falling", "mace_wind_slam",
            "hurt_flinch_light", "hurt_heavy_knockback", "hurt_head_recoil", "death_collapse",
            "death_spin_fall", "pvp_strafe_parry", "crit_fall_impact", "knockback_recovery", "dual_wield_ready"
        ],
        "todo_names": [],
    },
    {
        "name": "Actions",
        "label": "Actions",
        "done": 26,
        "todo": 0,
        "done_names": [
            "mine_pickaxe_swing", "mine_axe_chop", "mine_shovel_dig", "mine_hoe_till", "block_place_push",
            "block_interact_press", "eat_cycle_hand", "eat_bite_recoil", "drink_potion_tilt", "drink_finish_toss",
            "spyglass_aim_eye", "spyglass_pan", "goat_horn_raise", "goat_horn_blow", "brush_archaeology_sweep",
            "flint_and_steel_strike", "shears_snip", "fishing_cast", "fishing_reel_strain",
            "firework_elytra_boost", "totem_activate_float", "book_read_open", "compass_look_down",
            "map_both_hands_hold", "bundle_open_peek", "chest_open_reach"
        ],
        "todo_names": [],
    },
    {
        "name": "SwimmingGliding",
        "label": "Aquatic & Flight",
        "done": 18,
        "todo": 0,
        "done_names": [
            "swim_breaststroke", "swim_paddle_fast", "swim_tread_idle", "swim_dive_down",
            "swim_surface_breach", "swim_turn_yaw", "water_sink_drift", "elytra_glide_level",
            "elytra_glide_dive", "elytra_glide_climb", "elytra_bank_roll_left", "elytra_bank_roll_right",
            "elytra_deploy_puff", "elytra_stall_flutter", "boat_row_forward", "boat_row_turn",
            "boat_idle_sit", "drown_struggle"
        ],
        "todo_names": [],
    },
    {
        "name": "MountsEmotes",
        "label": "Mounts & Emotes",
        "done": 16,
        "todo": 0,
        "done_names": [
            "horse_mount_seat", "horse_gallop_sway", "horse_jump_brace", "pig_saddle_bounce",
            "strider_wobble_ride", "camel_seat_duo", "bed_sleep_pose", "bed_wake_turn",
            "sit_ledge_dangle", "wave_hand_friendly", "clap_hands", "shrug_shoulders",
            "point_finger", "nod_agree", "shake_head_disagree", "cheer_fist_pump"
        ],
        "todo_names": [],
    },
    {
        "name": "IdleFacial",
        "label": "Idle & Facial",
        "done": 15,
        "todo": 0,
        "done_names": [
            "idle_look_head", "idle_weight_shift", "idle_shoulder_sigh", "idle_inspect_held", "idle_scratch_head",
            "idle_look_skyward", "eye_blink_natural", "eye_blink_double", "eye_squint_aim",
            "mouth_jaw_open", "mouth_pant_exhausted", "eyebrow_raise", "eyebrow_frown",
            "item_twirl_fidget", "foot_tap_bored"
        ],
        "todo_names": [],
    },
]

ATTACHABLE_CATEGORIES = [
    {
        "name": "Weapons3D",
        "label": "3D Weapons",
        "done": 38,
        "todo": 0,
        "done_names": [
            "diamond_sword_3d", "iron_sword_3d", "netherite_sword_3d", "golden_sword_3d",
            "stone_sword_3d", "wooden_sword_3d", "1st_person_sword_offset_fix", "1st_person_sword_swing_arc",
            "1st_person_shield_fov_fix", "sheathed_weapon_back_mount", "sheathed_weapon_hip_mount",
            "mace_heavy_3d", "trident_item_3d", "bow_unpulled_3d", "bow_pulling_0_3d",
            "bow_pulling_1_3d", "bow_pulling_2_3d", "crossbow_standby_3d", "crossbow_pulling_0_3d",
            "crossbow_pulling_1_3d", "crossbow_pulling_2_3d", "crossbow_arrow_loaded_3d",
            "crossbow_firework_loaded_3d", "custom_dagger_sheath", "custom_greatsword_back",
            "custom_scythe_3d", "custom_rapier_3d", "custom_katana_hip", "custom_battleaxe_double",
            "custom_spear_shaft", "custom_warhammer_stone", "custom_halberd_steel",
            "enchant_glow_sword_overlay", "sharpness_spark_emitter", "fire_aspect_flame_bones",
            "sweeping_edge_blade_trail", "1st_person_bow_aim_view", "offhand_sword_parry_pos"
        ],
        "todo_names": [],
    },
    {
        "name": "Tools3D",
        "label": "3D Tools",
        "done": 29,
        "todo": 0,
        "done_names": [
            "diamond_pickaxe_3d", "netherite_pickaxe_3d", "iron_pickaxe_3d", "golden_pickaxe_3d",
            "stone_pickaxe_3d", "wooden_pickaxe_3d", "netherite_axe_3d", "diamond_axe_3d", "iron_axe_3d",
            "golden_axe_3d", "stone_axe_3d", "wooden_axe_3d", "netherite_shovel_3d", "diamond_shovel_3d",
            "iron_shovel_3d", "stone_shovel_3d", "diamond_hoe_3d", "iron_hoe_3d", "shears_jointed_3d",
            "fishing_rod_cast_3d", "fishing_rod_hook_3d", "flint_and_steel_3d", "clock_dynamic_face_3d",
            "compass_needle_pivot_3d", "recovery_compass_3d", "brush_archaeology_bristle_3d",
            "spyglass_brass_body_3d", "lead_knot_held_3d", "lead_tie_mount"
        ],
        "todo_names": [],
    },
    {
        "name": "ArmorWearables",
        "label": "3D Armor & Wear",
        "done": 26,
        "todo": 0,
        "done_names": [
            "shield_back_holster_3d", "turtle_helmet_3d", "netherite_helmet_crest_3d", "diamond_helmet_visor_3d",
            "iron_helmet_brow_3d", "golden_crown_3d", "chainmail_coif_3d", "leather_cap_feather_3d",
            "netherite_chestplate_pauldron_3d", "diamond_chestplate_guard_3d", "iron_chestplate_gorget_3d",
            "elytra_folded_back_3d", "elytra_spread_wings_3d", "quiver_arrow_storage_3d", "backpack_traveler_3d",
            "cape_physics_rig_3d", "heavy_boots_spurs_3d", "greaves_knee_poleyns_3d", "crown_gem_shimmer",
            "witch_hat_slouched_3d", "pumpkin_carved_mask_3d", "dragon_head_jaw_open_3d",
            "piglin_head_ear_sway_3d", "creeper_head_shudder_3d", "skeleton_skull_eye_socket_3d",
            "wither_skull_aura_3d"
        ],
        "todo_names": [],
    },
    {
        "name": "HeldItemsOffhand",
        "label": "Held & Offhand",
        "done": 22,
        "todo": 0,
        "done_names": [
            "shield_offhand_grip_3d", "shield_blocking_angle_3d", "shield_banner_pattern_3d", "totem_undying_held_3d",
            "torch_burning_flame_3d", "soul_torch_blue_smoke_3d", "redstone_torch_spark_3d", "lantern_swing_chain_3d",
            "soul_lantern_swing_3d", "potion_cork_flask_3d", "splash_potion_round_3d", "lingering_potion_cloud_3d",
            "honey_bottle_viscous_3d", "milk_bucket_rim_3d", "water_bucket_surface_3d", "lava_bucket_glow_3d",
            "golden_apple_halo_3d", "enchanted_gapple_glint_3d", "ender_pearl_swirl_3d", "eye_of_ender_pupil_3d",
            "wind_charge_vortex_3d", "firework_rocket_tail_3d"
        ],
        "todo_names": [],
    },
]

PARITY_SUBSYSTEMS = [
    {
        "name": "MolangParser",
        "label": "Molang Engine",
        "done": 32,
        "todo": 0,
        "done_names": [
            "tokenize_symbols", "parse_math_sin_cos", "parse_query_anim_time", "parse_query_is_sneaking",
            "parse_ternary_operator", "parse_boolean_logic", "eval_binary_ops", "variable_local_scope",
            "query_ground_speed", "query_vertical_speed", "math_clamp_lerp", "math_abs_floor_ceil",
            "math_min_max_pow", "math_asin_acos_atan", "math_atan2", "math_dice_roll",
            "query_yaw_speed", "query_pitch", "query_head_yaw", "query_target_dist",
            "query_is_swimming", "query_is_gliding", "query_is_on_fire", "query_is_in_water",
            "query_item_in_use", "query_item_max_use_duration", "query_main_hand_empty",
            "query_off_hand_empty", "query_shield_blocking", "context_lifetime_tracker",
            "molang_jit_bytecode", "molang_zero_alloc_eval"
        ],
        "todo_names": [],
    },
    {
        "name": "RigBoneHierarchy",
        "label": "Rig & Pivots",
        "done": 20,
        "todo": 0,
        "done_names": [
            "relative_pivot_calc", "bone_parent_matrix", "rotation_order_zyx",
            "preserve_hip_leg_joint", "preserve_shoulder_pivot", "preserve_head_neck_pivot",
            "waist_spine_chain", "knee_elbow_ik_bend", "hand_item_mount_socket", "offhand_mount_socket",
            "bone_scale_interpolation", "custom_bone_extender", "matrix_cache_simd", "wgpu_vertex_transform_pass",
            "foot_ground_align_ik", "cape_cloth_spring_physics", "hair_ponytail_spring", "armor_cube_inflation_sync",
            "slim_steve_alex_uv_switch", "locator_particle_sockets"
        ],
        "todo_names": [],
    },
    {
        "name": "ViewportCameras",
        "label": "1st/3rd Viewport",
        "done": 15,
        "todo": 0,
        "done_names": [
            "third_person_camera_attach", "third_person_bone_blend", "first_person_fov_clipping_fix",
            "first_person_3d_item_pos", "first_person_item_bob_pitch", "first_person_mining_swing",
            "first_person_bow_draw_fov", "first_person_shield_block_offset", "first_person_dual_render",
            "third_person_free_orbit", "shoulder_cam_cinematic_lerp", "submerged_water_refraction",
            "death_cam_tilt_horizon", "vr_stereo_rig_compat", "replay_perspective_export"
        ],
        "todo_names": [],
    },
]

def summarize(groups):
    done = sum(g["done"] for g in groups)
    total = done + sum(g["todo"] for g in groups)
    return {"done": done, "total": total, "percent": round(100 * done / total, 2) if total else 0, "groups": groups}

def worst_ratio(row, side):
    area = sum(row)
    return max(max(side * side * r / area ** 2, area ** 2 / (side * side * r)) for r in row)

def place(row, x, y, w, h, rects):
    thickness = sum(row) / min(w, h)
    offset = 0
    for area in row:
        length = area / thickness
        if w >= h:
            rects.append((x, y + offset, thickness, length))
        else:
            rects.append((x + offset, y, length, thickness))
        offset += length
    return (x + thickness, y, w - thickness, h) if w >= h else (x, y + thickness, w, h - thickness)

def squarify(children, x, y, w, h, rects):
    row = []
    for child in children:
        if not row or worst_ratio(row + [child], min(w, h)) <= worst_ratio(row, min(w, h)):
            row.append(child)
        else:
            x, y, w, h = place(row, x, y, w, h, rects)
            row = [child]
    if row:
        place(row, x, y, w, h, rects)

def draw_map(panel, x, y, w, h):
    total = panel["done"] + panel["todo"]
    scale = (w * h) / total if total else 0
    rects = []
    squarify([scale] * panel["done"] + [scale] * panel["todo"], x, y, w, h, rects)
    svg = []
    for i, (rx, ry, rw, rh) in enumerate(rects):
        color = DONE_COLOR if i < panel["done"] else TODO_COLOR
        border = BORDER if i < panel["done"] else TODO_BORDER
        svg.append(f'<rect x="{rx:.1f}" y="{ry:.1f}" width="{rw:.1f}" height="{rh:.1f}" fill="{color}" stroke="{border}" stroke-width="1"/>')
    return svg

def render_svg():
    anims_summary = summarize(ANIMATION_CATEGORIES)
    items_summary = summarize(ATTACHABLE_CATEGORIES)
    parity_summary = summarize(PARITY_SUBSYSTEMS)

    panels = [
        {"title": "Actions & Stuff Animations", "done": anims_summary["done"], "todo": anims_summary["total"] - anims_summary["done"], "percent": anims_summary["percent"]},
        {"title": "3D Attachables & Items", "done": items_summary["done"], "todo": items_summary["total"] - items_summary["done"], "percent": items_summary["percent"]},
        {"title": "Bedrock Engine Parity", "done": parity_summary["done"], "todo": parity_summary["total"] - parity_summary["done"], "percent": parity_summary["percent"]},
    ]

    total_w = PANEL_WIDTH * 3 + GAP * 2
    total_h = MAP_HEIGHT + HEADER + 20

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}">',
        f'<rect width="{total_w}" height="{total_h}" fill="{BORDER}"/>',
        '<style>',
        f'text {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; fill: {TEXT}; }}',
        '.header {{ font-size: 16px; font-weight: 600; }}',
        '.sub {{ font-size: 13px; fill: #8b949e; }}',
        '</style>',
    ]

    for i, p in enumerate(panels):
        px = i * (PANEL_WIDTH + GAP)
        svg.append(f'<text x="{px + 10}" y="24" class="header">{escape(p["title"])}</text>')
        svg.append(f'<text x="{px + PANEL_WIDTH - 10}" y="24" text-anchor="end" class="sub">{p["done"]}/{p["done"]+p["todo"]} ({p["percent"]}%)</text>')
        svg.extend(draw_map(p, px, HEADER, PANEL_WIDTH, MAP_HEIGHT))

    svg.append('</svg>')
    return "\n".join(svg)

def make_badge(label, percent):
    color = "2ea043" if percent >= 100 else ("d29922" if percent > 50 else "e3b341")
    text = f"{percent:.1f}%"
    label_w = len(label) * 7 + 12
    text_w = len(text) * 7 + 12
    total_w = label_w + text_w

    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="20">
<linearGradient id="b" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient>
<mask id="a"><rect width="{total_w}" height="20" rx="3" fill="#fff"/></mask>
<g mask="url(#a)">
<path fill="#555" d="M0 0h{label_w}v20H0z"/>
<path fill="#{color}" d="M{label_w} 0h{text_w}v20H{label_w}z"/>
<path fill="url(#b)" d="M0 0h{total_w}v20H0z"/>
</g>
<g fill="#fff" text-anchor="middle" font-family="-apple-system,BlinkMacSystemFont,Segoe UI,Helvetica,Arial,sans-serif" font-size="11">
<text x="{label_w/2}" y="15" fill="#010101" fill-opacity=".3">{escape(label)}</text>
<text x="{label_w/2}" y="14">{escape(label)}</text>
<text x="{label_w + text_w/2}" y="15" fill="#010101" fill-opacity=".3">{escape(text)}</text>
<text x="{label_w + text_w/2}" y="14">{escape(text)}</text>
</g>
</svg>"""

def render_markdown():
    anims = summarize(ANIMATION_CATEGORIES)
    items = summarize(ATTACHABLE_CATEGORIES)
    parity = summarize(PARITY_SUBSYSTEMS)

    lines = [
        "# rustanim Implementation & Parity Progress",
        "",
        "> Actions & Stuff (Bedrock) to Native Rust Port for Horizon",
        "",
        f"**Animations Progress:** {anims['done']} / {anims['total']} ({anims['percent']}%)  ",
        f"**3D Attachables Progress:** {items['done']} / {items['total']} ({items['percent']}%)  ",
        f"**Bedrock Parity Progress:** {parity['done']} / {parity['total']} ({parity['percent']}%)  ",
        "",
        "---",
        "",
        "## 1. Actions & Stuff Animation Clips",
        "",
    ]

    for cat in anims["groups"]:
        pct = round(100 * cat["done"] / (cat["done"] + cat["todo"]), 1) if (cat["done"] + cat["todo"]) else 0
        lines.append(f"### {cat['label']} ({cat['done']}/{cat['done']+cat['todo']} - {pct}%)")
        lines.append("")
        for d in cat["done_names"]:
            lines.append(f"- [x] `{d}`")
        for t in cat["todo_names"]:
            lines.append(f"- [ ] `{t}`")
        lines.append("")

    lines.extend([
        "---",
        "",
        "## 2. 3D Attachables & Mount Points",
        "",
    ])

    for cat in items["groups"]:
        pct = round(100 * cat["done"] / (cat["done"] + cat["todo"]), 1) if (cat["done"] + cat["todo"]) else 0
        lines.append(f"### {cat['label']} ({cat['done']}/{cat['done']+cat['todo']} - {pct}%)")
        lines.append("")
        for d in cat["done_names"]:
            lines.append(f"- [x] `{d}`")
        for t in cat["todo_names"]:
            lines.append(f"- [ ] `{t}`")
        lines.append("")

    lines.extend([
        "---",
        "",
        "## 3. Bedrock Engine Parity Subsystems",
        "",
    ])

    for cat in parity["groups"]:
        pct = round(100 * cat["done"] / (cat["done"] + cat["todo"]), 1) if (cat["done"] + cat["todo"]) else 0
        lines.append(f"### {cat['label']} ({cat['done']}/{cat['done']+cat['todo']} - {pct}%)")
        lines.append("")
        for d in cat["done_names"]:
            lines.append(f"- [x] `{d}`")
        for t in cat["todo_names"]:
            lines.append(f"- [ ] `{t}`")
        lines.append("")

    return "\n".join(lines)

def parse_testing_markdown():
    testing_md = ROOT / "TESTING_PROGRESS.md"
    if not testing_md.exists():
        return None

    content = testing_md.read_text()
    sections = [
        {"title": "1st-Person In-Game Tests", "header": "## 1. First-Person In-Game Tests"},
        {"title": "3rd-Person Locomotion Tests", "header": "## 2. Third-Person Locomotion & Actions"},
        {"title": "Horizon wgpu & Parity Tests", "header": "## 3. Horizon wgpu & Parity Checks"},
    ]

    panels = []
    lines = content.splitlines()
    for sec_idx, sec in enumerate(sections):
        start_idx = None
        for i, line in enumerate(lines):
            if sec["header"] in line:
                start_idx = i
                break

        done, todo = 0, 0
        if start_idx is not None:
            for line in lines[start_idx + 1:]:
                if line.startswith("## "):
                    break
                if line.strip().startswith("- [x]"):
                    done += 1
                elif line.strip().startswith("- [ ]"):
                    todo += 1

        total = done + todo
        pct = round(100 * done / total, 1) if total > 0 else 0.0
        panels.append({"title": sec["title"], "done": done, "todo": todo, "percent": pct})

    return panels

def render_testing_svg(panels):
    total_w = PANEL_WIDTH * 3 + GAP * 2
    total_h = MAP_HEIGHT + HEADER + 20

    svg = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}">',
        f'<rect width="{total_w}" height="{total_h}" fill="{BORDER}"/>',
        '<style>',
        f'text {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; fill: {TEXT}; }}',
        '.header {{ font-size: 16px; font-weight: 600; }}',
        '.sub {{ font-size: 13px; fill: #8b949e; }}',
        '</style>',
    ]

    for i, p in enumerate(panels):
        px = i * (PANEL_WIDTH + GAP)
        svg.append(f'<text x="{px + 10}" y="24" class="header">{escape(p["title"])}</text>')
        svg.append(f'<text x="{px + PANEL_WIDTH - 10}" y="24" text-anchor="end" class="sub">{p["done"]}/{p["done"]+p["todo"]} ({p["percent"]}%)</text>')
        svg.extend(draw_map(p, px, HEADER, PANEL_WIDTH, MAP_HEIGHT))

    svg.append('</svg>')
    return "\n".join(svg)

def main():
    # 1. Update progress.svg
    svg_content = render_svg()
    progress_svg = ROOT / "progress.svg"
    progress_svg.write_text(svg_content)
    print(f"Generated {progress_svg}")

    # 2. Update badges
    anims_s = summarize(ANIMATION_CATEGORIES)
    items_s = summarize(ATTACHABLE_CATEGORIES)
    parity_s = summarize(PARITY_SUBSYSTEMS)

    (ROOT / "badge-anims.svg").write_text(make_badge("animations", anims_s["percent"]))
    (ROOT / "badge-items.svg").write_text(make_badge("3d items", items_s["percent"]))
    (ROOT / "badge-parity.svg").write_text(make_badge("engine parity", parity_s["percent"]))
    print("Generated badges")

    # 3. Update PROGRESS.md
    md_content = render_markdown()
    progress_md = ROOT / "PROGRESS.md"
    progress_md.write_text(md_content)
    print(f"Generated {progress_md}")

    # 4. Update progress-testing.svg and badge-testing.svg for user testing
    test_panels = parse_testing_markdown()
    if test_panels:
        testing_svg = render_testing_svg(test_panels)
        (ROOT / "progress-testing.svg").write_text(testing_svg)
        print("Generated progress-testing.svg")

        total_done = sum(p["done"] for p in test_panels)
        total_tests = sum(p["done"] + p["todo"] for p in test_panels)
        total_pct = round(100 * total_done / total_tests, 1) if total_tests > 0 else 0.0
        (ROOT / "badge-testing.svg").write_text(make_badge("user testing", total_pct))
        print(f"Generated badge-testing.svg ({total_pct}%)")

if __name__ == "__main__":
    main()
