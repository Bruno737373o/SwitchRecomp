use recomp_pipeline::android::{emit_android_project, AndroidOptions};
use recomp_pipeline::{run_pipeline, PipelineOptions};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn emit_sample(sample: &str, out: &Path) -> PathBuf {
    let root = repo_root();
    let sample_dir = root.join("samples").join(sample);
    let project_dir = out.join("project");
    run_pipeline(PipelineOptions {
        module_path: sample_dir.join("module.json"),
        config_path: sample_dir.join("title.toml"),
        provenance_path: sample_dir.join("provenance.toml"),
        out_dir: project_dir.clone(),
        runtime_path: root.join("crates/recomp-runtime"),
    })
    .expect("pipeline");
    project_dir
}

#[test]
fn android_project_wraps_memory_image_sample() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_dir = emit_sample("memory-image", temp.path());
    let out_dir = temp.path().join("android");

    let mut options = AndroidOptions::new(
        project_dir.clone(),
        out_dir.clone(),
        repo_root().join("crates/recomp-runtime"),
    );
    options.app_name = Some("Memory & Image".to_string());
    options.abis = vec!["x86_64".to_string()];
    let report = emit_android_project(&options).expect("emit android project");

    assert_eq!(report.app_name, "Memory & Image");
    assert!(report.application_id.starts_with("org.switchrecomp."));
    assert_eq!(report.abis, vec!["x86_64".to_string()]);

    for rel in [
        "settings.gradle",
        "build.gradle",
        "gradlew",
        "gradle/wrapper/gradle-wrapper.jar",
        "app/build.gradle",
        "app/src/main/AndroidManifest.xml",
        "app/src/main/java/org/switchrecomp/android/RecompActivity.java",
        "rust/Cargo.toml",
        "rust/src/lib.rs",
    ] {
        assert!(out_dir.join(rel).is_file(), "missing {rel}");
    }

    let generated = fs::read_to_string(out_dir.join("rust/src/generated.rs")).unwrap();
    assert!(generated.contains("\npub fn main()"));
    assert!(generated.contains("segments/data-0.bin"));

    let assets: Vec<String> = report
        .assets
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();
    assert!(assets.contains(&"segments/data-0.bin".to_string()));
    assert!(assets.contains(&"manifest.json".to_string()));
    assert!(!assets.iter().any(|asset| asset.starts_with("src/")));
    assert_eq!(
        fs::read(out_dir.join("app/src/main/assets/recomp/segments/data-0.bin")).unwrap(),
        fs::read(project_dir.join("segments/data-0.bin")).unwrap()
    );

    let strings = fs::read_to_string(out_dir.join("app/src/main/res/values/strings.xml")).unwrap();
    assert!(strings.contains(">Memory &amp; Image<"));

    let properties = fs::read_to_string(out_dir.join("gradle.properties")).unwrap();
    assert!(properties.contains("recompAbis=x86_64\n"));

    let cargo_toml = fs::read_to_string(out_dir.join("rust/Cargo.toml")).unwrap();
    let cargo: toml::Value = toml::from_str(&cargo_toml).unwrap();
    let runtime = cargo["dependencies"]["recomp-runtime"]["path"]
        .as_str()
        .unwrap();
    assert!(out_dir
        .join("rust")
        .join(runtime)
        .join("Cargo.toml")
        .is_file());
    assert!(cargo.get("workspace").is_some());
}

#[test]
fn android_project_rejects_bad_inputs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project_dir = emit_sample("minimal", temp.path());
    let runtime = repo_root().join("crates/recomp-runtime");

    let mut options = AndroidOptions::new(
        project_dir.clone(),
        temp.path().join("android"),
        runtime.clone(),
    );
    options.application_id = Some("not-valid".to_string());
    assert!(emit_android_project(&options).is_err());

    let mut options = AndroidOptions::new(
        project_dir.clone(),
        temp.path().join("android"),
        runtime.clone(),
    );
    options.abis = vec!["mips".to_string()];
    assert!(emit_android_project(&options).is_err());

    let options = AndroidOptions::new(project_dir.clone(), project_dir.join("android"), runtime);
    assert!(emit_android_project(&options).is_err());

    let options = AndroidOptions::new(
        project_dir,
        temp.path().join("android"),
        temp.path().join("missing-runtime"),
    );
    assert!(emit_android_project(&options).is_err());
}
