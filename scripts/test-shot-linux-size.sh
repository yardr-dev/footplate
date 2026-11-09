#!/bin/sh
# SHOT_SIZE is two positive whole numbers joined by an x. A value of any
# other shape is refused at once and in words, before a container is touched
# or a file written, and that includes one with no x at all, which the check
# once let through as a width and a height of the same number: 1200 shot a
# window of 1200 by 1200 without a word. A size that is right is the
# resolution sway is told of, the width first; none asked is 1280 by 800.
set -eu

script_dir=$(CDPATH='' cd "$(dirname "$0")" && pwd)
script=$script_dir/shot-linux.sh
tmp=$(mktemp -d)
cleanup() {
    rm -rf "$tmp"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
mkdir -p "$tmp/bin"

# The shot script only needs Docker to answer whether a stale container is
# left and to report a visible window here. Stub it so this checks the real
# argument handling without provisioning a container or building Footplate;
# the run, rm and exec lines are logged, so this sees what sway is told, and
# that a size that is refused never reached docker at all.
cat > "$tmp/bin/docker" <<'MOCK_DOCKER'
#!/bin/sh
set -eu
command=$1
shift
case "$command" in
    inspect)
        # The only inspect asks whether a stale container of the name is left.
        exit 1
        ;;
    run)
        printf 'run %s\n' "$*" >> "$MOCK_LOG"
        exit 0
        ;;
    rm)
        printf 'rm %s\n' "$*" >> "$MOCK_LOG"
        exit 0
        ;;
    exec)
        shift
        printf '%s\n' "$*" >> "$MOCK_LOG"
        case "$*" in
            *swaymsg*-t\ get_tree*) echo '{"app_id":"footplate"}' ;;
        esac
        exit 0
        ;;
    cp)
        destination=$2
        printf 'mock screenshot\n' > "$destination"
        ;;
    *)
        echo "unexpected docker command: $command $*" >&2
        exit 1
        ;;
esac
MOCK_DOCKER
cat > "$tmp/bin/sleep" <<'MOCK_SLEEP'
#!/bin/sh
exit 0
MOCK_SLEEP
chmod +x "$tmp/bin/docker" "$tmp/bin/sleep"

fail() {
    echo "test-shot-linux-size.sh: $1" >&2
    exit 1
}

# Whatever size the caller's shell asks for is not this test's to take.
unset SHOT_SIZE

# A size that is not two positive whole numbers joined by an x is refused
# the same way, whatever its fault: exit 2, the words, no file, and nothing
# that reached docker.
export MOCK_LOG="$tmp/log-bogus"
for bogus in 1200 x800 1200x 1200x800x2 0x800 1200x0 wide; do
    rc=0
    err=$(SHOT_SIZE=$bogus PATH="$tmp/bin:$PATH" "$script" "$tmp/shot-size.png" 2>&1) || rc=$?
    [ "$rc" -eq 2 ] || fail "a size of $bogus exited $rc, not 2: $err"
    case "$err" in
        *"SHOT_SIZE must be WIDTHxHEIGHT"*) ;;
        *) fail "a size of $bogus said: $err" ;;
    esac
    [ ! -e "$tmp/shot-size.png" ] || fail "a size of $bogus wrote its file"
    [ ! -e "$MOCK_LOG" ] || fail "a size of $bogus reached docker: $(cat "$MOCK_LOG")"
done

# grep checks lines, so a valid first line once let later shell text through
# into the sway command. A multiline size is rejected before Docker is asked.
multiline=$(printf "1280x800\n'; touch /tmp/shot-size-injected; #")
export MOCK_LOG="$tmp/log-multiline"
rc=0
err=$(SHOT_SIZE="$multiline" PATH="$tmp/bin:$PATH" "$script" "$tmp/shot-multiline.png" 2>&1) || rc=$?
[ "$rc" -eq 2 ] || fail "a multiline size exited $rc, not 2: $err"
case "$err" in
    *"SHOT_SIZE must be WIDTHxHEIGHT"*) ;;
    *) fail "a multiline size said: $err" ;;
esac
[ ! -e "$tmp/shot-multiline.png" ] || fail "a multiline size wrote its file"
[ ! -e "$MOCK_LOG" ] || fail "a multiline size reached docker: $(cat "$MOCK_LOG")"

# The line that tells sway of its one output: the width, then the height.
expect_output() {
    what=$1
    log=$2
    width=$3
    height=$4
    grep -F -q -- "' '$width' '$height' > /tmp/shots/sway-config" "$log" \
        || fail "$what did not tell sway $width by $height: $(grep -F 'sway-config' "$log")"
}

# None asked is the usual 1280 by 800, and asked for it is the same.
export MOCK_LOG="$tmp/log-default"
PATH="$tmp/bin:$PATH" "$script" "$tmp/shot-default.png" >/dev/null 2>&1 \
    || fail "a shot with no size did not shoot"
[ -f "$tmp/shot-default.png" ] || fail "a shot with no size wrote no screenshot"
expect_output "a shot with no size" "$MOCK_LOG" 1280 800

export MOCK_LOG="$tmp/log-usual"
SHOT_SIZE=1280x800 PATH="$tmp/bin:$PATH" "$script" "$tmp/shot-usual.png" >/dev/null 2>&1 \
    || fail "a size of 1280x800 did not shoot"
[ -f "$tmp/shot-usual.png" ] || fail "a size of 1280x800 wrote no screenshot"
expect_output "a size of 1280x800" "$MOCK_LOG" 1280 800

# Another size, whose two numbers differ, so that a width taken for a height
# or the reverse shows.
export MOCK_LOG="$tmp/log-other"
SHOT_SIZE=1200x700 PATH="$tmp/bin:$PATH" "$script" "$tmp/shot-other.png" >/dev/null 2>&1 \
    || fail "a size of 1200x700 did not shoot"
[ -f "$tmp/shot-other.png" ] || fail "a size of 1200x700 wrote no screenshot"
expect_output "a size of 1200x700" "$MOCK_LOG" 1200 700

echo "test-shot-linux-size.sh: pass"
