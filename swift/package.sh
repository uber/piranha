#!/bin/bash

rm -rf artifact
rm -rf .build
swift build -c release

mkdir artifact
mkdir artifact/piranha
cd artifact
mkdir piranha/bin 
cp ../.build/release/Piranha piranha/bin
tar -zcvf piranha.tar.gz piranha
