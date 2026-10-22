#include "runtime_internal.hpp"

#include <chrono>
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
  ui_handle last_request = 0;
  ui_status create_status = UI_OK;
  ui_status bridge_status = UI_OK;
  ui_status shutdown_status = UI_OK;
  ui_status request_status = UI_PENDING;
  ui_status query_status = UI_OK;
  std::uint32_t query_state = UI_SHUTDOWN_REQUEST_PENDING;
  std::uint32_t query_reason = 0;
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
  static const freemvvm::detail::AbiApi api{
      &FakeCreate,       &FakeIsUiThread, &FakeSetBridge,
      &FakeShutdown,     &FakeRequestShutdown,
      &FakeQuery,        &FakeRelease,
      &FakeErrorStatus,  &FakeErrorCode,
  };
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

}  // namespace

int main() {
  if (!TestRuntimeLifecycleAndPendingResults()) return 1;
  if (!TestCloseFailureKeepsTokenForRetry()) return 1;
  if (!TestDestructorFailureIsRetriedBySupervisor()) return 1;
  if (!TestCreateFailurePreservesOwnedErrorSnapshot()) return 1;
  std::puts("C++ Runtime wrapper unit tests: OK");
  return 0;
}

namespace freemvvm::detail {

std::weak_ptr<RuntimeCleanupState> RuntimeTestAccess::state_of(
    const Runtime& runtime) {
  return runtime.state_;
}

}  // namespace freemvvm::detail
