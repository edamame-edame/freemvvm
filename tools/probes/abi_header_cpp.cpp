#include <freemvvm/abi.h>

namespace {

using RuntimeCreate = ui_status (*)(ui_handle *, ui_handle *);
using ValidatorRegister = ui_status (*)(ui_handle,
                                        ui_validate_utf8_callback, void *,
                                        ui_destroy_callback, ui_handle *,
                                        ui_handle *);
using StackPanelOrientation = ui_status (*)(ui_handle, uint32_t *);
using ButtonCommand = ui_status (*)(ui_handle, ui_handle *, ui_handle *);

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

}  // namespace

int freemvvm_abi_header_cpp_probe() {
  return runtime_create_signature != nullptr && validator_signature != nullptr &&
         orientation_signature != nullptr && button_command_signature != nullptr &&
         UI_ORIENTATION_HORIZONTAL == 2 && UI_ALIGN_CENTER == 2;
}
