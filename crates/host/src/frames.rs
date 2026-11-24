//! The frames' build a manifest names.

use serde::{Deserialize, Serialize};

use crate::frames_hash::FramesHash;

/// The frames' build: its folder, `frames/<hash>/`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Frames {
    /// The build's hash.
    pub hash: FramesHash,
}
