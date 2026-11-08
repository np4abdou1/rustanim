use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Instant, SystemTime};

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_walk_speed_mult() -> f32 {
    0.55
}

fn default_sprint_speed_mult() -> f32 {
    0.65
}

fn default_1_0() -> f32 {
    1.0
}

fn default_0_0() -> f32 {
    0.0
}

fn default_walk_body_pitch() -> f32 {
    2.3
}

fn default_walk_body_yaw() -> f32 {
    9.6
}

fn default_walk_body_roll() -> f32 {
    -3.0
}

fn default_bounce() -> f32 {
    1.0
}

fn default_sway() -> f32 {
    1.0
}

fn default_leg_swing() -> f32 {
    1.0
}

fn default_arm_swing() -> f32 {
    0.8
}

fn default_arm_carry_pitch() -> f32 {
    0.0
}

fn default_sprint_pitch() -> f32 {
    21.8
}

fn default_sprint_bounce() -> f32 {
    1.0
}

fn default_sprint_leg_swing() -> f32 {
    1.0
}

fn default_sprint_arm_swing() -> f32 {
    1.15
}

fn default_crouch_pitch() -> f32 {
    25.8
}

fn default_crouch_body_y() -> f32 {
    3.2
}

fn default_crouch_body_z() -> f32 {
    2.0
}

/// Live hot-reload configuration for Bedrock Actions & Stuff animations and transforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustanimConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub player_animations: bool,
    #[serde(default = "default_true")]
    pub mob_animations: bool,
    #[serde(default = "default_true")]
    pub first_person: bool,
    #[serde(default = "default_false")]
    pub debug_hud: bool,

    // Walk settings
    #[serde(default = "default_walk_speed_mult")]
    pub walk_speed_mult: f32,
    #[serde(default = "default_walk_body_pitch")]
    pub walk_body_pitch_deg: f32,
    #[serde(default = "default_walk_body_yaw")]
    pub walk_body_yaw_deg: f32,
    #[serde(default = "default_walk_body_roll")]
    pub walk_body_roll_deg: f32,
    #[serde(default = "default_bounce")]
    pub walk_bounce_mult: f32,
    #[serde(default = "default_sway")]
    pub walk_hip_sway: f32,
    #[serde(default = "default_leg_swing")]
    pub walk_leg_swing: f32,
    #[serde(default = "default_arm_swing")]
    pub walk_arm_swing: f32,
    #[serde(default = "default_arm_carry_pitch")]
    pub walk_arm_carry_pitch: f32,

    // Sprint settings
    #[serde(default = "default_sprint_speed_mult")]
    pub sprint_speed_mult: f32,
    #[serde(default = "default_sprint_pitch")]
    pub sprint_body_pitch_deg: f32,
    #[serde(default = "default_sprint_bounce")]
    pub sprint_bounce_mult: f32,
    #[serde(default = "default_sprint_leg_swing")]
    pub sprint_leg_swing: f32,
    #[serde(default = "default_sprint_arm_swing")]
    pub sprint_arm_swing: f32,

    // Crouch settings
    #[serde(default = "default_crouch_pitch")]
    pub crouch_body_pitch_deg: f32,
    #[serde(default = "default_crouch_body_y")]
    pub crouch_body_y: f32,
    #[serde(default = "default_crouch_body_z")]
    pub crouch_body_z: f32,

    // First person viewmodel
    #[serde(default = "default_0_0")]
    pub fp_offset_x: f32,
    #[serde(default = "default_0_0")]
    pub fp_offset_y: f32,
    #[serde(default = "default_0_0")]
    pub fp_offset_z: f32,
    #[serde(default = "default_1_0")]
    pub fp_bob_mult: f32,
    #[serde(default = "default_1_0")]
    pub fp_sway_mult: f32,

    // Mob multipliers
    #[serde(default = "default_1_0")]
    pub mob_zombie_sway: f32,
    #[serde(default = "default_1_0")]
    pub mob_creeper_sway: f32,
    #[serde(default = "default_1_0")]
    pub mob_spider_crawl: f32,
    #[serde(default = "default_1_0")]
    pub mob_quadruped_stride: f32,
}

impl Default for RustanimConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            player_animations: true,
            mob_animations: true,
            first_person: true,
            debug_hud: false,

            walk_speed_mult: 1.0,
            walk_body_pitch_deg: 2.3,
            walk_body_yaw_deg: 9.6,
            walk_body_roll_deg: -3.0,
            walk_bounce_mult: 1.0,
            walk_hip_sway: 1.0,
            walk_leg_swing: 1.0,
            walk_arm_swing: 0.8,
            walk_arm_carry_pitch: 0.0,

            sprint_speed_mult: 1.0,
            sprint_body_pitch_deg: 21.8,
            sprint_bounce_mult: 1.0,
            sprint_leg_swing: 1.0,
            sprint_arm_swing: 1.15,

            crouch_body_pitch_deg: 25.8,
            crouch_body_y: 3.2,
            crouch_body_z: 2.0,

            fp_offset_x: 0.0,
            fp_offset_y: 0.0,
            fp_offset_z: 0.0,
            fp_bob_mult: 1.0,
            fp_sway_mult: 1.0,

            mob_zombie_sway: 1.0,
            mob_creeper_sway: 1.0,
            mob_spider_crawl: 1.0,
            mob_quadruped_stride: 1.0,
        }
    }
}

static LIVE_CONFIG: RwLock<Option<(RustanimConfig, SystemTime)>> = RwLock::new(None);
static CUSTOM_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
static LAST_CHECK_TIME: AtomicU64 = AtomicU64::new(0);

/// Set custom config path override
pub fn set_config_path<P: AsRef<Path>>(path: P) {
    if let Ok(mut lock) = CUSTOM_PATH.write() {
        *lock = Some(path.as_ref().to_path_buf());
    }
}

/// Resolves candidates for the config file location
fn resolve_config_path() -> PathBuf {
    if let Ok(lock) = CUSTOM_PATH.read() {
        if let Some(p) = lock.as_ref() {
            return p.clone();
        }
    }
    if let Some(val) = std::env::var_os("RUSTANIM_CONFIG") {
        return PathBuf::from(val);
    }
    // Check known instance directories
    let rustani_path = PathBuf::from("/home/joyboy/.minecraft/instances/rustani/rustanim_config.json");
    if rustani_path.parent().map(|p| p.exists()).unwrap_or(false) {
        return rustani_path;
    }
    PathBuf::from("rustanim_config.json")
}

/// Retrieves the live configuration, checking for file modifications at most every 100ms.
pub fn get_live_config() -> RustanimConfig {
    let now = Instant::now();
    // Simple rate limiting: 100ms
    static START_INSTANT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let start = START_INSTANT.get_or_init(Instant::now);
    let now_ms = now.duration_since(*start).as_millis() as u64;
    let last = LAST_CHECK_TIME.load(Ordering::Relaxed);

    if now_ms >= last + 100 {
        LAST_CHECK_TIME.store(now_ms, Ordering::Relaxed);
        let path = resolve_config_path();

        if !path.exists() {
            // Write default config file for easy live editing
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let def = RustanimConfig::default();
            if let Ok(json) = serde_json::to_string_pretty(&def) {
                let _ = fs::write(&path, json);
            }
        }

        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                let need_reload = match LIVE_CONFIG.read() {
                    Ok(guard) => match &*guard {
                        Some((_, cached_time)) => *cached_time != mtime,
                        None => true,
                    },
                    Err(_) => false,
                };

                if need_reload {
                    if let Ok(contents) = fs::read_to_string(&path) {
                        match serde_json::from_str::<RustanimConfig>(&contents) {
                            Ok(parsed) => {
                                eprintln!(">>> [RUSTANIM] Live hot-reload updated from {}", path.display());
                                if let Ok(mut guard) = LIVE_CONFIG.write() {
                                    *guard = Some((parsed, mtime));
                                }
                            }
                            Err(e) => {
                                eprintln!(">>> [RUSTANIM] Warning: JSON syntax error in {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Ok(guard) = LIVE_CONFIG.read() {
        if let Some((cfg, _)) = &*guard {
            return cfg.clone();
        }
    }

    RustanimConfig::default()
}

/// Saves the configuration to the active config path and updates the in-memory cache.
pub fn save_live_config(cfg: &RustanimConfig) -> std::io::Result<()> {
    let path = resolve_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&path, json)?;
    if let Ok(meta) = fs::metadata(&path) {
        if let Ok(mtime) = meta.modified() {
            if let Ok(mut guard) = LIVE_CONFIG.write() {
                *guard = Some((cfg.clone(), mtime));
            }
        }
    }
    Ok(())
}
