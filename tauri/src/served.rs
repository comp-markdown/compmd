//! What the plugin keeps for the app: the frames' build and the scheme it serves them on.

use crate::frames::Frames;

/// The frames the app serves, and the scheme they are served on, as the app's state.
pub struct Served {
    /// The build read when the app started.
    pub frames: Frames,
    /// The scheme's name, such as `compmd`.
    pub scheme: String,
}

impl Served {
    /// The frames' `base` (`Frames::base`) on the plugin's scheme.
    #[must_use]
    pub fn base(&self) -> Option<String> {
        self.frames.base(&self.scheme)
    }
}
