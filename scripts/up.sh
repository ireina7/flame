#!/bin/bash

if [ $1 = 'c' ]
then
    ./target/debug/flame -p ./app/app.json -c up
else
    ./target/debug/flame -p ./app/app.json up
fi
