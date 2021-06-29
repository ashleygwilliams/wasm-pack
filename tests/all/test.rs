use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::env;
use utils::fixture;

#[test]
fn it_can_run_node_tests() {
    let fixture = fixture::wbg_test_node();
    fixture.install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .success();
}

#[test]
fn it_can_run_tests_with_different_wbg_test_and_wbg_versions() {
    let fixture = fixture::wbg_test_diff_versions();
    fixture.install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .success();
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn it_can_run_browser_tests() {
    let fixture = fixture::wbg_test_browser();
    fixture.install_local_wasm_bindgen();

    let firefox = cfg!(any(
        all(target_os = "linux", target_arch = "x86"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
        all(target_os = "windows", target_arch = "x86_64")
    ));
    if firefox {
        fixture.install_local_geckodriver();
    }

    let chrome = cfg!(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86")
    ));
    if chrome {
        fixture.install_local_chromedriver();
    }

    let safari = cfg!(target_os = "macos");

    if !firefox && !chrome && !safari {
        return;
    }

    let mut cmd = fixture.wasm_pack();
    cmd.arg("test").arg("--headless");

    if firefox {
        cmd.arg("--firefox");
    }
    if chrome {
        cmd.arg("--chrome");
    }
    if safari {
        cmd.arg("--safari");
    }

    let _lock = fixture.lock();
    cmd.assert().success();
}

#[test]
fn it_can_run_failing_tests() {
    let fixture = fixture::wbg_test_fail();
    fixture.install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Running Wasm tests with wasm-bindgen-test failed",
        ));
}

#[test]
#[cfg(any(
    all(target_os = "linux", target_arch = "x86"),
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86"),
    all(target_os = "windows", target_arch = "x86_64")
))]
fn it_can_find_a_webdriver_on_path() {
    let fixture = fixture::wbg_test_browser();
    let local_geckodriver = fixture.install_local_geckodriver();
    let local_wasm_bindgen = fixture.install_local_wasm_bindgen();

    let mut paths: Vec<_> = env::split_paths(&env::var("PATH").unwrap()).collect();
    paths.insert(0, local_geckodriver.parent().unwrap().to_path_buf());
    paths.insert(0, local_wasm_bindgen.parent().unwrap().to_path_buf());
    let path = env::join_paths(paths).unwrap();

    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .env("PATH", &path)
        .arg("test")
        .arg("--firefox")
        .arg("--headless")
        .arg("--mode")
        .arg("no-install")
        .assert()
        .success();
}

#[test]
fn it_requires_node_or_a_browser() {
    let fixture = fixture::wbg_test_node();
    fixture.install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Must specify at least one of"));
}

#[test]
fn the_headless_flag_requires_a_browser() {
    let fixture = fixture::wbg_test_node();
    fixture.install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .arg("--headless")
        .assert()
        .failure()
        .stderr(predicates::str::contains("only applies to browser tests"));
}

#[test]
fn complains_about_missing_wasm_bindgen_test_dependency() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = ["The wasm-pack developers"]
                description = "so awesome rust+wasm package"
                license = "WTFPL"
                name = "missing-wbg-test"
                repository = "https://github.com/rustwasm/wasm-pack.git"
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"

                [dev-dependencies]
                # no wasm-bindgen-test dep here!
            "#,
        )
        .hello_world_src_lib()
        .install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Ensure that you have \"wasm-bindgen-test\" as a dependency in your Cargo.toml file",
        ))
        .stderr(predicates::str::contains("[dev-dependencies]"))
        .stderr(predicates::str::contains("wasm-bindgen-test = \"0.2\""));
}

#[test]
fn renamed_crate_name_works() {
    let fixture = fixture::Fixture::new();
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

                [dev-dependencies]
                wasm-bindgen-test = "0.2"
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
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .success();
}

#[test]
fn cdylib_not_required() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [dependencies]
                wasm-bindgen = "0.2"

                [dev-dependencies]
                wasm-bindgen-test = "0.2"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 { 1 }
            "#,
        )
        .file(
            "tests/foo.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn smoke() {
                    foo::foo();
                }
            "#,
        )
        .install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .success();
}

#[test]
fn test_output_is_printed_once_in_both_stdout_and_failures() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .cargo_toml("test-output-printed-once")
        .hello_world_src_lib()
        .file(
            "tests/node.rs",
            r#"
                extern crate wasm_bindgen;
                extern crate wasm_bindgen_test;
                use wasm_bindgen::prelude::*;
                use wasm_bindgen_test::*;

                #[wasm_bindgen]
                extern {
                    #[wasm_bindgen(js_namespace = console)]
                    fn log(s: &str);
                }

                #[wasm_bindgen_test]
                fn yabba() {
                    log("YABBA DABBA DOO");
                    assert_eq!(1, 2);
                }
            "#,
        )
        .install_local_wasm_bindgen();
    let _lock = fixture.lock();

    // there will be only one log in stdout, and only one log in failures
    let log_cnt = 1;
    fixture
        .wasm_pack()
        .arg("test")
        .arg("--node")
        .assert()
        .failure()
        .stdout(predicate::function(|out: &str| {
            // but the out string will capture both stdout and failures,
            // so we will get a log that count twice
            out.matches("YABBA DABBA DOO").count() == log_cnt * 2
        }));
}

#[test]
fn extra_options_is_passed_to_cargo_when_building_tests() {
    let fixture = fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [dev-dependencies]
                wasm-bindgen-test = "0.2"

                [features]
                default = ["native"]
                native = []
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                pub fn foo() -> u32 {
                    #[cfg(feature = "native")]
                    compile_error!("Test should pass through `--no-default-features` for this to pass.");

                    1
                }
            "#,
        )
        .file(
            "tests/foo.rs",
            r#"
                extern crate wasm_bindgen_test;
                use wasm_bindgen_test::*;

                #[wasm_bindgen_test]
                fn smoke() {
                    foo::foo();
                }

                #[wasm_bindgen_test]
                fn fire() {
                    panic!("This should be filtered from test execution.");
                }
            "#,
        )
        .install_local_wasm_bindgen();
    let _lock = fixture.lock();
    fixture
        .wasm_pack()
        .args(&["test", "--node", "--no-default-features", "--", "smoke"])
        .assert()
        .success();
}
