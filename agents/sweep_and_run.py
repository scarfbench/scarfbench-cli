#!/usr/bin/env python3
import argparse, os, shlex, shutil, subprocess, sys
from pathlib import Path
from datetime import datetime

def run(cmd, cwd=None, check=True):
    print(f"$ {cmd}")
    proc = subprocess.run(cmd, shell=True, cwd=cwd)
    if check and proc.returncode != 0:
        raise RuntimeError(f"Command failed ({proc.returncode}): {cmd}")
    return proc.returncode

def ensure_devcontainer(app_dir: Path, template_dir: Path):
    dc = app_dir / ".devcontainer"
    if dc.exists():
        return
    print(f"[init] copy devcontainer template -> {dc}")
    shutil.copytree(template_dir, dc)

def main():
    ap = argparse.ArgumentParser(description="Run a specific agent for all apps under any folder that has run_<agent>.sh + prompt.txt")
    ap.add_argument("--root", required=True, help="Root to sweep (e.g., ./converted)")
    ap.add_argument("--agent", required=True, choices=["cursor","gemini","copilot"], help="Which agent to run")
    ap.add_argument("--devcontainer-template", required=True, help="Path to template .devcontainer folder")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    template = Path(args.devcontainer_template).resolve()
    if not root.exists(): sys.exit(f"root not found: {root}")
    if not (template.exists() and (template / "devcontainer.json").exists()):
        sys.exit(f"invalid devcontainer template: {template}")

    agent_script_name = f"run_{args.agent}.sh"
    batch_folders = []
    for dirpath, dirnames, filenames in os.walk(root):
        p = Path(dirpath)
        if agent_script_name in filenames and "prompt.txt" in filenames:
            batch_folders.append(p)

    if not batch_folders:
        print(f"[info] no batch folders found under {root} containing {agent_script_name} + prompt.txt")
        return 0

    batch_ts = datetime.now().strftime("%Y%m%d-%H%M%S")

    for batch in batch_folders:
        print(f"\n=== Batch: {batch} (agent={args.agent}) ===")
        prompt = batch / "prompt.txt"
        agent_script = batch / agent_script_name

        # For each immediate subdirectory = app
        for entry in sorted(batch.iterdir()):
            if not entry.is_dir():                 continue
            if entry.name in {".devcontainer","__runs__"}: continue
            if entry.name.startswith("."):         continue

            app = entry
            ensure_devcontainer(app, template)

            # sync agent script into app
            dst = app / agent_script_name
            if not dst.exists() or os.path.getmtime(agent_script) > os.path.getmtime(dst):
                print(f"[sync] copy {agent_script_name} -> {dst}")
                if not args.dry_run:
                    shutil.copy2(agent_script, dst)
                    dst.chmod(dst.stat().st_mode | 0o111)

            # bring up container
            up = f'devcontainer up --workspace-folder {shlex.quote(str(app))}'
            if args.dry_run:
                print(f"[dry-run] {up}")
            else:
                try:
                    run(up)
                except RuntimeError:
                    print("[warn] bring-up failed; retry once")
                    run(up)

            # build exec
            prompt_rel = os.path.relpath(str(prompt), start=str(app))  # e.g. '../prompt.txt'
            inner = f'./{agent_script_name} --prompt {shlex.quote(prompt_rel)} --out ./.agent_out'
            exec_cmd = f'devcontainer exec --workspace-folder {shlex.quote(str(app))} -- {inner}'

            if args.dry_run:
                print(f"[dry-run] {exec_cmd}")
            else:
                run(exec_cmd)

            # collect artifacts
            out_host = batch / "__runs__" / app.name / batch_ts
            out_host.mkdir(parents=True, exist_ok=True)
            container_out = app / ".agent_out"
            if container_out.exists():
                copy = f'rsync -a "{container_out}/" "{out_host}/"'
                if args.dry_run:
                    print(f"[dry-run] {copy}")
                else:
                    run(copy)
                    print(f"[ok] logs -> {out_host}")
            else:
                print("[warn] no .agent_out from", app)

    print("\nAll done.")
    return 0

if __name__ == "__main__":
    sys.exit(main())