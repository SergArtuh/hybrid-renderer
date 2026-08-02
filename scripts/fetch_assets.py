import gc
import os
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

ASSETS_DIR = Path("./assets")
TAG_NAME = "assets-v1.0.0"
ZIP_NAME = "engine_assets.zip"

GITHUB_USER = "SergArtuh"
GITHUB_REPO = "hybrid-renderer"

DOWNLOAD_URL = f"https://github.com/{GITHUB_USER}/{GITHUB_REPO}/releases/download/{TAG_NAME}/{ZIP_NAME}"


def fetch_assets():
    # Skip if assets directory exists and is not empty
    if ASSETS_DIR.exists() and any(ASSETS_DIR.iterdir()):
        print(
            f"✅ Directory '{ASSETS_DIR}' already exists and is not empty. Skipping download."
        )
        return

    print(
        f"🔍 Directory '{ASSETS_DIR}' is missing or empty. Downloading assets (~500 MB)..."
    )

    try:
        print("⬇️ Downloading archive from GitHub Releases...")
        req = urllib.request.Request(
            DOWNLOAD_URL, headers={"User-Agent": "Python-Assets-Fetcher"}
        )

        # Handle private repository authentication via environment variable
        token = os.environ.get("GITHUB_TOKEN")
        if token:
            req.add_header("Authorization", f"token {token}")

        with urllib.request.urlopen(req) as response, open(
            ZIP_NAME, "wb"
        ) as out_file:
            out_file.write(response.read())

    except urllib.error.HTTPError as e:
        if e.code == 404:
            print(f"❌ Failed to download assets (404 Not Found).")
            print("💡 Possible reasons:")
            print(
                f"   1. The repository '{GITHUB_USER}/{GITHUB_REPO}' is PRIVATE."
            )
            print(
                "      -> Set GITHUB_TOKEN environment variable or make the repo public."
            )
            print(
                f"   2. The file '{ZIP_NAME}' is not attached to release '{TAG_NAME}'."
            )
        else:
            print(f"❌ HTTP Error: {e.code} {e.reason}")
        return
    except Exception as e:
        print(f"❌ Failed to download assets: {e}")
        return

    print("📦 Extracting assets...")
    with zipfile.ZipFile(ZIP_NAME, "r") as zip_ref:
        zip_ref.extractall(".")

    # Force garbage collection to instantly release file handle on Windows
    gc.collect()

    print("🧹 Removing downloaded archive...")
    if os.path.exists(ZIP_NAME):
        try:
            os.remove(ZIP_NAME)
        except PermissionError:
            print(
                f"⚠️ Warning: Couldn't immediately delete {ZIP_NAME} due to OS file lock."
            )

    print(f"🎉 Assets successfully downloaded to '{ASSETS_DIR}'!")


if __name__ == "__main__":
    fetch_assets()