# build the python wheel and install it to

export OSX_SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
export IOS_SDKROOT=$(xcrun --sdk iphoneos --show-sdk-path)
export PYTHONDIR="/Users/marc/psydk/bubblesdemo/build/bubblesdemo/ios/xcode/Support/Python.xcframework/ios-arm64"
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
