// Copyright 2018 rel-eng
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//   1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//   2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//   3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#include <jni.h>
#include <jvmti.h>
#include <jvmticmlr.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    unsigned int vm_init_enabled;
    unsigned int vm_death_enabled;
    unsigned int thread_start_enabled;
    unsigned int thread_end_enabled;
    unsigned int class_file_load_hook_enabled;
    unsigned int class_load_enabled;
    unsigned int class_prepare_enabled;
    unsigned int vm_start_enabled;
    unsigned int exception_enabled;
    unsigned int exception_catch_enabled;
    unsigned int single_step_enabled;
    unsigned int frame_pop_enabled;
    unsigned int breakpoint_enabled;
    unsigned int field_access_enabled;
    unsigned int field_modification_enabled;
    unsigned int method_entry_enabled;
    unsigned int method_exit_enabled;
    unsigned int native_method_bind_enabled;
    unsigned int compiled_method_load_enabled;
    unsigned int compiled_method_unload_enabled;
    unsigned int dynamic_code_generated_enabled;
    unsigned int data_dump_request_enabled;
    unsigned int monitor_wait_enabled;
    unsigned int monitor_waited_enabled;
    unsigned int monitor_contended_enter_enabled;
    unsigned int monitor_contended_entered_enabled;
    unsigned int resource_exhausted_enabled;
    unsigned int garbage_collection_start_enabled;
    unsigned int garbage_collection_finish_enabled;
    unsigned int object_free_enabled;
    unsigned int vm_object_alloc_enabled;
} JvmtiEventCallbacksStatus;

jint java_vm_get_env(JavaVM *vm, jvmtiEnv **penv, jint version);

jvmtiError jvmti_env_dispose_environment(jvmtiEnv *env);

jvmtiError jvmti_env_add_capabilities(jvmtiEnv *env, jvmtiCapabilities *capabilities);

jvmtiCapabilities *alloc_empty_jvmti_capabilities();

void free_jvmti_capabilities(jvmtiCapabilities *capabilities);

void set_jvmti_capability_can_tag_objects(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_field_modification_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_field_access_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_bytecodes(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_synthetic_attribute(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_owned_monitor_info(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_current_contended_monitor(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_monitor_info(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_pop_frame(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_redefine_classes(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_signal_thread(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_source_file_name(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_line_numbers(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_source_debug_extension(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_access_local_variables(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_maintain_original_method_order(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_single_step_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_exception_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_frame_pop_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_breakpoint_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_suspend(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_redefine_any_class(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_current_thread_cpu_time(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_thread_cpu_time(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_method_entry_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_method_exit_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_all_class_hook_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_compiled_method_load_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_monitor_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_vm_object_alloc_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_native_method_bind_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_garbage_collection_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_object_free_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_force_early_return(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_owned_monitor_stack_depth_info(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_get_constant_pool(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_set_native_method_prefix(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_retransform_classes(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_retransform_any_class(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_resource_exhaustion_heap_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_resource_exhaustion_threads_events(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_early_vmstart(jvmtiCapabilities *capabilities, unsigned int value);

void set_jvmti_capability_can_generate_early_class_hook_events(jvmtiCapabilities *capabilities, unsigned int value);

unsigned int get_jvmti_capability_can_tag_objects(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_field_modification_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_field_access_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_bytecodes(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_synthetic_attribute(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_owned_monitor_info(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_current_contended_monitor(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_monitor_info(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_pop_frame(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_redefine_classes(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_signal_thread(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_source_file_name(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_line_numbers(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_source_debug_extension(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_access_local_variables(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_maintain_original_method_order(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_single_step_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_exception_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_frame_pop_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_breakpoint_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_suspend(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_redefine_any_class(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_current_thread_cpu_time(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_thread_cpu_time(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_method_entry_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_method_exit_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_all_class_hook_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_compiled_method_load_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_monitor_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_vm_object_alloc_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_native_method_bind_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_garbage_collection_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_object_free_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_force_early_return(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_owned_monitor_stack_depth_info(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_get_constant_pool(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_set_native_method_prefix(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_retransform_classes(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_retransform_any_class(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_resource_exhaustion_heap_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_resource_exhaustion_threads_events(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_early_vmstart(const jvmtiCapabilities *capabilities);

unsigned int get_jvmti_capability_can_generate_early_class_hook_events(const jvmtiCapabilities *capabilities);

JvmtiEventCallbacksStatus *alloc_empty_jvmti_event_callback_status();

void free_jvmti_event_callback_status(JvmtiEventCallbacksStatus *status);

jvmtiError set_jvmti_event_callbacks(jvmtiEnv *env, const JvmtiEventCallbacksStatus *status);

void set_jvmti_event_status_vm_init_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_vm_death_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_thread_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_thread_end_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_class_file_load_hook_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_class_load_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_class_prepare_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_vm_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_exception_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_exception_catch_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_single_step_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_frame_pop_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_breakpoint_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_field_access_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_field_modification_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_method_entry_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_method_exit_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_native_method_bind_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_compiled_method_load_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_compiled_method_unload_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_dynamic_code_generated_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_data_dump_request_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_monitor_wait_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_monitor_waited_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_monitor_contended_enter_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_monitor_contended_entered_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_resource_exhausted_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_garbage_collection_start_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_garbage_collection_finish_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_object_free_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

void set_jvmti_event_status_vm_object_alloc_enabled(JvmtiEventCallbacksStatus *status, unsigned int value);

unsigned int get_jvmti_event_status_vm_init_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_vm_death_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_thread_start_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_thread_end_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_class_file_load_hook_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_class_load_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_class_prepare_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_vm_start_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_exception_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_exception_catch_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_single_step_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_frame_pop_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_breakpoint_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_field_access_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_field_modification_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_method_entry_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_method_exit_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_native_method_bind_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_compiled_method_load_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_compiled_method_unload_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_dynamic_code_generated_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_data_dump_request_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_monitor_wait_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_monitor_waited_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_monitor_contended_enter_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_monitor_contended_entered_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_resource_exhausted_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_garbage_collection_start_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_garbage_collection_finish_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_object_free_enabled(const JvmtiEventCallbacksStatus *status);

unsigned int get_jvmti_event_status_vm_object_alloc_enabled(const JvmtiEventCallbacksStatus *status);

extern void jvmti_event_breakpoint_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method, jlocation location);

extern void jvmti_event_class_file_load_hook_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jclass class_being_redefined,
    jobject loader, const char* name, jobject protection_domain, jint class_data_len, const unsigned char* class_data,
    jint* new_class_data_len, unsigned char** new_class_data);

extern void jvmti_event_class_load_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jclass klass);

extern void jvmti_event_class_prepare_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jclass klass);

extern void jvmti_event_compiled_method_load_handler(jvmtiEnv *jvmti_env, jmethodID method, jint code_size,
    const void* code_addr, jint map_length, const jvmtiAddrLocationMap* map, const void* compile_info);

extern void jvmti_event_compiled_method_unload_handler(jvmtiEnv *jvmti_env, jmethodID method, const void* code_addr);

extern void jvmti_event_data_dump_request_handler(jvmtiEnv *jvmti_env);

extern void jvmti_event_dynamic_code_generated_handler(jvmtiEnv *jvmti_env, const char* name, const void* address, jint length);

extern void jvmti_event_exception_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jobject exception, jmethodID catch_method, jlocation catch_location);

extern void jvmti_event_exception_catch_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jobject exception);

extern void jvmti_event_field_access_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jclass field_klass, jobject object, jfieldID field);

extern void jvmti_event_field_modification_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location, jclass field_klass, jobject object, jfieldID field, char signature_type, jvalue new_value);

extern void jvmti_event_frame_pop_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jboolean was_popped_by_exception);

extern void jvmti_event_garbage_collection_finish_handler(jvmtiEnv *jvmti_env);

extern void jvmti_event_garbage_collection_start_handler(jvmtiEnv *jvmti_env);

extern void jvmti_event_method_entry_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method);

extern void jvmti_event_method_exit_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jboolean was_popped_by_exception, jvalue return_value);

extern void jvmti_event_monitor_contended_enter_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object);

extern void jvmti_event_monitor_contended_entered_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object);

extern void jvmti_event_monitor_wait_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object, jlong timeout);

extern void jvmti_event_monitor_waited_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object,
    jboolean timed_out);

extern void jvmti_event_native_method_bind_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    void* address, void** new_address_ptr);

extern void jvmti_event_object_free_handler(jvmtiEnv *jvmti_env, jlong tag);

extern void jvmti_event_resource_exhausted_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jint flags, const void* reserved,
    const char* description);

extern void jvmti_event_single_step_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jmethodID method,
    jlocation location);

extern void jvmti_event_thread_end_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread);

extern void jvmti_event_thread_start_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread);

extern void jvmti_event_vm_death_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env);

extern void jvmti_event_vm_init_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread);

extern void jvmti_event_vm_object_alloc_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env, jthread thread, jobject object,
    jclass object_klass, jlong size);

extern void jvmti_event_vm_start_handler(jvmtiEnv *jvmti_env, JNIEnv* jni_env);

jvmtiError jvmti_env_set_event_notification_mode(jvmtiEnv *env, jvmtiEventMode mode, jvmtiEvent event_type, jthread event_thread);

jvmtiError jvmti_env_get_method_name(jvmtiEnv* env, jmethodID method, char** name_ptr, char** signature_ptr, char** generic_ptr);

jvmtiError jvmti_env_deallocate(jvmtiEnv* env, void* mem);

jvmtiError jvmti_env_get_method_declaring_class(jvmtiEnv* env, jmethodID method, jclass* declaring_class_ptr);

jvmtiError jvmti_env_get_class_signature(jvmtiEnv* env, jclass klass, char** signature_ptr, char** generic_ptr);

jvmtiError jvmti_env_get_source_file_name(jvmtiEnv* env, jclass klass, char** source_name_ptr);

jvmtiError jvmti_env_get_line_number_table(jvmtiEnv* env, jmethodID method, jint* entry_count_ptr, jvmtiLineNumberEntry** table_ptr);

jvmtiError jvmti_env_is_method_native(jvmtiEnv* env, jmethodID method, jboolean* is_native_ptr);

#ifdef __cplusplus
}
#endif