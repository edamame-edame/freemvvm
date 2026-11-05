#include <freemvvm/abi.h>

namespace {

using RuntimeCreate = ui_status (*)(ui_handle *, ui_handle *);
using ValidatorRegister = ui_status (*)(ui_handle,
                                        ui_validate_utf8_callback, void *,
                                        ui_destroy_callback, ui_handle *,
                                        ui_handle *);
using StackPanelOrientation = ui_status (*)(ui_handle, uint32_t *);
using ButtonCommand = ui_status (*)(ui_handle, ui_handle *, ui_handle *);
using WindowCloseSubscribe = ui_status (*)(ui_handle,
                                           ui_window_close_callback, void *,
                                           ui_destroy_callback, ui_handle *,
                                           ui_handle *);
using WindowLifecycle = ui_status (*)(ui_handle, ui_handle *);
using InputSubscribe = ui_status (*)(ui_handle, uint64_t, ui_input_callback,
                                     void *, ui_destroy_callback,
                                     ui_handle *, ui_handle *);
using InputPointer = ui_status (*)(ui_handle, double *, double *, uint32_t *);

static_assert(sizeof(ui_handle) == 8);
static_assert(sizeof(ui_status) == 4);
static_assert(sizeof(ui_bool) == 1);
static_assert(UI_NUMERIC_NARROW_EXACT == 0);

RuntimeCreate runtime_create_signature = &ui_abi_1_runtime_create;
ValidatorRegister validator_signature =
    &ui_abi_1_property_validator_register_utf8;
StackPanelOrientation orientation_signature =
    &ui_abi_1_stack_panel_orientation;
ButtonCommand button_command_signature = &ui_abi_1_button_command;
WindowCloseSubscribe close_subscribe_signature =
    &ui_abi_1_window_close_request_subscribe;
WindowLifecycle window_open_signature = &ui_abi_1_window_open;
InputSubscribe input_subscribe_signature = &ui_abi_1_widget_input_subscribe;
InputPointer input_pointer_signature = &ui_abi_1_input_event_pointer;

}  // namespace

int freemvvm_abi_header_cpp_probe() {
  return runtime_create_signature != nullptr && validator_signature != nullptr &&
         orientation_signature != nullptr && button_command_signature != nullptr &&
         close_subscribe_signature != nullptr && window_open_signature != nullptr &&
         input_subscribe_signature != nullptr && input_pointer_signature != nullptr &&
         UI_CLOSE_ACCEPT == 1 && UI_CLOSE_DECLINE == 2 &&
         UI_INPUT_PRIMARY_POINTER_UP == 5 && UI_MODIFIER_CONTROL == 2 &&
         UI_ORIENTATION_HORIZONTAL == 2 && UI_ALIGN_CENTER == 2;
}
