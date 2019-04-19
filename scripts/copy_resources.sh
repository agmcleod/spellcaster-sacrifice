#!/bin/bash

mkdir -p target/debug/
mkdir -p target/release/
cp -r resources target/debug/
cp -r resources target/release/

rm -rf target/debug/resources/raw
rm -rf target/release/resources/raw
