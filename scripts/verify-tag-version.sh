#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

refspec_file="${1:-}"
tmp_file=""

if [[ -z "$refspec_file" ]]; then
  tmp_file="$(mktemp)"
  cat >"$tmp_file"
  refspec_file="$tmp_file"
fi

cleanup() {
  if [[ -n "$tmp_file" && -f "$tmp_file" ]]; then
    rm -f "$tmp_file"
  fi
}
trap cleanup EXIT

if [[ ! -f "$refspec_file" || ! -s "$refspec_file" ]]; then
  exit 0
fi

if [[ ! -f Cargo.toml ]]; then
  echo "Blocked pre-push: Cargo.toml is missing." >&2
  exit 1
fi

workspace_version="$(
  awk '
    /^\[workspace\.package\]/ { in_section=1; next }
    /^\[/ && in_section { in_section=0 }
    in_section && $0 ~ /^version = "/ {
      match($0, /"[^"]+"/)
      print substr($0, RSTART + 1, RLENGTH - 2)
      exit
    }
  ' Cargo.toml
)"
if [[ -z "$workspace_version" ]]; then
  echo "Blocked pre-push: cannot parse workspace.package.version from Cargo.toml." >&2
  exit 1
fi

mismatches=()

while read -r local_ref local_sha remote_ref remote_sha; do
  [[ -z "${local_ref:-}" ]] && continue
  [[ "$local_ref" != refs/tags/* ]] && continue

  # Skip tag deletion pushes.
  if [[ "${local_sha:-}" =~ ^0+$ ]]; then
    continue
  fi

  tag="${local_ref#refs/tags/}"
  tag_version="${tag#v}"
  if [[ "$tag_version" != "$workspace_version" ]]; then
    mismatches+=("${tag} (Cargo.toml version=${workspace_version})")
  fi
done <"$refspec_file"

if [[ ${#mismatches[@]} -gt 0 ]]; then
  {
    echo "Blocked pre-push: tag version and Cargo.toml version are not aligned:"
    for item in "${mismatches[@]}"; do
      echo "  - ${item}"
    done
    echo "Please bump [workspace.package].version before pushing release tags."
  } >&2
  exit 1
fi
