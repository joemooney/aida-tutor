#!/usr/bin/env bash
# Launch aida-tutor through Cargo, forwarding all CLI arguments.
#
#   ./run.sh                  # first-run welcome or the current exercise
#   ./run.sh shell            # recommended interactive onboarding
#   ./run.sh onboard          # full onboarding lesson
#   ./run.sh verify           # verify the current exercise
#
# Works from any directory: the script resolves the repository root before
# invoking Cargo. trace:FR-2 | ai:codex

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd -- "$repo_root"

# exec keeps Ctrl-C and the application exit status attached to the caller.
exec cargo run -- "$@"
