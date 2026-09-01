# Hybrid 3D Renderer

A real-time 3D rendering engine written in Rust on top of [wgpu](https://github.com/gfx-rs/wgpu). It blends a traditional forward geometry pipeline with GPU compute-driven lighting and environmental techniques, with plans to incorporate Signed Distance Fields (SDF) for shadows, ambient occlusion, and global illumination.

> **[Launch WebGPU Model Viewer Demo](https://model-viewer.artuh92.workers.dev/)**  
> *(Requires a WebGPU-compatible browser: Chrome 113+, Edge 113+, Safari 18+ on macOS/iOS, or Chrome 121+ on Android)*

## Features

### Core Capabilities
- **PBR Rendering:** Physically-based material pipeline (metallic-roughness workflow, base color, normal, occlusion, emissive, and clearcoat extensions)
- **Environment & Lighting:** High-quality Image-Based Lighting (IBL) powered by HDR skydomes
- **Asset Import:** Full glTF 2.0 model loading support
- **Interactive Cameras:** Built-in Orbit camera controls
- **Developer Workflow:** Live shader hot-reloading (`shader-hot-reload` feature)

### Technical & Graphics Details
- Forward rendering pipeline
- Full GPU-computed IBL generation via compute shaders: equirectangular-to-cubemap conversion, diffuse irradiance convolution, specular prefiltering, and mipmap generation

## Prerequisites

- **Rust toolchain**, edition 2024 (Rust 1.85+)
- **Python 3.x** — only needed for the asset management scripts

## System Requirements & Target Backends

- **Primary Target:** Cross-platform GPU execution across all major backends via `wgpu`, with rendering pipeline optimizations designed specifically around **Vulkan** architecture.
- **Secondary Targets:** WebGPU / Web compilation.

## Getting started

1. Clone the repository:
   ```bash
   git clone https://github.com/SergArtuh/hybrid-renderer.git
   cd hybrid-renderer
   ```
2. Fetch the assets (3D models/textures, ~500 MB — not stored in git):
   ```bash
   python scripts/fetch_assets.py
   ```
   This is a no-op if `./assets` already exists and is non-empty.
3. (Desktop) Run an example:
   ```bash
   cargo run --example model_viewer
   ```
   (Web) Run an example:
   ```bash
   python scripts/build.py
   python scripts/serve.py
   ```

## Asset Management

Large 3D models and textures are kept out of the git repository to maintain a clean history. They are managed and served through two channels:

- **Cloudflare R2 Storage** — Hosted at [pub-227d210eb508481688aaf1983484a58a.r2.dev](https://pub-227d210eb508481688aaf1983484a58a.r2.dev). Primarily serves high-bandwidth assets and texture suites directly to the WebGPU/WASM builds for fast in-browser loading (also used by desktop builds).
- **GitHub Releases** — Used for offline distribution, archival storage, and manual setup of sample assets.

### Fetching assets

```bash
python scripts/fetch_assets.py
```

Downloads `engine_assets.zip` from the `assets-v1.0.0` release and extracts it into `./assets`. Skips the download if `./assets` already exists and is non-empty. If the release is on a private repository, set the `GITHUB_TOKEN` environment variable before running the script.

### Publishing updated assets

Requires the [GitHub CLI](https://cli.github.com/) (`gh`), authenticated once via `gh auth login`.

```bash
python scripts/publish_assets.py
```

## Future plans

- **Distance Field Illumination** — shadows, AO, GI, based on distance fields
- **Progressive Asset Loading** — streaming model LODs and textures asynchronously for fast initial rendering
