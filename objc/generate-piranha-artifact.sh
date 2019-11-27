#!/usr/bin/env bash

set -exu

BRANCH_SCHEME=${BRANCH_SCHEME:-"swift-5.1-branch"}
PIRANHA_VERSION="${PIRANHA_VERSION:-"0.0.5"}"
PIRANHA_SRC=$(pwd)/src
PIRANHA_DIR=$(pwd)/piranha-objc
BASE_DIR=$(pwd)
PROCESS_DIR=$(pwd)/process
LLVM_PROJECT=$PROCESS_DIR/llvm-project
LLVM_BUILDDIR=$LLVM_PROJECT/llvm/build
DYLIB_OUTPUTDIR=$PROCESS_DIR/llvm-project/llvm/build

mkdir $PROCESS_DIR
mkdir $LLVM_PROJECT
cd $PROCESS_DIR

# set up the source for building the clang plugin
git clone https://github.com/apple/swift.git
./swift/utils/update-checkout --clone --scheme $BRANCH_SCHEME

cd $LLVM_PROJECT
mkdir clang/examples/XPFlagRefactoring
cp -R $BASE_DIR/src/XPFlagRefactoring/ clang/examples/XPFlagRefactoring
echo "add_subdirectory(XPFlagRefactoring)" >> clang/examples/CMakeLists.txt

# build the plugin
mkdir $LLVM_BUILDDIR
cd $LLVM_BUILDDIR
cmake -DCMAKE_BUILD_TYPE=Release .. -DLLVM_ENABLE_PROJECTS="clang" -DCLANG_BUILD_EXAMPLES="ON"

make -j 12

# generate the artifact
cd $BASE_DIR
rm -rf $PIRANHA_DIR
mkdir $PIRANHA_DIR
mkdir $PIRANHA_DIR/bin
mkdir $PIRANHA_DIR/lib
cp $DYLIB_OUTPUTDIR/lib/XPFlagRefactoring.dylib $PIRANHA_DIR/bin/
cp $DYLIB_OUTPUTDIR/bin/clang $PIRANHA_DIR/bin/
mv $DYLIB_OUTPUTDIR/lib/clang $PIRANHA_DIR/lib/
