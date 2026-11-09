//! The frames the crate brings (docs/tauri-plugin.md, decision 2): the build `build.rs` found in `frames/`, embedded,
//! with its hash; none when the crate was built without them.

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
