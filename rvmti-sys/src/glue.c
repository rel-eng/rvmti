// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#include <string.h>
#include <stdlib.h>

#include "glue.h"

#ifdef __cplusplus
extern "C" {
#endif

jint java_vm_get_env(JavaVM *vm, jvmtiEnv **penv, jint version) {
    return (*vm)->GetEnv(vm, (void **) penv, version);
}

jvmtiError jvmti_env_dispose_environment(jvmtiEnv *env) {
    return (*env)->DisposeEnvironment(env);
}

jvmtiError jvmti_env_add_capabilities(jvmtiEnv *env, jvmtiCapabilities *capabilities) {
    return (*env)->AddCapabilities(env, capabilities);
}

jvmtiCapabilities *alloc_empty_jvmti_capabilities() {
    jvmtiCapabilities *capabilities = (jvmtiCapabilities *) malloc(sizeof(jvmtiCapabilities));
    if (capabilities != NULL) {
        memset(capabilities, 0, sizeof(jvmtiCapabilities));
    }
    return capabilities;
}

void free_jvmti_capabilities(jvmtiCapabilities *capabilities) {
    free(capabilities);
}

void set_jvmti_capability_can_tag_objects(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_tag_objects = value;
}

void set_jvmti_capability_can_generate_field_modification_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_field_modification_events = value;
}

void set_jvmti_capability_can_generate_field_access_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_field_access_events = value;
}

void set_jvmti_capability_can_get_bytecodes(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_bytecodes = value;
}

void set_jvmti_capability_can_get_synthetic_attribute(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_synthetic_attribute = value;
}

void set_jvmti_capability_can_get_owned_monitor_info(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_owned_monitor_info = value;
}

void set_jvmti_capability_can_get_current_contended_monitor(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_current_contended_monitor = value;
}

void set_jvmti_capability_can_get_monitor_info(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_monitor_info = value;
}

void set_jvmti_capability_can_pop_frame(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_pop_frame = value;
}

void set_jvmti_capability_can_redefine_classes(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_redefine_classes = value;
}

void set_jvmti_capability_can_signal_thread(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_signal_thread = value;
}

void set_jvmti_capability_can_get_source_file_name(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_source_file_name = value;
}

void set_jvmti_capability_can_get_line_numbers(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_line_numbers = value;
}

void set_jvmti_capability_can_get_source_debug_extension(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_source_debug_extension = value;
}

void set_jvmti_capability_can_access_local_variables(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_access_local_variables = value;
}

void set_jvmti_capability_can_maintain_original_method_order(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_maintain_original_method_order = value;
}

void set_jvmti_capability_can_generate_single_step_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_single_step_events = value;
}

void set_jvmti_capability_can_generate_exception_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_exception_events = value;
}

void set_jvmti_capability_can_generate_frame_pop_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_frame_pop_events = value;
}

void set_jvmti_capability_can_generate_breakpoint_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_breakpoint_events = value;
}

void set_jvmti_capability_can_suspend(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_suspend = value;
}

void set_jvmti_capability_can_redefine_any_class(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_redefine_any_class = value;
}

void set_jvmti_capability_can_get_current_thread_cpu_time(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_current_thread_cpu_time = value;
}

void set_jvmti_capability_can_get_thread_cpu_time(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_thread_cpu_time = value;
}

void set_jvmti_capability_can_generate_method_entry_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_method_entry_events = value;
}

void set_jvmti_capability_can_generate_method_exit_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_method_exit_events = value;
}

void set_jvmti_capability_can_generate_all_class_hook_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_all_class_hook_events = value;
}

void set_jvmti_capability_can_generate_compiled_method_load_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_compiled_method_load_events = value;
}

void set_jvmti_capability_can_generate_monitor_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_monitor_events = value;
}

void set_jvmti_capability_can_generate_vm_object_alloc_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_vm_object_alloc_events = value;
}

void set_jvmti_capability_can_generate_native_method_bind_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_native_method_bind_events = value;
}

void set_jvmti_capability_can_generate_garbage_collection_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_garbage_collection_events = value;
}

void set_jvmti_capability_can_generate_object_free_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_object_free_events = value;
}

void set_jvmti_capability_can_force_early_return(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_force_early_return = value;
}

void set_jvmti_capability_can_get_owned_monitor_stack_depth_info(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_owned_monitor_stack_depth_info = value;
}

void set_jvmti_capability_can_get_constant_pool(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_get_constant_pool = value;
}

void set_jvmti_capability_can_set_native_method_prefix(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_set_native_method_prefix = value;
}

void set_jvmti_capability_can_retransform_classes(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_retransform_classes = value;
}

void set_jvmti_capability_can_retransform_any_class(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_retransform_any_class = value;
}

void set_jvmti_capability_can_generate_resource_exhaustion_heap_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_resource_exhaustion_heap_events = value;
}

void set_jvmti_capability_can_generate_resource_exhaustion_threads_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_resource_exhaustion_threads_events = value;
}

void set_jvmti_capability_can_generate_early_vmstart(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_early_vmstart = value;
}

void set_jvmti_capability_can_generate_early_class_hook_events(jvmtiCapabilities *capabilities, unsigned int value) {
    capabilities->can_generate_early_class_hook_events = value;
}

unsigned int get_jvmti_capability_can_tag_objects(const jvmtiCapabilities *capabilities) {
    return capabilities->can_tag_objects;
}

unsigned int get_jvmti_capability_can_generate_field_modification_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_field_modification_events;
}

unsigned int get_jvmti_capability_can_generate_field_access_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_field_access_events;
}

unsigned int get_jvmti_capability_can_get_bytecodes(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_bytecodes;
}

unsigned int get_jvmti_capability_can_get_synthetic_attribute(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_synthetic_attribute;
}

unsigned int get_jvmti_capability_can_get_owned_monitor_info(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_owned_monitor_info;
}

unsigned int get_jvmti_capability_can_get_current_contended_monitor(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_current_contended_monitor;
}

unsigned int get_jvmti_capability_can_get_monitor_info(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_monitor_info;
}

unsigned int get_jvmti_capability_can_pop_frame(const jvmtiCapabilities *capabilities) {
    return capabilities->can_pop_frame;
}

unsigned int get_jvmti_capability_can_redefine_classes(const jvmtiCapabilities *capabilities) {
    return capabilities->can_redefine_classes;
}

unsigned int get_jvmti_capability_can_signal_thread(const jvmtiCapabilities *capabilities) {
    return capabilities->can_signal_thread;
}

unsigned int get_jvmti_capability_can_get_source_file_name(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_source_file_name;
}

unsigned int get_jvmti_capability_can_get_line_numbers(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_line_numbers;
}

unsigned int get_jvmti_capability_can_get_source_debug_extension(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_source_debug_extension;
}

unsigned int get_jvmti_capability_can_access_local_variables(const jvmtiCapabilities *capabilities) {
    return capabilities->can_access_local_variables;
}

unsigned int get_jvmti_capability_can_maintain_original_method_order(const jvmtiCapabilities *capabilities) {
    return capabilities->can_maintain_original_method_order;
}

unsigned int get_jvmti_capability_can_generate_single_step_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_single_step_events;
}

unsigned int get_jvmti_capability_can_generate_exception_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_exception_events;
}

unsigned int get_jvmti_capability_can_generate_frame_pop_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_frame_pop_events;
}

unsigned int get_jvmti_capability_can_generate_breakpoint_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_breakpoint_events;
}

unsigned int get_jvmti_capability_can_suspend(const jvmtiCapabilities *capabilities) {
    return capabilities->can_suspend;
}

unsigned int get_jvmti_capability_can_redefine_any_class(const jvmtiCapabilities *capabilities) {
    return capabilities->can_redefine_any_class;
}

unsigned int get_jvmti_capability_can_get_current_thread_cpu_time(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_current_thread_cpu_time;
}

unsigned int get_jvmti_capability_can_get_thread_cpu_time(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_thread_cpu_time;
}

unsigned int get_jvmti_capability_can_generate_method_entry_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_method_entry_events;
}

unsigned int get_jvmti_capability_can_generate_method_exit_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_method_exit_events;
}

unsigned int get_jvmti_capability_can_generate_all_class_hook_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_all_class_hook_events;
}

unsigned int get_jvmti_capability_can_generate_compiled_method_load_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_compiled_method_load_events;
}

unsigned int get_jvmti_capability_can_generate_monitor_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_monitor_events;
}

unsigned int get_jvmti_capability_can_generate_vm_object_alloc_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_vm_object_alloc_events;
}

unsigned int get_jvmti_capability_can_generate_native_method_bind_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_native_method_bind_events;
}

unsigned int get_jvmti_capability_can_generate_garbage_collection_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_garbage_collection_events;
}

unsigned int get_jvmti_capability_can_generate_object_free_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_object_free_events;
}

unsigned int get_jvmti_capability_can_force_early_return(const jvmtiCapabilities *capabilities) {
    return capabilities->can_force_early_return;
}

unsigned int get_jvmti_capability_can_get_owned_monitor_stack_depth_info(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_owned_monitor_stack_depth_info;
}

unsigned int get_jvmti_capability_can_get_constant_pool(const jvmtiCapabilities *capabilities) {
    return capabilities->can_get_constant_pool;
}

unsigned int get_jvmti_capability_can_set_native_method_prefix(const jvmtiCapabilities *capabilities) {
    return capabilities->can_set_native_method_prefix;
}

unsigned int get_jvmti_capability_can_retransform_classes(const jvmtiCapabilities *capabilities) {
    return capabilities->can_retransform_classes;
}

unsigned int get_jvmti_capability_can_retransform_any_class(const jvmtiCapabilities *capabilities) {
    return capabilities->can_retransform_any_class;
}

unsigned int get_jvmti_capability_can_generate_resource_exhaustion_heap_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_resource_exhaustion_heap_events;
}

unsigned int get_jvmti_capability_can_generate_resource_exhaustion_threads_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_resource_exhaustion_threads_events;
}

unsigned int get_jvmti_capability_can_generate_early_vmstart(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_early_vmstart;
}

unsigned int get_jvmti_capability_can_generate_early_class_hook_events(const jvmtiCapabilities *capabilities) {
    return capabilities->can_generate_early_class_hook_events;
}

extern void JNICALL on_jvmti_event_breakpoint(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location)
{
    jvmti_event_breakpoint_handler(jvmti_env, jni_env, thread, method, location);
}

extern void JNICALL on_jvmti_event_class_file_load_hook(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jclass class_being_redefined,
    jobject loader, const char* name, jobject protection_domain, jint class_data_len, const unsigned char* class_data,
    jint* new_class_data_len, unsigned char** new_class_data)
{
    jvmti_event_class_file_load_hook_handler(jvmti_env, jni_env, class_being_redefined,loader, name, protection_domain,
        class_data_len, class_data, new_class_data_len, new_class_data);
}

extern void JNICALL on_jvmti_event_class_load(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jclass klass) {
    jvmti_event_class_load_handler(jvmti_env, jni_env, thread, klass);
}

extern void JNICALL on_jvmti_event_class_prepare(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jclass klass) {
    jvmti_event_class_prepare_handler(jvmti_env, jni_env, thread, klass);
}

extern void JNICALL on_jvmti_event_compiled_method_load(jvmtiEnv *jvmti_env, jmethodID method, jint code_size,
    const void* code_addr, jint map_length, const jvmtiAddrLocationMap* map, const void* compile_info)
{
    jvmti_event_compiled_method_load_handler(jvmti_env, method, code_size, code_addr, map_length, map, compile_info);
}

extern void JNICALL on_jvmti_event_compiled_method_unload(jvmtiEnv *jvmti_env, jmethodID method, const void* code_addr) {
    jvmti_event_compiled_method_unload_handler(jvmti_env, method, code_addr);
}

extern void JNICALL on_jvmti_event_data_dump_request(jvmtiEnv *jvmti_env) {
    jvmti_event_data_dump_request_handler(jvmti_env);
}

extern void JNICALL on_jvmti_event_dynamic_code_generated(jvmtiEnv *jvmti_env, const char* name, const void* address, jint length) {
    jvmti_event_dynamic_code_generated_handler(jvmti_env, name, address, length);
}

extern void JNICALL on_jvmti_event_exception(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jobject exception, jmethodID catch_method, jlocation catch_location)
{
    jvmti_event_exception_handler(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
}

extern void JNICALL on_jvmti_event_exception_catch(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jobject exception)
{
    jvmti_event_exception_catch_handler(jvmti_env, jni_env, thread, method, location, exception);
}

extern void JNICALL on_jvmti_event_field_access(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jclass field_klass, jobject object, jfieldID field)
{
    jvmti_event_field_access_handler(jvmti_env, jni_env, thread, method, location, field_klass, object, field);
}

extern void JNICALL on_jvmti_event_field_modification(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jclass field_klass, jobject object, jfieldID field, char signature_type, jvalue new_value)
{
    jvmti_event_field_modification_handler(jvmti_env, jni_env, thread, method, location, field_klass, object, field,
        signature_type, new_value);
}

extern void JNICALL on_jvmti_event_frame_pop(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jboolean was_popped_by_exception)
{
    jvmti_event_frame_pop_handler(jvmti_env, jni_env, thread, method, was_popped_by_exception);
}

extern void JNICALL on_jvmti_event_garbage_collection_finish(jvmtiEnv *jvmti_env) {
    jvmti_event_garbage_collection_finish_handler(jvmti_env);
}

extern void JNICALL on_jvmti_event_garbage_collection_start(jvmtiEnv *jvmti_env) {
    jvmti_event_garbage_collection_start_handler(jvmti_env);
}

extern void JNICALL on_jvmti_event_method_entry(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method) {
    jvmti_event_method_entry_handler(jvmti_env, jni_env, thread, method);
}

extern void JNICALL on_jvmti_event_method_exit(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jboolean was_popped_by_exception, jvalue return_value)
{
    jvmti_event_method_exit_handler(jvmti_env, jni_env, thread, method, was_popped_by_exception, return_value);
}

extern void JNICALL on_jvmti_event_monitor_contended_enter(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object) {
    jvmti_event_monitor_contended_enter_handler(jvmti_env, jni_env, thread, object);
}

extern void JNICALL on_jvmti_event_monitor_contended_entered(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object) {
    jvmti_event_monitor_contended_entered_handler(jvmti_env, jni_env, thread, object);
}

extern void JNICALL on_jvmti_event_monitor_wait(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object, jlong timeout) {
    jvmti_event_monitor_wait_handler(jvmti_env, jni_env, thread, object, timeout);
}

extern void JNICALL on_jvmti_event_monitor_waited(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object,
    jboolean timed_out)
{
    jvmti_event_monitor_waited_handler(jvmti_env, jni_env, thread, object, timed_out);
}

extern void JNICALL on_jvmti_event_native_method_bind(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    void* address, void** new_address_ptr)
{
    jvmti_event_native_method_bind_handler(jvmti_env, jni_env, thread, method, address, new_address_ptr);
}

extern void JNICALL on_jvmti_event_object_free(jvmtiEnv *jvmti_env, jlong tag) {
    jvmti_event_object_free_handler(jvmti_env, tag);
}

extern void JNICALL on_jvmti_event_resource_exhausted(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jint flags, const void* reserved,
    const char* description)
{
    jvmti_event_resource_exhausted_handler(jvmti_env, jni_env, flags, reserved, description);
}

extern void JNICALL on_jvmti_event_single_step(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location)
{
    jvmti_event_single_step_handler(jvmti_env, jni_env, thread, method, location);
}

extern void JNICALL on_jvmti_event_thread_end(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread) {
    jvmti_event_thread_end_handler(jvmti_env, jni_env, thread);
}

extern void JNICALL on_jvmti_event_thread_start(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread) {
    jvmti_event_thread_start_handler(jvmti_env, jni_env, thread);
}

extern void JNICALL on_jvmti_event_vm_death(jvmtiEnv *jvmti_env, JNIEnv* jni_env) {
    jvmti_event_vm_death_handler(jvmti_env, jni_env);
}

extern void JNICALL on_jvmti_event_vm_init(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread) {
    jvmti_event_vm_init_handler(jvmti_env, jni_env, thread);
}

extern void JNICALL on_jvmti_event_vm_object_alloc(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object,
    jclass object_klass, jlong size)
{
    jvmti_event_vm_object_alloc_handler(jvmti_env, jni_env, thread, object, object_klass, size);
}

extern void JNICALL on_jvmti_event_vm_start(jvmtiEnv *jvmti_env, JNIEnv* jni_env) {
    jvmti_event_vm_start_handler(jvmti_env, jni_env);
}

JvmtiEventCallbacksStatus *alloc_empty_jvmti_event_callback_status() {
    JvmtiEventCallbacksStatus *status = (JvmtiEventCallbacksStatus *) malloc(sizeof(JvmtiEventCallbacksStatus));
    if (status != NULL) {
        memset(status, 0, sizeof(JvmtiEventCallbacksStatus));
    }
    return status;
}

void free_jvmti_event_callback_status(JvmtiEventCallbacksStatus *status) {
    free(status);
}

jvmtiError set_jvmti_event_callbacks(jvmtiEnv *env, const JvmtiEventCallbacksStatus *status) {
    jvmtiEventCallbacks callbacks;
    memset(&callbacks, 0, sizeof(callbacks));
    if (status->vm_init_enabled != 0) {
        callbacks.VMInit = &on_jvmti_event_vm_init;
    }
    if (status->vm_death_enabled != 0) {
        callbacks.VMDeath = &on_jvmti_event_vm_death;
    }
    if (status->thread_start_enabled != 0) {
        callbacks.ThreadStart = &on_jvmti_event_thread_start;
    }
    if (status->thread_end_enabled != 0) {
        callbacks.ThreadEnd = &on_jvmti_event_thread_end;
    }
    if (status->class_file_load_hook_enabled != 0) {
        callbacks.ClassFileLoadHook = &on_jvmti_event_class_file_load_hook;
    }
    if (status->class_load_enabled != 0) {
        callbacks.ClassLoad = &on_jvmti_event_class_load;
    }
    if (status->class_prepare_enabled != 0) {
        callbacks.ClassPrepare = &on_jvmti_event_class_prepare;
    }
    if (status->vm_start_enabled != 0) {
        callbacks.VMStart = &on_jvmti_event_vm_start;
    }
    if (status->exception_enabled != 0) {
        callbacks.Exception = &on_jvmti_event_exception;
    }
    if (status->exception_catch_enabled != 0) {
        callbacks.ExceptionCatch = &on_jvmti_event_exception_catch;
    }
    if (status->single_step_enabled != 0) {
        callbacks.SingleStep = &on_jvmti_event_single_step;
    }
    if (status->frame_pop_enabled != 0) {
        callbacks.FramePop = &on_jvmti_event_frame_pop;
    }
    if (status->breakpoint_enabled != 0) {
        callbacks.Breakpoint = &on_jvmti_event_breakpoint;
    }
    if (status->field_access_enabled != 0) {
        callbacks.FieldAccess = &on_jvmti_event_field_access;
    }
    if (status->field_modification_enabled != 0) {
        callbacks.FieldModification = &on_jvmti_event_field_modification;
    }
    if (status->method_entry_enabled != 0) {
        callbacks.MethodEntry = &on_jvmti_event_method_entry;
    }
    if (status->method_exit_enabled != 0) {
        callbacks.MethodExit = &on_jvmti_event_method_exit;
    }
    if (status->native_method_bind_enabled != 0) {
        callbacks.NativeMethodBind = &on_jvmti_event_native_method_bind;
    }
    if (status->compiled_method_load_enabled != 0) {
        callbacks.CompiledMethodLoad = &on_jvmti_event_compiled_method_load;
    }
    if (status->compiled_method_unload_enabled != 0) {
        callbacks.CompiledMethodUnload = &on_jvmti_event_compiled_method_unload;
    }
    if (status->dynamic_code_generated_enabled != 0) {
        callbacks.DynamicCodeGenerated = &on_jvmti_event_dynamic_code_generated;
    }
    if (status->data_dump_request_enabled != 0) {
        callbacks.DataDumpRequest = &on_jvmti_event_data_dump_request;
    }
    if (status->monitor_wait_enabled != 0) {
        callbacks.MonitorWait = &on_jvmti_event_monitor_wait;
    }
    if (status->monitor_waited_enabled != 0) {
        callbacks.MonitorWaited = &on_jvmti_event_monitor_waited;
    }
    if (status->monitor_contended_enter_enabled != 0) {
        callbacks.MonitorContendedEnter = &on_jvmti_event_monitor_contended_enter;
    }
    if (status->monitor_contended_entered_enabled != 0) {
        callbacks.MonitorContendedEntered = &on_jvmti_event_monitor_contended_entered;
    }
    if (status->resource_exhausted_enabled != 0) {
        callbacks.ResourceExhausted = &on_jvmti_event_resource_exhausted;
    }
    if (status->garbage_collection_start_enabled != 0) {
        callbacks.GarbageCollectionStart = &on_jvmti_event_garbage_collection_start;
    }
    if (status->garbage_collection_finish_enabled != 0) {
        callbacks.GarbageCollectionFinish = &on_jvmti_event_garbage_collection_finish;
    }
    if (status->object_free_enabled != 0) {
        callbacks.ObjectFree = &on_jvmti_event_object_free;
    }
    if (status->vm_object_alloc_enabled != 0) {
        callbacks.VMObjectAlloc = &on_jvmti_event_vm_object_alloc;
    }
    return (*env)->SetEventCallbacks(env, &callbacks, (jint) sizeof(callbacks));
}

void set_jvmti_event_status_vm_init_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->vm_init_enabled = value;
}

void set_jvmti_event_status_vm_death_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->vm_death_enabled = value;
}

void set_jvmti_event_status_thread_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->thread_start_enabled = value;
}

void set_jvmti_event_status_thread_end_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->thread_end_enabled = value;
}

void set_jvmti_event_status_class_file_load_hook_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->class_file_load_hook_enabled = value;
}

void set_jvmti_event_status_class_load_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->class_load_enabled = value;
}

void set_jvmti_event_status_class_prepare_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->class_prepare_enabled = value;
}

void set_jvmti_event_status_vm_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->vm_start_enabled = value;
}

void set_jvmti_event_status_exception_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->exception_enabled = value;
}

void set_jvmti_event_status_exception_catch_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->exception_catch_enabled = value;
}

void set_jvmti_event_status_single_step_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->single_step_enabled = value;
}

void set_jvmti_event_status_frame_pop_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->frame_pop_enabled = value;
}

void set_jvmti_event_status_breakpoint_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->breakpoint_enabled = value;
}

void set_jvmti_event_status_field_access_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->field_access_enabled = value;
}

void set_jvmti_event_status_field_modification_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->field_modification_enabled = value;
}

void set_jvmti_event_status_method_entry_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->method_entry_enabled = value;
}

void set_jvmti_event_status_method_exit_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->method_exit_enabled = value;
}

void set_jvmti_event_status_native_method_bind_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->native_method_bind_enabled = value;
}

void set_jvmti_event_status_compiled_method_load_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->compiled_method_load_enabled = value;
}

void set_jvmti_event_status_compiled_method_unload_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->compiled_method_unload_enabled = value;
}

void set_jvmti_event_status_dynamic_code_generated_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->dynamic_code_generated_enabled = value;
}

void set_jvmti_event_status_data_dump_request_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->data_dump_request_enabled = value;
}

void set_jvmti_event_status_monitor_wait_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->monitor_wait_enabled = value;
}

void set_jvmti_event_status_monitor_waited_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->monitor_waited_enabled = value;
}

void set_jvmti_event_status_monitor_contended_enter_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->monitor_contended_enter_enabled = value;
}

void set_jvmti_event_status_monitor_contended_entered_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->monitor_contended_entered_enabled = value;
}

void set_jvmti_event_status_resource_exhausted_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->resource_exhausted_enabled = value;
}

void set_jvmti_event_status_garbage_collection_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->garbage_collection_start_enabled = value;
}

void set_jvmti_event_status_garbage_collection_finish_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->garbage_collection_finish_enabled = value;
}

void set_jvmti_event_status_object_free_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->object_free_enabled = value;
}

void set_jvmti_event_status_vm_object_alloc_enabled(JvmtiEventCallbacksStatus *status, unsigned int value) {
    status->vm_object_alloc_enabled = value;
}

unsigned int get_jvmti_event_status_vm_init_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->vm_init_enabled;
}

unsigned int get_jvmti_event_status_vm_death_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->vm_death_enabled;
}

unsigned int get_jvmti_event_status_thread_start_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->thread_start_enabled;
}

unsigned int get_jvmti_event_status_thread_end_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->thread_end_enabled;
}

unsigned int get_jvmti_event_status_class_file_load_hook_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->class_file_load_hook_enabled;
}

unsigned int get_jvmti_event_status_class_load_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->class_load_enabled;
}

unsigned int get_jvmti_event_status_class_prepare_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->class_prepare_enabled;
}

unsigned int get_jvmti_event_status_vm_start_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->vm_start_enabled;
}

unsigned int get_jvmti_event_status_exception_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->exception_enabled;
}

unsigned int get_jvmti_event_status_exception_catch_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->exception_catch_enabled;
}

unsigned int get_jvmti_event_status_single_step_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->single_step_enabled;
}

unsigned int get_jvmti_event_status_frame_pop_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->frame_pop_enabled;
}

unsigned int get_jvmti_event_status_breakpoint_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->breakpoint_enabled;
}

unsigned int get_jvmti_event_status_field_access_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->field_access_enabled;
}

unsigned int get_jvmti_event_status_field_modification_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->field_modification_enabled;
}

unsigned int get_jvmti_event_status_method_entry_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->method_entry_enabled;
}

unsigned int get_jvmti_event_status_method_exit_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->method_exit_enabled;
}

unsigned int get_jvmti_event_status_native_method_bind_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->native_method_bind_enabled;
}

unsigned int get_jvmti_event_status_compiled_method_load_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->compiled_method_load_enabled;
}

unsigned int get_jvmti_event_status_compiled_method_unload_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->compiled_method_unload_enabled;
}

unsigned int get_jvmti_event_status_dynamic_code_generated_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->dynamic_code_generated_enabled;
}

unsigned int get_jvmti_event_status_data_dump_request_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->data_dump_request_enabled;
}

unsigned int get_jvmti_event_status_monitor_wait_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->monitor_wait_enabled;
}

unsigned int get_jvmti_event_status_monitor_waited_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->monitor_waited_enabled;
}

unsigned int get_jvmti_event_status_monitor_contended_enter_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->monitor_contended_enter_enabled;
}

unsigned int get_jvmti_event_status_monitor_contended_entered_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->monitor_contended_entered_enabled;
}

unsigned int get_jvmti_event_status_resource_exhausted_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->resource_exhausted_enabled;
}

unsigned int get_jvmti_event_status_garbage_collection_start_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->garbage_collection_start_enabled;
}

unsigned int get_jvmti_event_status_garbage_collection_finish_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->garbage_collection_finish_enabled;
}

unsigned int get_jvmti_event_status_object_free_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->object_free_enabled;
}

unsigned int get_jvmti_event_status_vm_object_alloc_enabled(const JvmtiEventCallbacksStatus *status) {
    return status->vm_object_alloc_enabled;
}

jvmtiError jvmti_env_set_event_notification_mode(jvmtiEnv *env, jvmtiEventMode mode, jvmtiEvent event_type, jthread event_thread) {
    return (*env)->SetEventNotificationMode(env, mode, event_type, event_thread);
}

jvmtiError jvmti_env_get_method_name(jvmtiEnv* env, jmethodID method, char** name_ptr, char** signature_ptr, char** generic_ptr) {
    return (*env)->GetMethodName(env, method, name_ptr, signature_ptr, generic_ptr);
}

jvmtiError jvmti_env_deallocate(jvmtiEnv* env, void* mem) {
    return (*env)->Deallocate(env, (unsigned char *) mem);
}

jvmtiError jvmti_env_get_method_declaring_class(jvmtiEnv* env, jmethodID method, jclass* declaring_class_ptr) {
    return (*env)->GetMethodDeclaringClass(env, method, declaring_class_ptr);
}

jvmtiError jvmti_env_get_class_signature(jvmtiEnv* env, jclass klass, char** signature_ptr, char** generic_ptr) {
    return (*env)->GetClassSignature(env, klass, signature_ptr, generic_ptr);
}

jvmtiError jvmti_env_get_source_file_name(jvmtiEnv* env, jclass klass, char** source_name_ptr) {
    return (*env)->GetSourceFileName(env, klass, source_name_ptr);
}

jvmtiError jvmti_env_get_line_number_table(jvmtiEnv* env, jmethodID method, jint* entry_count_ptr, jvmtiLineNumberEntry** table_ptr) {
    return (*env)->GetLineNumberTable(env, method, entry_count_ptr, table_ptr);
}

#ifdef __cplusplus
}
#endif
