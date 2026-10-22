#include <freemvvm/abi.h>

#include <cstdio>

namespace {

void SmokeWake(void* userdata, uint32_t priority) {
  (void)userdata;
  (void)priority;
}

void SmokeDestroy(void* userdata) { (void)userdata; }

}  // namespace

int main() {
  uint32_t major = 0;
  uint32_t minor = 0;
  uint32_t patch = 0;
  if (ui_abi_1_get_version(&major, &minor, &patch) != UI_OK || major != 1 ||
      minor != 0 || patch != 0) {
    std::fputs("unexpected runtime version\n", stderr);
    return 1;
  }

  ui_handle runtime = 0;
  if (ui_abi_1_runtime_create(&runtime, nullptr) != UI_OK || runtime == 0) {
    std::fputs("Runtime creation failed\n", stderr);
    return 2;
  }
  if (ui_abi_1_runtime_set_ui_loop_bridge(runtime, SmokeWake, nullptr,
                                          SmokeDestroy, nullptr) != UI_OK) {
    std::fputs("UI loop bridge registration failed\n", stderr);
    (void)ui_abi_1_handle_release(runtime);
    return 3;
  }

  uint64_t host_scope = 0;
  if (ui_abi_1_runtime_host_begin_event(runtime, &host_scope) != UI_OK) {
    std::fputs("host event begin failed\n", stderr);
    (void)ui_abi_1_handle_release(runtime);
    return 4;
  }
  ui_handle shutdown_request = 0;
  if (ui_abi_1_runtime_request_shutdown(runtime, UI_SHUTDOWN_ASYNC,
                                        &shutdown_request, nullptr) !=
          UI_PENDING ||
      shutdown_request == 0) {
    std::fputs("Runtime shutdown request failed\n", stderr);
    (void)ui_abi_1_runtime_host_end_event(runtime, host_scope);
    (void)ui_abi_1_handle_release(runtime);
    return 5;
  }
  if (ui_abi_1_runtime_host_end_event(runtime, host_scope) != UI_OK) {
    std::fputs("host event end failed\n", stderr);
    (void)ui_abi_1_handle_release(shutdown_request);
    (void)ui_abi_1_handle_release(runtime);
    return 6;
  }

  uint32_t shutdown_state = UINT32_MAX;
  uint32_t shutdown_reason = UINT32_MAX;
  if (ui_abi_1_shutdown_request_query(shutdown_request, &shutdown_state,
                                      &shutdown_reason, nullptr) != UI_OK ||
      shutdown_state != UI_SHUTDOWN_REQUEST_COMPLETED || shutdown_reason != 0) {
    std::fputs("Runtime shutdown did not complete at the host boundary\n",
               stderr);
    (void)ui_abi_1_handle_release(shutdown_request);
    (void)ui_abi_1_handle_release(runtime);
    return 7;
  }
  if (ui_abi_1_handle_release(shutdown_request) != UI_OK ||
      ui_abi_1_handle_release(runtime) != UI_OK) {
    std::fputs("Runtime handle release failed\n", stderr);
    return 8;
  }

  std::puts("C++ C ABI native link/runtime smoke: OK");
  return 0;
}
