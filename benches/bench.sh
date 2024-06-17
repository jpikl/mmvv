#!/usr/bin/env sh

set -eu

. ./benches/command.sh

# Runs benchmark for the specified shell commands.
#
# Stdout: Benchmark results in markdown format.
# Stderr: Benchmarking progress.
#
# Usage: bench <cmd>...
#   <cmd>... Shell command(s) to be benchmarked.
bench() {
  NUMBER=$((NUMBER + 1))

  [ "$CLEANUP" ] && set -- --cleanup "$CLEANUP" "$@"
  [ "$SETUP" ] && set -- --setup "$SETUP" "$@"
  set -- hyperfine --warmup 5 "$@"

  echo
  echo "## Benchmark #$NUMBER"

  if [ "$NO_TEST" ]; then
    echo "> **Note:** $NO_TEST"
  fi

  echo
  echo "Command:"
  echo
  echo '```shell'

  preview "$@"

  echo
  echo '```'
  echo
  echo "Results:"
  echo

  execute "$@"
  reset
}

preview() {
  printf "%s" "$1"
  shift

  while [ $# -gt 0 ]; do
    printf ' \\\n    '

    case "$1" in
    --warmup)
      printf "%s %s" "$1" "$2"
      shift 2
      ;;
    --setup | --cleanup)
      printf "%s \"%s\"" "$1" "$(escape "$2")"
      shift 2
      ;;
    *)
      printf "\"%s\"" "$(escape "$1")"
      shift
      ;;
    esac
  done
}

escape() {
  echo "$1" | sed 's/"/\\"/g'
}

execute() {
  RESULTS_DIR=target/benches
  RESULTS_FILE=$RESULTS_DIR/$COMMAND.$NUMBER.md

  mkdir -p "$RESULTS_DIR"
  echo >&2
  "$@" --sort mean-time --export-markdown "$RESULTS_FILE" 1>&2
  cat "$RESULTS_FILE"
}

echo "# rew $COMMAND"
echo
echo "This page contains benchmarks for [rew $COMMAND](../reference/rew-$COMMAND.md) command."
echo
echo "See [benchmark setup](./setup.md) for information about testing environment and input data files."

# shellcheck disable=SC1090
. "./benches/commands/$COMMAND.sh"
