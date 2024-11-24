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
IR_DIR=ir

make clean && make

for T in $(ls $TEST_DIR)
do
  mkdir -p "$IR_DIR/$T"
  for F in $(ls $TEST_DIR/$T | grep ".bluejay$" | grep pass) # Only capture ir from passing tests.
  do
    TESTFILE="$TEST_DIR/$T/$F"
    IRFILE=$(sed 's/.bluejay/.aves_bytecode/g' <<<"$IR_DIR/$T/$F")
    ./$COMPILER $TESTFILE $IRFILE
  done
done