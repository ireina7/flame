#!/bin/bash

if [ "$1" == 'c' ]; then
    ./target/debug/flame -p ./app/app.json -c up
elif [ "$1" == 'r' ]; then
    ./target/debug/flame -p ./app/app.json -r up
elif [ "$1" == 'rc' ]; then
    ./target/debug/flame -p ./app/app.json -c -r up
else
    ./target/debug/flame -p ./app/app.json up
fi
