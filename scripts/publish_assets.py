import os
import subprocess
import sys
import zipfile
from pathlib import Path

TAG_NAME = "assets-v1.0.0"
ZIP_NAME = "engine_assets.zip"
ASSETS_DIR = Path("./assets")


def publish_assets():
    if not ASSETS_DIR.exists() or not ASSETS_DIR.is_dir():
        print(f"❌ Error: Directory '{ASSETS_DIR}' not found!")
        sys.exit(1)

    print(f"📦 Archiving '{ASSETS_DIR}' into '{ZIP_NAME}'...")

    # Create zip archive using Python's built-in zipfile (no external tools needed)
    with zipfile.ZipFile(ZIP_NAME, "w", zipfile.ZIP_DEFLATED) as zipf:
        for root, _, files in os.walk(ASSETS_DIR):
            for file in files:
                if file == ".DS_Store":
                    continue
                file_path = Path(root) / file
                arcname = file_path.relative_to(ASSETS_DIR.parent)
                zipf.write(file_path, arcname)

    print("🚀 Uploading to GitHub Releases...")

    # Check if release exists
    view_cmd = ["gh", "release", "view", TAG_NAME]
    release_exists = (
        subprocess.run(
            view_cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        ).returncode
        == 0
    )

    if release_exists:
        print(
            f"ℹ️ Release {TAG_NAME} already exists. Overwriting asset file..."
        )
        upload_cmd = ["gh", "release", "upload", TAG_NAME, ZIP_NAME, "--clobber"]
        subprocess.run(upload_cmd, check=True)
    else:
        print(f"✨ Creating new release {TAG_NAME}...")
        create_cmd = [
            "gh",
            "release",
            "create",
            TAG_NAME,
            ZIP_NAME,
            "--title",
            "Engine Demo & Test Assets",
            "--notes",
            "3D models and textures for local engine development (~500MB).",
        ]
        subprocess.run(create_cmd, check=True)

    print("🧹 Removing temporary archive...")
    if os.path.exists(ZIP_NAME):
        os.remove(ZIP_NAME)

    print("✅ Assets published successfully!")


if __name__ == "__main__":
    publish_assets()