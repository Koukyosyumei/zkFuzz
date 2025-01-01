#!/bin/bash

for submodule in $(git config --file .gitmodules --get-regexp path | awk '{ print $2 }'); do
    echo "Updating $submodule..."
    git submodule update --init "$submodule" || echo "Skipping $submodule due to error."
done

