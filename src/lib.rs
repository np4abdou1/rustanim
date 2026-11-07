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
    BonePose, CanonicalAnimState, ChannelType, FramePose, Keyframe, KeyframeChannel, KeyframePoint,
    LerpMode, LoopMode,
};
pub use attachable::{
    compute_underwater_refraction_matrix, compute_viewmodel_mainhand_base_offset,
    compute_viewmodel_matrix, compute_viewmodel_offhand_base_offset, compute_viewmodel_projection,
    is_viewmodel_vertex_safe, AttachableDefinition, AttachableMountPoint, BedrockAttachableFile,
    ItemTransform3D, ReplayCameraPath, ShoulderCamera, StereoRigVr, ThirdPersonCamera,
};
pub use mcpack::{McpackArchive, McpackError};
pub use molang::{ExecutionContext, MolangBytecode, MolangContext, MolangEngine, MolangExpr, MolangValue};
pub use rig::{
    ArmorCubeInflation, BoneHierarchy, BoneTransform, CapePhysicsRig, PlayerModelType,
    RelativePivotRig, SecondaryMotionSpring,
};
