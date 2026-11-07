//! # rustanim
//! High-performance native Rust Bedrock .mcpack animation and 3D attachable engine for Horizon.

pub mod anim;
pub mod attachable;
pub mod molang;
pub mod rig;

pub use anim::{AnimationClip, AnimationController, KeyframeChannel};
pub use attachable::{AttachableDefinition, AttachableMountPoint, ItemTransform3D};
pub use molang::{MolangContext, MolangEngine, MolangExpr, MolangValue};
pub use rig::{BoneHierarchy, BoneTransform, RelativePivotRig};
