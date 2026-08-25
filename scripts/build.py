import argparse
import subprocess
import sys

DEFAULT_EXAMPLE = "model_viewer"

OUTPUT_MODULE_NAME = "app"


def run_command(command: list[str]):
    print(f"\n[EXEC] {' '.join(command)}")
    try:
        subprocess.run(command, check=True)
    except subprocess.CalledProcessError as e:
        print(f"[ERROR] Build aborted (code {e.returncode})")
        sys.exit(e.returncode)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build WebGPU examples to WASM"
    )
    parser.add_argument(
        "example",
        nargs="?",
        default=DEFAULT_EXAMPLE,
        help=f"Example name to build (default: {DEFAULT_EXAMPLE})",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    example_name = args.example

    print(f"[INFO] Starting WASM build for example: '{example_name}'...")

    # 1. Компиляция Rust бинарника под target wasm32
    run_command([
        "cargo",
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--release",
        "--example",
        example_name,
    ])

    wasm_input_path = (
        f"target/wasm32-unknown-unknown/release/examples/{example_name}.wasm"
    )

    run_command([
        "wasm-bindgen",
        wasm_input_path,
        "--out-dir",
        "web/pkg",
        "--out-name",
        OUTPUT_MODULE_NAME,
        "--target",
        "web",
        "--no-typescript",
    ])

    print(
        f"\n[SUCCESS] '{example_name}' successfully compiled to web/pkg/{OUTPUT_MODULE_NAME}.js!"
    )


if __name__ == "__main__":
    main()