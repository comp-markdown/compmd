//! What a plan is made with: the primitives the client draws and the drawing's settings, as they are when the client
//! asks, so a change of either applies at the next request.

use super::primitive::Primitives;
use crate::settings::RenderSettings;

/// The primitives a plan may use and the settings it follows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Planning {
    /// What the client declared it draws; nothing for a client that declared no drawing.
    pub(crate) primitives: Primitives,
    /// The `compmd.render` settings.
    pub(crate) render: RenderSettings,
}
