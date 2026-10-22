#include <freemvvm/abi.h>

namespace {

using RuntimeCreate = ui_status (*)(ui_handle *, ui_handle *);
using ValidatorRegister = ui_status (*)(ui_handle,
                                        ui_validate_utf8_callback, void *,
                                        ui_destroy_callback, ui_handle *,
                                        ui_handle *);

static_assert(sizeof(ui_handle) == 8);
static_assert(sizeof(ui_status) == 4);
static_assert(sizeof(ui_bool) == 1);
static_assert(UI_NUMERIC_NARROW_EXACT == 0);

RuntimeCreate runtime_create_signature = &ui_abi_1_runtime_create;
ValidatorRegister validator_signature =
    &ui_abi_1_property_validator_register_utf8;

}  // namespace

int freemvvm_abi_header_cpp_probe() {
  return runtime_create_signature != nullptr && validator_signature != nullptr;
}
