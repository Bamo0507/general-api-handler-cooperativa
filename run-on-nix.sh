#!/usr/bin/env bash

source .env
nix develop . --command cargo run
