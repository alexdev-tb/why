#!/usr/bin/env python3
"""Validate all YAML files in db/ match the expected schema.

Requires PyYAML: pip install pyyaml

Usage:
    python scripts/validate.py

Exit codes:
    0 - All files valid
    1 - Validation errors found
"""

import os
import sys

import yaml


REQUIRED_FIELDS = {"id", "tool", "language", "title", "explain", "fix"}
OPTIONAL_STRING_FIELDS = {"example_error", "example_code"}
OPTIONAL_LIST_FIELDS = {"tags", "exclude", "patterns"}
OPTIONAL_LIST_OF_STRINGS_FIELDS = {"links"}

ALL_KNOWN_FIELDS = (
    REQUIRED_FIELDS
    | OPTIONAL_STRING_FIELDS
    | OPTIONAL_LIST_FIELDS
    | OPTIONAL_LIST_OF_STRINGS_FIELDS
)


def validate_file(filepath):
    """Validate a single YAML file. Returns a list of error strings."""
    errors = []

    try:
        with open(filepath, "r", encoding="utf-8") as f:
            doc = yaml.safe_load(f)
    except yaml.YAMLError as e:
        return [f"YAML parse error: {e}"]
    except OSError as e:
        return [f"Could not read file: {e}"]

    if doc is None:
        return ["File is empty or contains no YAML document"]

    if not isinstance(doc, dict):
        return [f"Expected a YAML mapping at top level, got {type(doc).__name__}"]

    # Check required fields
    for field in REQUIRED_FIELDS:
        if field not in doc:
            errors.append(f"Missing required field: {field}")

    # Check optional string fields
    for field in OPTIONAL_STRING_FIELDS:
        if field in doc and not isinstance(doc[field], str):
            errors.append(
                f"Field '{field}' should be a string, got {type(doc[field]).__name__}"
            )

    # Check optional list fields
    for field in OPTIONAL_LIST_FIELDS:
        if field in doc:
            if not isinstance(doc[field], list):
                errors.append(
                    f"Field '{field}' should be a list, got {type(doc[field]).__name__}"
                )

    # Check optional list-of-strings fields
    for field in OPTIONAL_LIST_OF_STRINGS_FIELDS:
        if field in doc:
            if not isinstance(doc[field], list):
                errors.append(
                    f"Field '{field}' should be a list of strings, "
                    f"got {type(doc[field]).__name__}"
                )
            else:
                for i, item in enumerate(doc[field]):
                    if not isinstance(item, str):
                        errors.append(
                            f"Field '{field}[{i}]' should be a string, "
                            f"got {type(item).__name__}"
                        )

    return errors


def find_yaml_files(db_dir):
    """Walk the db/ directory and yield all .yaml file paths."""
    for root, _dirs, files in os.walk(db_dir):
        for filename in sorted(files):
            if filename.endswith(".yaml"):
                yield os.path.join(root, filename)


def main():
    # Determine the db/ directory relative to the repo root
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(script_dir)
    db_dir = os.path.join(repo_root, "db")

    if not os.path.isdir(db_dir):
        print(f"Error: db/ directory not found at {db_dir}")
        sys.exit(1)

    total_files = 0
    total_errors = 0
    files_with_errors = 0

    for filepath in find_yaml_files(db_dir):
        total_files += 1
        rel_path = os.path.relpath(filepath, repo_root)
        errors = validate_file(filepath)

        if errors:
            files_with_errors += 1
            total_errors += len(errors)
            print(f"FAIL  {rel_path}")
            for error in errors:
                print(f"      - {error}")
        else:
            print(f"  OK  {rel_path}")

    # Summary
    print()
    print(f"Validated {total_files} file(s): ", end="")
    if total_errors == 0:
        print("all passed.")
    else:
        print(f"{files_with_errors} file(s) with {total_errors} error(s).")

    if total_files == 0:
        print("Warning: no .yaml files found in db/")

    sys.exit(1 if total_errors > 0 else 0)


if __name__ == "__main__":
    main()
