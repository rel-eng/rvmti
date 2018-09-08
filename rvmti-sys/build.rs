// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate bindgen;
extern crate cc;
extern crate log;
extern crate env_logger;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::error::Error;
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
            return Some(IntKind::Custom{name: "::std::os::raw::c_int", is_signed: true})
        }
        return None;
    }
}

fn main() {
    env_logger::init();
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/glue.c");
    println!("cargo:rerun-if-changed=src/glue.h");

    let mut java_process = match Command::new("java")
        .arg("-XshowSettings:properties")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        {
            Err(why) => panic!("Couldn't spawn process: {}", why.description()),
            Ok(process) => process,
        };
    match java_process.wait()
        {
            Err(why) => panic!("Couldn't wait until process is finished: {}", why.description()),
            Ok(_) => (),
        }
    let mut java_out = String::new();
    match java_process.stderr.expect("Couldn't get process stderr").read_to_string(&mut java_out) {
        Err(why) => panic!("Couldn't read process stderr: {}", why.description()),
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

    cc::Build::new()
        .cpp(false)
        .file("src/glue.c")
        .static_flag(true)
        .include(&java_include_path_str)
        .include(&java_md_include_path_str)
        .compile("libglue.a");

    println!("cargo:rustc-link-lib=static=glue");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", &java_include_path_str))
        .clang_arg(format!("-I{}", &java_md_include_path_str))
        .header("src/wrapper.h")
        .with_codegen_config(bindgen::CodegenConfig::all())
        .whitelist_recursively(true)
        .whitelist_type("JavaVM")
        .opaque_type("JavaVM")
        .whitelist_type("jvmtiEnv")
        .opaque_type("jvmtiEnv")
        .whitelist_type("jint")
        .whitelist_type("jthread")
        .whitelist_type("jmethodID")
        .whitelist_type("jlocation")
        .whitelist_type("jclass")
        .whitelist_type("jvmtiAddrLocationMap")
        .whitelist_type("jfieldID")
        .whitelist_type("jvalue")
        .whitelist_type("jboolean")
        .whitelist_type("jobject")
        .whitelist_type("jlong")
        .whitelist_type("jvmtiError")
        .whitelist_type("jvmtiCapabilities")
        .whitelist_type("JvmtiEventCallbacksStatus")
        .whitelist_type("JNIEnv")
        .whitelist_type("jvmtiEventMode")
        .whitelist_type("jvmtiEvent")
        .whitelist_type("jvmtiLineNumberEntry")
        .whitelist_type("jvmtiCMLRKind")
        .whitelist_type("jvmtiCompiledMethodLoadRecordHeader")
        .whitelist_type("jvmtiCompiledMethodLoadInlineRecord")
        .whitelist_type("jvmtiCompiledMethodLoadDummyRecord")
        .whitelist_type("PCStackInfo")
        .opaque_type("jvmtiCapabilities")
        .opaque_type("JvmtiEventCallbacksStatus")
        .opaque_type("JNIEnv")
        .whitelist_var("JVMTI_VERSION_1_0")
        .whitelist_var("JVMTI_VERSION_1_1")
        .whitelist_var("JVMTI_VERSION_1_2")
        .whitelist_var("JVMTI_VERSION_9")
        .whitelist_var("JVMTI_VERSION")
        .whitelist_var("JNI_OK")
        .whitelist_var("JNI_ERR")
        .whitelist_var("JNI_EDETACHED")
        .whitelist_var("JNI_EVERSION")
        .whitelist_var("JNI_ENOMEM")
        .whitelist_var("JNI_EEXIST")
        .whitelist_var("JNI_EINVAL")
        .whitelist_var("JVMTI_CMLR_MAJOR_VERSION.*")
        .whitelist_var("JVMTI_CMLR_MINOR_VERSION.*")
        .whitelist_function("java_vm_get_env")
        .whitelist_function("jvmti_env_dispose_environment")
        .whitelist_function("jvmti_env_add_capabilities")
        .whitelist_function("set_jvmti_capability_.*")
        .whitelist_function("get_jvmti_capability_.*")
        .whitelist_function("alloc_empty_jvmti_capabilities")
        .whitelist_function("free_jvmti_capabilities")
        .whitelist_function("alloc_empty_jvmti_event_callback_status")
        .whitelist_function("free_jvmti_event_callback_status")
        .whitelist_function("set_jvmti_event_callbacks")
        .whitelist_function("set_jvmti_event_status_.*")
        .whitelist_function("get_jvmti_event_status_.*")
        .whitelist_function("jvmti_env_set_event_notification_mode")
        .whitelist_function("jvmti_env_get_method_name")
        .whitelist_function("jvmti_env_deallocate")
        .whitelist_function("jvmti_env_get_method_declaring_class")
        .whitelist_function("jvmti_env_get_class_signature")
        .whitelist_function("jvmti_env_get_source_file_name")
        .whitelist_function("jvmti_env_get_line_number_table")
        .whitelist_function("jvmti_env_is_method_native")
        .parse_callbacks(Box::new(MacroCallback {}))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
