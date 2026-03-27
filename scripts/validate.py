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
OPTIONAL_LIST_OF_STRINGS_FIELDS = {"links", "tags", "exclude"}

ALL_KNOWN_FIELDS = (
    REQUIRED_FIELDS
    | OPTIONAL_STRING_FIELDS
    | OPTIONAL_LIST_OF_STRINGS_FIELDS
    | {"patterns"}
)


def validate_file(filepath, filename_stem):
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

    # Check for unknown fields
    for field in doc:
        if field not in ALL_KNOWN_FIELDS:
            errors.append(f"Unknown field: '{field}'")

    # Check required fields exist and are non-empty strings
    for field in REQUIRED_FIELDS:
        if field not in doc:
            errors.append(f"Missing required field: {field}")
        elif not isinstance(doc[field], str):
            errors.append(
                f"Field '{field}' should be a string, got {type(doc[field]).__name__}"
            )
        elif not doc[field].strip():
            errors.append(f"Field '{field}' is empty")

    # id must match filename
    if "id" in doc and isinstance(doc["id"], str):
        if doc["id"] != filename_stem:
            errors.append(
                f"id '{doc['id']}' does not match filename '{filename_stem}.yaml'"
            )

    # language must match parent directory name
    parent_dir = os.path.basename(os.path.dirname(filepath))
    if "language" in doc and isinstance(doc["language"], str):
        if doc["language"] != parent_dir:
            errors.append(
                f"language '{doc['language']}' does not match "
                f"directory '{parent_dir}'"
            )

    # Check optional string fields
    for field in OPTIONAL_STRING_FIELDS:
        if field in doc and not isinstance(doc[field], str):
            errors.append(
                f"Field '{field}' should be a string, got {type(doc[field]).__name__}"
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

    # Validate patterns structure: list of lists of strings
    if "patterns" in doc:
        patterns = doc["patterns"]
        if not isinstance(patterns, list):
            errors.append(
                f"Field 'patterns' should be a list, got {type(patterns).__name__}"
            )
        elif len(patterns) == 0:
            errors.append("Field 'patterns' is empty — remove it or add entries")
        else:
            for i, group in enumerate(patterns):
                if not isinstance(group, list):
                    errors.append(
                        f"patterns[{i}] should be a list of strings, "
                        f"got {type(group).__name__}"
                    )
                elif len(group) == 0:
                    errors.append(f"patterns[{i}] is empty")
                else:
                    for j, item in enumerate(group):
                        if not isinstance(item, str):
                            errors.append(
                                f"patterns[{i}][{j}] should be a string, "
                                f"got {type(item).__name__}"
                            )
                        elif not item.strip():
                            errors.append(f"patterns[{i}][{j}] is an empty string")

    return errors


def find_yaml_files(db_dir):
    """Walk the db/ directory and yield (filepath, filename_stem) for .yaml files."""
    for root, _dirs, files in os.walk(db_dir):
        for filename in sorted(files):
            if not filename.endswith(".yaml"):
                continue
            stem = filename[: -len(".yaml")]
            yield os.path.join(root, filename), stem


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(script_dir)
    db_dir = os.path.join(repo_root, "db")

    if not os.path.isdir(db_dir):
        print(f"Error: db/ directory not found at {db_dir}")
        sys.exit(1)

    total_files = 0
    total_errors = 0
    files_with_errors = 0
    seen_ids = {}  # "language/id" -> filepath, for duplicate detection

    for filepath, stem in find_yaml_files(db_dir):
        rel_path = os.path.relpath(filepath, repo_root)

        # Skip TEMPLATE files
        if stem == "TEMPLATE":
            continue

        # Skip root-level files (only subdirectories contain real entries)
        parent = os.path.basename(os.path.dirname(filepath))
        if parent == "db":
            continue

        total_files += 1
        errors = validate_file(filepath, stem)

        # Check for duplicate IDs across the entire database
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                doc = yaml.safe_load(f)
            if isinstance(doc, dict) and "id" in doc:
                entry_id = doc["id"]
                lang = doc.get("language", parent)
                key = f"{lang}/{entry_id}"
                if key in seen_ids:
                    errors.append(
                        f"Duplicate id '{entry_id}' (also in {seen_ids[key]})"
                    )
                else:
                    seen_ids[key] = rel_path
        except Exception:
            pass  # parse errors already caught in validate_file

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
