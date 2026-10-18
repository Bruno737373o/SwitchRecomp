# Android Build Target

The `android` command wraps an emitted project (the output of `recomp-cli run`) in a
Gradle app. The Rust code is cross-compiled with `cargo ndk` into
`librecomp_android.so` and packaged into an APK together with the project's runtime
data files (for example memory image segments).

The target only repackages code that the pipeline already emitted. It does not change
intake, provenance, or asset policy (see `docs/LEGAL-POLICY.md`).

## Prerequisites
- JDK 17 and the Android SDK (set `ANDROID_HOME`, platform 34).
- Android NDK (default `26.3.11579264`, override with `--ndk-version`).
- Rust Android targets and `cargo-ndk`:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk --locked
```

## Usage
```bash
cargo run -p recomp-cli -- run \
  --module samples/memory-image/module.json \
  --config samples/memory-image/title.toml \
  --provenance samples/memory-image/provenance.toml \
  --out-dir out/memory-image

cargo run -p recomp-cli -- android \
  --project-dir out/memory-image \
  --out-dir out/android-memory-image \
  --build
```

The APK is written to `out/android-memory-image/app/build/outputs/apk/debug/app-debug.apk`.

Options:
- `--runtime-path` (default `crates/recomp-runtime`): runtime crate used by the wrapper.
- `--app-name` (default: manifest title) and `--application-id`
  (default: `org.switchrecomp.<title>`).
- `--abi` (repeatable, default `arm64-v8a`, `armeabi-v7a`, `x86_64`; `x86` also supported).
- `--min-sdk` (default 21), `--ndk-version`.
- `--build` runs `./gradlew assembleDebug`; `--release` runs `assembleRelease`
  (unsigned APK). Without either flag only the Gradle project is generated.

## Generated Layout
```
out/android-<title>/
  settings.gradle, build.gradle, gradle.properties, gradlew, gradle/wrapper/
  app/build.gradle              # runs `cargo ndk` before preBuild
  app/src/main/AndroidManifest.xml
  app/src/main/java/org/switchrecomp/android/RecompActivity.java
  app/src/main/res/values/strings.xml
  app/src/main/assets/recomp/   # project files except Cargo.toml, src/, target/
  rust/Cargo.toml               # cdylib `recomp_android`, depends on recomp-runtime + jni
  rust/src/lib.rs               # JNI entry point
  rust/src/generated.rs         # emitted src/main.rs with `main` made public
```

## Runtime Behavior
- On launch, `RecompActivity` extracts `assets/recomp/` into the app files directory.
- The JNI entry point changes into that directory (so relative segment paths resolve),
  redirects stdout/stderr to a log file, and calls the emitted `main`.
- The log and exit code are shown on screen and mirrored to logcat with tag `SwitchRecomp`.
- The emitted program runs once per process.
