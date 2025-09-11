# Unnamed...

## How to Build
### With Nix
```bash
nix develop
cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build
gradle app:assembleDebug
```
### Or manually
Requirements
- rustc supports `2024`
- `aarch64-android-linux` target support
- android sdk `36`
- android ndk `35.0.0`
then run
```bash
cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build
./gradlew app:assembleDebug
```

# Acknowledgement
This app uses inspiration from [https://github.com/andraantariksa/wgpu-winit-android-example](https://github.com/andraantariksa/wgpu-winit-android-example)