/*
 Copyright 2021 Google LLC

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

      https://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 */

extern crate cc;

#[cfg(feature="binja")]
use std::env;

#[cfg(feature="binja")]
use std::fs::File;


#[cfg(feature="binja")]
use std::io::BufReader;

#[cfg(feature="binja")]
use std::path::PathBuf;

#[cfg(all(feature="binja", target_os = "macos"))]
static LASTRUN_PATH: (&str, &str) = ("HOME", "Library/Application Support/Binary Ninja/lastrun");

#[cfg(all(feature="binja", target_os = "linux"))]
static LASTRUN_PATH: (&str, &str) = ("HOME", ".binaryninja/lastrun");

#[cfg(all(feature="binja", windows))]
static LASTRUN_PATH: (&str, &str) = ("APPDATA", "Binary Ninja\\lastrun");


// Check last run location for path to BinaryNinja; Otherwise check the default install locations
#[cfg(feature="binja")]
fn link_path() -> PathBuf {
    use std::io::prelude::*;

    let home = PathBuf::from(env::var(LASTRUN_PATH.0).unwrap());
    let lastrun = PathBuf::from(&home).join(LASTRUN_PATH.1);

    File::open(lastrun)
        .and_then(|f| {
            let mut binja_path = String::new();
            let mut reader = BufReader::new(f);

            reader.read_line(&mut binja_path)?;
            Ok(PathBuf::from(binja_path.trim()))
        })
        .unwrap_or_else(|_| {
            #[cfg(target_os = "macos")]
            return PathBuf::from("/Applications/Binary Ninja.app/Contents/MacOS");

            #[cfg(target_os = "linux")]
            return home.join("binaryninja");

            #[cfg(windows)]
            return PathBuf::from(env::var("PROGRAMFILES").unwrap())
                .join("Vector35\\BinaryNinja\\");
        })
}

#[cfg(feature="binja")]
fn binja_build() {
    // Use BINARYNINJADIR first for custom BN builds/configurations (BN devs/build server), fallback on defaults
    let install_path = env::var("BINARYNINJADIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| link_path());

    #[cfg(target_os = "linux")]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{},-L{},-l:libbinaryninjacore.so.1",
        install_path.to_str().unwrap(),
        install_path.to_str().unwrap(),
    );

    #[cfg(target_os = "macos")]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{},-L{},-lbinaryninjacore",
        install_path.to_str().unwrap(),
        install_path.to_str().unwrap(),
    );

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=binaryninjacore");
        println!("cargo:rustc-link-search={}", install_path.to_str().unwrap());
    }
}

fn main() {
    cc::Build::new()
        .include("third_party/grammars/")
        .file("third_party/grammars/weggli-c/src/parser.c")
        .flag("-std=c99")
        .flag("-w")
        .compile("tree-sitter-c");

    cc::Build::new()
        .cpp(true)
        .include("third_party/grammars/")
        .file("third_party/grammars/weggli-cpp/src/scanner.cc")
        .flag("-w")
        .compile("tree-sitter-cpp-scanner");

    cc::Build::new()
        .include("third_party/grammars/")
        .file("third_party/grammars/weggli-cpp/src/parser.c")
        .flag("-w")
        .compile("tree-sitter-cpp-parser");

    #[cfg(feature="binja")]
    binja_build();
}
