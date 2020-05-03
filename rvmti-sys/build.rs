// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::io::Read;
use std::path::Path;
use bindgen::callbacks::ParseCallbacks;
use bindgen::callbacks::IntKind;

#[derive(Debug)]
struct MacroCallback {
}

impl ParseCallbacks for MacroCallback {
    fn int_macro(&self, _name: &str, _value: i64) -> Option<IntKind> {
        if _name.starts_with("JNI_OK") {
            return Some(IntKind::I32)
        }
        return None;
    }
}

fn main() {
    env_logger::init();
    println!("cargo:rerun-if-changed=src/wrapper.h");

    let mut java_process = match Command::new("java")
        .arg("-XshowSettings:properties")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        {
            Err(why) => panic!("Couldn't spawn process: {}", why),
            Ok(process) => process,
        };
    match java_process.wait()
        {
            Err(why) => panic!("Couldn't wait until process is finished: {}", why),
            Ok(_) => (),
        }
    let mut java_out = String::new();
    match java_process.stderr.expect("Couldn't get process stderr").read_to_string(&mut java_out) {
        Err(why) => panic!("Couldn't read process stderr: {}", why),
        Ok(_) => (),
    }
    let java_home = java_out.lines().find(|line| line.contains("java.home")).and_then(|line| line.split(" = ").nth(1))
        .expect("java.home not found");
    let java_include_path = Path::new(java_home).join("include");
    let java_include_path_str = java_include_path.to_str().expect("Couldn't convert include path to string").to_owned();
    let java_md_include_path_str = if cfg!(target_os = "linux") {
        let java_md_include_path = java_include_path.join("linux");
        java_md_include_path.to_str().expect("Couldn't convert include path to string").to_owned()
    } else if cfg!(target_os = "windows") {
        let java_md_include_path = java_include_path.join("win32");
        java_md_include_path.to_str().expect("Couldn't convert include path to string").to_owned()
    } else if cfg!(target_os = "macos") {
        let java_md_include_path = java_include_path.join("darwin");
        java_md_include_path.to_str().expect("Couldn't convert include path to string").to_owned()
    } else if cfg!(target_os = "freebsd") {
        let java_md_include_path = java_include_path.join("freebsd");
        java_md_include_path.to_str().expect("Couldn't convert include path to string").to_owned()
    } else if cfg!(target_os = "openbsd") {
        let java_md_include_path = java_include_path.join("openbsd");
        java_md_include_path.to_str().expect("Couldn't convert include path to string").to_owned()
    } else {
        panic!("Path to include files is unknown for the current platform");
    };

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", &java_include_path_str))
        .clang_arg(format!("-I{}", &java_md_include_path_str))
        .header("src/wrapper.h")
        .with_codegen_config(bindgen::CodegenConfig::all())
        .whitelist_recursively(true)
        .parse_callbacks(Box::new(MacroCallback {}))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
