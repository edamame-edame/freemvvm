#include <freemvvm/abi.h>

#include <stdio.h>

static void smoke_wake(void *userdata, uint32_t priority) {
  (void)userdata;
  (void)priority;
}

static void smoke_destroy(void *userdata) { (void)userdata; }

int main(void) {
  uint32_t major = 0;
  uint32_t minor = 0;
  uint32_t patch = 0;
  if (ui_abi_1_get_version(&major, &minor, &patch) != UI_OK || major != 1 ||
      minor != 0 || patch != 0) {
    fputs("unexpected runtime version\n", stderr);
    return 1;
  }

  ui_handle runtime = 0;
  if (ui_abi_1_runtime_create(&runtime, 0) != UI_OK || runtime == 0) {
    fputs("Runtime creation failed\n", stderr);
    return 2;
  }
  if (ui_abi_1_runtime_set_ui_loop_bridge(runtime, smoke_wake, 0,
                                          smoke_destroy, 0) != UI_OK) {
    fputs("UI loop bridge registration failed\n", stderr);
    (void)ui_abi_1_handle_release(runtime);
    return 3;
  }

  ui_handle window = 0;
  ui_handle panel = 0;
  ui_handle text = 0;
  ui_handle text_box = 0;
  ui_handle button = 0;
  ui_handle property = 0;
  if (ui_abi_1_window_create(runtime, &window, 0) != UI_OK ||
      ui_abi_1_stack_panel_create(runtime, &panel, 0) != UI_OK ||
      ui_abi_1_text_create(runtime, (const uint8_t *)"hello", 5, &text, 0) !=
          UI_OK ||
      ui_abi_1_text_box_create(runtime, 0, 0, &text_box, 0) != UI_OK ||
      ui_abi_1_button_create(runtime, (const uint8_t *)"run", 3, &button, 0) !=
          UI_OK ||
      ui_abi_1_text_content_property(text, &property) != UI_OK || property == 0 ||
      ui_abi_1_window_set_content(window, panel, 0) != UI_OK ||
      ui_abi_1_container_insert(panel, text, 0, 0) != UI_OK ||
      ui_abi_1_container_insert(panel, button, 1, 0) != UI_OK) {
    fputs("UI Control creation/property/Tree operation failed\n", stderr);
    return 9;
  }
  uint64_t child_count = 0;
  uint32_t orientation = 0;
  if (ui_abi_1_container_child_count(panel, &child_count) != UI_OK ||
      child_count != 2 ||
      ui_abi_1_stack_panel_orientation(panel, &orientation) != UI_OK ||
      orientation != UI_ORIENTATION_VERTICAL ||
      ui_abi_1_stack_panel_set_orientation(panel, UI_ORIENTATION_HORIZONTAL,
                                           0) != UI_OK ||
      ui_abi_1_stack_panel_set_child_align(panel, button, UI_ALIGN_CENTER, 0) !=
          UI_OK) {
    fputs("UI Control layout operation failed\n", stderr);
    return 10;
  }
  ui_handle command = 0;
  if (ui_abi_1_button_command(button, &command, 0) != UI_OK || command != 0 ||
      ui_abi_1_window_clear_content(window, 0) != UI_OK) {
    fputs("UI Control optional state operation failed\n", stderr);
    return 11;
  }
  if (ui_abi_1_handle_release(property) != UI_OK ||
      ui_abi_1_handle_release(window) != UI_OK ||
      ui_abi_1_handle_release(panel) != UI_OK ||
      ui_abi_1_handle_release(text) != UI_OK ||
      ui_abi_1_handle_release(text_box) != UI_OK ||
      ui_abi_1_handle_release(button) != UI_OK) {
    fputs("UI Control handle release failed\n", stderr);
    return 12;
  }

  uint64_t host_scope = 0;
  if (ui_abi_1_runtime_host_begin_event(runtime, &host_scope) != UI_OK) {
    fputs("host event begin failed\n", stderr);
    (void)ui_abi_1_handle_release(runtime);
    return 4;
  }
  ui_handle shutdown_request = 0;
  if (ui_abi_1_runtime_request_shutdown(runtime, UI_SHUTDOWN_ASYNC,
                                        &shutdown_request, 0) != UI_PENDING ||
      shutdown_request == 0) {
    fputs("Runtime shutdown request failed\n", stderr);
    (void)ui_abi_1_runtime_host_end_event(runtime, host_scope);
    (void)ui_abi_1_handle_release(runtime);
    return 5;
  }
  if (ui_abi_1_runtime_host_end_event(runtime, host_scope) != UI_OK) {
    fputs("host event end failed\n", stderr);
    (void)ui_abi_1_handle_release(shutdown_request);
    (void)ui_abi_1_handle_release(runtime);
    return 6;
  }
  uint32_t shutdown_state = UINT32_MAX;
  uint32_t shutdown_reason = UINT32_MAX;
  if (ui_abi_1_shutdown_request_query(shutdown_request, &shutdown_state,
                                      &shutdown_reason, 0) != UI_OK ||
      shutdown_state != UI_SHUTDOWN_REQUEST_COMPLETED || shutdown_reason != 0) {
    fputs("Runtime shutdown did not complete at the host boundary\n", stderr);
    (void)ui_abi_1_handle_release(shutdown_request);
    (void)ui_abi_1_handle_release(runtime);
    return 7;
  }
  if (ui_abi_1_handle_release(shutdown_request) != UI_OK ||
      ui_abi_1_handle_release(runtime) != UI_OK) {
    fputs("Runtime handle release failed\n", stderr);
    return 8;
  }

  puts("C ABI native link/runtime smoke: OK");
  return 0;
}
