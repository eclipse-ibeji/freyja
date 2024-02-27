#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

if [ $# -ne 1 ]; then
  echo "Usage: $0 {directory}"
  exit 1
fi

TEMP_FILE=`mktemp`
TARGET_DIR="$1"
ACCEPTED_WORDS_FILEPATH="$TARGET_DIR/.accepted_words.txt"

EXIT_CODE=0
while IFS= read -r -d '' file; do
  if [ -e $ACCEPTED_WORDS_FILEPATH ]; then
    spell -d "$ACCEPTED_WORDS_FILEPATH" "$file" -n > $TEMP_FILE
  else
    spell $1 -n > $TEMP_FILE
  fi
  NUM_SPELLING_ERRORS=`wc -l $TEMP_FILE | cut -d ' ' -f 1`
  if [ $NUM_SPELLING_ERRORS -ne 0 ]; then
    echo "Error: $file has spelling mistakes. Please fix them."
    cat $TEMP_FILE
    EXIT_CODE=1
  fi
done < <(find . -path ./target -prune -o -path ./node_modules -prune -o -name "*.md" -print0)

rm -f $TEMP_FILE

exit $EXIT_CODE