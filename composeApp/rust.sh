cd src/main/rust &&

/home/void/.cargo/bin/cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -o ../jniLibs \
  build --release &&

cargo run --features=uniffi/cli --bin uniffi-bindgen generate \
  --library target/aarch64-linux-android/release/libplayer_core.so \
  --language kotlin \
  --out-dir src/commonMain/kotlin/