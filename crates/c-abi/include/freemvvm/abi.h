#ifndef FREEMVVM_ABI_H_
#define FREEMVVM_ABI_H_

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t ui_handle;
typedef uint32_t ui_status;
typedef uint8_t ui_bool;

typedef void (*ui_callback)(void *userdata);
typedef void (*ui_destroy_callback)(void *userdata);
typedef void (*ui_wake_callback)(void *userdata, uint32_t priority);
typedef ui_status (*ui_validate_bool_callback)(void *userdata, ui_bool candidate,
                                               ui_handle *out_error);
typedef ui_status (*ui_validate_i64_callback)(void *userdata, int64_t candidate,
                                              ui_handle *out_error);
typedef ui_status (*ui_validate_utf8_callback)(void *userdata,
                                               const uint8_t *candidate,
                                               uint64_t candidate_byte_length,
                                               ui_handle *out_error);
typedef ui_status (*ui_command_start_callback)(void *userdata,
                                               ui_handle borrowed_command,
                                               uint64_t operation_id,
                                               ui_handle borrowed_cancel_token);

enum {
  UI_OK = 0,
  UI_INVALID_ARGUMENT = 1,
  UI_INVALID_HANDLE = 2,
  UI_WRONG_TYPE = 3,
  UI_WRONG_RUNTIME = 4,
  UI_WRONG_THREAD = 5,
  UI_CLOSED = 6,
  UI_BUFFER_TOO_SMALL = 7,
  UI_OUT_OF_MEMORY = 8,
  UI_INTERNAL_ERROR = 9,
  UI_UNSUPPORTED = 10,
  UI_VALIDATION_FAILED = 11,
  UI_COMMAND_EXECUTION_FAILED = 12,
  UI_PENDING = 14
};

enum {
  UI_COMMAND_STATE_IDLE = 0,
  UI_COMMAND_STATE_RUNNING = 1,
  UI_COMMAND_STATE_CANCELLING = 2,
  UI_COMMAND_STATE_SUCCEEDED = 3,
  UI_COMMAND_STATE_FAILED = 4,
  UI_COMMAND_STATE_CANCELLED = 5
};

enum {
  UI_NUMERIC_NARROW_EXACT = 0,
  UI_NUMERIC_NARROW_NEAREST_EVEN = 1
};

enum {
  UI_SHUTDOWN_ASYNC = 0,
  UI_SHUTDOWN_SYNC = 1,
  UI_SHUTDOWN_REQUEST_PENDING = 0,
  UI_SHUTDOWN_REQUEST_COMPLETED = 1,
  UI_SHUTDOWN_REQUEST_HOST_UNAVAILABLE = 2
};

ui_status ui_abi_1_runtime_create(ui_handle *out_runtime, ui_handle *out_error);
ui_status ui_abi_1_runtime_is_ui_thread(ui_handle runtime, ui_bool *out_yes);

ui_status ui_abi_1_property_create_bool(ui_handle runtime, ui_bool initial,
                                        ui_handle *out_property,
                                        ui_handle *out_error);
ui_status ui_abi_1_property_create_i64(ui_handle runtime, int64_t initial,
                                       ui_handle *out_property,
                                       ui_handle *out_error);
ui_status ui_abi_1_property_create_utf8(ui_handle runtime, const uint8_t *bytes,
                                        uint64_t byte_length,
                                        ui_handle *out_property,
                                        ui_handle *out_error);
ui_status ui_abi_1_property_create_f64(ui_handle runtime, double initial,
                                       ui_handle *out_property,
                                       ui_handle *out_error);
ui_status ui_abi_1_property_create_decimal(ui_handle runtime,
                                           uint64_t initial_bits,
                                           ui_handle *out_property,
                                           ui_handle *out_error);
ui_status ui_abi_1_property_kind(ui_handle property, uint32_t *out_kind);
ui_status ui_abi_1_property_get_bool(ui_handle property, ui_bool *out_value);
ui_status ui_abi_1_property_get_i64(ui_handle property, int64_t *out_value);
ui_status ui_abi_1_property_get_utf8(ui_handle property, uint8_t *buffer,
                                     uint64_t capacity, uint64_t *out_length);
ui_status ui_abi_1_property_get_f64(ui_handle property, double *out_value);
ui_status ui_abi_1_property_get_decimal(ui_handle property,
                                        uint64_t *out_bits);
ui_status ui_abi_1_property_set_bool(ui_handle property, ui_bool value,
                                     ui_handle *out_error);
ui_status ui_abi_1_property_set_i64(ui_handle property, int64_t value,
                                    ui_handle *out_error);
ui_status ui_abi_1_property_set_utf8(ui_handle property, const uint8_t *bytes,
                                     uint64_t byte_length,
                                     ui_handle *out_error);
ui_status ui_abi_1_property_set_f64(ui_handle property, double value,
                                    ui_handle *out_error);
ui_status ui_abi_1_property_set_decimal(ui_handle property, uint64_t bits,
                                        ui_handle *out_error);

ui_status ui_abi_1_validation_error_create(uint32_t domain, uint32_t code,
                                           const uint8_t *message,
                                           uint64_t message_byte_length,
                                           ui_handle *out_error);
ui_status ui_abi_1_error_status(ui_handle error, uint32_t *out_status);
ui_status ui_abi_1_error_code(ui_handle error, uint32_t *out_domain,
                              uint32_t *out_code);
ui_status ui_abi_1_error_message_utf8(ui_handle error, uint8_t *buffer,
                                      uint64_t capacity,
                                      uint64_t *out_length);

ui_status ui_abi_1_property_validator_register_bool(
    ui_handle property, ui_validate_bool_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_registration,
    ui_handle *out_error);
ui_status ui_abi_1_property_validator_register_i64(
    ui_handle property, ui_validate_i64_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_registration,
    ui_handle *out_error);
ui_status ui_abi_1_property_validator_register_utf8(
    ui_handle property, ui_validate_utf8_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_registration,
    ui_handle *out_error);
ui_status ui_abi_1_property_validator_cancel(ui_handle registration,
                                             ui_handle *out_error);
ui_status ui_abi_1_property_subscribe(ui_handle property,
                                      ui_callback callback, void *userdata,
                                      ui_destroy_callback destroy,
                                      ui_handle *out_subscription,
                                      ui_handle *out_error);
ui_status ui_abi_1_subscription_cancel(ui_handle subscription,
                                       ui_handle *out_error);

ui_status ui_abi_1_binding_create(ui_handle source, ui_handle target,
                                  uint32_t mode, ui_handle *out_binding,
                                  ui_handle *out_error);
ui_status ui_abi_1_binding_cancel(ui_handle binding, ui_handle *out_error);
ui_status ui_abi_1_binding_error_snapshot(
    ui_handle binding, ui_handle *out_owned_error,
    ui_handle *out_owned_rejected_property, uint32_t *out_direction);
ui_status ui_abi_1_binding_subscribe_error(
    ui_handle binding, ui_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_subscription,
    ui_handle *out_error);

ui_status ui_abi_1_command_create_async(
    ui_handle runtime, ui_handle eligibility_bool_or_zero,
    ui_command_start_callback start, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_command,
    ui_handle *out_error);
ui_status ui_abi_1_command_execute(ui_handle command,
                                   uint64_t *out_operation_id,
                                   ui_handle *out_error);
ui_status ui_abi_1_command_cancel(ui_handle command, uint64_t operation_id,
                                  ui_handle *out_error);
ui_status ui_abi_1_command_close(ui_handle command, ui_handle *out_error);
ui_status ui_abi_1_command_state_property(
    ui_handle command, ui_handle *out_owned_property);
ui_status ui_abi_1_command_can_execute_property(
    ui_handle command, ui_handle *out_owned_property);
ui_status ui_abi_1_command_failure_snapshot(
    ui_handle command, ui_handle *out_owned_error);
ui_status ui_abi_1_command_cancel_token_requested(ui_handle token,
                                                  ui_bool *out_yes);
ui_status ui_abi_1_command_complete_success(ui_handle command,
                                            uint64_t operation_id);
ui_status ui_abi_1_command_complete_failure_utf8(
    ui_handle command, uint64_t operation_id, uint32_t domain,
    uint32_t code, const uint8_t *message,
    uint64_t message_byte_length);
ui_status ui_abi_1_command_complete_cancelled(ui_handle command,
                                              uint64_t operation_id);

ui_status ui_abi_1_runtime_set_ui_loop_bridge(
    ui_handle runtime, ui_wake_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_error);
ui_status ui_abi_1_runtime_request_shutdown(ui_handle runtime, uint32_t mode,
                                            ui_handle *out_request,
                                            ui_handle *out_error);
ui_status ui_abi_1_shutdown_request_query(ui_handle request,
                                          uint32_t *out_state,
                                          uint32_t *out_reason,
                                          ui_handle *out_error);
ui_status ui_abi_1_runtime_host_begin_event(ui_handle runtime,
                                            uint64_t *out_scope_id);
ui_status ui_abi_1_runtime_host_end_event(ui_handle runtime,
                                          uint64_t scope_id);
ui_status ui_abi_1_runtime_host_stopped(ui_handle runtime, uint32_t reason,
                                        ui_handle *out_error);
ui_status ui_abi_1_runtime_post(ui_handle runtime, ui_callback callback,
                                void *userdata, ui_destroy_callback destroy,
                                ui_handle *out_error);
ui_status ui_abi_1_runtime_dispatch_turn(ui_handle runtime);
ui_status ui_abi_1_runtime_shutdown(ui_handle runtime, ui_handle *out_error);

ui_status ui_abi_1_handle_release(ui_handle owned);
ui_status ui_abi_1_handle_retain(ui_handle borrowed, ui_handle *out_owned);
ui_status ui_abi_1_get_version(uint32_t *out_major, uint32_t *out_minor,
                               uint32_t *out_patch);
ui_status ui_abi_1_numeric_expand_f32(float value, double *out_value,
                                      ui_handle *out_error);
ui_status ui_abi_1_numeric_narrow_f64(double value, uint32_t mode,
                                     float *out_value, ui_handle *out_error);
ui_status ui_abi_1_has_feature(uint32_t feature_id, ui_bool *out_supported);

#ifdef __cplusplus
}  // extern "C"
#endif

#endif  // FREEMVVM_ABI_H_
