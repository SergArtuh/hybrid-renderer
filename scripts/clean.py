import os
import shutil


def main():
  paths_to_remove = ["web/pkg", "target/wasm32-unknown-unknown"]

  print("[INFO] Cleaning build artifacts...")
  for path in paths_to_remove:
    if os.path.exists(path):
      if os.path.isdir(path):
        shutil.rmtree(path)
        print(f"  Removed folder: {path}")
    else:
      print(f"  Skipped (not found): {path}")

  print("[SUCCESS] Cleaning complete.")


if __name__ == "__main__":
  main()