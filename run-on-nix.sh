#!/usr/bin/env bash

source .env
<<<<<<< HEAD
nix run nixpkgs#cargo run
=======
nix develop . --command cargo run
>>>>>>> file-s3-upload
