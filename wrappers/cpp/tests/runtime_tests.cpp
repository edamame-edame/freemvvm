#include "runtime_internal.hpp"
#include "freemvvm/window.hpp"

#include <chrono>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <memory>
#include <mutex>
#include <unordered_map>
#include <thread>

namespace {

struct FakeAbi final {
  std::mutex mutex;
  std::condition_variable condition;
  ui_handle next_handle = 100;
  ui_handle last_runtime = 0;
  ui_handle last_window = 0;
  ui_handle last_request = 0;
  ui_status create_status = UI_OK;
  ui_status bridge_status = UI_OK;
  ui_status shutdown_status = UI_OK;
  ui_status request_status = UI_PENDING;
  ui_status query_status = UI_OK;
  ui_status window_create_status = UI_OK;
  ui_status window_get_status = UI_OK;
  ui_status window_set_status = UI_OK;
  ui_status window_reset_status = UI_OK;
  std::uint32_t query_state = UI_SHUTDOWN_REQUEST_PENDING;
  std::uint32_t query_reason = 0;
  float background_red = 1.0f;
  float background_green = 1.0f;
  float background_blue = 1.0f;
  ui_handle create_error = 0;
  ui_handle bridge_error = 0;
  ui_handle shutdown_error = 0;
  ui_handle request_error = 0;
  ui_handle query_error = 0;
  std::unordered_map<ui_handle, int> release_failures;
  std::unordered_map<ui_handle, int> release_counts;
  int bridge_calls = 0;
  int shutdown_calls = 0;
  ui_wake_callback wake = nullptr;
  void* wake_userdata = nullptr;
  ui_destroy_callback destroy = nullptr;
};

FakeAbi* active = nullptr;

ui_status FakeCreate(ui_handle* out_runtime, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_runtime = 0;
  *out_error = active->create_error;
  if (active->create_status == UI_OK) {
    *out_runtime = ++active->next_handle;
    active->last_runtime = *out_runtime;
  }
  return active->create_status;
}

ui_status FakeIsUiThread(ui_handle runtime, ui_bool* out_yes) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_yes = runtime == active->last_runtime ? 1 : 0;
  return UI_OK;
}

ui_status FakeSetBridge(ui_handle runtime, ui_wake_callback wake,
                        void* userdata, ui_destroy_callback destroy,
                        ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  ++active->bridge_calls;
  active->wake = wake;
  active->wake_userdata = userdata;
  active->destroy = destroy;
  *out_error = active->bridge_error;
  return runtime == active->last_runtime ? active->bridge_status
                                         : UI_INVALID_HANDLE;
}

ui_status FakeShutdown(ui_handle runtime, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  ++active->shutdown_calls;
  *out_error = active->shutdown_error;
  return runtime == active->last_runtime ? active->shutdown_status
                                         : UI_INVALID_HANDLE;
}

ui_status FakeRequestShutdown(ui_handle runtime, std::uint32_t mode,
                              ui_handle* out_request, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_request = 0;
  *out_error = active->request_error;
  if (runtime != active->last_runtime || mode != UI_SHUTDOWN_ASYNC) {
    return UI_INVALID_ARGUMENT;
  }
  if (active->request_status == UI_OK || active->request_status == UI_PENDING) {
    *out_request = ++active->next_handle;
    active->last_request = *out_request;
  }
  return active->request_status;
}

ui_status FakeQuery(ui_handle request, std::uint32_t* out_state,
                    std::uint32_t* out_reason, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_state = active->query_state;
  *out_reason = active->query_reason;
  *out_error = active->query_error;
  return request == active->last_request ? active->query_status
                                         : UI_INVALID_HANDLE;
}

ui_status FakeRelease(ui_handle token) {
  std::lock_guard<std::mutex> lock(active->mutex);
  ++active->release_counts[token];
  active->condition.notify_all();
  int& failures = active->release_failures[token];
  if (failures > 0) {
    --failures;
    return UI_INTERNAL_ERROR;
  }
  return UI_OK;
}

ui_status FakeWindowCreate(ui_handle runtime, ui_handle* out_window,
                           ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_window = 0;
  *out_error = 0;
  if (runtime != active->last_runtime) return UI_INVALID_HANDLE;
  if (active->window_create_status != UI_OK) {
    return active->window_create_status;
  }
  *out_window = ++active->next_handle;
  active->last_window = *out_window;
  active->background_red = 1.0f;
  active->background_green = 1.0f;
  active->background_blue = 1.0f;
  return UI_OK;
}

ui_status FakeWindowBackground(ui_handle window, float* out_red,
                               float* out_green, float* out_blue,
                               ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_error = 0;
  if (window != active->last_window) return UI_INVALID_HANDLE;
  if (active->window_get_status != UI_OK) return active->window_get_status;
  *out_red = active->background_red;
  *out_green = active->background_green;
  *out_blue = active->background_blue;
  return UI_OK;
}

ui_status FakeWindowSetBackground(ui_handle window, float red, float green,
                                  float blue, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_error = 0;
  if (window != active->last_window) return UI_INVALID_HANDLE;
  if (active->window_set_status != UI_OK) return active->window_set_status;
  if (!std::isfinite(red) || !std::isfinite(green) || !std::isfinite(blue) ||
      red < 0.0f || red > 1.0f || green < 0.0f || green > 1.0f ||
      blue < 0.0f || blue > 1.0f) {
    return UI_INVALID_ARGUMENT;
  }
  active->background_red = red;
  active->background_green = green;
  active->background_blue = blue;
  return UI_OK;
}

ui_status FakeWindowResetBackground(ui_handle window, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_error = 0;
  if (window != active->last_window) return UI_INVALID_HANDLE;
  if (active->window_reset_status != UI_OK) {
    return active->window_reset_status;
  }
  active->background_red = 1.0f;
  active->background_green = 1.0f;
  active->background_blue = 1.0f;
  return UI_OK;
}

ui_status FakeErrorStatus(ui_handle, std::uint32_t* out_status) {
  *out_status = UI_INTERNAL_ERROR;
  return UI_OK;
}

ui_status FakeErrorCode(ui_handle, std::uint32_t* out_domain,
                        std::uint32_t* out_code) {
  *out_domain = 1;
  *out_code = 7;
  return UI_OK;
}

void Wake(void*, std::uint32_t) {}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  static const freemvvm::detail::AbiApi api = [] {
    freemvvm::detail::AbiApi value{};
    value.runtime_create = &FakeCreate;
    value.runtime_is_ui_thread = &FakeIsUiThread;
    value.runtime_set_ui_loop_bridge = &FakeSetBridge;
    value.runtime_shutdown = &FakeShutdown;
    value.runtime_request_shutdown = &FakeRequestShutdown;
    value.shutdown_request_query = &FakeQuery;
    value.handle_release = &FakeRelease;
    value.error_status = &FakeErrorStatus;
    value.error_code = &FakeErrorCode;
    value.window_create = &FakeWindowCreate;
    value.window_background_rgb = &FakeWindowBackground;
    value.window_set_background_rgb = &FakeWindowSetBackground;
    value.window_reset_background_rgb = &FakeWindowResetBackground;
    return value;
  }();
  return std::shared_ptr<const freemvvm::detail::AbiApi>(
      std::shared_ptr<const freemvvm::detail::AbiApi>{}, &api);
}

bool Check(bool condition, const char* message) {
  if (condition) return true;
  std::fprintf(stderr, "FAILED: %s\n", message);
  return false;
}

freemvvm::Result<freemvvm::Runtime> Create(FakeAbi& fake) {
  active = &fake;
  return freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
}

bool TestRuntimeLifecycleAndPendingResults() {
  FakeAbi fake;
  auto created = Create(fake);
  if (!Check(created.has_value(), "Runtime creation should succeed")) return false;
  freemvvm::Runtime runtime = std::move(*created.value());

  auto is_ui_thread = runtime.is_ui_thread();
  if (!Check(is_ui_thread.has_value() && *is_ui_thread.value(),
             "UI-thread query should preserve the ABI result")) {
    return false;
  }

  fake.shutdown_status = UI_PENDING;
  const auto shutdown = runtime.shutdown();
  if (!Check(!shutdown.has_error() &&
                 shutdown.disposition() ==
                     freemvvm::ShutdownDisposition::kAcceptedPending,
             "UI_PENDING must be represented as accepted, not failure")) {
    return false;
  }

  auto request_result = runtime.request_shutdown();
  if (!Check(request_result.has_value(), "shutdown request should be owned")) {
    return false;
  }
  freemvvm::ShutdownRequest request = std::move(*request_result.value());
  fake.query_state = UI_SHUTDOWN_REQUEST_HOST_UNAVAILABLE;
  fake.query_reason = 23;
  auto query = request.query();
  if (!Check(query.has_value() &&
                 query.value()->state == freemvvm::ShutdownState::kHostUnavailable &&
                 query.value()->reason == 23,
             "shutdown query should preserve terminal state and reason")) {
    return false;
  }

  auto request_close = request.close();
  auto runtime_close = runtime.close();
  return Check(request_close.has_value() && runtime_close.has_value(),
               "explicit close should release both owned tokens");
}

bool TestCloseFailureKeepsTokenForRetry() {
  FakeAbi fake;
  auto created = Create(fake);
  if (!Check(created.has_value(), "Runtime creation should succeed")) return false;
  freemvvm::Runtime runtime = std::move(*created.value());
  const ui_handle token = fake.last_runtime;
  fake.release_failures[token] = 1;

  auto first = runtime.close();
  if (!Check(!first.has_value() && first.error()->raw_status() == UI_INTERNAL_ERROR,
             "failed explicit close should expose raw status")) {
    return false;
  }
  auto second = runtime.close();
  if (!Check(second.has_value(), "failed close must retain token for retry")) {
    return false;
  }
  return Check(fake.release_counts[token] == 2,
               "retry should call release exactly once more");
}

bool TestDestructorFailureIsRetriedBySupervisor() {
  FakeAbi fake;
  std::weak_ptr<freemvvm::detail::RuntimeCleanupState> state;
  ui_handle token = 0;
  {
    auto created = Create(fake);
    if (!Check(created.has_value(), "Runtime creation should succeed")) return false;
    freemvvm::Runtime runtime = std::move(*created.value());
    state = freemvvm::detail::RuntimeTestAccess::state_of(runtime);
    token = fake.last_runtime;
    fake.release_failures[token] = 2;
  }

  std::unique_lock<std::mutex> lock(fake.mutex);
  const bool retried = fake.condition.wait_for(
      lock, std::chrono::seconds(3), [&fake, token] {
        return fake.release_counts[token] >= 3;
      });
  lock.unlock();
  if (!Check(retried, "supervisor should retry failed destructor releases")) {
    return false;
  }
  for (int attempt = 0; attempt < 100 && !state.expired(); ++attempt) {
    std::this_thread::sleep_for(std::chrono::milliseconds(5));
  }
  return Check(state.expired(),
               "supervisor should stop retaining a drained Runtime state");
}

bool TestCreateFailurePreservesOwnedErrorSnapshot() {
  FakeAbi fake;
  fake.create_status = 77;
  fake.create_error = 900;
  auto created = Create(fake);
  if (!Check(!created.has_value(), "native Runtime creation failure should fail")) {
    return false;
  }
  const freemvvm::UiError* error = created.error();
  if (!Check(error != nullptr && error->raw_status() == 77,
             "UiError should preserve the unknown raw status")) {
    return false;
  }
  return Check(error->snapshot() != nullptr && error->snapshot()->valid(),
               "UiError should own an optional native error snapshot");
}

bool TestWindowBackgroundAndMoveCloseRetry() {
  FakeAbi fake;
  auto runtime_result = Create(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  freemvvm::Runtime runtime = std::move(*runtime_result.value());

  auto window_result = freemvvm::Window::create(runtime);
  if (!Check(window_result.has_value(), "Window creation should succeed")) {
    return false;
  }
  freemvvm::Window window = std::move(*window_result.value());
  const ui_handle window_token = fake.last_window;

  auto initial = window.background_rgb();
  if (!Check(initial.has_value() && initial.value()->red == 1.0f &&
                 initial.value()->green == 1.0f &&
                 initial.value()->blue == 1.0f,
             "default background should be opaque white")) {
    return false;
  }

  const freemvvm::RgbColor requested{0.2f, 0.4f, 0.8f};
  if (!Check(window.set_background_rgb(requested).has_value(),
             "valid background should be accepted")) {
    return false;
  }
  auto updated = window.background_rgb();
  if (!Check(updated.has_value() && updated.value()->red == requested.red &&
                 updated.value()->green == requested.green &&
                 updated.value()->blue == requested.blue,
             "background getter should reflect Core value")) {
    return false;
  }

  const freemvvm::RgbColor invalid{1.1f, 0.0f, 0.0f};
  auto invalid_result = window.set_background_rgb(invalid);
  auto unchanged = window.background_rgb();
  if (!Check(!invalid_result.has_value() &&
                 invalid_result.error()->raw_status() == UI_INVALID_ARGUMENT &&
                 unchanged.has_value() &&
                 unchanged.value()->red == requested.red &&
                 unchanged.value()->green == requested.green &&
                 unchanged.value()->blue == requested.blue,
             "invalid input should preserve previous color and raw error")) {
    return false;
  }

  if (!Check(window.reset_background_rgb().has_value(),
             "background reset should succeed")) {
    return false;
  }
  auto reset = window.background_rgb();
  if (!Check(reset.has_value() && reset.value()->red == 1.0f &&
                 reset.value()->green == 1.0f && reset.value()->blue == 1.0f,
             "reset should restore opaque white")) {
    return false;
  }

  fake.window_get_status = UI_INTERNAL_ERROR;
  auto get_failure = window.background_rgb();
  fake.window_get_status = UI_OK;
  fake.window_set_status = 73;
  auto set_failure = window.set_background_rgb(requested);
  fake.window_set_status = UI_OK;
  fake.window_reset_status = 74;
  auto reset_failure = window.reset_background_rgb();
  fake.window_reset_status = UI_OK;
  if (!Check(!get_failure.has_value() &&
                 get_failure.error()->raw_status() == UI_INTERNAL_ERROR &&
                 !set_failure.has_value() &&
                 set_failure.error()->raw_status() == 73 &&
                 !reset_failure.has_value() &&
                 reset_failure.error()->raw_status() == 74,
             "C ABI failures should preserve raw status in Result")) {
    return false;
  }

  freemvvm::Window moved(std::move(window));
  if (!Check(!window.background_rgb().has_value(),
             "moved-from Window should be invalid")) {
    return false;
  }
  fake.release_failures[window_token] = 1;
  auto first_close = moved.close();
  if (!Check(!first_close.has_value() &&
                 first_close.error()->raw_status() == UI_INTERNAL_ERROR,
             "failed close should expose error and retain ownership")) {
    return false;
  }
  if (!Check(moved.close().has_value() &&
                 fake.release_counts[window_token] == 2,
             "close retry should release the token exactly once")) {
    return false;
  }
  return Check(runtime.close().has_value(), "Runtime should close after Window");
}

bool TestWindowCreateFailureAndDestructorCleanup() {
  FakeAbi fake;
  auto runtime_result = Create(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  freemvvm::Runtime runtime = std::move(*runtime_result.value());

  fake.window_create_status = UI_INTERNAL_ERROR;
  auto failed = freemvvm::Window::create(runtime);
  if (!Check(!failed.has_value() &&
                 failed.error()->raw_status() == UI_INTERNAL_ERROR,
             "native Window create failure should preserve status")) {
    return false;
  }

  fake.window_create_status = UI_OK;
  ui_handle token = 0;
  {
    auto created = freemvvm::Window::create(runtime);
    if (!Check(created.has_value(), "Window creation should recover after failure")) {
      return false;
    }
    token = fake.last_window;
    fake.release_failures[token] = 2;
  }

  std::unique_lock<std::mutex> lock(fake.mutex);
  const bool retried = fake.condition.wait_for(
      lock, std::chrono::seconds(3), [&fake, token] {
        return fake.release_counts[token] >= 3;
      });
  lock.unlock();
  if (!Check(retried,
             "destructor release failure should be retried by supervisor")) {
    return false;
  }
  return Check(runtime.close().has_value(), "Runtime should close after cleanup");
}

}  // namespace

int main() {
  if (!TestRuntimeLifecycleAndPendingResults()) return 1;
  if (!TestCloseFailureKeepsTokenForRetry()) return 1;
  if (!TestDestructorFailureIsRetriedBySupervisor()) return 1;
  if (!TestCreateFailurePreservesOwnedErrorSnapshot()) return 1;
  if (!TestWindowBackgroundAndMoveCloseRetry()) return 1;
  if (!TestWindowCreateFailureAndDestructorCleanup()) return 1;
  std::puts("C++ Runtime wrapper unit tests: OK");
  return 0;
}

namespace freemvvm::detail {

std::weak_ptr<RuntimeCleanupState> RuntimeTestAccess::state_of(
    const Runtime& runtime) {
  return runtime.state_;
}

}  // namespace freemvvm::detail
