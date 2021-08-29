#!/bin/bash

cargo watch -x 'r -- -t "$APPLICATION_TOKEN" -a $APPLICATION_ID -m $MONGODB_URI -g $DEVELOPMENT_GUILD_ID'
