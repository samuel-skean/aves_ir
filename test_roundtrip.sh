#!/usr/bin/env bash

# This is highly inspired from Prof Pina's testing scripts. This tests to make
# sure that printing all the ir, parsing that printed version, and writing it as
# bytecode results in the original bytecode, on all the examples of known
# bytecode.

set -u # Don't let me refer to uninitialized variables.
set -e # Fail immediately when any command fails.

cargo build
AVES_BIN_PATH='./target/debug/aves_interpreter'
PRINTED='printed.aves_text'
REASSEMBLED='rust_out.aves_bytecode'
XXD_BYTECODE_DIFFERENCE=xxd_bytecode_difference
TEXT_DIFFERENCE=text_difference
IR_DIR=ir_samples

GREEN='\033[42m'
RESET='\033[0m'

for ORIGINAL in $(find $IR_DIR -type f | grep ".aves_bytecode$" | sort)
do
    echo "Checking $ORIGINAL"
    $AVES_BIN_PATH --bytecode $ORIGINAL print > $PRINTED 2> /dev/null
    $AVES_BIN_PATH --text $PRINTED --output-bytecode $REASSEMBLED print &> /dev/null
    diff <($AVES_BIN_PATH --bytecode $REASSEMBLED print 2> /dev/null) $PRINTED > $TEXT_DIFFERENCE
    # Is diffing the result of xxd always gonna work?
    diff <(xxd $REASSEMBLED) <(xxd $ORIGINAL) > $XXD_BYTECODE_DIFFERENCE
    
    rm $PRINTED $REASSEMBLED
done

echo -e "${GREEN}EVERYTHING IS PASSING! :)${RESET}"