//! # rustanim
//! High-performance native Rust Bedrock .mcpack animation and 3D attachable engine for Horizon.

pub mod anim;
pub mod attachable;
pub mod mcpack;
pub mod molang;
pub mod rig;

pub use anim::{
    AnimChannel, AnimCoord, AnimVector, AnimationClip, AnimationController, AnimationFile,
    BedrockAnimationFile, BedrockAnimationClip, BedrockBoneAnimation, BoneAnimation, BoneChannel,
    BonePose, ChannelType, FramePose, Keyframe, KeyframeChannel, KeyframePoint, LerpMode, LoopMode,
};
pub use attachable::{AttachableDefinition, AttachableMountPoint, ItemTransform3D};
pub use mcpack::{McpackArchive, McpackError};
pub use molang::{MolangContext, MolangEngine, MolangExpr, MolangValue};
pub use rig::{BoneHierarchy, BoneTransform, RelativePivotRig};
