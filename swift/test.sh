#!/bin/bash

cleanup() {
    rv=$?
    rm $tempfile || true
    exit $rv
}

trap "cleanup" INT TERM EXIT

piranha_exe=artifact/piranha/bin/Piranha

tempfile=mktemp
$piranha_exe cleanup-stale-flags piranha.properties tests/testfile.swift test_experiment true > "$tempfile"
CHANGES=$(diff -wB $tempfile tests/treated.swift)

if [ "$CHANGES" != "" ]
then
   echo "Treatment tests failed. Differences given below:"
   echo "$CHANGES"
   exit 64
fi


$piranha_exe cleanup-stale-flags piranha.properties tests/testfile.swift test_experiment false > "$tempfile"
CHANGES=$(diff -wB $tempfile tests/control.swift)

if [ "$CHANGES" != "" ]
then
   echo "Control tests failed. Differences given below:"
   echo "$CHANGES"
   exit 64
fi

echo "Tests successfully passed"

