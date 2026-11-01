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
typedef void (*ui_window_close_callback)(void *userdata,
                                         ui_handle borrowed_close_request);

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

enum {
  UI_ORIENTATION_VERTICAL = 1,
  UI_ORIENTATION_HORIZONTAL = 2,
  UI_ALIGN_START = 1,
  UI_ALIGN_CENTER = 2,
  UI_ALIGN_END = 3,
  UI_ALIGN_STRETCH = 4,
  UI_CLOSE_ACCEPT = 1,
  UI_CLOSE_DECLINE = 2
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

ui_status ui_abi_1_window_create(ui_handle runtime, ui_handle *out_window,
                                 ui_handle *out_error);
ui_status ui_abi_1_stack_panel_create(ui_handle runtime,
                                      ui_handle *out_panel,
                                      ui_handle *out_error);
ui_status ui_abi_1_text_create(ui_handle runtime,
                               const uint8_t *initial_utf8,
                               uint64_t initial_byte_length,
                               ui_handle *out_text, ui_handle *out_error);
ui_status ui_abi_1_text_box_create(ui_handle runtime,
                                   const uint8_t *initial_utf8,
                                   uint64_t initial_byte_length,
                                   ui_handle *out_text_box,
                                   ui_handle *out_error);
ui_status ui_abi_1_button_create(ui_handle runtime,
                                 const uint8_t *initial_label_utf8,
                                 uint64_t initial_byte_length,
                                 ui_handle *out_button, ui_handle *out_error);
ui_status ui_abi_1_widget_is_enabled_property(
    ui_handle widget, ui_handle *out_owned_property);
ui_status ui_abi_1_widget_accessible_name_property(
    ui_handle widget, ui_handle *out_owned_property);
ui_status ui_abi_1_widget_request_focus(ui_handle widget, ui_handle *out_error);
ui_status ui_abi_1_window_content(ui_handle window,
                                  ui_handle *out_owned_child,
                                  ui_handle *out_error);
ui_status ui_abi_1_window_set_content(ui_handle window,
                                      ui_handle borrowed_child,
                                      ui_handle *out_error);
ui_status ui_abi_1_window_clear_content(ui_handle window,
                                        ui_handle *out_error);
ui_status ui_abi_1_window_title_property(ui_handle window,
                                         ui_handle *out_owned_property);
ui_status ui_abi_1_window_background_rgb(ui_handle window, float *out_red,
                                         float *out_green, float *out_blue,
                                         ui_handle *out_error);
ui_status ui_abi_1_window_set_background_rgb(ui_handle window, float red,
                                             float green, float blue,
                                             ui_handle *out_error);
ui_status ui_abi_1_window_reset_background_rgb(ui_handle window,
                                               ui_handle *out_error);
ui_status ui_abi_1_window_open(ui_handle window, ui_handle *out_error);
ui_status ui_abi_1_window_close(ui_handle window, ui_handle *out_error);
ui_status ui_abi_1_window_close_request_subscribe(
    ui_handle window, ui_window_close_callback callback, void *userdata,
    ui_destroy_callback destroy, ui_handle *out_subscription,
    ui_handle *out_error);
ui_status ui_abi_1_window_close_request_defer(
    ui_handle borrowed_request, ui_handle *out_owned_request,
    ui_handle *out_error);
ui_status ui_abi_1_window_close_request_resolve(
    ui_handle request, uint32_t decision, ui_handle *out_error);
ui_status ui_abi_1_container_insert(ui_handle container,
                                    ui_handle borrowed_child, uint64_t index,
                                    ui_handle *out_error);
ui_status ui_abi_1_container_remove(ui_handle container,
                                    ui_handle borrowed_child,
                                    ui_handle *out_error);
ui_status ui_abi_1_container_move(ui_handle container, uint64_t from_index,
                                  uint64_t to_index, ui_handle *out_error);
ui_status ui_abi_1_container_child_count(ui_handle container,
                                         uint64_t *out_count);
ui_status ui_abi_1_container_child_at(ui_handle container, uint64_t index,
                                      ui_handle *out_owned_child,
                                      ui_handle *out_error);
ui_status ui_abi_1_stack_panel_orientation(ui_handle panel,
                                           uint32_t *out_orientation);
ui_status ui_abi_1_stack_panel_set_orientation(ui_handle panel,
                                               uint32_t orientation,
                                               ui_handle *out_error);
ui_status ui_abi_1_stack_panel_set_child_align(ui_handle panel,
                                              ui_handle borrowed_child,
                                              uint32_t align,
                                              ui_handle *out_error);
ui_status ui_abi_1_text_content_property(ui_handle text,
                                         ui_handle *out_owned_property);
ui_status ui_abi_1_text_box_value_property(ui_handle text_box,
                                           ui_handle *out_owned_property);
ui_status ui_abi_1_button_label_property(ui_handle button,
                                         ui_handle *out_owned_property);
ui_status ui_abi_1_button_set_command(ui_handle button,
                                      ui_handle borrowed_command_or_zero,
                                      ui_handle *out_error);
ui_status ui_abi_1_button_command(ui_handle button,
                                  ui_handle *out_owned_command,
                                  ui_handle *out_error);

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
