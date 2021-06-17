#! /bin/bash

# Cargo install the package
cargo install --path .
# create the directories for the language files
mkdir ~/.local/share/matheriser/ && mkdir ~/.local/share/matheriser/assets/
# copy the language files across
cp assets/*.ron ~/.local/share/matheriser/assets/
# remove the manifest, it's hard-coded
rm ~/.local/share/matheriser/assets/manifest.ron
