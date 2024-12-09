#!/usr/bin/env bash

# This is highly inspired from Prof Pina's testing scripts. It is meant to be
# run in wherever there is a compiler to the Aves IR, on tests that are valid
# programs. As I speak, this means it can be run in the Assignment 4 directory
# (with a solution to assignment 4) and on the assignment 4 and assignment 3
# positive tests. Positive tests from previous assignments may not generate any
# IR, because they are meant only to test syntactic correctness.

set -u # Don't let me refer to uninitialized variables.
set -e # Fail immediately when any command fails.

EXEC=interp
COMPILER=bluejaycc
TEST_DIR=tests
IR_SAMPLES_DIR=ir/from_a4
SHOULD_EXTRACT_EXPECTED_OUTPUT=true # Only set this to true when there is *program* output to extract.

make clean && make

for T in $(ls $TEST_DIR)
do
  mkdir -p "$IR_SAMPLES_DIR/$T"
  for F in $(ls $TEST_DIR/$T | grep ".bluejay$" | grep pass) # Only capture ir from passing tests.
  do
    TESTFILE="$TEST_DIR/$T/$F"
    # TODO: I don't quite understand how this line works. Is it a here document?
    IRFILE=$(sed 's/.bluejay/.aves_bytecode/g' <<<"$IR_SAMPLES_DIR/$T/$F")
    ./$COMPILER $TESTFILE $IRFILE
    if $SHOULD_EXTRACT_EXPECTED_OUTPUT # This does not cause the whole script to fail even when it evaluates to false.
    then
      EXPECTEDFILE_NAMEONLY=$(sed 's/.bluejay/.expected/g' <<<"$F")
      cp "$TEST_DIR/$T/$EXPECTEDFILE_NAMEONLY" "$IR_SAMPLES_DIR/$T/$EXPECTEDFILE_NAMEONLY"
    fi
  done
done