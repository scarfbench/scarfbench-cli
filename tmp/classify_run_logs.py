from __future__ import annotations

import re
from pathlib import Path

BASE = Path(
    "/Users/rkrsn/workspace/scarfbench/conversions-latest-conversions-march-2026/test"
)


def container_section(log: str) -> str:
    """Return the slice of the log after the first `docker run -d` line,
    representing what the running container emitted.
    Returns an empty string when the container was never started."""
    pos = log.find("docker run -d")
    if pos == -1:
        return ""
    after = log[pos:]
    nl = after.find("\n")
    if nl == -1:
        return after
    return after[nl + 1 :]


def analyze_compile(log: str) -> str:
    # ── 1. Unambiguous Java compiler errors ──────────────────────────────────
    if "COMPILATION ERROR" in log:
        return "FALSE"

    if "cannot find symbol" in log:
        return "FALSE"

    if re.search(r"package [\w.]+ does not exist", log):
        return "FALSE"

    if re.search(r"\[ERROR\] Failed to execute goal.*maven-compiler-plugin", log):
        return "FALSE"

    # ── 2. Target-framework plugin missing from pom.xml ──────────────────────
    if "NoPluginFoundForPrefixException" in log or "No plugin found for prefix" in log:
        return "FALSE"

    # ── 3. Dependency resolution failure ─────────────────────────────────────
    if (
        "Could not resolve dependencies" in log
        or "Failed to collect dependencies" in log
    ):
        return "FALSE"

    # ── 4. BUILD FAILURE inside the running container ────────────────────────
    container = container_section(log)
    if "BUILD FAILURE" in container:
        return "FALSE"

    # ── 5. Makefile build-step failure ────────────────────────────────────────
    if "make: *** [build]" in log:
        return "FALSE"

    # ── 6. Framework started in container → compile must have succeeded ───────
    if (
        "Listening on:" in container
        or " seconds (process running for" in container
        or "CWWKF0011I" in container
        or "ready to run a smarter planet" in container
        or "BUILD SUCCESS" in container
    ):
        return "TRUE"

    # ── 7. No conclusive evidence ─────────────────────────────────────────────
    return "UNK"


def analyze_deploy(log: str, compile_ok: str) -> str:
    # ── 1. Cascade from compile ───────────────────────────────────────────────
    if compile_ok == "FALSE":
        return "FALSE"

    container = container_section(log)

    # ── 2. Tests reached execution stage → deploy was healthy ────────────────
    has_test_evidence = (
        "PASSED" in container
        or "FAILED smoke.py" in container
        or "short test summary info" in container
        or bool(re.search(r"=+\s+\d+ (?:passed|failed|error)", container))
    )
    if has_test_evidence:
        return "TRUE"

    # ── 3. Hard in-container failure signals ─────────────────────────────────
    has_app_exception = (
        "CWWKZ0002E" in container
        or "CNTR4002E" in container
        or "CNTR0201E" in container
        or "StateChangeException" in container
    )

    curl_fails = container.count("curl: (7)")
    has_persistent_conn_failure = curl_fails >= 3 or (
        "Connection refused" in container and curl_fails > 0
    )

    if has_app_exception or has_persistent_conn_failure:
        return "FALSE"

    if "container exited" in container:
        return "FALSE"

    # ── 4. Harness-level deploy errors ────────────────────────────────────────
    if "make: *** [up]" in log or "make: *** [deploy]" in log:
        return "FALSE"

    if "pull access denied" in container:
        return "FALSE"

    if "container name" in container and "already in use" in container:
        return "FALSE"

    if "Terminated: 15" in log:
        return "FALSE"

    # ── 5. Readiness signals (only trusted when no hard failures above) ───────
    if (
        "pplication started and ready." in container
        or "Listening on:" in container
        or "CWWKF0011I" in container
        or "ready to run a smarter planet" in container
    ):
        return "TRUE"

    # ── 6. Log truncated before deploy concluded ──────────────────────────────
    if "waiting for app to start..." in log:
        after_wait = log.split("waiting for app to start...")[-1]
        if (
            "pplication started and ready" not in after_wait
            and "PASSED" not in after_wait
            and "FAILED" not in after_wait
            and "Listening on:" not in after_wait
        ):
            return "UNK"  # ValidationTruncated

    if compile_ok == "TRUE":
        return "UNK"

    return "UNK"


def analyze_tests(log: str, deploy_ok: str) -> str:
    # ── 1. Cascade from deploy ────────────────────────────────────────────────
    if deploy_ok == "FALSE":
        return "UNK"

    # ── 2. Parse pytest summary – use the LAST match ──────────────────────────
    summaries = list(
        re.finditer(
            r"=+ (.*?(?:passed|failed|error).*?) =+",
            log,
            re.IGNORECASE | re.DOTALL,
        )
    )

    if summaries:
        summary = summaries[-1].group(1)

        passed_m = re.search(r"(\d+) passed", summary)
        failed_m = re.search(r"(\d+) failed", summary)
        error_m = re.search(r"(\d+) error", summary)

        passed = int(passed_m.group(1)) if passed_m else 0
        failed = int(failed_m.group(1)) if failed_m else 0
        errors = int(error_m.group(1)) if error_m else 0

        total = passed + failed + errors
        if total > 0:
            pct = round(passed / total * 100.0)
            return str(pct)

    # ── 3. Fallback: make-level test error codes ───────────────────────────────
    if "make: *** [test] Error 137" in log:
        return "UNK"

    if "make: *** [test] Error 1" in log:
        return "0"

    if deploy_ok == "TRUE":
        return "UNK"

    return "UNK"


def main() -> None:
    logs = sorted(BASE.rglob("run.log"))
    print(f"log_count={len(logs)}")
    for path in logs:
        log = path.read_text(errors="replace")
        compile_ok = analyze_compile(log)
        deploy_ok = analyze_deploy(log, compile_ok)
        test_ok = analyze_tests(log, deploy_ok)
        label = (
            str(path).replace(str(BASE) + "/", "").replace("/validation/run.log", "")
        )
        print(f"{label}: compile={compile_ok} deploy={deploy_ok} test={test_ok}")


if __name__ == "__main__":
    main()
