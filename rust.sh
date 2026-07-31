cd "$(dirname "$0")/composeApp/src/main/rust"

"${CARGO:-cargo}" ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -o ../jniLibs \
  build --release

"${CARGO:-cargo}" build --release


"${CARGO:-cargo}" run --release --features=uniffi/cli --bin uniffi-bindgen generate \
  --library target/release/libvociferate_core.so \
  --language kotlin \
  --out-dir ../../commonMain/kotlin/

"${CARGO:-cargo}" build --release --target x86_64-pc-windows-gnu