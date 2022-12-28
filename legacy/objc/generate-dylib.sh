#!/usr/bin/env bash

#should be invoked after downloading the necessary llvm code

set -exu

PIRANHA_SRC=$(pwd)/src
PIRANHA_DIR=$(pwd)/piranha-objc
BASE_DIR=$(pwd)
PROCESS_DIR=$(pwd)/process
LLVM_PROJECT=$PROCESS_DIR/llvm-project
LLVM_BUILDDIR=$LLVM_PROJECT/llvm/build
DYLIB_OUTPUTDIR=$PROCESS_DIR/llvm-project/llvm/build

cd $LLVM_PROJECT
cp -R $BASE_DIR/src/XPFlagRefactoring/ clang/examples/XPFlagRefactoring

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

