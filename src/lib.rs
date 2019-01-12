// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

mod rvmti;
mod perf;
mod demangle;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate nix;

use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::MutexGuard;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::slice;

pub use crate::rvmti::Agent_OnLoad;
pub use crate::rvmti::Agent_OnUnload;
pub use crate::rvmti::jvmti_event_breakpoint_handler;
pub use crate::rvmti::jvmti_event_class_file_load_hook_handler;
pub use crate::rvmti::jvmti_event_class_load_handler;
pub use crate::rvmti::jvmti_event_class_prepare_handler;
pub use crate::rvmti::jvmti_event_compiled_method_load_handler;
pub use crate::rvmti::jvmti_event_compiled_method_unload_handler;
pub use crate::rvmti::jvmti_event_data_dump_request_handler;
pub use crate::rvmti::jvmti_event_dynamic_code_generated_handler;
pub use crate::rvmti::jvmti_event_exception_handler;
pub use crate::rvmti::jvmti_event_exception_catch_handler;
pub use crate::rvmti::jvmti_event_field_access_handler;
pub use crate::rvmti::jvmti_event_field_modification_handler;
pub use crate::rvmti::jvmti_event_frame_pop_handler;
pub use crate::rvmti::jvmti_event_garbage_collection_finish_handler;
pub use crate::rvmti::jvmti_event_garbage_collection_start_handler;
pub use crate::rvmti::jvmti_event_method_entry_handler;
pub use crate::rvmti::jvmti_event_method_exit_handler;
pub use crate::rvmti::jvmti_event_monitor_contended_enter_handler;
pub use crate::rvmti::jvmti_event_monitor_contended_entered_handler;
pub use crate::rvmti::jvmti_event_monitor_wait_handler;
pub use crate::rvmti::jvmti_event_monitor_waited_handler;
pub use crate::rvmti::jvmti_event_native_method_bind_handler;
pub use crate::rvmti::jvmti_event_object_free_handler;
pub use crate::rvmti::jvmti_event_resource_exhausted_handler;
pub use crate::rvmti::jvmti_event_single_step_handler;
pub use crate::rvmti::jvmti_event_thread_end_handler;
pub use crate::rvmti::jvmti_event_thread_start_handler;
pub use crate::rvmti::jvmti_event_vm_death_handler;
pub use crate::rvmti::jvmti_event_vm_init_handler;
pub use crate::rvmti::jvmti_event_vm_object_alloc_handler;
pub use crate::rvmti::jvmti_event_vm_start_handler;

lazy_static! {
    static ref AGENT_ENV: Mutex<Option<AgentEnv>> = Mutex::new(None);
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

pub fn agent_on_unload(_vm: &rvmti::Jvm) {
    info!("Agent unloading...");
    match AGENT_ENV.lock() {
        Ok(mut guard) => {
            match guard.take() {
                Some(ref mut env) => {
                    unload_environment(&mut env.env);
                },
                None => {
                    warn!("Agent was not initialized, skipping shutdown");
                }
            }
            debug!("Environments freed");
        },
        Err(err) => {
            warn!("Failed to lock agent environment for unloading: {}", err);
        }
    }
    info!("Agent unloaded");
}

pub fn jvmti_event_compiled_method_load(env: &mut rvmti::JvmtiEnv, method_id: &rvmti::JMethodId,
                                        address_locations: &Option<Vec<rvmti::AddressLocationEntry>>,
                                        compile_info: &Option<Vec<rvmti::CompiledMethodLoadRecord>>, address: usize, length: usize)
{
    match on_compiled_method_load(env, method_id, address_locations, compile_info, address, length) {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to handle compiled method load event: {}", e);
        }
    }
}

pub fn jvmti_event_dynamic_code_generated(env: &mut rvmti::JvmtiEnv, name: &Option<String>, address: usize, length: usize) {
    match on_dynamic_code_generated(env, name, address, length) {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to handle dynamic code generation event: {}", e);
        }
    }
}

fn on_compiled_method_load(env: &mut rvmti::JvmtiEnv, method_id: &rvmti::JMethodId,
                           address_locations: &Option<Vec<rvmti::AddressLocationEntry>>,
                           compile_info: &Option<Vec<rvmti::CompiledMethodLoadRecord>>,
                           address: usize, length: usize) -> Result<(), CompiledMethodLoadHandlerError>
{
    let mut guard = AGENT_ENV.lock().map_err(|_e| CompiledMethodLoadHandlerError::FailedToLockAgentEnvironment)?;
    match *guard {
        Some(ref mut agent_env) => {
            let method_info = method_info(env, method_id).map_err(CompiledMethodLoadHandlerError::UnableToGetMethodInfo)?;
            let stack_info = stack_info(env, compile_info).map_err(CompiledMethodLoadHandlerError::UnableToGetStackInfo)?;
            let timestamp = perf::get_timestamp().map_err(CompiledMethodLoadHandlerError::UnableToGetTimestamp)?;
            agent_env.compiled_method_load(method_info.name, method_info.class.signature,
                                           method_info.class.source_file_name, address, length,
                                           method_info.line_numbers, address_locations.clone(),
                                           stack_info, timestamp);
            Ok(())
        },
        None => Err(CompiledMethodLoadHandlerError::AgentNotInitialized),
    }
}

fn on_dynamic_code_generated(_env: &mut rvmti::JvmtiEnv, name: &Option<String>, address: usize,
                             length: usize) -> Result<(), DynamicCodeGeneratedHandlerError>
{
    let mut guard = AGENT_ENV.lock().map_err(|_e| DynamicCodeGeneratedHandlerError::FailedToLockAgentEnvironment)?;
    match *guard {
        Some(ref mut env) => {
            let timestamp = perf::get_timestamp().map_err(DynamicCodeGeneratedHandlerError::UnableToGetTimestamp)?;
            env.dynamic_code_generated(name, address, length, timestamp);
            Ok(())
        },
        None => Err(DynamicCodeGeneratedHandlerError::AgentNotInitialized),
    }
}

fn unload_environment(_env: &mut rvmti::JvmtiEnv) {
}

fn do_on_load<'a>(vm: &rvmti::Jvm, options: &Option<String>) -> Result<(), AgentInitError> {
    let mut guard = AGENT_ENV.lock().map_err(AgentInitError::from)?;
    return match *guard {
        Some(_) => {
            warn!("Agent was already initialized, skipping initialization");
            Ok(())
        },
        None => {
            let mut jvmti_env = vm.get_jvmti_env(rvmti::JvmtiVersion::CurrentVersion)
                .map_err(AgentInitError::UnableToObtainJvmtiEnvironment)?;
            debug!("Environment obtained");
            let _ = initialize_agent(&mut jvmti_env, &options)?;
            debug!("Agent environment successfully initialized");
            let dump_dir = perf::create_dump_dir()
                .map_err(AgentInitError::UnableToCreateDumpDir)?;
            debug!("Jit dump directory created");
            let dump_file = perf::DumpFile::new(dump_dir)
                .map_err(AgentInitError::UnableToCreateDumpFile)?;
            debug!("Jit dump file created");
            *guard = Some(AgentEnv::new(jvmti_env, dump_file));
            Ok(())
        }
    }
}

fn initialize_agent<'a>(env: &mut rvmti::JvmtiEnv, _options: &Option<String>) -> Result<(), AgentInitError> {
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

impl AgentEnv {

    fn new(env: rvmti::JvmtiEnv, dump_file: perf::DumpFile) -> AgentEnv {
        debug!("Spawning agent worker thread...");
        let (sender, receiver) = channel();
        let worker = thread::spawn(move|| {
            debug!("Agent worker thread running...");
            run_worker(receiver, dump_file);
        });
        debug!("Agent worker thread spawned");
        AgentEnv{env, sender, worker: Some(worker)}
    }

    fn dynamic_code_generated(&mut self, name: &Option<String>, address: usize, length: usize, timestamp: i64) {
        let code = unsafe{
            slice::from_raw_parts(address as *const u8, length)
        };
        match self.sender.send(AgentMessage::DynamicCodeGenerated {name: name.clone(), address, length, timestamp, code: code.to_vec()}) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to send dynamic code generation event to worker thread: {}", e);
            },
        };
    }

    fn compiled_method_load(&mut self, name: rvmti::MethodName, class_signature: rvmti::ClassSignature,
                            class_source_file_name: Option<String>, address: usize, length: usize,
                            line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
                            address_locations: Option<Vec<rvmti::AddressLocationEntry>>,
                            stack_info: Option<Vec<StackInfo>>, timestamp: i64)
    {
        let code = unsafe{
            slice::from_raw_parts(address as *const u8, length)
        };
        match self.sender.send(AgentMessage::CompiledMethodLoad {name, class_signature, class_source_file_name,
            address, length, line_numbers, address_locations, stack_info, timestamp, code: code.to_vec()})
        {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to send dynamic code generation event to worker thread: {}", e);
            },
        };
    }
}

impl Drop for AgentEnv {

    fn drop(&mut self) {
        debug!("Stopping agent worker thread...");
        let send_shutdown_result = self.sender.send(AgentMessage::Shutdown);
        match send_shutdown_result {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to send shutdown request to agent worker thread: {}", e)
            },
        };
        match self.worker.take() {
            Some(w) => {
                let worker_thread_result = w.join();
                match worker_thread_result {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Failed to wait for worker thread shutdown: {:?}", e);
                    }
                }
            },
            None => {},
        }
        debug!("Agent worker thread stopped");
    }

}

fn run_worker(receiver: Receiver<AgentMessage>, mut dump_file: perf::DumpFile) {
    match dump_file.write_header() {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to write jit dump header: {}", e);
        },
    }
    let mut code_index = 0u64;
    loop {
        match receiver.recv() {
            Ok(message) => {
                match message {
                    AgentMessage::DynamicCodeGenerated { name, address, length, timestamp, code } => {
                        debug!("'Dynamic code generated' event fired: {}, 0x{:x}, {}",
                              name.as_ref().unwrap_or(&"".to_string()), address, length);
                        if name.is_some() && address != 0 as usize && length > 0 as usize {
                            match dump_file.write_jit_code_load(name.unwrap(), address, length, code_index, timestamp, &code) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Failed to write jit code load record for dynamically generated code: {}", e);
                                }
                            }
                            code_index += 1u64;
                        }
                    },
                    AgentMessage::CompiledMethodLoad { name, class_signature, class_source_file_name,
                        address, length, line_numbers, address_locations, stack_info, timestamp, code } => {
                        debug!("'Compiled method load' event fired: {:?}, {:?}, {:?}, 0x{:x}, {}, {:?}, {:?}, {:?}",
                              name, class_signature, class_source_file_name, address, length,
                              line_numbers, address_locations, stack_info);
                        if address != 0 as usize && length > 0 as usize {
                            match dump_file.write_line_numbers(&name, &class_signature,
                                                               &class_source_file_name,
                                                               address, &line_numbers,
                                                               &address_locations, &stack_info,
                                                               timestamp)
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Failed to write jit code load line numbers record for compiled method: {}", e);
                                }
                            }
                            match dump_file.write_compiled_method_load(name, class_signature,
                                                                       class_source_file_name,
                                                                       address, length, line_numbers,
                                                                       address_locations, stack_info,
                                                                       code_index, timestamp, &code)
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Failed to write jit code load record for compiled method: {}", e);
                                }
                            }
                            code_index += 1u64;
                        }
                    }
                    AgentMessage::Shutdown => {
                        debug!("Received shutdown request for agent worker thread...");
                        break;
                    },
                }
            },
            Err(e) => {
                error!("Agent message bus was shutdown unexpectedly: {}", e);
                break;
            }
        }
    }
    match dump_file.write_code_close_record() {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to write jit dump code close record: {}", e);
        },
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Option<AgentEnv>>>> for AgentInitError {

    fn from(_error: PoisonError<MutexGuard<'a, Option<AgentEnv>>>) -> AgentInitError {
        AgentInitError::PoisonedMutexError
    }

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

#[derive(Debug)]
enum AgentMessage {
    Shutdown,
    DynamicCodeGenerated { name: Option<String>, address: usize, length: usize, timestamp: i64, code: Vec<u8> },
    CompiledMethodLoad { name: rvmti::MethodName, class_signature: rvmti::ClassSignature, class_source_file_name: Option<String>,
        address: usize, length: usize, line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
        address_locations: Option<Vec<rvmti::AddressLocationEntry>>, stack_info: Option<Vec<StackInfo>>,
        timestamp: i64, code: Vec<u8> },
}

#[derive(Debug)]
struct AgentEnv {
    env: rvmti::JvmtiEnv,
    sender: Sender<AgentMessage>,
    worker: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub struct MethodInfo {
    name: rvmti::MethodName,
    class: ClassInfo,
    native_method: bool,
    line_numbers: Option<Vec<rvmti::LineNumberEntry>>,
}

#[derive(Debug)]
pub struct ClassInfo {
    signature: rvmti::ClassSignature,
    source_file_name: Option<String>,
}

#[derive(Debug)]
pub struct StackFrameInfo {
    method: MethodInfo,
    byte_code_index: i32,
}

#[derive(Debug)]
pub struct StackInfo {
    pc_address: usize,
    stack_frames: Vec<StackFrameInfo>,
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
    #[fail(display = "Failed to create jit dump directory: {}", _0)]
    UnableToCreateDumpDir(#[cause] perf::CreteDumpDirError),
    #[fail(display = "Failed to create jit dump file: {}", _0)]
    UnableToCreateDumpFile(#[cause] perf::NewDumpFileError),
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

#[derive(Fail, Debug)]
enum DynamicCodeGeneratedHandlerError {
    #[fail(display = "Unable to get timestamp: {}", _0)]
    UnableToGetTimestamp(#[cause] nix::errno::Errno),
    #[fail(display = "Agent is not initialized")]
    AgentNotInitialized,
    #[fail(display = "Failed to lock agent environment")]
    FailedToLockAgentEnvironment,
}

#[derive(Fail, Debug)]
enum CompiledMethodLoadHandlerError {
    #[fail(display = "Unable to get timestamp: {}", _0)]
    UnableToGetTimestamp(#[cause] nix::errno::Errno),
    #[fail(display = "Agent is not initialized")]
    AgentNotInitialized,
    #[fail(display = "Failed to lock agent environment")]
    FailedToLockAgentEnvironment,
    #[fail(display = "Unable to get method info: {}", _0)]
    UnableToGetMethodInfo(#[cause] MethodInfoError),
    #[fail(display = "Unable to get stack info: {}", _0)]
    UnableToGetStackInfo(#[cause] StackInfoError),
}
