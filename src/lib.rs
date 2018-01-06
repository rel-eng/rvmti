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

use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::MutexGuard;
use std::fmt;
use std::error;

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
    // TODO Log errors
    let method_name = env.get_method_name(&method_id).ok();
    let class_id = env.get_method_declaring_class(&method_id).ok();
    let (class_signature, source_file_name) = match class_id {
        Some(id) => {
            (env.get_class_signature(&id).ok(), env.get_source_file_name(&id).ok().and_then(|e| e))
        },
        None => (None, None),
    };
    let line_numbers = env.get_line_number_table(&method_id).ok().and_then(|e| e);
    info!("'Compiled method load' event fired: {:?}, {:?}, {:?}, 0x{:x}, {}, {:?}, {:?}, {:?}", method_name, class_signature,
             source_file_name, address, length, line_numbers, address_locations, compile_info);
}

pub fn jvmti_event_dynamic_code_generated(env: &mut rvmti::JvmtiEnv, name: &Option<String>, address: usize, length: usize) {
    info!("'Dynamic code generated' event fired: {}, 0x{:x}, {}", name.as_ref().unwrap_or(&"".to_string()), address, length);
}

fn unload_environment(env: &mut rvmti::JvmtiEnv) {
}

fn do_on_load<'a>(vm: &rvmti::Jvm, options: &Option<String>) -> Result<(), AgentInitError<'a>> {
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

fn initialize_agent<'a>(env: &mut rvmti::JvmtiEnv, options: &Option<String>) -> Result<(), AgentInitError<'a>> {
    let _ = add_capabilities(env)?;
    let _ = set_event_callbacks(env)?;
    let _ = enable_events(env)?;
    Ok(())
}

fn add_capabilities<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError<'a>> {
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

fn set_event_callbacks<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError<'a>> {
    let mut settings = rvmti::JvmtiEventCallbacksSettings::new_empty_settings()
        .map_err(AgentInitError::UnableToAllocateEventCallbackSettings)?;
    settings.set_compiled_method_load_enabled(true);
    settings.set_dynamic_code_generated_enabled(true);
    env.set_event_callbacks_settings(&settings).map_err(AgentInitError::UnableToSetEventCallbacks)?;
    debug!("Event callbacks set for the environment");
    Ok(())
}

fn enable_events<'a>(env: &mut rvmti::JvmtiEnv) -> Result<(), AgentInitError<'a>> {
    env.set_event_notification_mode(rvmti::JvmtiEventMode::Enable, rvmti::JvmtiEvent::CompiledMethodLoad, None)
        .map_err(AgentInitError::UnableToEnableEvents)?;
    env.set_event_notification_mode(rvmti::JvmtiEventMode::Enable, rvmti::JvmtiEvent::DynamicCodeGenerated, None)
        .map_err(AgentInitError::UnableToEnableEvents)?;
    debug!("Events enabled for the environment");
    Ok(())
}

#[derive(Debug)]
enum AgentInitError<'a> {
    UnableToAllocateCapabilities(rvmti::AllocError),
    UnableToAddCapabilities(rvmti::JvmtiError),
    UnableToAllocateEventCallbackSettings(rvmti::AllocError),
    UnableToSetEventCallbacks(rvmti::JvmtiError),
    UnableToEnableEvents(rvmti::JvmtiError),
    UnableToObtainJvmtiEnvironment(rvmti::JniError),
    MutexError(PoisonError<MutexGuard<'a, Vec<rvmti::JvmtiEnv>>>),
}

impl<'a> fmt::Display for AgentInitError<'a> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AgentInitError::UnableToAllocateCapabilities(ref error) => write!(f, "Unable to allocate capabilities: {}", error),
            AgentInitError::UnableToAddCapabilities(ref error) => write!(f, "Unable to add capabilities: {}", error),
            AgentInitError::UnableToAllocateEventCallbackSettings(ref error) => write!(f, "Unable to allocate event callback settings: {}", error),
            AgentInitError::UnableToSetEventCallbacks(ref error) => write!(f, "Unable to set event callbacks: {}", error),
            AgentInitError::UnableToEnableEvents(ref error) => write!(f, "Unable to enable events: {}", error),
            AgentInitError::UnableToObtainJvmtiEnvironment(ref error) => write!(f, "Unable to obtain environment: {}", error),
            AgentInitError::MutexError(ref error) => write!(f, "Mutex error: {}", error),
        }
    }

}

impl<'a> error::Error for AgentInitError<'a> {

    fn description(&self) -> &str {
        match *self {
            AgentInitError::UnableToAllocateCapabilities(ref error) => error.description(),
            AgentInitError::UnableToAddCapabilities(ref error) => error.description(),
            AgentInitError::UnableToAllocateEventCallbackSettings(ref error) => error.description(),
            AgentInitError::UnableToSetEventCallbacks(ref error) => error.description(),
            AgentInitError::UnableToEnableEvents(ref error) => error.description(),
            AgentInitError::UnableToObtainJvmtiEnvironment(ref error) => error.description(),
            AgentInitError::MutexError(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            AgentInitError::UnableToAllocateCapabilities(ref error) => Some(error),
            AgentInitError::UnableToAddCapabilities(ref error) => Some(error),
            AgentInitError::UnableToAllocateEventCallbackSettings(ref error) => Some(error),
            AgentInitError::UnableToSetEventCallbacks(ref error) => Some(error),
            AgentInitError::UnableToEnableEvents(ref error) => Some(error),
            AgentInitError::UnableToObtainJvmtiEnvironment(ref error) => Some(error),
            AgentInitError::MutexError(ref error) => Some(error),
        }
    }

}

impl<'a> From<PoisonError<MutexGuard<'a, Vec<rvmti::JvmtiEnv>>>> for AgentInitError<'a> {

    fn from(error: PoisonError<MutexGuard<'a, Vec<rvmti::JvmtiEnv>>>) -> AgentInitError<'a> {
        AgentInitError::MutexError(error)
    }

}
