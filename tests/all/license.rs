extern crate failure;
extern crate wasm_pack;

use std::fs;

use utils::{self, fixture};
use wasm_pack::license;
use wasm_pack::manifest::CrateData;

#[test]
fn it_copies_a_license_default_path() {
    let fixture = fixture::single_license();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None, None);

    assert!(license::copy_from_crate(&crate_data.unwrap(), &fixture.path, &out_dir).is_ok());

    let crate_license_path = fixture.path.join("LICENSE");
    let pkg_license_path = out_dir.join("LICENSE");
    println!(
        "wasm-pack: should have copied LICENSE from '{}' to '{}'",
        crate_license_path.display(),
        pkg_license_path.display()
    );
    assert!(fs::metadata(&crate_license_path).is_ok());

    assert!(fs::metadata(&pkg_license_path).is_ok());

    let crate_license = utils::file::read_file(&crate_license_path).unwrap();
    let pkg_license = utils::file::read_file(&pkg_license_path).unwrap();
    assert_eq!(crate_license, pkg_license);
}

#[test]
fn it_copies_a_license_provided_path() {
    let fixture = fixture::single_license();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None, None);

    assert!(license::copy_from_crate(&crate_data.unwrap(), &fixture.path, &out_dir).is_ok());
    let crate_license_path = fixture.path.join("LICENSE");
    let pkg_license_path = out_dir.join("LICENSE");
    println!(
        "wasm-pack: should have copied LICENSE from '{}' to '{}'",
        crate_license_path.display(),
        pkg_license_path.display()
    );
    assert!(fs::metadata(&crate_license_path).is_ok());
    assert!(fs::metadata(&pkg_license_path).is_ok());

    let crate_license = utils::file::read_file(&crate_license_path).unwrap();
    let pkg_license = utils::file::read_file(&pkg_license_path).unwrap();
    assert_eq!(crate_license, pkg_license);
}

#[test]
fn it_copies_all_licenses_default_path() {
    let fixture = fixture::dual_license();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None, None);

    assert!(license::copy_from_crate(&crate_data.unwrap(), &fixture.path, &out_dir).is_ok());

    let crate_license_path = fixture.path.join("LICENSE-WTFPL");
    let pkg_license_path = out_dir.join("LICENSE-WTFPL");

    let crate_license_path_2 = fixture.path.join("LICENSE-MIT");
    let pkg_license_path_2 = out_dir.join("LICENSE-MIT");

    println!(
        "wasm-pack: should have copied LICENSE from '{}' to '{}'",
        crate_license_path.display(),
        pkg_license_path.display()
    );
    assert!(fs::metadata(&crate_license_path).is_ok());
    assert!(fs::metadata(&pkg_license_path).is_ok());

    assert!(fs::metadata(&crate_license_path_2).is_ok());
    assert!(fs::metadata(&pkg_license_path_2).is_ok());

    let crate_license = utils::file::read_file(&crate_license_path).unwrap();
    let pkg_license = utils::file::read_file(&pkg_license_path).unwrap();
    assert_eq!(crate_license, pkg_license);

    let crate_license_2 = utils::file::read_file(&crate_license_path_2).unwrap();
    let pkg_license_2 = utils::file::read_file(&pkg_license_path_2).unwrap();
    assert_eq!(crate_license_2, pkg_license_2);
}

#[test]
fn it_copies_all_licenses_provided_path() {
    let fixture = fixture::dual_license();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None, None);

    assert!(license::copy_from_crate(&crate_data.unwrap(), &fixture.path, &out_dir).is_ok());

    let crate_license_path = fixture.path.join("LICENSE-WTFPL");
    let pkg_license_path = out_dir.join("LICENSE-WTFPL");

    let crate_license_path_2 = fixture.path.join("LICENSE-MIT");
    let pkg_license_path_2 = out_dir.join("LICENSE-MIT");

    println!(
        "wasm-pack: should have copied LICENSE from '{}' to '{}'",
        crate_license_path.display(),
        pkg_license_path.display()
    );
    assert!(fs::metadata(&crate_license_path).is_ok());
    assert!(fs::metadata(&pkg_license_path).is_ok());

    assert!(fs::metadata(&crate_license_path_2).is_ok());
    assert!(fs::metadata(&pkg_license_path_2).is_ok());

    let crate_license = utils::file::read_file(&crate_license_path).unwrap();
    let pkg_license = utils::file::read_file(&pkg_license_path).unwrap();
    assert_eq!(crate_license, pkg_license);

    let crate_license_2 = utils::file::read_file(&crate_license_path_2).unwrap();
    let pkg_license_2 = utils::file::read_file(&pkg_license_path_2).unwrap();
    assert_eq!(crate_license_2, pkg_license_2);
}

#[test]
fn it_copies_a_non_standard_license_provided_path() {
    let license_file = "NON-STANDARD-LICENSE";
    let fixture = fixture::non_standard_license(license_file);
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None, None);

    assert!(license::copy_from_crate(&crate_data.unwrap(), &fixture.path, &out_dir).is_ok());

    let crate_license_path = fixture.path.join(license_file);
    let pkg_license_path = out_dir.join(license_file);
    println!(
        "wasm-pack: should have copied LICENSE from '{}' to '{}'",
        crate_license_path.display(),
        pkg_license_path.display()
    );
    assert!(fs::metadata(&crate_license_path).is_ok());

    assert!(fs::metadata(&pkg_license_path).is_ok());

    let crate_license = utils::file::read_file(&crate_license_path).unwrap();
    let pkg_license = utils::file::read_file(&pkg_license_path).unwrap();
    assert_eq!(crate_license, pkg_license);
}
