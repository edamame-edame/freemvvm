#include "runtime_internal.hpp"

#include <freemvvm/abi.h>
#include <freemvvm/command.hpp>

#include <cstdint>
#include <cstdio>
#include <memory>
#include <optional>
#include <string>
#include <utility>

namespace {

struct Posted final {
  ui_callback callback = nullptr;
  void* userdata = nullptr;
  ui_destroy_callback destroy = nullptr;
};

struct FakeAbi final {
  ui_command_start_callback start = nullptr;
  void* start_userdata = nullptr;
  ui_destroy_callback start_destroy = nullptr;
  Posted posted;
  ui_handle next_handle = 100;
  ui_handle command = 20;
  ui_handle state_property = 30;
  ui_handle can_execute_property = 31;
  ui_handle failure_snapshot = 80;
  std::uint64_t operation_id = 42;
  std::uint32_t state = 1;
  bool can_execute = true;
  bool cancellation_requested = false;
  ui_status post_status = UI_OK;
  ui_handle post_error = 0;
  int callback_destroy_count = 0;
  int cancel_count = 0;
  int success_count = 0;
  int failure_count = 0;
  int cancelled_count = 0;
  int command_close_count = 0;
  int releases = 0;
  std::string failure_message;
};

FakeAbi* active = nullptr;

ui_status FakeRuntimeCreate(ui_handle* runtime, ui_handle*) {
  *runtime = 10;
  return UI_OK;
}

ui_status FakeRuntimeIsUiThread(ui_handle, ui_bool* is_ui_thread) {
  *is_ui_thread = 1;
  return UI_OK;
}

ui_status FakeSetBridge(ui_handle, ui_wake_callback, void*,
                        ui_destroy_callback, ui_handle*) {
  return UI_OK;
}

ui_status FakeShutdown(ui_handle, ui_handle*) { return UI_OK; }

ui_status FakeRelease(ui_handle) {
  ++active->releases;
  return UI_OK;
}

ui_status FakeRetain(ui_handle token, ui_handle* retained) {
  *retained = ++active->next_handle;
  if (token == 0) return UI_INVALID_HANDLE;
  return UI_OK;
}

ui_status FakeCreateCommand(ui_handle, ui_handle,
                            ui_command_start_callback start, void* userdata,
                            ui_destroy_callback destroy,
                            ui_handle* command, ui_handle*) {
  active->start = start;
  active->start_userdata = userdata;
  active->start_destroy = destroy;
  *command = active->command;
  return UI_OK;
}

ui_status FakeExecute(ui_handle command, std::uint64_t* operation_id,
                      ui_handle*) {
  if (command != active->command) return UI_INVALID_HANDLE;
  *operation_id = active->operation_id;
  const ui_status callback_status = active->start(
      active->start_userdata, command, active->operation_id, 50);
  active->state = callback_status == UI_OK ? 1 : 4;
  return UI_OK;
}

ui_status FakeCancel(ui_handle, std::uint64_t, ui_handle*) {
  ++active->cancel_count;
  active->cancellation_requested = true;
  active->state = 2;
  return UI_OK;
}

ui_status FakeClose(ui_handle command, ui_handle*) {
  if (command != active->command) return UI_INVALID_HANDLE;
  ++active->command_close_count;
  if (active->start_destroy != nullptr) {
    active->start_destroy(active->start_userdata);
    active->start_destroy = nullptr;
    ++active->callback_destroy_count;
  }
  return UI_OK;
}

ui_status FakeStateProperty(ui_handle, ui_handle* property) {
  *property = active->state_property;
  return UI_OK;
}

ui_status FakeCanExecuteProperty(ui_handle, ui_handle* property) {
  *property = active->can_execute_property;
  return UI_OK;
}

ui_status FakeFailureSnapshot(ui_handle, ui_handle* snapshot) {
  *snapshot = active->state == 4 ? active->failure_snapshot : 0;
  return UI_OK;
}

ui_status FakeCancelRequested(ui_handle, ui_bool* requested) {
  *requested = active->cancellation_requested ? 1 : 0;
  return UI_OK;
}

ui_status FakeCompleteSuccess(ui_handle, std::uint64_t) {
  ++active->success_count;
  active->state = 3;
  return UI_OK;
}

ui_status FakeCompleteFailure(ui_handle, std::uint64_t, std::uint32_t,
                              std::uint32_t, const std::uint8_t* message,
                              std::uint64_t length) {
  ++active->failure_count;
  active->state = 4;
  active->failure_message.assign(reinterpret_cast<const char*>(message),
                                 static_cast<std::size_t>(length));
  return UI_OK;
}

ui_status FakeCompleteCancelled(ui_handle, std::uint64_t) {
  ++active->cancelled_count;
  active->state = 5;
  return UI_OK;
}

ui_status FakePost(ui_handle, ui_callback callback, void* userdata,
                   ui_destroy_callback destroy, ui_handle* error) {
  *error = active->post_error;
  if (active->post_status != UI_OK) return active->post_status;
  active->posted = Posted{callback, userdata, destroy};
  return UI_OK;
}

ui_status FakeGetI64(ui_handle property, std::int64_t* value) {
  if (property != active->state_property) return UI_INVALID_HANDLE;
  *value = static_cast<std::int64_t>(active->state);
  return UI_OK;
}

ui_status FakeGetBool(ui_handle property, ui_bool* value) {
  if (property != active->can_execute_property) return UI_INVALID_HANDLE;
  *value = active->can_execute ? 1 : 0;
  return UI_OK;
}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  static const freemvvm::detail::AbiApi api = [] {
    freemvvm::detail::AbiApi value{};
    value.runtime_create = &FakeRuntimeCreate;
    value.runtime_is_ui_thread = &FakeRuntimeIsUiThread;
    value.runtime_set_ui_loop_bridge = &FakeSetBridge;
    value.runtime_shutdown = &FakeShutdown;
    value.handle_release = &FakeRelease;
    value.property_get_bool = &FakeGetBool;
    value.property_get_i64 = &FakeGetI64;
    value.handle_retain = &FakeRetain;
    value.command_create_async = &FakeCreateCommand;
    value.command_execute = &FakeExecute;
    value.command_cancel = &FakeCancel;
    value.command_close = &FakeClose;
    value.command_state_property = &FakeStateProperty;
    value.command_can_execute_property = &FakeCanExecuteProperty;
    value.command_failure_snapshot = &FakeFailureSnapshot;
    value.command_cancel_token_requested = &FakeCancelRequested;
    value.command_complete_success = &FakeCompleteSuccess;
    value.command_complete_failure_utf8 = &FakeCompleteFailure;
    value.command_complete_cancelled = &FakeCompleteCancelled;
    value.runtime_post = &FakePost;
    return value;
  }();
  return std::shared_ptr<const freemvvm::detail::AbiApi>(
      std::shared_ptr<const freemvvm::detail::AbiApi>{}, &api);
}

void Wake(void*, std::uint32_t) {}

bool Check(bool condition, const char* message) {
  if (condition) return true;
  std::fprintf(stderr, "FAILED: %s\n", message);
  return false;
}

template <typename T>
bool Dispatch(freemvvm::Result<T>& result) {
  if (!result.has_value() || active->posted.callback == nullptr) return false;
  Posted posted = std::exchange(active->posted, Posted{});
  posted.callback(posted.userdata);
  posted.destroy(posted.userdata);
  return true;
}

bool TestCommandWorkerLifecycle() {
  FakeAbi fake;
  active = &fake;
  auto runtime_result = freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
  if (!Check(runtime_result.has_value(), "create fake Runtime")) return false;
  auto runtime = std::move(*runtime_result.value());
  std::optional<freemvvm::CommandWorker> worker;
  auto command_result = freemvvm::Command::create_async(
      runtime, nullptr,
      [&worker](freemvvm::CommandWorker value) -> freemvvm::Result<void> {
        worker.emplace(std::move(value));
        return freemvvm::Result<void>::success();
      });
  if (!Check(command_result.has_value(), "create async Command")) return false;
  auto command = std::move(*command_result.value());

  auto state_result = command.state_property();
  if (!Check(state_result.has_value(), "get state Property")) return false;
  auto state = std::move(*state_result.value());
  auto can_execute_result = command.can_execute_property();
  if (!Check(can_execute_result.has_value(), "get can-execute Property")) {
    return false;
  }
  auto can_execute = std::move(*can_execute_result.value());
  auto can_execute_value = can_execute.get();
  if (!Check(can_execute_value.has_value() && *can_execute_value.value(),
             "read can-execute Property")) {
    return false;
  }
  auto failure = command.failure_snapshot();
  if (!Check(failure.has_value() && !failure.value()->has_value(),
             "no failure snapshot before execution")) {
    return false;
  }
  auto operation = command.execute();
  if (!Check(operation.has_value() && *operation.value() == 42 && worker,
             "execute transfers worker lease")) {
    return false;
  }
  auto cancellation = worker->is_cancellation_requested();
  if (!Check(cancellation.has_value() && !*cancellation.value(),
             "query cancellation token")) {
    return false;
  }
  if (!Check(command.cancel(*operation.value()).has_value() &&
                 fake.cancel_count == 1,
             "request cooperative cancellation")) {
    return false;
  }
  cancellation = worker->is_cancellation_requested();
  if (!Check(cancellation.has_value() && *cancellation.value(),
             "observe cooperative cancellation request")) {
    return false;
  }
  auto report = worker->report_success();
  if (!Check(Dispatch(report), "post terminal result to UI loop")) return false;
  auto state_value = state.get();
  if (!Check(state_value.has_value() && *state_value.value() == 3 &&
                 fake.success_count == 1,
             "apply completion on UI dispatch")) {
    return false;
  }
  if (!Check(command.close().has_value(), "close command registration")) {
    return false;
  }
  worker.reset();
  if (!Check(fake.callback_destroy_count == 1 &&
                 fake.command_close_count == 1,
             "destroy callback once and close once")) {
    return false;
  }
  if (!Check(state.close().has_value() && can_execute.close().has_value(),
             "close independently owned Properties")) {
    return false;
  }
  return runtime.close().has_value();
}

bool TestRejectedPostCanRetry() {
  FakeAbi fake;
  active = &fake;
  auto runtime_result = freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
  if (!Check(runtime_result.has_value(), "create retry Runtime")) return false;
  auto runtime = std::move(*runtime_result.value());
  std::optional<freemvvm::CommandWorker> worker;
  auto command_result = freemvvm::Command::create_async(
      runtime, nullptr,
      [&worker](freemvvm::CommandWorker value) -> freemvvm::Result<void> {
        worker.emplace(std::move(value));
        return freemvvm::Result<void>::success();
      });
  if (!Check(command_result.has_value(), "create retry Command")) return false;
  auto command = std::move(*command_result.value());
  if (!Check(command.execute().has_value() && worker,
             "start retryable worker")) {
    return false;
  }

  fake.post_status = UI_UNSUPPORTED;
  auto rejected = worker->report_failure(7, 19, "worker failed");
  if (!Check(!rejected.has_value() && fake.posted.callback == nullptr,
             "rejected post is reported and not dispatched")) {
    return false;
  }
  fake.post_status = UI_OK;
  auto accepted = worker->report_failure(7, 19, "worker failed");
  if (!Check(Dispatch(accepted), "retry terminal report after post rejection")) {
    return false;
  }
  if (!Check(fake.failure_count == 1 && fake.state == 4 &&
                 fake.failure_message == "worker failed",
             "retry applies one failure with its UTF-8 message")) {
    return false;
  }
  worker.reset();
  if (!Check(command.close().has_value(), "close retry Command")) return false;
  return runtime.close().has_value();
}

bool TestStartFailureKeepsAcceptedOperation() {
  FakeAbi fake;
  active = &fake;
  auto runtime_result = freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
  if (!Check(runtime_result.has_value(), "create failure Runtime")) return false;
  auto runtime = std::move(*runtime_result.value());
  auto command_result = freemvvm::Command::create_async(
      runtime, nullptr,
      [](freemvvm::CommandWorker) -> freemvvm::Result<void> {
        return freemvvm::Result<void>::failure(freemvvm::UiError(77));
      });
  if (!Check(command_result.has_value(), "create failing Command")) return false;
  auto command = std::move(*command_result.value());
  auto operation = command.execute();
  if (!Check(operation.has_value() && *operation.value() == 42,
             "accepted operation keeps its ID when start callback fails")) {
    return false;
  }
  auto state_result = command.state_property();
  if (!Check(state_result.has_value(), "get failed state Property")) {
    return false;
  }
  auto state = std::move(*state_result.value());
  auto state_value = state.get();
  if (!Check(state_value.has_value() && *state_value.value() == 4,
             "start failure is reflected as failed state")) {
    return false;
  }
  auto failure = command.failure_snapshot();
  if (!Check(failure.has_value() && failure.value()->has_value(),
             "failed start exposes an owned failure snapshot")) {
    return false;
  }
  if (!Check(command.close().has_value() && state.close().has_value(),
             "close failed Command and state owner")) {
    return false;
  }
  return runtime.close().has_value();
}

}  // namespace

int main() {
  return TestCommandWorkerLifecycle() && TestRejectedPostCanRetry() &&
                 TestStartFailureKeepsAcceptedOperation()
             ? 0
             : 1;
}
