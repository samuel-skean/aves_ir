#!/usr/bin/env bash

# This script should be rerun every time the contents of ir_samples changes
# before testing. Ideally, it should not be run often. The human-readable files
# it creates (both ".aves_text" and ".expected") should be checked by hand
# before relying on them, because this script simply invokes the rust and c
# code, which it is also meant to help test.

# This script generates human-readable ".aves_text" files for the bytecode files
# in IR_DIR that aren't in HANDWRITTEN_DIR, and ".aves_bytecode" files for the
# ".aves_text" files in HANDWRITTEN_DIR. If any of the files it might generate
# already exist, it indicates so on stderr and doesn't regenerate that file.

# Then, the script interprets all the bytecode files and produces files ending
# in ".expected" with their output. If, for any file, a corresponding
# ".expected" file already exists, the file is not interpreted and this is
# indicated on stderr.

PRINT='cargo run --bin aves_interpreter -- --print --bytecode'
ASSEMBLE='cargo run --bin aves_interpreter -- --print --text' # TODO: Bad name.
BYTECODE_EXTENSION=".aves_bytecode"
TEXT_EXTENSION=".aves_text"
IR_DIR=ir_samples
HANDWRITTEN_DIR=ir_samples/handwritten

set -u # Fail on access to unset variables.
set -e # Fail on first failing command.

for BYTECODE_FILE in $(find "$IR_DIR" -type f ! -path "$HANDWRITTEN_DIR" | grep "${BYTECODE_EXTENSION}\$" | sort)
do
    OUTPUT_TEXT_FILE=$(sed "s/${BYTECODE_EXTENSION}/${TEXT_EXTENSION}/g" <<<"$BYTECODE_FILE")
    if [ -e "$OUTPUT_TEXT_FILE" ]
    then
        echo "Skipping printing of ${BYTECODE_FILE} because ${OUTPUT_TEXT_FILE} already exists." >&2
        continue
    fi
    "$PRINT" "$BYTECODE_FILE" > "$OUTPUT_TEXT_FILE"
done

for HANDWRITTEN_FILE in $(find $HANDWRITTEN_DIR -type f | grep "${TEXT_EXTENSION}\$" | sort)
do
    OUTPUT_BYTECODE_FILE=$(sed "s/${TEXT_EXTENSION}/${BYTECODE_EXTENSION}/g" <<<"$HANDWRITTEN_FILE")
    if [ -e "$OUTPUT_BYTECODE_FILE" ]
    then
        echo "Skipping assembly of ${HANDWRITTEN_FILE} because ${OUTPUT_BYTECODE_FILE} already exists." >&2
        continue
    fi
    "$ASSEMBLE" "$HANDWRITTEN_FILE" --output-bytecode "$OUTPUT_BYTECODE_FILE" 
done

# TODO: Interpret.