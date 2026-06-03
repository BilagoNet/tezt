#!/usr/bin/env python3
"""Deterministic test-suite generator for benchmarking tezt vs pytest.

Generates synthetic test suites whose files can be executed by either
runner. Two styles:

  trivial : every test is a plain function with pure-arithmetic asserts.
            With --flavor plain (the default) the files contain NO
            imports at all, so the *exact same files* run under both
            tezt and pytest -- the fairest possible comparison.

  mixed   : a more realistic shape -- 70% trivial functions,
            15% parametrized tests (5 cases each), 10% fixture-using
            tests (one module-scoped fixture per file that has them),
            5% class-based tests. Parametrize/fixture APIs differ
            between runners, so mixed requires --flavor pytest or
            --flavor tezt and you generate the suite twice (once per
            flavor) into different directories.

Everything is deterministic: same arguments -> byte-identical output.
All generated tests pass (asserts are tautologies built from seeded
arithmetic), so a full run must exit 0 under both runners.

Output: prints the absolute suite path and the nominal test-case count
(accounting for parametrize expansion) on stdout, machine-parseable:

    suite_path=/abs/path
    test_cases=12345
"""

import argparse
import os
import shutil
import sys

PARAM_CASES = 5  # cases per parametrized test


def _arith(rng_a: int, rng_b: int) -> str:
    """Return a deterministic always-true assert expression."""
    a = (rng_a * 1103515245 + 12345) % 1000
    b = (rng_b * 69069 + 1) % 1000
    return f"assert {a} + {b} == {a + b}"


def _trivial_test(file_idx: int, test_idx: int) -> str:
    body = "\n".join(
        "    " + _arith(file_idx * 100 + test_idx, test_idx * 7 + k)
        for k in range(3)
    )
    return f"def test_trivial_{test_idx:04d}():\n{body}\n"


def _parametrized_test(flavor: str, file_idx: int, test_idx: int) -> str:
    cases = []
    for c in range(PARAM_CASES):
        a = (file_idx * 31 + test_idx * 7 + c) % 500
        b = (test_idx * 13 + c * 3) % 500
        cases.append(f"({a}, {b}, {a + b})")
    case_list = ", ".join(cases)
    mod = "pytest" if flavor == "pytest" else "tezt"
    return (
        f'@{mod}.mark.parametrize("a,b,expected", [{case_list}])\n'
        f"def test_param_{test_idx:04d}(a, b, expected):\n"
        f"    assert a + b == expected\n"
    )


def _fixture_def(flavor: str, file_idx: int) -> str:
    mod = "pytest" if flavor == "pytest" else "tezt"
    val = (file_idx * 17 + 3) % 1000
    return (
        f'@{mod}.fixture(scope="module")\n'
        f"def shared_value():\n"
        f"    return {val}\n"
    )


def _fixture_test(file_idx: int, test_idx: int) -> str:
    val = (file_idx * 17 + 3) % 1000
    return (
        f"def test_fixture_{test_idx:04d}(shared_value):\n"
        f"    assert shared_value == {val}\n"
        f"    assert shared_value * 2 == {val * 2}\n"
    )


def _class_test(file_idx: int, test_idx: int) -> str:
    a = (file_idx + test_idx * 11) % 300
    return (
        f"class TestGroup{test_idx:04d}:\n"
        f"    def test_method_a(self):\n"
        f"        assert {a} * 2 == {a * 2}\n"
        f"    def test_method_b(self):\n"
        f"        assert {a} - {a} == 0\n"
    )


def _gen_file(style: str, flavor: str, file_idx: int, n_tests: int):
    """Return (source_text, case_count) for one test file."""
    chunks = []
    cases = 0
    needs_import = False
    fixture_emitted = False

    if style == "trivial":
        for t in range(n_tests):
            chunks.append(_trivial_test(file_idx, t))
            cases += 1
    else:  # mixed
        for t in range(n_tests):
            # Deterministic kind selection: 70/15/10/5 split via bucket.
            bucket = (file_idx * 37 + t * 101 + 13) % 20
            if bucket < 14:  # 70% trivial
                chunks.append(_trivial_test(file_idx, t))
                cases += 1
            elif bucket < 17:  # 15% parametrized
                chunks.append(_parametrized_test(flavor, file_idx, t))
                cases += PARAM_CASES
                needs_import = True
            elif bucket < 19:  # 10% fixture-using
                if not fixture_emitted:
                    chunks.insert(0, _fixture_def(flavor, file_idx))
                    fixture_emitted = True
                    needs_import = True
                chunks.append(_fixture_test(file_idx, t))
                cases += 1
            else:  # 5% class-based (2 methods each)
                chunks.append(_class_test(file_idx, t))
                cases += 2

    header = ""
    if needs_import:
        mod = "pytest" if flavor == "pytest" else "tezt"
        header = f"import {mod}\n\n"
    return header + "\n\n".join(chunks) + "\n", cases


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--out", required=True, help="output directory for the suite")
    p.add_argument("--files", type=int, required=True, help="number of test files")
    p.add_argument("--tests-per-file", type=int, required=True,
                   help="number of test definitions per file")
    p.add_argument("--style", choices=("trivial", "mixed"), default="trivial")
    p.add_argument("--flavor", choices=("plain", "pytest", "tezt"), default="plain",
                   help="which runner's API to import for parametrize/fixtures; "
                        "'plain' emits no imports (only valid with --style trivial)")
    args = p.parse_args(argv)

    if args.files < 1 or args.tests_per_file < 1:
        p.error("--files and --tests-per-file must be >= 1")
    if args.style == "mixed" and args.flavor == "plain":
        p.error("--style mixed requires --flavor pytest or --flavor tezt "
                "(parametrize/fixture APIs differ between runners); "
                "generate the suite twice, once per flavor")

    out = os.path.abspath(args.out)
    if os.path.isdir(out):
        # Refuse to nuke anything that doesn't look like one of ours.
        entries = os.listdir(out)
        foreign = [e for e in entries
                   if not (e.startswith("test_") and e.endswith(".py"))
                   and e not in ("__pycache__", ".pytest_cache")]
        if foreign:
            sys.exit(f"error: {out} exists and contains non-suite files "
                     f"({foreign[:3]}...); refusing to overwrite")
        shutil.rmtree(out)
    os.makedirs(out, exist_ok=True)

    total_cases = 0
    width = max(4, len(str(args.files - 1)))
    for f in range(args.files):
        src, cases = _gen_file(args.style, args.flavor, f, args.tests_per_file)
        total_cases += cases
        path = os.path.join(out, f"test_gen_{f:0{width}d}.py")
        with open(path, "w") as fh:
            fh.write(src)

    print(f"suite_path={out}")
    print(f"test_cases={total_cases}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
