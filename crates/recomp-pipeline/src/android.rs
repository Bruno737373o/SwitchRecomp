//! Android build target: wraps an emitted recompiled project in a Gradle app
//! that cross-compiles the Rust code with `cargo ndk` and packages it as an APK.

use pathdiff::diff_paths;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_ABIS: &[&str] = &["arm64-v8a", "armeabi-v7a", "x86_64"];
pub const SUPPORTED_ABIS: &[&str] = &["arm64-v8a", "armeabi-v7a", "x86", "x86_64"];
pub const DEFAULT_MIN_SDK: u32 = 21;
pub const DEFAULT_NDK_VERSION: &str = "26.3.11579264";

const ASSET_ROOT: &str = "app/src/main/assets/recomp";
const SKIPPED_PROJECT_ENTRIES: &[&str] = &["Cargo.toml", "Cargo.lock", "src", "target"];

const TEMPLATE_SETTINGS: &str = include_str!("../templates/android/settings.gradle");
const TEMPLATE_ROOT_BUILD: &str = include_str!("../templates/android/build.gradle");
const TEMPLATE_APP_BUILD: &str = include_str!("../templates/android/app.build.gradle");
const TEMPLATE_MANIFEST: &str = include_str!("../templates/android/AndroidManifest.xml");
const TEMPLATE_ACTIVITY: &str = include_str!("../templates/android/RecompActivity.java");
const TEMPLATE_LIB_RS: &str = include_str!("../templates/android/lib.rs");
const TEMPLATE_GRADLEW: &str = include_str!("../templates/android/gradlew");
const TEMPLATE_GRADLEW_BAT: &str = include_str!("../templates/android/gradlew.bat");
const TEMPLATE_WRAPPER_PROPERTIES: &str =
    include_str!("../templates/android/gradle/wrapper/gradle-wrapper.properties");
const TEMPLATE_WRAPPER_JAR: &[u8] =
    include_bytes!("../templates/android/gradle/wrapper/gradle-wrapper.jar");

#[derive(Debug, Clone)]
pub struct AndroidOptions {
    pub project_dir: PathBuf,
    pub out_dir: PathBuf,
    pub runtime_path: PathBuf,
    pub app_name: Option<String>,
    pub application_id: Option<String>,
    pub abis: Vec<String>,
    pub min_sdk: u32,
    pub ndk_version: String,
}

impl AndroidOptions {
    pub fn new(project_dir: PathBuf, out_dir: PathBuf, runtime_path: PathBuf) -> Self {
        Self {
            project_dir,
            out_dir,
            runtime_path,
            app_name: None,
            application_id: None,
            abis: DEFAULT_ABIS.iter().map(|abi| abi.to_string()).collect(),
            min_sdk: DEFAULT_MIN_SDK,
            ndk_version: DEFAULT_NDK_VERSION.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct AndroidReport {
    pub out_dir: PathBuf,
    pub app_name: String,
    pub application_id: String,
    pub abis: Vec<String>,
    pub assets: Vec<PathBuf>,
    pub files_written: Vec<PathBuf>,
}

pub fn emit_android_project(options: &AndroidOptions) -> Result<AndroidReport, String> {
    let project_dir = fs::canonicalize(&options.project_dir).map_err(|err| {
        format!(
            "project dir {} not found: {err}",
            options.project_dir.display()
        )
    })?;
    let main_rs_path = project_dir.join("src/main.rs");
    let main_rs = fs::read_to_string(&main_rs_path)
        .map_err(|err| format!("read {}: {err}", main_rs_path.display()))?;
    let generated_rs = expose_main(&main_rs)?;

    let runtime_path = fs::canonicalize(&options.runtime_path).map_err(|err| {
        format!(
            "runtime path {} not found: {err}",
            options.runtime_path.display()
        )
    })?;
    if !runtime_path.join("Cargo.toml").is_file() {
        return Err(format!(
            "runtime path {} does not contain Cargo.toml",
            runtime_path.display()
        ));
    }

    let title = read_title(&project_dir)?;
    let app_name = options.app_name.clone().unwrap_or_else(|| title.clone());
    if app_name.trim().is_empty() {
        return Err("app name must not be empty".to_string());
    }
    let application_id = match &options.application_id {
        Some(id) => id.clone(),
        None => default_application_id(&title),
    };
    validate_application_id(&application_id)?;
    let abis = validate_abis(&options.abis)?;
    if options.min_sdk < DEFAULT_MIN_SDK {
        return Err(format!(
            "min sdk {} is below the supported minimum {DEFAULT_MIN_SDK}",
            options.min_sdk
        ));
    }

    fs::create_dir_all(&options.out_dir)
        .map_err(|err| format!("create {}: {err}", options.out_dir.display()))?;
    let out_dir = fs::canonicalize(&options.out_dir).map_err(|err| err.to_string())?;
    if out_dir.starts_with(&project_dir) {
        return Err("out dir must not be inside the project dir".to_string());
    }

    let mut writer = Writer::new(&out_dir);
    writer.text("settings.gradle", TEMPLATE_SETTINGS)?;
    writer.text("build.gradle", TEMPLATE_ROOT_BUILD)?;
    writer.text(
        "gradle.properties",
        &gradle_properties(
            &application_id,
            &abis,
            options.min_sdk,
            &options.ndk_version,
        ),
    )?;
    writer.text("gradlew", TEMPLATE_GRADLEW)?;
    make_executable(&out_dir.join("gradlew"))?;
    writer.text("gradlew.bat", TEMPLATE_GRADLEW_BAT)?;
    writer.text(
        "gradle/wrapper/gradle-wrapper.properties",
        TEMPLATE_WRAPPER_PROPERTIES,
    )?;
    writer.bytes("gradle/wrapper/gradle-wrapper.jar", TEMPLATE_WRAPPER_JAR)?;
    writer.text("app/build.gradle", TEMPLATE_APP_BUILD)?;
    writer.text("app/src/main/AndroidManifest.xml", TEMPLATE_MANIFEST)?;
    writer.text(
        "app/src/main/java/org/switchrecomp/android/RecompActivity.java",
        TEMPLATE_ACTIVITY,
    )?;
    writer.text(
        "app/src/main/res/values/strings.xml",
        &strings_xml(&app_name),
    )?;

    let rust_dir = out_dir.join("rust");
    let runtime_dep = diff_paths(&runtime_path, &rust_dir).unwrap_or(runtime_path.clone());
    writer.text("rust/Cargo.toml", &rust_cargo_toml(&title, &runtime_dep)?)?;
    writer.text("rust/src/lib.rs", TEMPLATE_LIB_RS)?;
    writer.text("rust/src/generated.rs", &generated_rs)?;

    let asset_root = out_dir.join(ASSET_ROOT);
    if asset_root.exists() {
        fs::remove_dir_all(&asset_root)
            .map_err(|err| format!("clean {}: {err}", asset_root.display()))?;
    }
    let mut assets = Vec::new();
    copy_project_assets(&project_dir, &project_dir, &asset_root, &mut assets)?;
    assets.sort();
    for asset in &assets {
        writer.written.push(asset_root.join(asset));
    }
    let files_written = writer.written;

    Ok(AndroidReport {
        out_dir,
        app_name,
        application_id,
        abis,
        assets,
        files_written,
    })
}

/// Runs the Gradle wrapper in an emitted Android project and returns the APK path.
pub fn build_android_project(out_dir: &Path, release: bool) -> Result<PathBuf, String> {
    let (task, variant, file) = if release {
        ("assembleRelease", "release", "app-release-unsigned.apk")
    } else {
        ("assembleDebug", "debug", "app-debug.apk")
    };
    let status = Command::new(out_dir.join("gradlew"))
        .arg(task)
        .current_dir(out_dir)
        .status()
        .map_err(|err| format!("failed to run gradlew: {err}"))?;
    if !status.success() {
        return Err(format!("gradlew {task} failed with {status}"));
    }
    let apk = out_dir
        .join("app/build/outputs/apk")
        .join(variant)
        .join(file);
    if !apk.is_file() {
        return Err(format!("expected APK not found at {}", apk.display()));
    }
    Ok(apk)
}

struct Writer<'a> {
    root: &'a Path,
    written: Vec<PathBuf>,
}

impl<'a> Writer<'a> {
    fn new(root: &'a Path) -> Self {
        Self {
            root,
            written: Vec::new(),
        }
    }

    fn text(&mut self, rel: &str, contents: &str) -> Result<(), String> {
        self.bytes(rel, contents.as_bytes())
    }

    fn bytes(&mut self, rel: &str, contents: &[u8]) -> Result<(), String> {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create {}: {err}", parent.display()))?;
        }
        fs::write(&path, contents).map_err(|err| format!("write {}: {err}", path.display()))?;
        self.written.push(path);
        Ok(())
    }
}

fn expose_main(main_rs: &str) -> Result<String, String> {
    const SIGNATURE: &str = "\nfn main()";
    if !main_rs.contains(SIGNATURE) {
        return Err("emitted src/main.rs does not define `fn main()`".to_string());
    }
    Ok(main_rs.replacen(SIGNATURE, "\npub fn main()", 1))
}

fn read_title(project_dir: &Path) -> Result<String, String> {
    let manifest_path = project_dir.join("manifest.json");
    if let Ok(text) = fs::read_to_string(&manifest_path) {
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("parse {}: {err}", manifest_path.display()))?;
        if let Some(title) = value.get("title").and_then(|title| title.as_str()) {
            return Ok(title.to_string());
        }
    }
    let cargo_path = project_dir.join("Cargo.toml");
    let text = fs::read_to_string(&cargo_path)
        .map_err(|err| format!("read {}: {err}", cargo_path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|err| format!("parse {}: {err}", cargo_path.display()))?;
    value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string())
        .ok_or_else(|| "unable to determine project title".to_string())
}

pub fn default_application_id(title: &str) -> String {
    let mut segment: String = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    segment = segment.trim_matches('_').to_string();
    if segment.is_empty() {
        segment = "app".to_string();
    }
    if !segment.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
        segment = format!("app_{segment}");
    }
    format!("org.switchrecomp.{segment}")
}

pub fn validate_application_id(id: &str) -> Result<(), String> {
    let segments: Vec<&str> = id.split('.').collect();
    let valid = segments.len() >= 2
        && segments.iter().all(|segment| {
            let mut chars = segment.chars();
            matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic())
                && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid application id `{id}` (expected e.g. org.example.app)"
        ))
    }
}

fn validate_abis(abis: &[String]) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for abi in abis {
        if !SUPPORTED_ABIS.contains(&abi.as_str()) {
            return Err(format!(
                "unsupported ABI `{abi}` (supported: {})",
                SUPPORTED_ABIS.join(", ")
            ));
        }
        if !out.contains(abi) {
            out.push(abi.clone());
        }
    }
    if out.is_empty() {
        return Err("at least one ABI is required".to_string());
    }
    Ok(out)
}

fn gradle_properties(application_id: &str, abis: &[String], min_sdk: u32, ndk: &str) -> String {
    format!(
        "org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8\n\
         recompApplicationId={application_id}\n\
         recompAbis={abis}\n\
         recompMinSdk={min_sdk}\n\
         recompNdkVersion={ndk}\n",
        abis = abis.join(",")
    )
}

fn strings_xml(app_name: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n    <string name=\"app_name\">{}</string>\n</resources>\n",
        escape_android_string(app_name)
    )
}

fn escape_android_string(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '@' | '?' if out.is_empty() => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

fn rust_cargo_toml(title: &str, runtime_dep: &Path) -> Result<String, String> {
    let package = format!("recomp-android-{}", crate_suffix(title));
    let runtime =
        serde_json::to_string(&runtime_dep.to_string_lossy()).map_err(|err| err.to_string())?;
    Ok(format!(
        "[package]\n\
         name = \"{package}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\
         publish = false\n\
         \n\
         [lib]\n\
         name = \"recomp_android\"\n\
         crate-type = [\"cdylib\"]\n\
         path = \"src/lib.rs\"\n\
         \n\
         [dependencies]\n\
         recomp-runtime = {{ path = {runtime} }}\n\
         jni = \"0.21\"\n\
         \n\
         [workspace]\n"
    ))
}

fn crate_suffix(title: &str) -> String {
    let suffix: String = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let suffix = suffix.trim_matches('-').to_string();
    if suffix.is_empty() {
        "app".to_string()
    } else {
        suffix
    }
}

fn copy_project_assets(
    root: &Path,
    dir: &Path,
    dest_root: &Path,
    assets: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|err| format!("read {}: {err}", dir.display()))?
        .collect::<Result<_, _>>()
        .map_err(|err| err.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|err| err.to_string())?
            .to_path_buf();
        if dir == root
            && SKIPPED_PROJECT_ENTRIES
                .iter()
                .any(|skip| entry.file_name() == *skip)
        {
            continue;
        }
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        if file_type.is_dir() {
            copy_project_assets(root, &path, dest_root, assets)?;
        } else if file_type.is_file() {
            let dest = dest_root.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create {}: {err}", parent.display()))?;
            }
            fs::copy(&path, &dest).map_err(|err| format!("copy {}: {err}", path.display()))?;
            assets.push(rel);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|err| err.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|err| err.to_string())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_application_id_is_valid() {
        let id = default_application_id("Minimal Sample!");
        assert_eq!(id, "org.switchrecomp.minimal_sample");
        validate_application_id(&id).unwrap();
        let id = default_application_id("123");
        assert_eq!(id, "org.switchrecomp.app_123");
        validate_application_id(&id).unwrap();
    }

    #[test]
    fn rejects_invalid_application_ids() {
        for id in ["app", "org..app", "org.1app", "org.app-name", ""] {
            assert!(validate_application_id(id).is_err(), "{id}");
        }
    }

    #[test]
    fn escapes_app_name_for_android_resources() {
        assert_eq!(
            escape_android_string("Tom & Jerry's <\"Demo\">"),
            "Tom &amp; Jerry\\'s &lt;\\\"Demo\\\"&gt;"
        );
        assert_eq!(escape_android_string("@app?"), "\\@app?");
    }

    #[test]
    fn rejects_unknown_abis() {
        assert!(validate_abis(&["mips".to_string()]).is_err());
        assert!(validate_abis(&[]).is_err());
        assert_eq!(
            validate_abis(&["x86_64".to_string(), "x86_64".to_string()]).unwrap(),
            vec!["x86_64".to_string()]
        );
    }

    #[test]
    fn expose_main_makes_entry_public() {
        let src = "use recomp_runtime;\n\nfn main() -> Result<(), ()> {\n    Ok(())\n}\n";
        let out = expose_main(src).unwrap();
        assert!(out.contains("\npub fn main() -> Result<(), ()>"));
        assert!(expose_main("fn other() {}").is_err());
    }
}
