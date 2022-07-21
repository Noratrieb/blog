#!/usr/bin/env bash

git stash
git checkout gh-pages
hugo -D
git add ./public
git commit -m "deploy"
git push origin gh-pages
git checkout -
git stash pop