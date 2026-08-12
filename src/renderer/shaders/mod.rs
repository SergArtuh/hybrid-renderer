pub mod embedded;

use std::path::Path;

use anyhow::Context;

/// Loads WGSL source for a shader path. On native targets this reads the file
/// from disk (enabling shader hot-reload); on wasm32 there is no filesystem,
/// so the source is pulled from the embedded copies baked in at compile time.
pub fn load_shader_source(path: &Path) -> anyhow::Result<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .with_context(|| format!("Invalid shader path: {:?}", path))?;

        embedded::get_embedded_shader(name)
            .map(str::to_string)
            .with_context(|| format!("No embedded shader source for {:?}", path))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read shader file at {:?}", path))
    }
}
