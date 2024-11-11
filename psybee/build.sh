# build the python wheel

export OSX_SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
export IOS_SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)
export PYTHONDIR="/Users/marc/psybee/bubblesdemo/build/bubblesdemo/ios/xcode/Support/Python.xcframework/ios-arm64"
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

# find the wheel
WHEEL=$(find ../target/wheels -name "*.whl")

# convert to absolute path
WHEEL=$(realpath $WHEEL)

# unzip the wheel
rm -rf ../target/wheels/unzipped
unzip $WHEEL -d ../target/wheels/unzipped

# rename all .*so files to *.dylib
find ../target/wheels/unzipped -name "*.so" -exec bash -c 'mv "$1" "${1%.so}.dylib"' _ {} \;

# zip the wheel back up
cd ../target/wheels/unzipped
rm -f $WHEEL
zip -r $WHEEL *
