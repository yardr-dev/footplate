#!/bin/sh
# The states src/shot.rs serves, read from its STATES list: sourced by
# scripts/shot-linux.sh and scripts/shot-mac.sh, not run. The caller sets
# shot_rs to that file before it sources this one (a sourced file has no $0
# of its own), and gets back `states`, the names with a space between them,
# and is_state(). A state added to STATES is then served by both scripts,
# with no list of theirs to edit.
states=$(sed -n '/^pub const STATES/,/^];/s/^ *"\([^"]*\)",$/\1/p' "$shot_rs" | tr '\n' ' ')
states=${states% }
if [ -z "$states" ]; then
    echo "shot-states.sh: no STATES in $shot_rs" >&2
    exit 2
fi
is_state() {
    case " $states " in
        *" $1 "*) return 0 ;;
        *) return 1 ;;
    esac
}
