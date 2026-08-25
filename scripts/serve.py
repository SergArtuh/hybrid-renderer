import os
import sys


def main():
  web_dir = "web"
  if not os.path.isdir(web_dir):
    print(f"[ERROR] Directory '{web_dir}' not found.")
    sys.exit(1)

  print(
      f"[INFO] Starting local server from directory '{web_dir}/' at"
      " http://localhost:8080"
  )
  print("[INFO] Press Ctrl+C to stop\n")

  # Run python -m http.server from inside web directory
  os.chdir(web_dir)
  os.execve(
      sys.executable,
      [sys.executable, "-m", "http.server", "8080"],
      os.environ,
  )


if __name__ == "__main__":
  main()