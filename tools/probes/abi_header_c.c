#include <freemvvm/abi.h>

static ui_status (*const runtime_create_signature)(ui_handle *, ui_handle *) =
    &ui_abi_1_runtime_create;
static ui_status (*const property_set_i64_signature)(ui_handle, int64_t,
                                                     ui_handle *) =
    &ui_abi_1_property_set_i64;
static ui_status (*const subscribe_signature)(ui_handle, ui_callback, void *,
                                              ui_destroy_callback, ui_handle *,
                                              ui_handle *) =
    &ui_abi_1_property_subscribe;
static ui_status (*const window_create_signature)(ui_handle, ui_handle *,
                                                  ui_handle *) =
    &ui_abi_1_window_create;
static ui_status (*const container_insert_signature)(ui_handle, ui_handle,
                                                     uint64_t, ui_handle *) =
    &ui_abi_1_container_insert;
static ui_status (*const button_command_signature)(ui_handle, ui_handle *,
                                                   ui_handle *) =
    &ui_abi_1_button_command;
static ui_status (*const widget_focus_signature)(ui_handle, ui_handle *) =
    &ui_abi_1_widget_request_focus;
static ui_status (*const window_close_subscribe_signature)(
    ui_handle, ui_window_close_callback, void *, ui_destroy_callback,
    ui_handle *, ui_handle *) = &ui_abi_1_window_close_request_subscribe;
static ui_status (*const close_request_defer_signature)(ui_handle, ui_handle *,
                                                        ui_handle *) =
    &ui_abi_1_window_close_request_defer;

int freemvvm_abi_header_c_probe(void) {
  return runtime_create_signature != 0 && property_set_i64_signature != 0 &&
         subscribe_signature != 0 && window_create_signature != 0 &&
         container_insert_signature != 0 && button_command_signature != 0 &&
         widget_focus_signature != 0 && window_close_subscribe_signature != 0 &&
         close_request_defer_signature != 0 && UI_CLOSE_ACCEPT == 1 &&
         UI_CLOSE_DECLINE == 2 &&
         UI_ORIENTATION_VERTICAL == 1 && UI_ALIGN_STRETCH == 4 &&
         sizeof(ui_handle) == 8 &&
         sizeof(ui_status) == 4 && sizeof(ui_bool) == 1;
}
