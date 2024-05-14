# build the python wheel and install it to 

export OSX_SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
export IOS_SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)
export PYTHONDIR="/Users/marc/psychophysics/example_experiments/demo_app/build/demo_app/ios/xcode/Support/Python.xcframework/ios-arm64"
export PYO3_CROSS_LIB_DIR="$PYTHONDIR"
export PYO3_CROSS_PYTHON_VERSION=3.9
env SDKROOT="$IOS_SDKROOT" \
PYO3_CROSS_LIB_DIR="$PYTHONDIR" \
CARGO_TARGET_AARCH64_APPLE_IOS_RUSTFLAGS="-C link-arg=-isysroot -C link-arg=$IOS_SDKROOT \
	-C link-arg=-arch -C link-arg=arm64 -C link-arg=-miphoneos-version-min=14.0 -C link-arg=-L \
	-C link-arg=$PYTHONDIR \
	-C link-arg=-undefined \
	-C link-arg=dynamic_lookup" \
maturin build --target aarch64-apple-ios --release

# unzip the wheel 
export TARGET_DIR="/Users/marc/psychophysics/example_experiments/demo_app/build/demo_app/ios/xcode/DemoApp/app_packages.iphoneos"
unzip -o ../target/wheels/psychophysics_py-0.1.0-cp38-abi3-ios_23_2_0_arm64.whl -d $TARGET_DIR

# rename psychophysics_py.cpython-39-darwin.so files to *.dylib and codesign them
mv -f $TARGET_DIR/psychophysics_py/psychophysics_py.abi3.so $TARGET_DIR/psychophysics_py/psychophysics_py.abi3.dylib

codesign --force --timestamp --sign 0801B5387CDC573AB560315B0C3D1C309906D961 $TARGET_DIR/psychophysics_py/psychophysics_py.abi3.dylib
# # remove the signature
#codesign --remove-signature $TARGET_DIR/psychophysics_py/psychophysics_py.abi3.dylib



