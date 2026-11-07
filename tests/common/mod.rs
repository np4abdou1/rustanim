//! Shared test utilities, mathematical reference models, and test fixtures for `rustanim`.
#![allow(dead_code, unused_imports)]

use glam::{EulerRot, Mat4, Quat, Vec3};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Path to the authoritative resource pack archive.
pub const MCPACK_PATH: &str = "/home/ubuntu/rustani/Actions_and_Stuff_112.mcpack";

/// Helper for accessing the Actions & Stuff Bedrock mcpack archive.
pub struct McpackFixture {
    path: PathBuf,
}

impl McpackFixture {
    pub fn new() -> Option<Self> {
        let path = PathBuf::from(MCPACK_PATH);
        if path.exists() {
            Some(Self { path })
        } else {
            None
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read raw file text from within the mcpack zip archive using unzip.
    pub fn read_file_text(&self, inner_path: &str) -> Result<String, String> {
        let output = Command::new("unzip")
            .args(["-p", self.path.to_str().unwrap(), inner_path])
            .output()
            .map_err(|e| format!("Failed to execute unzip: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "unzip failed for {}: {}",
                inner_path,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| format!("Invalid UTF-8 in {}: {}", inner_path, e))
    }
}

// ---------------------------------------------------------------------------
// Canonical Vanilla Java Humanoid Rig Anchors (pixel coordinates, 16px = 1 block)
// ---------------------------------------------------------------------------

pub const JAVA_HEAD_ANCHOR: Vec3 = Vec3::new(0.0, 24.0, 0.0);
pub const JAVA_BODY_ANCHOR: Vec3 = Vec3::new(0.0, 24.0, 0.0);
pub const JAVA_RIGHT_ARM_ANCHOR: Vec3 = Vec3::new(-5.0, 22.0, 0.0);
pub const JAVA_LEFT_ARM_ANCHOR: Vec3 = Vec3::new(5.0, 22.0, 0.0);
pub const JAVA_RIGHT_LEG_ANCHOR: Vec3 = Vec3::new(-1.9, 12.0, 0.0);
pub const JAVA_LEFT_LEG_ANCHOR: Vec3 = Vec3::new(1.9, 12.0, 0.0);

// Virtual & Bedrock specific sockets
pub const VIRTUAL_WAIST_ANCHOR: Vec3 = Vec3::new(0.0, 12.0, 0.0);
pub const BEDROCK_RIGHT_ITEM_ANCHOR: Vec3 = Vec3::new(-6.0, 15.0, 1.0);
pub const BEDROCK_LEFT_ITEM_ANCHOR: Vec3 = Vec3::new(6.0, 15.0, 1.0);

// ---------------------------------------------------------------------------
// Reference Spline Evaluation (Cubic Hermite Catmull-Rom)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct KeyframePoint3D {
    pub time: f32,
    pub pre: Vec3,
    pub post: Vec3,
    pub is_catmullrom: bool,
}

impl KeyframePoint3D {
    pub fn continuous(time: f32, value: Vec3, is_catmullrom: bool) -> Self {
        Self {
            time,
            pre: value,
            post: value,
            is_catmullrom,
        }
    }

    pub fn discontinuous(time: f32, pre: Vec3, post: Vec3, is_catmullrom: bool) -> Self {
        Self {
            time,
            pre,
            post,
            is_catmullrom,
        }
    }
}

/// Evaluates a Catmull-Rom spline curve with non-uniform time normalization and chord reflection tangents.
pub fn evaluate_catmull_rom_spline(
    keyframes: &[KeyframePoint3D],
    time: f32,
    is_looping: bool,
    anim_length: f32,
) -> Vec3 {
    if keyframes.is_empty() {
        return Vec3::ZERO;
    }
    if keyframes.len() == 1 {
        return keyframes[0].post;
    }

    // Clamp or wrap time
    let sample_time = if is_looping && anim_length > 0.0 {
        let t = time % anim_length;
        if t < 0.0 { t + anim_length } else { t }
    } else {
        time.clamp(keyframes.first().unwrap().time, keyframes.last().unwrap().time)
    };

    // If before first keyframe
    if sample_time <= keyframes[0].time {
        return keyframes[0].pre;
    }
    // If after last keyframe
    if sample_time >= keyframes.last().unwrap().time {
        return keyframes.last().unwrap().post;
    }

    // Locate segment [i, i+1]
    let mut idx = 0;
    while idx + 1 < keyframes.len() && keyframes[idx + 1].time < sample_time {
        idx += 1;
    }

    let k_i = &keyframes[idx];
    let k_next = &keyframes[idx + 1];

    let dt = k_next.time - k_i.time;
    if dt.abs() < 1e-6 {
        return k_next.post;
    }

    let u = (sample_time - k_i.time) / dt;

    if !k_i.is_catmullrom && !k_next.is_catmullrom {
        // Linear interpolation between k_i.post and k_next.pre
        return k_i.post.lerp(k_next.pre, u);
    }

    // Determine 4 control points P_{i-1}, P_i, P_{i+1}, P_{i+2}
    let p_i = k_i.post;
    let p_next = k_next.pre;

    // Previous point P_{i-1}
    let (p_prev, dt_prev) = if idx > 0 {
        (keyframes[idx - 1].post, k_i.time - keyframes[idx - 1].time)
    } else if is_looping && keyframes.len() > 2 {
        let last = &keyframes[keyframes.len() - 2];
        (last.post, k_i.time - (last.time - anim_length))
    } else {
        // Chord reflection: P_{-1} = 2 * P_0 - P_1
        (2.0 * p_i - p_next, dt)
    };

    // Outgoing next point P_{i+2}
    let (p_after, dt_after) = if idx + 2 < keyframes.len() {
        (keyframes[idx + 2].pre, keyframes[idx + 2].time - k_next.time)
    } else if is_looping && keyframes.len() > 2 {
        let first = &keyframes[1];
        (first.pre, (first.time + anim_length) - k_next.time)
    } else {
        // Chord reflection: P_{N} = 2 * P_{N-1} - P_{N-2}
        (2.0 * p_next - p_i, dt)
    };

    // Compute tangent slopes scaled by interval dt
    // m_i = dt * (p_next - p_prev) / (t_next - t_prev)
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

    // Cubic Hermite basis
    let u2 = u * u;
    let u3 = u2 * u;

    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;

    h00 * p_i + h10 * m_i + h01 * p_next + h11 * m_next
}

// ---------------------------------------------------------------------------
// Reference Degree Trigonometry & Molang Math
// ---------------------------------------------------------------------------

#[inline]
pub fn molang_sin(deg: f32) -> f32 {
    deg.to_radians().sin()
}

#[inline]
pub fn molang_cos(deg: f32) -> f32 {
    deg.to_radians().cos()
}

#[inline]
pub fn molang_asin(val: f32) -> f32 {
    val.clamp(-1.0, 1.0).asin().to_degrees()
}

#[inline]
pub fn molang_acos(val: f32) -> f32 {
    val.clamp(-1.0, 1.0).acos().to_degrees()
}

#[inline]
pub fn molang_atan(val: f32) -> f32 {
    val.atan().to_degrees()
}

#[inline]
pub fn molang_atan2(y: f32, x: f32) -> f32 {
    y.atan2(x).to_degrees()
}

#[inline]
pub fn molang_clamp(val: f32, min: f32, max: f32) -> f32 {
    val.clamp(min, max)
}

#[inline]
pub fn molang_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
pub fn molang_hermite_blend(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    3.0 * t * t - 2.0 * t * t * t
}

#[inline]
pub fn molang_ease_out_cubic(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let factor = 1.0 - (1.0 - t).powi(3);
    start + (end - start) * factor
}

#[inline]
pub fn molang_ease_in_expo(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let factor = if t == 0.0 { 0.0 } else { 2.0f32.powf(10.0 * (t - 1.0)) };
    start + (end - start) * factor
}

#[inline]
pub fn molang_ease_in_out_sine(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let factor = -0.5 * ((std::f32::consts::PI * t).cos() - 1.0);
    start + (end - start) * factor
}

// ---------------------------------------------------------------------------
// Relative Pivot Rigging Reference Math
// ---------------------------------------------------------------------------

/// Computes the local transform matrix of a bone relative to its parent anchor:
/// M_local = Translate(A_i - A_p + delta_t) * Rotate(R) * Scale(S)
pub fn compute_relative_local_matrix(
    bone_anchor: Vec3,
    parent_anchor: Vec3,
    delta_translation: Vec3,
    rotation_deg_zyx: Vec3,
    scale: Vec3,
) -> Mat4 {
    let relative_origin = bone_anchor - parent_anchor + delta_translation;
    let rotation = Quat::from_euler(
        EulerRot::ZYX,
        rotation_deg_zyx.z.to_radians(),
        rotation_deg_zyx.y.to_radians(),
        rotation_deg_zyx.x.to_radians(),
    );
    Mat4::from_scale_rotation_translation(scale, rotation, relative_origin)
}

/// Dedicated 1st-person viewmodel projection matrix:
/// Fixed 70° vertical FOV, z_near = 0.01 to prevent near-plane weapon clipping.
pub fn compute_viewmodel_projection(aspect: f32) -> Mat4 {
    Mat4::perspective_rh(70.0f32.to_radians(), aspect, 0.01, 100.0)
}

/// Standard 1st-person camera eye base offset matrix for the main hand item:
/// Offset [+0.56, -0.52, -0.72] in camera eye space.
pub fn compute_viewmodel_mainhand_base_offset() -> Mat4 {
    Mat4::from_translation(Vec3::new(0.56, -0.52, -0.72))
}
