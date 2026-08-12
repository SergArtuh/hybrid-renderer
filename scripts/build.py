import subprocess
import sys


def run_command(command: list[str]):
  print(f"\n[EXEC] {' '.join(command)}")
  try:
    subprocess.run(command, check=True)
  except subprocess.CalledProcessError as e:
    print(f"[ERROR] Build aborted (code {e.returncode})")
    sys.exit(e.returncode)


def main():
  print("[INFO] Starting WASM example build...")

  run_command([
      "cargo",
      "build",
      "--target",
      "wasm32-unknown-unknown",
      "--release",
      "--example",
      "simple_sphere",
  ])

  run_command([
      "wasm-bindgen",
      "target/wasm32-unknown-unknown/release/examples/simple_sphere.wasm",
      "--out-dir",
      "web/pkg",
      "--target",
      "web",
      "--no-typescript",
  ])

  print("\n[SUCCESS] Build completed successfully! Files updated in web/pkg")


if __name__ == "__main__":
  main()