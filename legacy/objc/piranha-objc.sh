
#./piranha-objc.sh ObjectiveCFile.m flag_name optimistic

SOURCE=$1
FLAGNAME=$2
FLAGTYPE=$3

XcodeSDK="/Applications/Xcode.11.1.0.11A1027.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.15.sdk" 
PIRANHA_LIB="./piranha-objc/bin/XPFlagRefactoring.dylib"

./piranha-objc/bin/clang -x objective-c \
    -c $SOURCE \
    -fobjc-arc \
    -isysroot $XcodeSDK \
    -fsyntax-only \
    -Xclang -load \
    -Xclang $PIRANHA_LIB \
    -Xclang -plugin \
    -Xclang XPFlagRefactorPlugin \
    -Xclang -plugin-arg-XPFlagRefactorPlugin \
    -Xclang $FLAGNAME,$FLAGTYPE
