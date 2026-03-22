from __future__ import annotations

import re
from collections import Counter, defaultdict
from pathlib import Path

BASE = Path(
    "/Users/rkrsn/workspace/scarfbench/conversions-latest-conversions-march-2026/test"
)

LOG_NAMES = ("run.log", "run.logs")


def find_logs(base: Path) -> list[Path]:
    logs: list[Path] = []
    for name in LOG_NAMES:
        logs.extend(base.rglob(name))
    return sorted(set(logs))


def read_text(path: Path) -> str:
    return path.read_text(errors="replace")


def detect_patterns(log: str) -> dict[str, bool]:
    return {
        "build_success": "BUILD SUCCESS" in log,
        "build_failure": "BUILD FAILURE" in log,
        "compilation_error": "COMPILATION ERROR" in log,
        "cannot_find_symbol": "cannot find symbol" in log,
        "package_missing": "package" in log and "does not exist" in log,
        "dependency_resolution_failed": (
            "Could not resolve dependencies" in log
            or "Failed to collect dependencies" in log
        ),
        "plugin_not_found": (
            "NoPluginFoundForPrefixException" in log
            or "No plugin found for prefix" in log
        ),
        "make_build_error": "make: *** [build]" in log,
        "make_up_error": "make: *** [up]" in log,
        "make_test_error_1": "make: *** [test] Error 1" in log,
        "make_test_error_137": "make: *** [test] Error 137" in log,
        "container_started": "docker run -d" in log,
        "ready_signal": "pplication started and ready." in log,
        "waiting_for_start": "waiting for app to start..." in log,
        "liberty_app_start_exception": "CWWKZ0002E" in log,
        "generic_failed_to_start": "failed to start" in log,
        "state_change_exception": "StateChangeException" in log,
        "cntr0201e": "CNTR0201E" in log,
        "container_exited": "container exited" in log,
        "connection_refused": "Connection refused" in log,
        "curl_7": "curl: (7)" in log,
        "pull_access_denied": "pull access denied" in log,
        "container_name_conflict": (
            "container name" in log and "already in use" in log
        ),
        "terminated_15": "Terminated: 15" in log,
        "short_test_summary_info": "short test summary info" in log,
        "pytest_passed_token": "PASSED" in log,
        "pytest_failed_smoke": "FAILED smoke.py" in log,
        "liberty_server_ready": (
            "ready to run a smarter planet" in log or "server is ready to run" in log
        ),
        "port_8080": "port 8080" in log,
        "port_9080": "port 9080" in log,
    }


def classify_compile(log: str) -> str:
    if "BUILD FAILURE" in log:
        return "FALSE"
    if "COMPILATION ERROR" in log:
        return "FALSE"
    if "cannot find symbol" in log:
        return "FALSE"
    if "package" in log and "does not exist" in log:
        return "FALSE"
    if "NoPluginFoundForPrefixException" in log or "No plugin found for prefix" in log:
        return "FALSE"
    if (
        "Could not resolve dependencies" in log
        or "Failed to collect dependencies" in log
    ):
        return "FALSE"
    if "make: *** [build]" in log:
        return "FALSE"
    if "BUILD SUCCESS" in log:
        return "TRUE"

    # Heuristic: if logs clearly progressed into in-container runtime/server startup,
    # compilation likely completed without compile-stage failure.
    if (
        "Scanning for projects..." in log
        and (
            "Application started and ready." in log
            or "CWWKF0011I" in log
            or "Started " in log
            or "Tomcat started on port" in log
            or "Listening on:" in log
        )
        and "COMPILATION ERROR" not in log
        and "cannot find symbol" not in log
        and not ("package" in log and "does not exist" in log)
    ):
        return "TRUE"

    return "UNK"


def classify_deploy(log: str, compile_ok: str) -> str:
    if compile_ok == "FALSE":
        return "FALSE"

    has_ready_signal = "pplication started and ready." in log
    has_test_output = (
        "short test summary info" in log or "PASSED" in log or "FAILED smoke.py" in log
    )
    has_test_summary = bool(
        re.search(r"=+ .*(?:passed|failed|error).*=+", log, re.IGNORECASE | re.DOTALL)
    )
    has_startup_exception = any(
        marker in log
        for marker in (
            "failed to start",
            "CWWKZ0002E",
            "StateChangeException",
            "CNTR0201E",
        )
    )
    has_smoke_connection_failure = any(
        marker in log
        for marker in (
            "curl: (7)",
            "Failed to connect to localhost port",
            "Connection refused",
        )
    )

    smoke_failed_after_ready = (
        has_ready_signal
        and has_smoke_connection_failure
        and not has_test_output
        and not has_test_summary
    )

    if "docker run -d" in log and (
        has_startup_exception or "container exited" in log or smoke_failed_after_ready
    ):
        return "FALSE"

    if has_test_summary or has_test_output:
        return "TRUE"

    if has_ready_signal:
        return "TRUE"

    if "pull access denied" in log:
        return "FALSE"

    if "container name" in log and "already in use" in log:
        return "FALSE"

    if "make: *** [up]" in log:
        return "FALSE"

    if "Terminated: 15" in log:
        return "FALSE"

    if "waiting for app to start..." in log:
        after_waiting = log.split("waiting for app to start...")[-1]
        if (
            "pplication started and ready" not in after_waiting
            and "PASSED" not in after_waiting
            and "FAILED" not in after_waiting
            and "===" not in after_waiting
        ):
            return "UNK"

    if compile_ok == "TRUE":
        return "UNK"

    return "UNK"


def classify_tests(log: str, deploy_ok: str) -> str:
    if deploy_ok == "FALSE":
        return "UNK"

    summaries = list(
        re.finditer(
            r"=+ (.*?(?:passed|failed|error).*?) =+",
            log,
            re.IGNORECASE | re.DOTALL,
        )
    )

    if summaries:
        summary = summaries[-1].group(1)

        passed_match = re.search(r"(\d+) passed", summary)
        failed_match = re.search(r"(\d+) failed", summary)
        error_match = re.search(r"(\d+) error", summary)

        passed = int(passed_match.group(1)) if passed_match else 0
        failed = int(failed_match.group(1)) if failed_match else 0
        errors = int(error_match.group(1)) if error_match else 0

        total = passed + failed + errors
        if total > 0:
            pct = round(passed / total * 100.0)
            return str(pct)

    if "make: *** [test] Error 137" in log:
        return "UNK"

    if "make: *** [test] Error 1" in log:
        return "0"

    if deploy_ok == "TRUE":
        return "UNK"

    return "UNK"


def extract_test_summary(log: str) -> str | None:
    summaries = list(
        re.finditer(
            r"=+ (.*?(?:passed|failed|error).*?) =+",
            log,
            re.IGNORECASE | re.DOTALL,
        )
    )
    if not summaries:
        return None
    return " ".join(summaries[-1].group(1).split())


def extract_key_lines(log: str) -> list[str]:
    interesting = [
        r"BUILD SUCCESS",
        r"BUILD FAILURE",
        r"COMPILATION ERROR",
        r"cannot find symbol",
        r"package .* does not exist",
        r"NoPluginFoundForPrefixException",
        r"No plugin found for prefix",
        r"Could not resolve dependencies",
        r"Failed to collect dependencies",
        r"docker run -d",
        r"waiting for app to start\.\.\.",
        r"Application started and ready\.",
        r"CWWKZ0002E",
        r"StateChangeException",
        r"CNTR0201E",
        r"container exited",
        r"Connection refused",
        r"curl: \(7\)",
        r"make: \*\*\* \[build\]",
        r"make: \*\*\* \[up\]",
        r"make: \*\*\* \[test\] Error 1",
        r"make: \*\*\* \[test\] Error 137",
        r"short test summary info",
        r"PASSED",
        r"FAILED smoke.py",
        r"port 8080",
        r"port 9080",
    ]
    pattern = re.compile("|".join(f"(?:{p})" for p in interesting))
    lines: list[str] = []
    for line in log.splitlines():
        if pattern.search(line):
            lines.append(line.strip())
    return lines[:20]


def print_counter(title: str, counter: Counter[str]) -> None:
    print(f"\n== {title} ==")
    for key, value in sorted(counter.items()):
        print(f"{key}: {value}")


def main() -> None:
    logs = find_logs(BASE)
    print(f"base={BASE}")
    print(f"log_count={len(logs)}")

    compile_counts: Counter[str] = Counter()
    deploy_counts: Counter[str] = Counter()
    test_counts: Counter[str] = Counter()
    pattern_counts: Counter[str] = Counter()

    contradictory_logs: list[Path] = []
    no_compile_signal_but_runtime: list[Path] = []
    with_test_summaries: list[tuple[Path, str]] = []
    classifications: dict[Path, tuple[str, str, str]] = {}
    grouped_by_triplet: dict[tuple[str, str, str], list[Path]] = defaultdict(list)

    for path in logs:
        log = read_text(path)
        patterns = detect_patterns(log)
        for name, present in patterns.items():
            if present:
                pattern_counts[name] += 1

        compile_ok = classify_compile(log)
        deploy_ok = classify_deploy(log, compile_ok)
        test_ok = classify_tests(log, deploy_ok)

        classifications[path] = (compile_ok, deploy_ok, test_ok)
        grouped_by_triplet[(compile_ok, deploy_ok, test_ok)].append(path)

        compile_counts[compile_ok] += 1
        deploy_counts[deploy_ok] += 1
        test_counts[test_ok] += 1

        if patterns["ready_signal"] and (
            patterns["liberty_app_start_exception"]
            or patterns["state_change_exception"]
            or patterns["cntr0201e"]
            or patterns["curl_7"]
            or patterns["connection_refused"]
        ):
            contradictory_logs.append(path)

        if compile_ok == "UNK" and (
            patterns["ready_signal"]
            or patterns["liberty_server_ready"]
            or patterns["container_started"]
        ):
            no_compile_signal_but_runtime.append(path)

        summary = extract_test_summary(log)
        if summary:
            with_test_summaries.append((path, summary))

    print_counter("compile counts", compile_counts)
    print_counter("deploy counts", deploy_counts)
    print_counter("test counts", test_counts)

    print("\n== classification groups ==")
    for key, paths in sorted(grouped_by_triplet.items()):
        print(f"{key}: {len(paths)}")

    print("\n== pattern counts ==")
    for key, value in sorted(pattern_counts.items()):
        print(f"{key}: {value}")

    print("\n== per-log classification ==")
    for path in logs:
        compile_ok, deploy_ok, test_ok = classifications[path]
        print(f"{path}: compile={compile_ok} deploy={deploy_ok} test={test_ok}")

    print("\n== contradictory logs (ready signal but later failure evidence) ==")
    if contradictory_logs:
        for path in contradictory_logs:
            print(path)
    else:
        print("none")

    print("\n== logs with runtime evidence but compile still UNK ==")
    if no_compile_signal_but_runtime:
        for path in no_compile_signal_but_runtime:
            print(path)
    else:
        print("none")

    print("\n== test summaries found ==")
    if with_test_summaries:
        for path, summary in with_test_summaries:
            print(f"{path}: {summary}")
    else:
        print("none")

    print("\n== sample key lines for each log ==")
    for path in logs:
        print(f"\n-- {path}")
        lines = extract_key_lines(read_text(path))
        if not lines:
            print("(no key lines matched)")
            continue
        for line in lines:
            print(line)


if __name__ == "__main__":
    main()
