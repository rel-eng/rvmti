// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar, c_void};
use std::string::FromUtf8Error;
use std::str;
use std::ptr;
use std::panic;
use std::slice;
use std::mem::size_of;

use log::{debug, warn, error};
use failure_derive::Fail;

use crate::agent_on_load;
use crate::agent_on_unload;
use crate::jvmti_event_dynamic_code_generated;
use crate::jvmti_event_compiled_method_load;

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Agent_OnLoad(vm: *mut rvmti_sys::JavaVM, options: *const c_char, reserved: *const c_void) -> rvmti_sys::jint {
    let log_init_result = panic::catch_unwind(|| {
        env_logger::init()
    });
    match log_init_result {
        Ok(r) => {},
        Err(e) => {
            println!("Failed to initialize logger: {:?}", e);
            return -1;
        }
    }
    let result = panic::catch_unwind(|| {
        debug!("Agent 'on load'");
        // Options is a platform string, not modified utf-8 string, see https://bugs.openjdk.java.net/browse/JDK-5049313
        let options_string = from_platform(options);
        return match options_string {
            Ok(s) => agent_on_load(&Jvm { vm }, &s),
            Err(e) => {
                error!("Failed to process options string {}", e);
                -1
            },
        }
    });
    match result {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to load agent: {:?}", e);
            -1
        }
    }
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Agent_OnUnload(vm: *mut rvmti_sys::JavaVM) {
    debug!("Agent 'on unload'");
    let result = panic::catch_unwind(|| {
        agent_on_unload(&Jvm { vm })
    });
    match result {
        Ok(()) => (),
        Err(e) => {
            warn!("Failed to unload agent: {:?}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn jvmti_event_breakpoint_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                 _jni_env: *mut rvmti_sys::JNIEnv,
                                                 _thread: rvmti_sys::jthread,
                                                 _method: rvmti_sys::jmethodID,
                                                 _location: rvmti_sys::jlocation)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_class_file_load_hook_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                           _jni_env: *mut rvmti_sys::JNIEnv,
                                                           _class_being_redefined: rvmti_sys::jclass,
                                                           _loader: rvmti_sys::jobject,
                                                           _name: *const ::std::os::raw::c_char,
                                                           _protection_domain: rvmti_sys::jobject,
                                                           _class_data_len: rvmti_sys::jint,
                                                           _class_data: *const ::std::os::raw::c_uchar,
                                                           _new_class_data_len: *mut rvmti_sys::jint,
                                                           _new_class_data: *mut *mut ::std::os::raw::c_uchar)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_class_load_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                 _jni_env: *mut rvmti_sys::JNIEnv,
                                                 _thread: rvmti_sys::jthread,
                                                 _klass: rvmti_sys::jclass)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_class_prepare_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                    _jni_env: *mut rvmti_sys::JNIEnv,
                                                    _thread: rvmti_sys::jthread,
                                                    _klass: rvmti_sys::jclass)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_compiled_method_load_handler(jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                           method: rvmti_sys::jmethodID,
                                                           code_size: rvmti_sys::jint,
                                                           code_addr: *const ::std::os::raw::c_void,
                                                           map_length: rvmti_sys::jint,
                                                           map: *const rvmti_sys::jvmtiAddrLocationMap,
                                                           compile_info: *const ::std::os::raw::c_void)
{
    let result = panic::catch_unwind(|| {
        let env = &mut JvmtiEnv::cons(jvmti_env);
        let method_id = JMethodId{method};
        let address_locations = as_address_location_slice(map_length, map).map(|t| t.iter()
            .map(|e| AddressLocationEntry{start_address: e.start_address as usize, location: e.location}).collect());
        let compile_infos = to_compile_infos(compile_info);
        jvmti_event_compiled_method_load(env, &method_id, &address_locations, &compile_infos, code_addr as usize, code_size as usize)
    });
    match result {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to handle 'compiled method load' event: {:?}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn jvmti_event_compiled_method_unload_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                             _method: rvmti_sys::jmethodID,
                                                             _code_addr: *const ::std::os::raw::c_void)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_data_dump_request_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv) {
}

#[no_mangle]
pub extern "C" fn jvmti_event_dynamic_code_generated_handler(jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                             name: *const ::std::os::raw::c_char,
                                                             address: *const ::std::os::raw::c_void,
                                                             length: rvmti_sys::jint)
{
    let result = panic::catch_unwind(|| {
        match from_modified_utf8(name) {
            Ok(s) => jvmti_event_dynamic_code_generated(&mut JvmtiEnv::cons(jvmti_env), &s, address as usize, length as usize),
            Err(e) => warn!("Incorrect function name for 'dynamic code generated' event: {:?}", e),
        }
    });
    match result {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to handle 'dynamic code generated' event: {:?}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn jvmti_event_exception_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                _jni_env: *mut rvmti_sys::JNIEnv,
                                                _thread: rvmti_sys::jthread,
                                                _method: rvmti_sys::jmethodID,
                                                _location: rvmti_sys::jlocation,
                                                _exception: rvmti_sys::jobject,
                                                _catch_method: rvmti_sys::jmethodID,
                                                _catch_location: rvmti_sys::jlocation)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_exception_catch_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                      _jni_env: *mut rvmti_sys::JNIEnv,
                                                      _thread: rvmti_sys::jthread,
                                                      _method: rvmti_sys::jmethodID,
                                                      _location: rvmti_sys::jlocation,
                                                      _exception: rvmti_sys::jobject)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_field_access_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                   _jni_env: *mut rvmti_sys::JNIEnv,
                                                   _thread: rvmti_sys::jthread,
                                                   _method: rvmti_sys::jmethodID,
                                                   _location: rvmti_sys::jlocation,
                                                   _field_klass: rvmti_sys::jclass,
                                                   _object: rvmti_sys::jobject,
                                                   _field: rvmti_sys::jfieldID)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_field_modification_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                         _jni_env: *mut rvmti_sys::JNIEnv,
                                                         _thread: rvmti_sys::jthread,
                                                         _method: rvmti_sys::jmethodID,
                                                         _location: rvmti_sys::jlocation,
                                                         _field_klass: rvmti_sys::jclass,
                                                         _object: rvmti_sys::jobject,
                                                         _field: rvmti_sys::jfieldID,
                                                         _signature_type: ::std::os::raw::c_char,
                                                         _new_value: rvmti_sys::jvalue)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_frame_pop_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                _jni_env: *mut rvmti_sys::JNIEnv,
                                                _thread: rvmti_sys::jthread,
                                                _method: rvmti_sys::jmethodID,
                                                _was_popped_by_exception: rvmti_sys::jboolean)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_garbage_collection_finish_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv) {
}

#[no_mangle]
pub extern "C" fn jvmti_event_garbage_collection_start_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv) {
}

#[no_mangle]
pub extern "C" fn jvmti_event_method_entry_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                   _jni_env: *mut rvmti_sys::JNIEnv,
                                                   _thread: rvmti_sys::jthread,
                                                   _method: rvmti_sys::jmethodID)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_method_exit_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                  _jni_env: *mut rvmti_sys::JNIEnv,
                                                  _thread: rvmti_sys::jthread,
                                                  _method: rvmti_sys::jmethodID,
                                                  _was_popped_by_exception: rvmti_sys::jboolean,
                                                  _return_value: rvmti_sys::jvalue)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_monitor_contended_enter_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                              _jni_env: *mut rvmti_sys::JNIEnv,
                                                              _thread: rvmti_sys::jthread,
                                                              _object: rvmti_sys::jobject)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_monitor_contended_entered_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                                _jni_env: *mut rvmti_sys::JNIEnv,
                                                                _thread: rvmti_sys::jthread,
                                                                _object: rvmti_sys::jobject)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_monitor_wait_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                   _jni_env: *mut rvmti_sys::JNIEnv,
                                                   _thread: rvmti_sys::jthread,
                                                   _object: rvmti_sys::jobject,
                                                   _timeout: rvmti_sys::jlong)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_monitor_waited_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                     _jni_env: *mut rvmti_sys::JNIEnv,
                                                     _thread: rvmti_sys::jthread,
                                                     _object: rvmti_sys::jobject,
                                                     _timed_out: rvmti_sys::jboolean)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_native_method_bind_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                         _jni_env: *mut rvmti_sys::JNIEnv,
                                                         _thread: rvmti_sys::jthread,
                                                         _method: rvmti_sys::jmethodID,
                                                         _address: *mut ::std::os::raw::c_void,
                                                         _new_address_ptr: *mut *mut ::std::os::raw::c_void)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_object_free_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv, _tag: rvmti_sys::jlong) {
}

#[no_mangle]
pub extern "C" fn jvmti_event_resource_exhausted_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                         _jni_env: *mut rvmti_sys::JNIEnv,
                                                         _flags: rvmti_sys::jint,
                                                         _reserved: *const ::std::os::raw::c_void,
                                                         _description: *const ::std::os::raw::c_char)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_single_step_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                  _jni_env: *mut rvmti_sys::JNIEnv,
                                                  _thread: rvmti_sys::jthread,
                                                  _method: rvmti_sys::jmethodID,
                                                  _location: rvmti_sys::jlocation)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_thread_end_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                 _jni_env: *mut rvmti_sys::JNIEnv,
                                                 _thread: rvmti_sys::jthread)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_thread_start_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                   _jni_env: *mut rvmti_sys::JNIEnv,
                                                   _thread: rvmti_sys::jthread)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_vm_death_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                               _jni_env: *mut rvmti_sys::JNIEnv)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_vm_init_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                              _jni_env: *mut rvmti_sys::JNIEnv,
                                              _thread: rvmti_sys::jthread)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_vm_object_alloc_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                      _jni_env: *mut rvmti_sys::JNIEnv,
                                                      _thread: rvmti_sys::jthread,
                                                      _object: rvmti_sys::jobject,
                                                      _object_klass: rvmti_sys::jclass,
                                                      _size: rvmti_sys::jlong)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_vm_start_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                               _jni_env: *mut rvmti_sys::JNIEnv)
{
}

#[no_mangle]
pub extern "C" fn jvmti_event_sampled_object_alloc_handler(_jvmti_env: *mut rvmti_sys::jvmtiEnv,
                                                           _jni_env: *mut rvmti_sys::JNIEnv,
                                                           _thread: rvmti_sys::jthread,
                                                           _object: rvmti_sys::jobject,
                                                           _object_klass: rvmti_sys::jclass,
                                                           _size: rvmti_sys::jlong)
{
}

#[derive(Debug)]
pub struct Jvm {
    vm: *mut rvmti_sys::JavaVM,
}

#[derive(Debug)]
pub struct JvmtiEnv {
    env: *mut rvmti_sys::jvmtiEnv,
    owned: bool,
}

// Required for global thread-safe store of initialized environments
unsafe impl Send for JvmtiEnv {}

#[derive(Debug)]
pub struct JvmtiCapabilities {
    caps: rvmti_sys::jvmtiCapabilities,
}

#[derive(Debug)]
pub struct JvmtiEventCallbacksSettings {
    settings: rvmti_sys::jvmtiEventCallbacks,
}

#[derive(Debug)]
pub struct JThread {
    thread: rvmti_sys::jthread,
}

#[derive(Debug)]
pub struct JMethodId {
    method: rvmti_sys::jmethodID,
}

#[derive(Debug)]
pub struct JClass {
    class: rvmti_sys::jclass,
}

pub type JLocation = rvmti_sys::jlocation;

#[derive(Debug)]
pub struct MethodName {
    pub name: String,
    pub signature: String,
    pub generic_signature: Option<String>,
}

#[derive(Debug)]
pub struct ClassSignature {
    pub signature: String,
    pub generic_signature: Option<String>,
}

#[derive(Debug)]
pub struct LineNumberEntry {
    pub start_location: JLocation,
    pub line_number: i32,
}

#[derive(Debug, Clone)]
pub struct AddressLocationEntry {
    pub start_address: usize,
    pub location: JLocation,
}

#[derive(Debug)]
pub struct StackInfo {
    pub pc_address: usize,
    pub stack_frames: Vec<StackFrame>,
}

#[derive(Debug)]
pub struct StackFrame {
    pub method_id: JMethodId,
    pub byte_code_index: i32,
}

#[derive(Debug)]
pub enum CompiledMethodLoadRecord {
    Inline{stack_infos: Vec<StackInfo>},
    Dummy,
}

#[derive(Debug)]
pub enum JvmtiEventMode {
    Enable,
    Disable,
}

#[derive(Debug)]
pub enum JvmtiEvent {
    VmInit,
    VmDeath,
    ThreadStart,
    ThreadEnd,
    ClassFileLoadHook,
    ClassLoad,
    ClassPrepare,
    VmStart,
    Exception,
    ExceptionCatch,
    SingleStep,
    FramePop,
    Breakpoint,
    FieldAccess,
    FieldModification,
    MethodEntry,
    MethodExit,
    NativeMethodBind,
    CompiledMethodLoad,
    CompiledMethodUnload,
    DynamicCodeGenerated,
    DataDumpRequest,
    MonitorWait,
    MonitorWaited,
    MonitorContendedEnter,
    MonitorContendedEntered,
    ResourceExhausted,
    GarbageCollectionStart,
    GarbageCollectionFinish,
    ObjectFree,
    VmObjectAlloc,
    SampledObjectAlloc,
}

#[derive(Fail, Debug)]
pub enum JniError {
    #[fail(display = "Unknown JNI error")]
    UnknownError,
    #[fail(display = "A thread is detached from the VM")]
    ThreadDetachedFromVm,
    #[fail(display = "JNI version error")]
    JniVersionError,
    #[fail(display = "Not enough memory")]
    NotEnoughMemory,
    #[fail(display = "VM is already created")]
    VmAlreadyCreated,
    #[fail(display = "Invalid arguments")]
    InvalidArguments,
    #[fail(display = "Unsupported JNI error code: {}", _0)]
    UnsupportedError(i32),
}

#[derive(Debug)]
pub enum JvmtiVersion {
    Version1dot0,
    Version1dot1,
    Version1dot2,
    Version9,
    Version11,
    CurrentVersion,
}

#[derive(Fail, Debug)]
pub enum JvmtiError {
    #[fail(display = "Invalid thread")]
    InvalidThread,
    #[fail(display = "Invalid thread group")]
    InvalidThreadGroup,
    #[fail(display = "Invalid priority")]
    InvalidPriority,
    #[fail(display = "Thread is not suspended")]
    ThreadNotSuspended,
    #[fail(display = "Thread is already suspended")]
    ThreadSuspended,
    #[fail(display = "Thread is not alive")]
    ThreadNotAlive,
    #[fail(display = "Invalid object")]
    InvalidObject,
    #[fail(display = "Invalid class")]
    InvalidClass,
    #[fail(display = "The class is not prepared yet")]
    ClassNotPrepared,
    #[fail(display = "Invalid method id")]
    InvalidMethodId,
    #[fail(display = "Invalid location")]
    InvalidLocation,
    #[fail(display = "Invalid field id")]
    InvalidFieldId,
    #[fail(display = "Invalid module")]
    InvalidModule,
    #[fail(display = "There are no more stack frames")]
    NoMoreFrames,
    #[fail(display = "No information is available about the stack frame")]
    OpaqueFrame,
    #[fail(display = "Variable type mismatch")]
    TypeMismatch,
    #[fail(display = "Invalid slot")]
    InvalidSlot,
    #[fail(display = "The item is already set")]
    Duplicate,
    #[fail(display = "Element is not found")]
    NotFound,
    #[fail(display = "Invalid raw monitor")]
    InvalidMonitor,
    #[fail(display = "The raw monitor is not owned by this thread")]
    NotMonitorOwner,
    #[fail(display = "The call has been interrupted")]
    Interrupt,
    #[fail(display = "Malformed class file")]
    InvalidClassFormat,
    #[fail(display = "Circular class definition")]
    CircularClassDefinition,
    #[fail(display = "The class fails verification")]
    FailsVerification,
    #[fail(display = "Class redefinition not possible, method addition is unsupported")]
    UnsupportedRedefinitionMethodAdded,
    #[fail(display = "Class redefinition not possible, field change is unsupported")]
    UnsupportedRedefinitionSchemaChanged,
    #[fail(display = "The thread state is inconsistent due to it having been modified")]
    InvalidTypeState,
    #[fail(display = "Class redefinition not possible, class hierarchy change is unsupported")]
    UnsupportedRedefinitionHierarchyChanged,
    #[fail(display = "Class redefinition not possible, method deletion is unsupported")]
    UnsupportedRedefinitionMethodDeleted,
    #[fail(display = "Class file version is unsupported")]
    UnsupportedVersion,
    #[fail(display = "Class names do not match")]
    NamesDontMatch,
    #[fail(display = "Class redefinition not possible, class modifiers change is unsupported")]
    UnsupportedRedefinitionClassModifiersChanged,
    #[fail(display = "Class redefinition not possible, method modifiers change is unsupported")]
    UnsupportedRedefinitionMethodModifiersChanged,
    #[fail(display = "The class is unmodifiable")]
    UnmodifiableClass,
    #[fail(display = "The module is unmodifiable")]
    UnmodifiableModule,
    #[fail(display = "The functionality is not available")]
    NotAvaliable,
    #[fail(display = "This environment does not possess the required capability")]
    MustPosessCapability,
    #[fail(display = "Unexpected null pointer")]
    NullPointer,
    #[fail(display = "Information is not available")]
    AbsentInformation,
    #[fail(display = "Invalid event type")]
    InvalidEventType,
    #[fail(display = "Illegal argument")]
    IllegalArgument,
    #[fail(display = "Information is not available for native method")]
    NativeMethod,
    #[fail(display = "This class loader does not support the requested operation")]
    ClassLoaderUnsupported,
    #[fail(display = "Out of memory")]
    OutOfMemory,
    #[fail(display = "Access denied")]
    AccessDenied,
    #[fail(display = "The functionality is not available in the current phase")]
    WrongPhase,
    #[fail(display = "Unexpected internal error")]
    Internal,
    #[fail(display = "The thread is not attached to the virtual machine")]
    UnattachedThread,
    #[fail(display = "Invalid environment")]
    InvalidEnvironment,
    #[fail(display = "Unsupported JVMTI error code: {}", _0)]
    UnsupportedError(u32),
}

#[derive(Fail, Debug)]
pub enum GetMethodNameError {
    #[fail(display = "JVMTI method call error: {}", _0)]
    VmError(#[cause] JvmtiError),
    #[fail(display = "Failed to decode method name: {}", _0)]
    NameDecodeError(#[cause] StringDecodeError),
    #[fail(display = "Failed to decode method signature: {}", _0)]
    SignatureDecodeError(#[cause] StringDecodeError),
    #[fail(display = "Failed to decode method generic signature: {}", _0)]
    GenericSignatureDecodeError(#[cause] StringDecodeError),
}

#[derive(Fail, Debug)]
pub enum GetClassSignatureError {
    #[fail(display = "JVMTI method call error: {}", _0)]
    VmError(#[cause] JvmtiError),
    #[fail(display = "Failed to decode class signature: {}", _0)]
    SignatureDecodeError(#[cause] StringDecodeError),
    #[fail(display = "Failed to decode class generic signature: {}", _0)]
    GenericSignatureDecodeError(#[cause] StringDecodeError),
}

#[derive(Fail, Debug)]
pub enum GetSourceFileNameError {
    #[fail(display = "JVMTI method call error: {}", _0)]
    VmError(#[cause] JvmtiError),
    #[fail(display = "Failed to decode source file name: {}", _0)]
    SourceFileNameDecodeError(#[cause] StringDecodeError),
}

#[derive(Fail, Debug)]
pub enum StringDecodeError {
    #[fail(display = "Invalid modified UTF-8 encoding")]
    ModifiedUtf8Error,
    #[fail(display = "Invalid UTF-8 byte string: {}", _0)]
    FromUtf8Error(#[cause] FromUtf8Error),
    #[fail(display = "Invalid UTF-8 byte string: {}", _0)]
    Utf8Error(#[cause] str::Utf8Error),
}

impl Jvm {

    pub fn get_jvmti_env(&self, version: JvmtiVersion) -> Result<JvmtiEnv, JniError> {
        unsafe {
            let mut env: *mut c_void = ptr::null_mut();
            let result = (*(*self.vm)).GetEnv.unwrap()(self.vm, &mut env, rvmti_sys::jint::from(version));
            if result == rvmti_sys::JNI_OK {
                if !env.is_null() {
                    Ok(JvmtiEnv{env: env as *mut rvmti_sys::jvmtiEnv, owned: true})
                } else {
                    Err(JniError::UnknownError)
                }
            } else {
                Err(JniError::from(result))
            }
        }
    }

}

impl JvmtiEnv {

    fn cons(env: *mut rvmti_sys::jvmtiEnv) -> JvmtiEnv {
        JvmtiEnv{env: env, owned: false}
    }

    pub fn add_capabilities(&mut self, capabilities: &JvmtiCapabilities) -> Result<(), JvmtiError> {
        unsafe {
            let result = (*(*self.env)).AddCapabilities.unwrap()(self.env, &capabilities.caps);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(());
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn relinquish_capabilities(&mut self, capabilities: &JvmtiCapabilities) -> Result<(), JvmtiError> {
        unsafe {
            let result = (*(*self.env)).RelinquishCapabilities.unwrap()(self.env, &capabilities.caps);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(());
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn set_event_callbacks_settings(&mut self, settings: &JvmtiEventCallbacksSettings) -> Result<(), JvmtiError> {
        unsafe {
            let result = (*(*self.env)).SetEventCallbacks.unwrap()(self.env, &settings.settings,
                                                                   size_of::<rvmti_sys::jvmtiEventCallbacks>() as rvmti_sys::jint);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(());
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn set_event_notification_mode(&mut self, mode: JvmtiEventMode, event_type: JvmtiEvent,
                                       event_thread: Option<JThread>) -> Result<(), JvmtiError>
    {
        unsafe {
            let result = (*(*self.env)).SetEventNotificationMode.unwrap()(self.env,
                                                                          rvmti_sys::jvmtiEventMode::from(mode),
                                                                          rvmti_sys::jvmtiEvent::from(event_type),
                                                                          event_thread.map_or_else(|| ptr::null_mut(), |t| t.thread ));
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(());
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn set_heap_sampling_interval(&mut self, sampling_interval: i32) -> Result<(), JvmtiError> {
        unsafe {
            let result = (*(*self.env)).SetHeapSamplingInterval.unwrap()(self.env,
                                                                         sampling_interval as rvmti_sys::jint);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(());
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn get_method_name(&mut self, method: &JMethodId) -> Result<MethodName, GetMethodNameError> {
        unsafe {
            let mut name_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let mut signature_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let mut generic_signature_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let result = (*(*self.env)).GetMethodName.unwrap()(self.env, method.method, &mut name_ptr,
                                                               &mut signature_ptr, &mut generic_signature_ptr);
            let name = name_ptr.as_ref().map(|v| VmOwnedString {ptr:v, env: &self});
            let signature = signature_ptr.as_ref().map(|v| VmOwnedString {ptr: v, env: &self});
            let generic_signature = generic_signature_ptr.as_ref().map(|v| VmOwnedString {ptr: v, env: &self});
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                let name_string = name.as_ref().map_or_else(|| Ok("".to_string()), |s| s.to_string()
                    .map(|v| v.unwrap_or_else(|| "".to_string()))).map_err(|e| GetMethodNameError::NameDecodeError(e))?;
                let signature_string = signature.as_ref().map_or_else(|| Ok("".to_string()), |s| s.to_string()
                    .map(|v| v.unwrap_or_else(|| "".to_string()))).map_err(|e| GetMethodNameError::SignatureDecodeError(e))?;
                let generic_signature_string = generic_signature.as_ref().map_or_else(|| Ok(None), |s| s.to_string()
                    .map_err(|e| GetMethodNameError::GenericSignatureDecodeError(e)))?;
                return Ok(MethodName{name: name_string, signature: signature_string, generic_signature: generic_signature_string});
            } else {
                return Err(GetMethodNameError::VmError(JvmtiError::from(result)));
            }
        }
    }

    pub fn get_method_declaring_class(&mut self, method: &JMethodId) -> Result<JClass, JvmtiError> {
        unsafe {
            let mut class: rvmti_sys::jclass = ptr::null_mut();
            let result = (*(*self.env)).GetMethodDeclaringClass.unwrap()(self.env, method.method, &mut class);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(JClass{class});
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn get_class_signature(&mut self, class: &JClass) -> Result<ClassSignature, GetClassSignatureError> {
        unsafe {
            let mut signature_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let mut generic_signature_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let result = (*(*self.env)).GetClassSignature.unwrap()(self.env, class.class,
                                                                   &mut signature_ptr, &mut generic_signature_ptr);
            let signature = signature_ptr.as_ref().map(|v| VmOwnedString {ptr: v, env: &self});
            let generic_signature = generic_signature_ptr.as_ref().map(|v| VmOwnedString {ptr: v, env: &self});
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                let signature_string = signature.as_ref().map_or_else(|| Ok("".to_string()), |s| s.to_string()
                    .map(|v| v.unwrap_or_else(|| "".to_string()))).map_err(|e| GetClassSignatureError::SignatureDecodeError(e))?;
                let generic_signature_string = generic_signature.as_ref().map_or_else(|| Ok(None), |s| s.to_string()
                    .map_err(|e| GetClassSignatureError::GenericSignatureDecodeError(e)))?;
                return Ok(ClassSignature{signature: signature_string, generic_signature: generic_signature_string});
            } else {
                return Err(GetClassSignatureError::VmError(JvmtiError::from(result)));
            }
        }
    }

    pub fn get_source_file_name(&mut self, class: &JClass) -> Result<Option<String>, GetSourceFileNameError> {
        unsafe {
            let mut source_name_ptr: *mut ::std::os::raw::c_char = ptr::null_mut();
            let result = (*(*self.env)).GetSourceFileName.unwrap()(self.env, class.class, &mut source_name_ptr);
            let source_name = source_name_ptr.as_ref().map(|v| VmOwnedString {ptr: v, env: &self});
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                let source_name_string = source_name.as_ref().map_or_else(|| Ok(None), |s| s.to_string()
                    .map_err(|e| GetSourceFileNameError::SourceFileNameDecodeError(e)))?;
                return Ok(source_name_string);
            } else if result == rvmti_sys::jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION {
                return Ok(None);
            } else {
                return Err(GetSourceFileNameError::VmError(JvmtiError::from(result)));
            }
        }
    }

    pub fn get_line_number_table(&mut self, method: &JMethodId) -> Result<Option<Vec<LineNumberEntry>>, JvmtiError> {
        unsafe {
            let mut entry_count: rvmti_sys::jint = 0 as rvmti_sys::jint;
            let mut table_ptr: *mut rvmti_sys::jvmtiLineNumberEntry = ptr::null_mut();
            let result = (*(*self.env)).GetLineNumberTable.unwrap()(self.env, method.method, &mut entry_count, &mut table_ptr);
            let table = table_ptr.as_ref().map(|v| VmOwnedLineNumberTable{ptr:v, entry_count: entry_count, env: &self});
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(table.as_ref().and_then(|t| t.as_line_number_slice()).map(|t| t.iter()
                    .map(|e| LineNumberEntry{start_location: e.start_location, line_number: e.line_number}).collect()));
            } else if result == rvmti_sys::jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION {
                return Ok(None);
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

    pub fn check_is_method_native(&mut self, method: &JMethodId) -> Result<bool, JvmtiError> {
        unsafe {
            let mut is_native: rvmti_sys::jboolean = 0 as rvmti_sys::jboolean;
            let result = (*(*self.env)).IsMethodNative.unwrap()(self.env, method.method, &mut is_native);
            if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                return Ok(is_native != 0);
            } else {
                return Err(JvmtiError::from(result));
            }
        }
    }

}

impl Drop for JvmtiEnv {

    fn drop(&mut self) {
        if !self.owned {
            return;
        }
        unsafe {
            let result = (*(*self.env)).DisposeEnvironment.unwrap()(self.env);
            if result != rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                warn!("Failed to dispose of JVMTI environment: {}", JvmtiError::from(result))
            } else {
                debug!("Disposed of JVMTI environment")
            }
        }
    }

}

impl JvmtiCapabilities {

    pub fn new_empty_capabilities() -> JvmtiCapabilities {
        JvmtiCapabilities{caps: rvmti_sys::jvmtiCapabilities{
            _bitfield_1: rvmti_sys::jvmtiCapabilities::new_bitfield_1(0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                                                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                                                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                                                      0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                                                      0, 0, 0, 0),
        }}
    }

    pub fn can_tag_objects(&mut self) {
        self.caps.set_can_tag_objects(1)
    }

    pub fn can_generate_field_modification_events(&mut self) {
        self.caps.set_can_generate_field_modification_events(1)
    }

    pub fn can_generate_field_access_events(&mut self) {
        self.caps.set_can_generate_field_access_events(1)
    }

    pub fn can_get_bytecodes(&mut self) {
        self.caps.set_can_get_bytecodes(1)
    }

    pub fn can_get_synthetic_attribute(&mut self) {
        self.caps.set_can_get_synthetic_attribute(1)
    }

    pub fn can_get_owned_monitor_info(&mut self) {
        self.caps.set_can_get_owned_monitor_info(1)
    }

    pub fn can_get_current_contended_monitor(&mut self) {
        self.caps.set_can_get_current_contended_monitor(1)
    }

    pub fn can_get_monitor_info(&mut self) {
        self.caps.set_can_get_monitor_info(1)
    }

    pub fn can_pop_frame(&mut self) {
        self.caps.set_can_pop_frame(1)
    }

    pub fn can_redefine_classes(&mut self) {
        self.caps.set_can_redefine_classes(1)
    }

    pub fn can_signal_thread(&mut self) {
        self.caps.set_can_signal_thread(1)
    }

    pub fn can_get_source_file_name(&mut self) {
        self.caps.set_can_get_source_file_name(1)
    }

    pub fn can_get_line_numbers(&mut self) {
        self.caps.set_can_get_line_numbers(1)
    }

    pub fn can_get_source_debug_extension(&mut self) {
        self.caps.set_can_get_source_debug_extension(1)
    }

    pub fn can_access_local_variables(&mut self) {
        self.caps.set_can_access_local_variables(1)
    }

    pub fn can_maintain_original_method_order(&mut self) {
        self.caps.set_can_maintain_original_method_order(1)
    }

    pub fn can_generate_single_step_events(&mut self) {
        self.caps.set_can_generate_single_step_events(1)
    }

    pub fn can_generate_exception_events(&mut self) {
        self.caps.set_can_generate_exception_events(1)
    }

    pub fn can_generate_frame_pop_events(&mut self) {
        self.caps.set_can_generate_frame_pop_events(1)
    }

    pub fn can_generate_breakpoint_events(&mut self) {
        self.caps.set_can_generate_breakpoint_events(1)
    }

    pub fn can_suspend(&mut self) {
        self.caps.set_can_suspend(1)
    }

    pub fn can_redefine_any_class(&mut self) {
        self.caps.set_can_redefine_any_class(1)
    }

    pub fn can_get_current_thread_cpu_time(&mut self) {
        self.caps.set_can_get_current_thread_cpu_time(1)
    }

    pub fn can_get_thread_cpu_time(&mut self) {
        self.caps.set_can_get_thread_cpu_time(1)
    }

    pub fn can_generate_method_entry_events(&mut self) {
        self.caps.set_can_generate_method_entry_events(1)
    }

    pub fn can_generate_method_exit_events(&mut self) {
        self.caps.set_can_generate_method_exit_events(1)
    }

    pub fn can_generate_all_class_hook_events(&mut self) {
        self.caps.set_can_generate_all_class_hook_events(1)
    }

    pub fn can_generate_compiled_method_load_events(&mut self) {
        self.caps.set_can_generate_compiled_method_load_events(1)
    }

    pub fn can_generate_monitor_events(&mut self) {
        self.caps.set_can_generate_monitor_events(1)
    }

    pub fn can_generate_vm_object_alloc_events(&mut self) {
        self.caps.set_can_generate_vm_object_alloc_events(1)
    }

    pub fn can_generate_native_method_bind_events(&mut self) {
        self.caps.set_can_generate_native_method_bind_events(1)
    }

    pub fn can_generate_garbage_collection_events(&mut self) {
        self.caps.set_can_generate_garbage_collection_events(1)
    }

    pub fn can_generate_object_free_events(&mut self) {
        self.caps.set_can_generate_object_free_events(1)
    }

    pub fn can_force_early_return(&mut self) {
        self.caps.set_can_force_early_return(1)
    }

    pub fn can_get_owned_monitor_stack_depth_info(&mut self) {
        self.caps.set_can_get_owned_monitor_stack_depth_info(1)
    }

    pub fn can_get_constant_pool(&mut self) {
        self.caps.set_can_get_constant_pool(1)
    }

    pub fn can_set_native_method_prefix(&mut self) {
        self.caps.set_can_set_native_method_prefix(1)
    }

    pub fn can_retransform_classes(&mut self) {
        self.caps.set_can_retransform_classes(1)
    }

    pub fn can_retransform_any_class(&mut self) {
        self.caps.set_can_retransform_any_class(1)
    }

    pub fn can_generate_resource_exhaustion_heap_events(&mut self) {
        self.caps.set_can_generate_resource_exhaustion_heap_events(1)
    }

    pub fn can_generate_resource_exhaustion_threads_events(&mut self) {
        self.caps.set_can_generate_resource_exhaustion_threads_events(1)
    }

    pub fn can_generate_early_vmstart(&mut self) {
        self.caps.set_can_generate_early_vmstart(1)
    }

    pub fn can_generate_early_class_hook_events(&mut self) {
        self.caps.set_can_generate_early_class_hook_events(1)
    }

    pub fn can_generate_sampled_object_alloc_events(&mut self) {
        self.caps.set_can_generate_sampled_object_alloc_events(1)
    }

}

impl JvmtiEventCallbacksSettings {

    pub fn new_empty_settings() -> JvmtiEventCallbacksSettings {
        JvmtiEventCallbacksSettings{ settings: rvmti_sys::jvmtiEventCallbacks { VMInit: None,
            VMDeath: None, ThreadStart: None, ThreadEnd: None, ClassFileLoadHook: None,
            ClassLoad: None, ClassPrepare: None, VMStart: None, Exception: None,
            ExceptionCatch: None, SingleStep: None, FramePop: None, Breakpoint: None,
            FieldAccess: None, FieldModification: None, MethodEntry: None, MethodExit: None,
            NativeMethodBind: None, CompiledMethodLoad: None, CompiledMethodUnload: None,
            DynamicCodeGenerated: None, DataDumpRequest: None, reserved72: None, MonitorWait: None,
            MonitorWaited: None, MonitorContendedEnter: None, MonitorContendedEntered: None,
            reserved77: None, reserved78: None, reserved79: None, ResourceExhausted: None,
            GarbageCollectionStart: None, GarbageCollectionFinish: None, ObjectFree: None,
            VMObjectAlloc: None, reserved85: None, SampledObjectAlloc: None }}
    }

    pub fn vm_init_enabled(&mut self) {
        self.settings.VMInit = Some(jvmti_event_vm_init_handler)
    }

    pub fn vm_death_enabled(&mut self) {
        self.settings.VMDeath = Some(jvmti_event_vm_death_handler)
    }

    pub fn thread_start_enabled(&mut self) {
        self.settings.ThreadStart = Some(jvmti_event_thread_start_handler)
    }

    pub fn thread_end_enabled(&mut self) {
        self.settings.ThreadEnd = Some(jvmti_event_thread_end_handler)
    }

    pub fn class_file_load_hook_enabled(&mut self) {
        self.settings.ClassFileLoadHook = Some(jvmti_event_class_file_load_hook_handler)
    }

    pub fn class_load_enabled(&mut self) {
        self.settings.ClassLoad = Some(jvmti_event_class_load_handler)
    }

    pub fn class_prepare_enabled(&mut self) {
        self.settings.ClassPrepare = Some(jvmti_event_class_prepare_handler)
    }

    pub fn vm_start_enabled(&mut self) {
        self.settings.VMStart = Some(jvmti_event_vm_start_handler)
    }

    pub fn exception_enabled(&mut self) {
        self.settings.Exception = Some(jvmti_event_exception_handler)
    }

    pub fn exception_catch_enabled(&mut self) {
        self.settings.ExceptionCatch = Some(jvmti_event_exception_catch_handler)
    }

    pub fn single_step_enabled(&mut self) {
        self.settings.SingleStep = Some(jvmti_event_single_step_handler)
    }

    pub fn frame_pop_enabled(&mut self) {
        self.settings.FramePop = Some(jvmti_event_frame_pop_handler)
    }

    pub fn breakpoint_enabled(&mut self) {
        self.settings.Breakpoint = Some(jvmti_event_breakpoint_handler)
    }

    pub fn field_access_enabled(&mut self) {
        self.settings.FieldAccess = Some(jvmti_event_field_access_handler)
    }

    pub fn field_modification_enabled(&mut self) {
        self.settings.FieldModification = Some(jvmti_event_field_modification_handler)
    }

    pub fn method_entry_enabled(&mut self) {
        self.settings.MethodEntry = Some(jvmti_event_method_entry_handler)
    }

    pub fn method_exit_enabled(&mut self) {
        self.settings.MethodExit = Some(jvmti_event_method_exit_handler)
    }

    pub fn native_method_bind_enabled(&mut self) {
        self.settings.NativeMethodBind = Some(jvmti_event_native_method_bind_handler)
    }

    pub fn compiled_method_load_enabled(&mut self) {
        self.settings.CompiledMethodLoad = Some(jvmti_event_compiled_method_load_handler)
    }

    pub fn compiled_method_unload_enabled(&mut self) {
        self.settings.CompiledMethodUnload = Some(jvmti_event_compiled_method_unload_handler)
    }

    pub fn dynamic_code_generated_enabled(&mut self) {
        self.settings.DynamicCodeGenerated = Some(jvmti_event_dynamic_code_generated_handler)
    }

    pub fn data_dump_request_enabled(&mut self) {
        self.settings.DataDumpRequest = Some(jvmti_event_data_dump_request_handler)
    }

    pub fn monitor_wait_enabled(&mut self) {
        self.settings.MonitorWait = Some(jvmti_event_monitor_wait_handler)
    }

    pub fn monitor_waited_enabled(&mut self) {
        self.settings.MonitorWaited = Some(jvmti_event_monitor_waited_handler)
    }

    pub fn monitor_contended_enter_enabled(&mut self) {
        self.settings.MonitorContendedEnter = Some(jvmti_event_monitor_contended_enter_handler)
    }

    pub fn monitor_contended_entered_enabled(&mut self) {
        self.settings.MonitorContendedEntered = Some(jvmti_event_monitor_contended_entered_handler)
    }

    pub fn resource_exhausted_enabled(&mut self) {
        self.settings.ResourceExhausted = Some(jvmti_event_resource_exhausted_handler)
    }

    pub fn garbage_collection_start_enabled(&mut self) {
        self.settings.GarbageCollectionStart = Some(jvmti_event_garbage_collection_start_handler)
    }

    pub fn garbage_collection_finish_enabled(&mut self) {
        self.settings.GarbageCollectionFinish = Some(jvmti_event_garbage_collection_finish_handler)
    }

    pub fn object_free_enabled(&mut self) {
        self.settings.ObjectFree = Some(jvmti_event_object_free_handler)
    }

    pub fn vm_object_alloc_enabled(&mut self) {
        self.settings.VMObjectAlloc = Some(jvmti_event_vm_object_alloc_handler)
    }

    pub fn sampled_object_alloc_enabled(&mut self) {
        self.settings.SampledObjectAlloc = Some(jvmti_event_sampled_object_alloc_handler)
    }
}

impl From<JvmtiVersion> for rvmti_sys::jint {

    fn from(ver: JvmtiVersion) -> rvmti_sys::jint {
        match ver {
            JvmtiVersion::Version1dot0 => rvmti_sys::JVMTI_VERSION_1_0 as rvmti_sys::jint,
            JvmtiVersion::Version1dot1 => rvmti_sys::JVMTI_VERSION_1_1 as rvmti_sys::jint,
            JvmtiVersion::Version1dot2 => rvmti_sys::JVMTI_VERSION_1_2 as rvmti_sys::jint,
            JvmtiVersion::Version9 => rvmti_sys::JVMTI_VERSION_9 as rvmti_sys::jint,
            JvmtiVersion::Version11 => rvmti_sys::JVMTI_VERSION_11 as rvmti_sys::jint,
            JvmtiVersion::CurrentVersion => rvmti_sys::JVMTI_VERSION as rvmti_sys::jint,
        }
    }

}

// TODO Support platform encodings other than utf-8
#[cfg(not(target_os = "windows"))]
fn from_platform(input: *const c_char) -> Result<Option<String>, StringDecodeError> {
    unsafe {
        if input.is_null() {
            return Ok(None);
        }
        return CStr::from_ptr(input).to_str().map(str::to_string).map(Option::Some).map_err(StringDecodeError::from);
    }
}

fn from_modified_utf8(input: *const c_char) -> Result<Option<String>, StringDecodeError> {
    unsafe {
        if input.is_null() {
            return Ok(None);
        }
        let bytes = CStr::from_ptr(input).to_bytes();
        let mut converted: Vec<u8> = Vec::new();
        let mut state: ModifiedUtf8DecoderState = ModifiedUtf8DecoderState::OneByte;
        let mut accumulator: u32 = 0u32;
        for b in bytes.into_iter() {
            match state {
                ModifiedUtf8DecoderState::OneByte => {
                    if (*b & 0x80u8) == 0x00u8 {
                        converted.push(*b & 0x7Fu8);
                    } else if (*b & 0xe0u8) == 0xC0u8 {
                        state = ModifiedUtf8DecoderState::TwoBytes;
                        accumulator += ((*b & 0x1fu8) as u32) << 6;
                    } else if (*b & 0xf0u8) == 0x70u8 {
                        state = ModifiedUtf8DecoderState::ThreeBytesOne;
                        accumulator += ((*b & 0x0fu8) as u32) << 12;
                    } else if *b == 0xedu8 {
                        state = ModifiedUtf8DecoderState::SixBytesOne;
                        accumulator += 0x10000u32;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::TwoBytes => {
                    if (*b & 0xc0u8) == 0x80u8 {
                        state = ModifiedUtf8DecoderState::OneByte;
                        accumulator += (*b & 0x3fu8) as u32;
                        if accumulator == 0u32 {
                            converted.push(0u8);
                        } else if accumulator >= 0x80u32 && accumulator <= 0x7ffu32 {
                            converted.push(0xc0u8 | ((accumulator >> 6) & 0xffu32) as u8);
                            converted.push(0x80u8 | (accumulator & 0x3fu32) as u8);
                        } else {
                            return Err(StringDecodeError::ModifiedUtf8Error);
                        }
                        accumulator = 0u32;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::ThreeBytesOne => {
                    if (*b & 0xc0u8) == 0x80u8 {
                        state = ModifiedUtf8DecoderState::ThreeBytesTwo;
                        accumulator += ((*b & 0x3fu8) as u32) << 6;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::ThreeBytesTwo => {
                    if (*b & 0xc0u8) == 0x80u8 {
                        state = ModifiedUtf8DecoderState::OneByte;
                        accumulator += (*b & 0x3fu8) as u32;
                        if accumulator >= 0x800u32 && accumulator <= 0xffffu32 {
                            converted.push(0xe0u8 | ((accumulator >> 12) & 0xffu32) as u8);
                            converted.push(0x80u8 | ((accumulator >> 6) & 0x3fu32) as u8);
                            converted.push(0x80u8 | (accumulator & 0x3fu32) as u8);
                        } else {
                            return Err(StringDecodeError::ModifiedUtf8Error);
                        }
                        accumulator = 0u32;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::SixBytesOne => {
                    if *b & 0xf0u8 == 0xa0u8 {
                        state = ModifiedUtf8DecoderState::SixBytesTwo;
                        accumulator += ((*b & 0x0f) as u32) << 16;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::SixBytesTwo => {
                    if *b & 0xc0u8 == 0x80u8 {
                        state = ModifiedUtf8DecoderState::SixBytesThree;
                        accumulator += ((*b & 0x3fu8) as u32) << 10;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::SixBytesThree => {
                    if *b == 0xedu8 {
                        state = ModifiedUtf8DecoderState::SixBytesFour;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::SixBytesFour => {
                    if *b & 0xf0u8 == 0xb0u8 {
                        state = ModifiedUtf8DecoderState::SixBytesFive;
                        accumulator += ((*b & 0x0fu8) as u32) << 6;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
                ModifiedUtf8DecoderState::SixBytesFive => {
                    if *b & 0xc0u8 == 0x80u8 {
                        state = ModifiedUtf8DecoderState::OneByte;
                        accumulator += (*b & 0x3fu8) as u32;
                        if accumulator > 0xffffu32 {
                            converted.push(0xf0u8 | ((accumulator >> 18) & 0xffu32) as u8);
                            converted.push(0x80u8 | ((accumulator >> 12) & 0x3fu32) as u8);
                            converted.push(0x80u8 | ((accumulator >> 6) & 0x3fu32) as u8);
                            converted.push(0x80u8 | (accumulator & 0x3fu32) as u8);
                        } else {
                            return Err(StringDecodeError::ModifiedUtf8Error);
                        }
                        accumulator = 0u32;
                    } else {
                        return Err(StringDecodeError::ModifiedUtf8Error);
                    }
                },
            }
        }
        String::from_utf8(converted).map_err(StringDecodeError::from).map(Option::Some)
    }
}

fn as_address_location_slice<'a>(map_length: rvmti_sys::jint,
                                 map: *const rvmti_sys::jvmtiAddrLocationMap) -> Option<&'a [rvmti_sys::jvmtiAddrLocationMap]>
{
    unsafe {
        if map_length == 0 || map.is_null() {
            return None;
        }
        Some(slice::from_raw_parts(map, map_length as usize))
    }
}

fn to_compile_infos(compile_info: *const ::std::os::raw::c_void) -> Option<Vec<CompiledMethodLoadRecord>> {
    unsafe {
        if compile_info.is_null() {
            return None;
        }
        let mut result = Vec::new();
        let mut record_ptr = compile_info as *const rvmti_sys::jvmtiCompiledMethodLoadRecordHeader;
        loop {
            if (*record_ptr).majorinfoversion == (rvmti_sys::JVMTI_CMLR_MAJOR_VERSION as i32)
                && (*record_ptr).minorinfoversion == (rvmti_sys::JVMTI_CMLR_MINOR_VERSION as i32)
                {
                    match (*record_ptr).kind {
                        rvmti_sys::jvmtiCMLRKind_JVMTI_CMLR_DUMMY => {
                            result.push(CompiledMethodLoadRecord::Dummy);
                        },
                        rvmti_sys::jvmtiCMLRKind_JVMTI_CMLR_INLINE_INFO => {
                            let inline_record_ptr = record_ptr as *const rvmti_sys::jvmtiCompiledMethodLoadInlineRecord;
                            let stack_infos = as_stack_infos_slice((*inline_record_ptr).numpcs, (*inline_record_ptr).pcinfo)
                                .iter().map(|e|
                                {
                                    let method_ids = as_method_ids_slice(e.numstackframes, e.methods);
                                    let byte_code_indices = as_byte_code_indices_slice(e.numstackframes, e.bcis);
                                    let mut stack_frames = Vec::new();
                                    if e.numstackframes > 0 {
                                        for i in 0..(e.numstackframes as usize) {
                                            stack_frames.push(StackFrame{method_id: JMethodId{method: method_ids[i]},
                                                byte_code_index: byte_code_indices[i]});
                                        }
                                    }
                                    StackInfo{pc_address: e.pc as usize, stack_frames: stack_frames}
                                }).collect();
                            result.push(CompiledMethodLoadRecord::Inline {stack_infos: stack_infos})
                        },
                        _ => {},
                    }
                }
            if (*record_ptr).next.is_null() {
                break;
            } else {
                record_ptr = (*record_ptr).next;
            }
        }
        Some(result)
    }
}

fn as_stack_infos_slice<'a>(numpcs: rvmti_sys::jint, pcinfo: *const rvmti_sys::PCStackInfo) -> &'a [rvmti_sys::PCStackInfo]
{
    unsafe {
        if numpcs == 0 || pcinfo.is_null() {
            return &[];
        }
        slice::from_raw_parts(pcinfo, numpcs as usize)
    }
}

fn as_method_ids_slice<'a>(numstackframes: rvmti_sys::jint, methods: *const rvmti_sys::jmethodID) -> &'a [rvmti_sys::jmethodID] {
    unsafe {
        if numstackframes == 0 || methods.is_null() {
            return &[];
        }
        slice::from_raw_parts(methods, numstackframes as usize)
    }
}

fn as_byte_code_indices_slice<'a>(numstackframes: rvmti_sys::jint, bcis: *const rvmti_sys::jint) -> &'a [rvmti_sys::jint] {
    unsafe {
        if numstackframes == 0 || bcis.is_null() {
            return &[];
        }
        slice::from_raw_parts(bcis, numstackframes as usize)
    }
}

#[derive(Debug)]
enum ModifiedUtf8DecoderState {
    OneByte,
    TwoBytes,
    ThreeBytesOne,
    ThreeBytesTwo,
    SixBytesOne,
    SixBytesTwo,
    SixBytesThree,
    SixBytesFour,
    SixBytesFive,
}

struct VmOwnedString<'a> {
    ptr: *const ::std::os::raw::c_char,
    env: &'a JvmtiEnv,
}

struct VmOwnedLineNumberTable<'a> {
    ptr: *const rvmti_sys::jvmtiLineNumberEntry,
    entry_count: rvmti_sys::jint,
    env: &'a JvmtiEnv,
}

impl<'a> VmOwnedString<'a> {

    fn to_string(&self) -> Result<Option<String>, StringDecodeError> {
        from_modified_utf8(self.ptr)
    }

}

impl<'a> VmOwnedLineNumberTable<'a> {

    fn as_line_number_slice(&'a self) -> Option<&'a [rvmti_sys::jvmtiLineNumberEntry]> {
        unsafe {
            if self.entry_count == 0 || self.ptr.is_null() {
                return None
            }
            Some(slice::from_raw_parts(self.ptr, self.entry_count as usize))
        }
    }
}

impl<'a> Drop for VmOwnedString<'a> {

    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let result = (*(*(self.env.env))).Deallocate.unwrap()(self.env.env, self.ptr as *mut c_uchar);
                if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                    debug!("VM owned string is deallocated");
                } else {
                    warn!("Failed to deallocate VM owned string {}", JvmtiError::from(result));
                }
            }
        }
    }

}

impl<'a> Drop for VmOwnedLineNumberTable<'a> {

    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let result = (*(*(self.env.env))).Deallocate.unwrap()(self.env.env, self.ptr as *mut c_uchar);
                if result == rvmti_sys::jvmtiError_JVMTI_ERROR_NONE {
                    debug!("VM owned line number table is deallocated");
                } else {
                    warn!("Failed to deallocate VM owned line number table {}", JvmtiError::from(result));
                }
            }
        }
    }

}

impl From<FromUtf8Error> for StringDecodeError {

    fn from(error: FromUtf8Error) -> StringDecodeError {
        StringDecodeError::FromUtf8Error(error)
    }

}

impl From<str::Utf8Error> for StringDecodeError {

    fn from(error: str::Utf8Error) -> StringDecodeError {
        StringDecodeError::Utf8Error(error)
    }

}

impl From<rvmti_sys::jvmtiError> for JvmtiError {

    fn from(error: rvmti_sys::jvmtiError) -> JvmtiError {
        match error {
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_THREAD => JvmtiError::InvalidThread,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_THREAD_GROUP => JvmtiError::InvalidThreadGroup,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_PRIORITY => JvmtiError::InvalidPriority,
            rvmti_sys::jvmtiError_JVMTI_ERROR_THREAD_NOT_SUSPENDED => JvmtiError::ThreadNotSuspended,
            rvmti_sys::jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED => JvmtiError::ThreadSuspended,
            rvmti_sys::jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE => JvmtiError::ThreadNotAlive,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_OBJECT => JvmtiError::InvalidObject,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_CLASS => JvmtiError::InvalidClass,
            rvmti_sys::jvmtiError_JVMTI_ERROR_CLASS_NOT_PREPARED => JvmtiError::ClassNotPrepared,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_METHODID => JvmtiError::InvalidMethodId,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_LOCATION => JvmtiError::InvalidLocation,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_FIELDID => JvmtiError::InvalidFieldId,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_MODULE => JvmtiError::InvalidModule,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES => JvmtiError::NoMoreFrames,
            rvmti_sys::jvmtiError_JVMTI_ERROR_OPAQUE_FRAME => JvmtiError::OpaqueFrame,
            rvmti_sys::jvmtiError_JVMTI_ERROR_TYPE_MISMATCH => JvmtiError::TypeMismatch,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_SLOT => JvmtiError::InvalidSlot,
            rvmti_sys::jvmtiError_JVMTI_ERROR_DUPLICATE => JvmtiError::Duplicate,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NOT_FOUND => JvmtiError::NotFound,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_MONITOR => JvmtiError::InvalidMonitor,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NOT_MONITOR_OWNER => JvmtiError::NotMonitorOwner,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INTERRUPT => JvmtiError::Interrupt,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_CLASS_FORMAT => JvmtiError::InvalidClassFormat,
            rvmti_sys::jvmtiError_JVMTI_ERROR_CIRCULAR_CLASS_DEFINITION => JvmtiError::CircularClassDefinition,
            rvmti_sys::jvmtiError_JVMTI_ERROR_FAILS_VERIFICATION => JvmtiError::FailsVerification,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_METHOD_ADDED => JvmtiError::UnsupportedRedefinitionMethodAdded,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_SCHEMA_CHANGED => JvmtiError::UnsupportedRedefinitionSchemaChanged,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_TYPESTATE => JvmtiError::InvalidTypeState,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_HIERARCHY_CHANGED => JvmtiError::UnsupportedRedefinitionHierarchyChanged,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_METHOD_DELETED => JvmtiError::UnsupportedRedefinitionMethodDeleted,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_VERSION => JvmtiError::UnsupportedVersion,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NAMES_DONT_MATCH => JvmtiError::NamesDontMatch,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_CLASS_MODIFIERS_CHANGED => JvmtiError::UnsupportedRedefinitionClassModifiersChanged,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNSUPPORTED_REDEFINITION_METHOD_MODIFIERS_CHANGED => JvmtiError::UnsupportedRedefinitionMethodModifiersChanged,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNMODIFIABLE_CLASS => JvmtiError::UnmodifiableClass,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNMODIFIABLE_MODULE => JvmtiError::UnmodifiableModule,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NOT_AVAILABLE => JvmtiError::NotAvaliable,
            rvmti_sys::jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY => JvmtiError::MustPosessCapability,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NULL_POINTER => JvmtiError::NullPointer,
            rvmti_sys::jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION => JvmtiError::AbsentInformation,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_EVENT_TYPE => JvmtiError::InvalidEventType,
            rvmti_sys::jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT => JvmtiError::IllegalArgument,
            rvmti_sys::jvmtiError_JVMTI_ERROR_NATIVE_METHOD => JvmtiError::NativeMethod,
            rvmti_sys::jvmtiError_JVMTI_ERROR_CLASS_LOADER_UNSUPPORTED => JvmtiError::ClassLoaderUnsupported,
            rvmti_sys::jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY => JvmtiError::OutOfMemory,
            rvmti_sys::jvmtiError_JVMTI_ERROR_ACCESS_DENIED => JvmtiError::AccessDenied,
            rvmti_sys::jvmtiError_JVMTI_ERROR_WRONG_PHASE => JvmtiError::WrongPhase,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INTERNAL => JvmtiError::Internal,
            rvmti_sys::jvmtiError_JVMTI_ERROR_UNATTACHED_THREAD => JvmtiError::UnattachedThread,
            rvmti_sys::jvmtiError_JVMTI_ERROR_INVALID_ENVIRONMENT => JvmtiError::InvalidEnvironment,
            _ => JvmtiError::UnsupportedError(error),
        }
    }

}

impl From<rvmti_sys::jint> for JniError {

    fn from(error: rvmti_sys::jint) -> JniError {
        match error {
            rvmti_sys::JNI_ERR => JniError::UnknownError,
            rvmti_sys::JNI_EDETACHED => JniError::ThreadDetachedFromVm,
            rvmti_sys::JNI_EVERSION => JniError::JniVersionError,
            rvmti_sys::JNI_ENOMEM => JniError::NotEnoughMemory,
            rvmti_sys::JNI_EEXIST => JniError::VmAlreadyCreated,
            rvmti_sys::JNI_EINVAL => JniError::InvalidArguments,
            code => JniError::UnsupportedError(code),
        }
    }

}

impl From<JvmtiEventMode> for rvmti_sys::jvmtiEventMode {

    fn from(value: JvmtiEventMode) -> rvmti_sys::jvmtiEventMode {
        match value {
            JvmtiEventMode::Enable => rvmti_sys::jvmtiEventMode_JVMTI_ENABLE,
            JvmtiEventMode::Disable => rvmti_sys::jvmtiEventMode_JVMTI_DISABLE,
        }
    }

}

impl From<JvmtiEvent> for rvmti_sys::jvmtiEvent {

    fn from(value: JvmtiEvent) -> rvmti_sys::jvmtiEvent {
        match value {
            JvmtiEvent::VmInit => rvmti_sys::jvmtiEvent_JVMTI_EVENT_VM_INIT,
            JvmtiEvent::VmDeath => rvmti_sys::jvmtiEvent_JVMTI_EVENT_VM_DEATH,
            JvmtiEvent::ThreadStart => rvmti_sys::jvmtiEvent_JVMTI_EVENT_THREAD_START,
            JvmtiEvent::ThreadEnd => rvmti_sys::jvmtiEvent_JVMTI_EVENT_THREAD_END,
            JvmtiEvent::ClassFileLoadHook => rvmti_sys::jvmtiEvent_JVMTI_EVENT_CLASS_FILE_LOAD_HOOK,
            JvmtiEvent::ClassLoad => rvmti_sys::jvmtiEvent_JVMTI_EVENT_CLASS_LOAD,
            JvmtiEvent::ClassPrepare => rvmti_sys::jvmtiEvent_JVMTI_EVENT_CLASS_PREPARE,
            JvmtiEvent::VmStart => rvmti_sys::jvmtiEvent_JVMTI_EVENT_VM_START,
            JvmtiEvent::Exception => rvmti_sys::jvmtiEvent_JVMTI_EVENT_EXCEPTION,
            JvmtiEvent::ExceptionCatch => rvmti_sys::jvmtiEvent_JVMTI_EVENT_EXCEPTION_CATCH,
            JvmtiEvent::SingleStep => rvmti_sys::jvmtiEvent_JVMTI_EVENT_SINGLE_STEP,
            JvmtiEvent::FramePop => rvmti_sys::jvmtiEvent_JVMTI_EVENT_FRAME_POP,
            JvmtiEvent::Breakpoint => rvmti_sys::jvmtiEvent_JVMTI_EVENT_BREAKPOINT,
            JvmtiEvent::FieldAccess => rvmti_sys::jvmtiEvent_JVMTI_EVENT_FIELD_ACCESS,
            JvmtiEvent::FieldModification => rvmti_sys::jvmtiEvent_JVMTI_EVENT_FIELD_MODIFICATION,
            JvmtiEvent::MethodEntry => rvmti_sys::jvmtiEvent_JVMTI_EVENT_METHOD_ENTRY,
            JvmtiEvent::MethodExit => rvmti_sys::jvmtiEvent_JVMTI_EVENT_METHOD_EXIT,
            JvmtiEvent::NativeMethodBind => rvmti_sys::jvmtiEvent_JVMTI_EVENT_NATIVE_METHOD_BIND,
            JvmtiEvent::CompiledMethodLoad => rvmti_sys::jvmtiEvent_JVMTI_EVENT_COMPILED_METHOD_LOAD,
            JvmtiEvent::CompiledMethodUnload => rvmti_sys::jvmtiEvent_JVMTI_EVENT_COMPILED_METHOD_UNLOAD,
            JvmtiEvent::DynamicCodeGenerated => rvmti_sys::jvmtiEvent_JVMTI_EVENT_DYNAMIC_CODE_GENERATED,
            JvmtiEvent::DataDumpRequest => rvmti_sys::jvmtiEvent_JVMTI_EVENT_DATA_DUMP_REQUEST,
            JvmtiEvent::MonitorWait => rvmti_sys::jvmtiEvent_JVMTI_EVENT_MONITOR_WAIT,
            JvmtiEvent::MonitorWaited => rvmti_sys::jvmtiEvent_JVMTI_EVENT_MONITOR_WAITED,
            JvmtiEvent::MonitorContendedEnter => rvmti_sys::jvmtiEvent_JVMTI_EVENT_MONITOR_CONTENDED_ENTER,
            JvmtiEvent::MonitorContendedEntered => rvmti_sys::jvmtiEvent_JVMTI_EVENT_MONITOR_CONTENDED_ENTERED,
            JvmtiEvent::ResourceExhausted => rvmti_sys::jvmtiEvent_JVMTI_EVENT_RESOURCE_EXHAUSTED,
            JvmtiEvent::GarbageCollectionStart => rvmti_sys::jvmtiEvent_JVMTI_EVENT_GARBAGE_COLLECTION_START,
            JvmtiEvent::GarbageCollectionFinish => rvmti_sys::jvmtiEvent_JVMTI_EVENT_GARBAGE_COLLECTION_FINISH,
            JvmtiEvent::ObjectFree => rvmti_sys::jvmtiEvent_JVMTI_EVENT_OBJECT_FREE,
            JvmtiEvent::VmObjectAlloc => rvmti_sys::jvmtiEvent_JVMTI_EVENT_VM_OBJECT_ALLOC,
            JvmtiEvent::SampledObjectAlloc => rvmti_sys::jvmtiEvent_JVMTI_EVENT_SAMPLED_OBJECT_ALLOC,
        }
    }

}
