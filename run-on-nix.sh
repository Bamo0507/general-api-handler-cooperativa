#!/usr/bin/env bash

source .env
nix run nixpkgs#cargo run
