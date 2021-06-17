#! /bin/bash

# Cargo install the package
cargo install --path .
# create the directories for the language files
mkdir ~/Library/Application Support/matheriser && mkdir ~/Library/Application Support/matheriser/assets
# copy the language files across
cp assets/*.ron ~/Library/Application Support/matheriser/assets/
# remove the manifest, it's hard-coded
rm ~/Library/Application Support/matheriser/assets/manifest.ron
