// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

mod rvmti;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate failure;
#[macro_use]
extern crate failure_derive;

use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::MutexGuard;

pub use rvmti::Agent_OnLoad;
pub use rvmti::Agent_OnUnload;
pub use rvmti::jvmti_event_breakpoint_handler;
pub use rvmti::jvmti_event_class_file_load_hook_handler;
pub use rvmti::jvmti_event_class_load_handler;
pub use rvmti::jvmti_event_class_prepare_handler;
pub use rvmti::jvmti_event_compiled_method_load_handler;
pub use rvmti::jvmti_event_compiled_method_unload_handler;
pub use rvmti::jvmti_event_data_dump_request_handler;
pub use rvmti::jvmti_event_dynamic_code_generated_handler;
pub use rvmti::jvmti_event_exception_handler;
pub use rvmti::jvmti_event_exception_catch_handler;
pub use rvmti::jvmti_event_field_access_handler;
pub use rvmti::jvmti_event_field_modification_handler;
pub use rvmti::jvmti_event_frame_pop_handler;
pub use rvmti::jvmti_event_garbage_collection_finish_handler;
pub use rvmti::jvmti_event_garbage_collection_start_handler;
pub use rvmti::jvmti_event_method_entry_handler;
pub use rvmti::jvmti_event_method_exit_handler;
pub use rvmti::jvmti_event_monitor_contended_enter_handler;
pub use rvmti::jvmti_event_monitor_contended_entered_handler;
pub use rvmti::jvmti_event_monitor_wait_handler;
pub use rvmti::jvmti_event_monitor_waited_handler;
pub use rvmti::jvmti_event_native_method_bind_handler;
pub use rvmti::jvmti_event_object_free_handler;
pub use rvmti::jvmti_event_resource_exhausted_handler;
pub use rvmti::jvmti_event_single_step_handler;
pub use rvmti::jvmti_event_thread_end_handler;
pub use rvmti::jvmti_event_thread_start_handler;
pub use rvmti::jvmti_event_vm_death_handler;
pub use rvmti::jvmti_event_vm_init_handler;
pub use rvmti::jvmti_event_vm_object_alloc_handler;
pub use rvmti::jvmti_event_vm_start_handler;

lazy_static! {
    static ref JVMTI_ENVIRONMENTS: Mutex<Vec<rvmti::JvmtiEnv>> = Mutex::new(Vec::new());
}

pub fn agent_on_load(vm: &rvmti::Jvm, options: &Option<String>) -> i32 {
    info!("Agent starting...");
    debug!("Agent options: {}", options.as_ref().unwrap_or(&"".to_string()));
    match do_on_load(vm, options) {
        Ok(_) => {
            info!("Agent started");
            return 0;
        },
        Err(err) => {
            error!("Agent initialization error: {}", err);
            return -1
        }
    }
}

pub fn agent_on_unload(vm: &rvmti::Jvm) {
    info!("Agent unloading...");
    match JVMTI_ENVIRONMENTS.lock() {
        Ok(mut guard) => {
            for mut env in guard.as_mut_slice() {
                unload_environment(&mut env);
            }
            guard.clear();
            debug!("Environments freed");
        },
        Err(err) => {
            warn!("Failed to lock environments global storage: {}", err);
        }
    }
    info!("Agent unloaded");
}

pub fn jvmti_event_compiled_method_load(env: &mut rvmti::JvmtiEnv, method_id: &rvmti::JMethodId,
                                        address_locations: &Option<Vec<rvmti::AddressLocationEntry>>,
                                        compile_info: &Option<Vec<rvmti::CompiledMethodLoadRecord>>, address: usize, length: usize)
{
    // Agent_OnUnload may be called while this handler is still running so some methods calls may fail, even Deallocate calls
    // It does not look like a huge problem as Agent_OnUnload is called on JVM shutdown anyway
    // Lock on the environment would prevent this but then all handlers would be executed serially which is undesirable
    // Maybe name resolution can be moved to the dedicated thread using some lock-free queue...
    let method_info = method_info(env, method_id);
    match method_info {
        Ok(info) => {
            let stack_info = stack_info(env, compile_info);
            match stack_info {
                Ok(stack_info_value) => {
                    info!("'Compiled method load' event fired: {:?}, {:?}, {:?}, 0x{:x}, {}, {:?}, {:?}, {:?}",
                          info.name, info.class.signature, info.class.source_file_name, address, length,
                          info.line_numbers, address_locations, stack_info_value);
                },
                Err(e) => {
                    info!("'Compiled method load' event fired: {:?}, {:?}, {:?}, 0x{:x}, {}, {:?}, {:?}, {:?}",
                          info.name, info.class.signature, info.class.source_file_name, address, length,
                          info.line_numbers, address_locations, compile_info);
                    error!("Failed to process method inlining info: {}", e);
                }
            }
        },
        Err(e) => {
            info!("'Compiled method load' event fired: {:?}, 0x{:x}, {}, {:?}, {:?}",
                  method_id, address, length, address_locations, compile_info);
            error!("Failed to get compiled method info: {}", e);
        }
    }
}

pub fn jvmti_event_dynamic_code_generated(env: &mut rvmti::JvmtiEnv, name: &Option<String>, address: usize, length: usize) {
    info!("'Dynamic code generated' event fired: {}, 0x{:x}, {}", name.as_ref().unwrap_or(&"".to_string()), address, length);
}

fn unload_environment(env: &mut rvmti::JvmtiEnv) {
}

fn do_on_load<'a>(vm: &rvmti::Jvm, options: &Option<String>) -> Result<(), AgentInitError> {
    let mut jvmti_env = vm.get_jvmti_env(rvmti::JvmtiVersion::CurrentVersion)
        .map_err(|e| AgentInitError::UnableToObtainJvmtiEnvironment(e))?;
    debug!("Environment obtained");
    {
        let mut guard = JVMTI_ENVIRONMENTS.lock().map_err(AgentInitError::from)?;
        let initialization_result = initialize_agent(&mut jvmti_env, &options);
        guard.push(jvmti_env);
        return initialization_result;
    }
}

fn initialize_agent<'a>(env: &mut rvmti::JvmtiEnv, options: &Option<String>) -> Result<(), AgentInitError> {
    let _ = add_capabilities(env)?;
    let _ = set_event_callbacks(env)?;
    let _ = enable_events(env)?;
    Ok(())
}

fn add_capabilities<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError> {
    let mut capabilities = rvmti::JvmtiCapabilities::new_empty_capabilities()
        .map_err(AgentInitError::UnableToAllocateCapabilities)?;

    capabilities.set_can_generate_all_class_hook_events(true);
    capabilities.set_can_tag_objects(true);
    capabilities.set_can_generate_object_free_events(true);
    capabilities.set_can_get_source_file_name(true);
    capabilities.set_can_get_line_numbers(true);
    capabilities.set_can_generate_vm_object_alloc_events(true);
    capabilities.set_can_generate_compiled_method_load_events(true);
    env.add_capabilities(&capabilities).map_err(AgentInitError::UnableToAddCapabilities)?;
    debug!("Capabilities added to the environment");
    Ok(())
}

fn set_event_callbacks<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError> {
    let mut settings = rvmti::JvmtiEventCallbacksSettings::new_empty_settings()
        .map_err(AgentInitError::UnableToAllocateEventCallbackSettings)?;
    settings.set_compiled_method_load_enabled(true);
    settings.set_dynamic_code_generated_enabled(true);
    env.set_event_callbacks_settings(&settings).map_err(AgentInitError::UnableToSetEventCallbacks)?;
    debug!("Event callbacks set for the environment");
    Ok(())
}

fn enable_events<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError> {
    env.set_event_notification_mode(rvmti::JvmtiEventMode::Enable, rvmti::JvmtiEvent::CompiledMethodLoad, None)
        .map_err(AgentInitError::UnableToEnableEvents)?;
    env.set_event_notification_mode(rvmti::JvmtiEventMode::Enable, rvmti::JvmtiEvent::DynamicCodeGenerated, None)
        .map_err(AgentInitError::UnableToEnableEvents)?;
    debug!("Events enabled for the environment");
    Ok(())
}

#[derive(Fail, Debug)]
enum AgentInitError {
    #[fail(display = "Failed to allocate capabilities: {}", _0)]
    UnableToAllocateCapabilities(#[cause] rvmti::AllocError),
    #[fail(display = "Failed to add capabilities: {}", _0)]
    UnableToAddCapabilities(#[cause] rvmti::JvmtiError),
    #[fail(display = "Failed to allocate event callback settings: {}", _0)]
    UnableToAllocateEventCallbackSettings(#[cause] rvmti::AllocError),
    #[fail(display = "Failed to set event callbacks: {}", _0)]
    UnableToSetEventCallbacks(#[cause] rvmti::JvmtiError),
    #[fail(display = "Failed to enable events: {}", _0)]
    UnableToEnableEvents(#[cause] rvmti::JvmtiError),
    #[fail(display = "Failed to obtain jvmti environment: {}", _0)]
    UnableToObtainJvmtiEnvironment(#[cause] rvmti::JniError),
    #[fail(display = "The mutex was poisoned")]
    PoisonedMutexError,
}

impl<'a> From<PoisonError<MutexGuard<'a, Vec<rvmti::JvmtiEnv>>>> for AgentInitError {

    fn from(_error: PoisonError<MutexGuard<'a, Vec<rvmti::JvmtiEnv>>>) -> AgentInitError {
        AgentInitError::PoisonedMutexError
    }

}

#[derive(Debug)]
struct MethodInfo {
    name: rvmti::MethodName,
    class: ClassInfo,
    native_method: bool,
    line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
}

#[derive(Debug)]
struct ClassInfo {
    signature: rvmti::ClassSignature,
    source_file_name: Option<String>,
}

#[derive(Debug)]
struct StackFrameInfo {
    method: MethodInfo,
    byte_code_index: i32,
}

#[derive(Debug)]
struct StackInfo {
    pc_address: usize,
    stack_frames: Vec<StackFrameInfo>,
}

#[derive(Fail, Debug)]
enum MethodInfoError {
    #[fail(display = "Failed to obtain method name: {}", _0)]
    UnableToGetMethodName(#[cause] rvmti::GetMethodNameError),
    #[fail(display = "Failed to obtain method declaring class id: {}", _0)]
    UnableToGetMethodDeclaringClass(#[cause] rvmti::JvmtiError),
    #[fail(display = "Failed to obtain method declaring class info: {}", _0)]
    UnableToGetDeclaringClassInfo(#[cause] ClassInfoError),
    #[fail(display = "Failed to check if method is native: {}", _0)]
    UnableToCheckIfMethodIsNative(#[cause] rvmti::JvmtiError),
    #[fail(display = "Failed to obtain method line numbers: {}", _0)]
    UnableToGetMethodLineNumbers(#[cause] rvmti::JvmtiError),
}

#[derive(Fail, Debug)]
enum ClassInfoError {
    #[fail(display = "Failed to obtain class signature: {}", _0)]
    UnableToGetClassSignature(#[cause] rvmti::GetClassSignatureError),
    #[fail(display = "Failed to obtain class source file name: {}", _0)]
    UnableToGetClassSourceFileName(#[cause] rvmti::GetSourceFileNameError),
}

#[derive(Fail, Debug)]
enum StackInfoError {
    #[fail(display = "Failed to obtain method metadata: {}", _0)]
    UnableToGetMethodInfo(#[cause] MethodInfoError),
}

fn method_info(env: &mut rvmti::JvmtiEnv, method_id: &rvmti::JMethodId) -> Result<MethodInfo, MethodInfoError> {
    let name = env.get_method_name(&method_id)
        .map_err(MethodInfoError::UnableToGetMethodName)?;
    let declaring_class_id = env.get_method_declaring_class(&method_id)
        .map_err(MethodInfoError::UnableToGetMethodDeclaringClass)?;
    let class = class_info(env, &declaring_class_id)
        .map_err(MethodInfoError::UnableToGetDeclaringClassInfo)?;
    let native_method = env.check_is_method_native(method_id)
        .map_err(MethodInfoError::UnableToCheckIfMethodIsNative)?;
    let line_numbers = if !native_method {
        env.get_line_number_table(&method_id).map_err(MethodInfoError::UnableToGetMethodLineNumbers)?
    } else {
        None
    };
    let method_info = MethodInfo{name, class, native_method, line_numbers};
    Ok(method_info)
}

fn class_info(env: &mut rvmti::JvmtiEnv, class_id: &rvmti::JClass) -> Result<ClassInfo, ClassInfoError> {
    let signature = env.get_class_signature(class_id)
        .map_err(ClassInfoError::UnableToGetClassSignature)?;
    let source_file_name = env.get_source_file_name(class_id)
        .map_err(ClassInfoError::UnableToGetClassSourceFileName)?;
    let class_info = ClassInfo{signature, source_file_name};
    Ok(class_info)
}

fn stack_info(env: &mut rvmti::JvmtiEnv, compile_info: &Option<Vec<rvmti::CompiledMethodLoadRecord>>)
    -> Result<Option<Vec<StackInfo>>, StackInfoError>
{
    match compile_info {
        &Some(ref infos) => {
            let mut result = Vec::new();
            for info in infos.iter() {
                match info {
                    &rvmti::CompiledMethodLoadRecord::Inline{ref stack_infos} => {
                        for stack_info in stack_infos.iter() {
                            let mut stack_frame_infos: Vec<StackFrameInfo> = Vec::new();
                            for stack_frame in stack_info.stack_frames.iter() {
                                let method_info = method_info(env, &stack_frame.method_id)
                                    .map_err(StackInfoError::UnableToGetMethodInfo)?;
                                stack_frame_infos.push(StackFrameInfo{method: method_info,
                                    byte_code_index: stack_frame.byte_code_index});
                            }
                            result.push(StackInfo{pc_address: stack_info.pc_address, stack_frames: stack_frame_infos});
                        }
                    },
                    _ => {},
                }
            }
            return Ok(Some(result));
        },
        &None => Ok(None),
    }
}
