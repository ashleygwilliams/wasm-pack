use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use utils;

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::not_a_crate();
    fixture
        .wasm_pack()
        .arg("build")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing a `Cargo.toml`"));
}

#[test]
fn it_should_build_js_hello_world_example() {
    let fixture = utils::fixture::js_hello_world();
    fixture.wasm_pack().arg("build").assert().success();
}

#[test]
fn it_should_build_crates_in_a_workspace() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["blah"]
            "#,
        )
        .file(
            Path::new("blah").join("Cargo.toml"),
            r#"
                [package]
                authors = ["The wasm-pack developers"]
                description = "so awesome rust+wasm package"
                license = "WTFPL"
                name = "blah"
                repository = "https://github.com/rustwasm/wasm-pack.git"
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"
            "#,
        )
        .file(
            Path::new("blah").join("src").join("lib.rs"),
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn hello() -> u32 { 42 }
            "#,
        )
        .install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .current_dir(&fixture.path.join("blah"))
        .arg("build")
        .assert()
        .success();
}

#[test]
fn renamed_crate_name_works() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["cdylib"]
                name = 'bar'

                [dependencies]
                wasm-bindgen = "0.2"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn one() -> u32 { 1 }
            "#,
        )
        .install_local_wasm_bindgen();
    fixture.wasm_pack().arg("build").assert().success();
}

#[test]
fn dash_dash_web_target_has_error_on_old_bindgen() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["cdylib"]
                name = 'bar'

                [dependencies]
                wasm-bindgen = "=0.2.37"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn one() -> u32 { 1 }
            "#,
        )
        .install_local_wasm_bindgen();
    let cmd = fixture
        .wasm_pack()
        .arg("build")
        .arg("--target")
        .arg("web")
        .assert()
        .failure();
    let output = String::from_utf8(cmd.get_output().stderr.clone()).unwrap();

    assert!(
        output.contains("0.2.39"),
        "Output did not contain '0.2.39', output was {}",
        output
    );
}

#[test]
fn it_should_build_nested_project_with_transitive_dependencies() {
    let fixture = utils::fixture::transitive_dependencies();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .current_dir(fixture.path.join("main"))
        .arg("build")
        .assert()
        .success();
}

#[test]
fn build_different_profiles() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();

    for profile in ["--dev", "--debug", "--profiling", "--release"]
        .iter()
        .cloned()
    {
        fixture
            .wasm_pack()
            .arg("build")
            .arg(profile)
            .assert()
            .success();
    }
}

#[test]
fn build_with_and_without_wasm_bindgen_debug() {
    for debug in [true, false].iter().cloned() {
        let fixture = utils::fixture::Fixture::new();
        fixture
            .readme()
            .file(
                "Cargo.toml",
                format!(
                    r#"
                    [package]
                    authors = ["The wasm-pack developers"]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "whatever"
                    repository = "https://github.com/rustwasm/wasm-pack.git"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    wasm-bindgen = "0.2"

                    [package.metadata.wasm-pack.profile.dev.wasm-bindgen]
                    debug-js-glue = {}
                    "#,
                    debug
                ),
            )
            .file(
                "src/lib.rs",
                r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub struct MyThing {}

                #[wasm_bindgen]
                impl MyThing {
                    #[wasm_bindgen(constructor)]
                    pub fn new() -> MyThing {
                        MyThing {}
                    }
                }

                #[wasm_bindgen]
                pub fn take(foo: MyThing) {
                    drop(foo);
                }
                "#,
            )
            .install_local_wasm_bindgen();

        fixture
            .wasm_pack()
            .arg("build")
            .arg("--dev")
            .assert()
            .success();

        let contents = fs::read_to_string(fixture.path.join("pkg/whatever_bg.js")).unwrap();
        let contains_move_assertions =
            contents.contains("throw new Error('Attempt to use a moved value')");
        assert_eq!(
            contains_move_assertions, debug,
            "Should contain moved value assertions iff debug assertions are enabled. \
             Contains move assertions? {}. \
             Is a debug JS glue build? {}.",
            contains_move_assertions, debug,
        );
    }
}

#[test]
fn build_with_arbitrary_cargo_options() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--")
        .arg("--no-default-features")
        .assert()
        .success();
}

#[test]
fn build_no_install() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--mode")
        .arg("no-install")
        .assert()
        .success();
}

#[test]
fn build_force() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--mode")
        .arg("force")
        .assert()
        .success();
}

#[test]
fn bin_crate_behavior_identical() {
    let fixture = utils::fixture::bin_crate();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--target")
        .arg("nodejs")
        .assert()
        .success();
    let native_output = fixture.command("cargo").arg("run").output().unwrap();
    assert!(native_output.status.success());
    assert_eq!(native_output.stdout, b"Hello, World\n");
    let wasm_output = fixture.command("node").arg("pkg/foo.js").output().unwrap();
    assert!(wasm_output.status.success());
    assert_eq!(wasm_output.stdout, b"Hello, World\n");
}

#[test]
fn multi_bin_crate_procs_all() {
    let fixture = utils::fixture::multi_bin_crate();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--target")
        .arg("nodejs")
        .assert()
        .success();
    let pkg_path = |x: &str| {
        let mut path = fixture.path.clone();
        path.push("pkg");
        path.push(x);
        path
    };
    assert!(pkg_path("foo.js").exists());
    assert!(pkg_path("sample.js").exists());
}

#[test]
fn builds_examples() {
    let fixture = utils::fixture::bin_example_crate();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--target")
        .arg("nodejs")
        .arg("--example")
        .arg("example")
        .assert()
        .success();
    let mut path = fixture.path.clone();
    path.push("pkg");
    path.push("example.js");
    assert!(path.exists());
}

#[test]
fn build_from_new() {
    let fixture = utils::fixture::not_a_crate();
    let name = "generated-project";
    fixture.wasm_pack().arg("new").arg(name).assert().success();
    let project_location = fixture.path.join(&name);
    fixture
        .wasm_pack()
        .arg("build")
        .arg(&project_location)
        .assert()
        .success();
}

#[test]
fn build_crates_with_same_names() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "somename1/Cargo.toml",
            r#"
            [package]
            authors = ["The wasm-pack developers"]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "somename"
            repository = "https://github.com/rustwasm/wasm-pack.git"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "0.2"
            somenameother = { path = "../somename2", package = "somename" }
            "#,
        )
        .file(
            "somename1/src/lib.rs",
            r#"
            extern crate wasm_bindgen;
            use wasm_bindgen::prelude::*;
            #[wasm_bindgen]
            pub fn method() -> i32 {
                somenameother::method()
            }
            "#,
        )
        .file(
            "somename2/Cargo.toml",
            r#"
            [package]
            authors = ["The wasm-pack developers"]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "somename"
            repository = "https://github.com/rustwasm/wasm-pack.git"
            version = "0.1.1"

            [lib]
            crate-type = ["rlib"]
            "#,
        )
        .file(
            "somename2/src/lib.rs",
            r#"
            pub fn method() -> i32 {
                0
            }
            "#,
        );
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .current_dir(fixture.path.join("somename1"))
        .arg("build")
        .assert()
        .success();
}
