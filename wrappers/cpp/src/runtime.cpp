#include "freemvvm/property.hpp"
#include "freemvvm/runtime.hpp"
#include "freemvvm/subscription.hpp"

#include "runtime_internal.hpp"

#include <algorithm>
#include <chrono>
#include <cstdio>
#include <exception>
#include <new>
#include <stdexcept>
#include <thread>

namespace freemvvm::detail {
namespace {

constexpr std::uint32_t kUiOk = 0;
constexpr std::uint32_t kUiInvalidArgument = 1;
constexpr std::uint32_t kUiOutOfMemory = 8;
constexpr std::uint32_t kUiInternalError = 9;
constexpr std::uint32_t kUiPending = 14;
constexpr std::uint32_t kShutdownAsync = 0;

ui_status api_runtime_create(ui_handle* out_runtime, ui_handle* out_error) {
  return ui_abi_1_runtime_create(out_runtime, out_error);
}

ui_status api_runtime_is_ui_thread(ui_handle runtime, ui_bool* out_yes) {
  return ui_abi_1_runtime_is_ui_thread(runtime, out_yes);
}

ui_status api_runtime_set_ui_loop_bridge(ui_handle runtime,
                                         ui_wake_callback wake, void* userdata,
                                         ui_destroy_callback destroy,
                                         ui_handle* out_error) {
  return ui_abi_1_runtime_set_ui_loop_bridge(runtime, wake, userdata, destroy,
                                             out_error);
}

ui_status api_runtime_shutdown(ui_handle runtime, ui_handle* out_error) {
  return ui_abi_1_runtime_shutdown(runtime, out_error);
}

ui_status api_runtime_request_shutdown(ui_handle runtime, std::uint32_t mode,
                                       ui_handle* out_request,
                                       ui_handle* out_error) {
  return ui_abi_1_runtime_request_shutdown(runtime, mode, out_request,
                                           out_error);
}

ui_status api_shutdown_request_query(ui_handle request, std::uint32_t* state,
                                     std::uint32_t* reason,
                                     ui_handle* out_error) {
  return ui_abi_1_shutdown_request_query(request, state, reason, out_error);
}

ui_status api_handle_release(ui_handle token) {
  return ui_abi_1_handle_release(token);
}

ui_status api_error_status(ui_handle error, std::uint32_t* status) {
  return ui_abi_1_error_status(error, status);
}

ui_status api_error_code(ui_handle error, std::uint32_t* domain,
                         std::uint32_t* code) {
  return ui_abi_1_error_code(error, domain, code);
}

ui_status api_property_create_bool(ui_handle runtime, ui_bool initial,
                                   ui_handle* out_property,
                                   ui_handle* out_error) {
  return ui_abi_1_property_create_bool(runtime, initial, out_property,
                                       out_error);
}

ui_status api_property_create_i64(ui_handle runtime, std::int64_t initial,
                                  ui_handle* out_property,
                                  ui_handle* out_error) {
  return ui_abi_1_property_create_i64(runtime, initial, out_property,
                                      out_error);
}

ui_status api_property_create_f64(ui_handle runtime, double initial,
                                  ui_handle* out_property,
                                  ui_handle* out_error) {
  return ui_abi_1_property_create_f64(runtime, initial, out_property,
                                      out_error);
}

ui_status api_property_create_decimal(ui_handle runtime, std::uint64_t initial,
                                      ui_handle* out_property,
                                      ui_handle* out_error) {
  return ui_abi_1_property_create_decimal(runtime, initial, out_property,
                                          out_error);
}

ui_status api_property_create_utf8(ui_handle runtime,
                                   const std::uint8_t* initial,
                                   std::uint64_t byte_length,
                                   ui_handle* out_property,
                                   ui_handle* out_error) {
  return ui_abi_1_property_create_utf8(runtime, initial, byte_length,
                                       out_property, out_error);
}

ui_status api_property_get_bool(ui_handle property, ui_bool* out_value) {
  return ui_abi_1_property_get_bool(property, out_value);
}

ui_status api_property_get_i64(ui_handle property, std::int64_t* out_value) {
  return ui_abi_1_property_get_i64(property, out_value);
}

ui_status api_property_get_f64(ui_handle property, double* out_value) {
  return ui_abi_1_property_get_f64(property, out_value);
}

ui_status api_property_get_decimal(ui_handle property,
                                   std::uint64_t* out_value) {
  return ui_abi_1_property_get_decimal(property, out_value);
}

ui_status api_property_get_utf8(ui_handle property, std::uint8_t* buffer,
                                std::uint64_t capacity,
                                std::uint64_t* out_length) {
  return ui_abi_1_property_get_utf8(property, buffer, capacity, out_length);
}

ui_status api_property_set_bool(ui_handle property, ui_bool value,
                                ui_handle* out_error) {
  return ui_abi_1_property_set_bool(property, value, out_error);
}

ui_status api_property_set_i64(ui_handle property, std::int64_t value,
                               ui_handle* out_error) {
  return ui_abi_1_property_set_i64(property, value, out_error);
}

ui_status api_property_set_f64(ui_handle property, double value,
                               ui_handle* out_error) {
  return ui_abi_1_property_set_f64(property, value, out_error);
}

ui_status api_property_set_decimal(ui_handle property, std::uint64_t value,
                                   ui_handle* out_error) {
  return ui_abi_1_property_set_decimal(property, value, out_error);
}

ui_status api_property_set_utf8(ui_handle property, const std::uint8_t* bytes,
                                std::uint64_t byte_length,
                                ui_handle* out_error) {
  return ui_abi_1_property_set_utf8(property, bytes, byte_length, out_error);
}

ui_status api_property_subscribe(ui_handle property, ui_callback callback,
                                 void* userdata, ui_destroy_callback destroy,
                                 ui_handle* out_subscription,
                                 ui_handle* out_error) {
  return ui_abi_1_property_subscribe(property, callback, userdata, destroy,
                                     out_subscription, out_error);
}

ui_status api_subscription_cancel(ui_handle subscription,
                                  ui_handle* out_error) {
  return ui_abi_1_subscription_cancel(subscription, out_error);
}

ui_status api_binding_create(ui_handle source, ui_handle target,
                             std::uint32_t mode, ui_handle* out_binding,
                             ui_handle* out_error) {
  return ui_abi_1_binding_create(source, target, mode, out_binding, out_error);
}

ui_status api_binding_cancel(ui_handle binding, ui_handle* out_error) {
  return ui_abi_1_binding_cancel(binding, out_error);
}

ui_status api_binding_error_snapshot(ui_handle binding,
                                      ui_handle* out_error,
                                      ui_handle* out_rejected_property,
                                      std::uint32_t* out_direction) {
  return ui_abi_1_binding_error_snapshot(binding, out_error,
                                         out_rejected_property,
                                         out_direction);
}

ui_status api_binding_subscribe_error(ui_handle binding, ui_callback callback,
                                      void* userdata,
                                      ui_destroy_callback destroy,
                                      ui_handle* out_subscription,
                                      ui_handle* out_error) {
  return ui_abi_1_binding_subscribe_error(binding, callback, userdata, destroy,
                                          out_subscription, out_error);
}

void report_callback_exception(const std::exception_ptr& exception) noexcept {
  if (!exception) {
    std::fputs("FreeMvvm: C++ callback failed without an exception payload\n",
               stderr);
    return;
  }
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  try {
    std::rethrow_exception(exception);
  } catch (const std::exception& error) {
    std::fprintf(stderr,
                 "FreeMvvm: C++ property notification callback threw: %s\n",
                 error.what());
  } catch (...) {
    std::fputs(
        "FreeMvvm: C++ property notification callback threw a "
        "non-standard exception\n",
        stderr);
  }
#else
  (void)exception;
  std::fputs("FreeMvvm: C++ property notification callback failed\n", stderr);
#endif
}

struct SubscriptionCallbackState final {
  SubscriptionCallbackState(std::function<void()> callback_value,
                           std::shared_ptr<RuntimeCleanupState> runtime_value,
                           CallbackKind callback_kind)
      : callback(std::move(callback_value)),
        runtime(std::move(runtime_value)),
        kind(callback_kind) {}

  std::function<void()> callback;
  std::shared_ptr<RuntimeCleanupState> runtime;
  CallbackKind kind;
};

void subscription_callback(void* userdata) noexcept {
  auto* state = static_cast<SubscriptionCallbackState*>(userdata);
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  try {
    state->callback();
  } catch (...) {
    state->runtime->dispatch_callback_failure(
        CallbackFailure{state->kind, std::current_exception()});
  }
#else
  state->callback();
#endif
}

void subscription_destroy(void* userdata) noexcept {
  delete static_cast<SubscriptionCallbackState*>(userdata);
}

}  // namespace

RuntimeCleanupState::RuntimeCleanupState(std::shared_ptr<const AbiApi> api_value)
    : api(std::move(api_value)) {}

CallbackHandlerRecord::~CallbackHandlerRecord() noexcept {
  if (destroy == nullptr) return;
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  try {
    destroy(userdata);
  } catch (...) {
    std::fputs("FreeMvvm: C++ callback handler destroy function threw\n",
               stderr);
  }
#else
  destroy(userdata);
#endif
}

void RuntimeCleanupState::dispatch_callback_failure(
    const CallbackFailure& failure) noexcept {
  std::shared_ptr<CallbackHandlerRecord> handler;
  {
    std::lock_guard<std::mutex> lock(callback_handler_mutex);
    handler = callback_handler;
  }
  if (!handler || handler->handler == nullptr) {
    report_callback_exception(failure.exception);
    return;
  }
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  try {
    handler->handler(failure, handler->userdata);
  } catch (...) {
    std::fputs("FreeMvvm: C++ callback failure handler threw\n", stderr);
  }
#else
  handler->handler(failure, handler->userdata);
#endif
}

RuntimeCleanupState::~RuntimeCleanupState() {
  PendingNode* node = pending_head;
  while (node != nullptr) {
    PendingNode* next = node->next;
    delete node;
    node = next;
  }
}

bool RuntimeCleanupState::enqueue(PendingNode* node) noexcept {
  if (node == nullptr || node->token == 0) return false;
  node->next = nullptr;
  {
    std::lock_guard<std::mutex> lock(queue_mutex);
    if (pending_tail == nullptr) {
      pending_head = node;
    } else {
      pending_tail->next = node;
    }
    pending_tail = node;
  }
  CleanupSupervisor::instance().register_pending(shared_from_this());
  return true;
}

PendingNode* RuntimeCleanupState::pop() noexcept {
  std::lock_guard<std::mutex> lock(queue_mutex);
  PendingNode* node = pending_head;
  if (node != nullptr) {
    pending_head = node->next;
    if (pending_head == nullptr) pending_tail = nullptr;
    node->next = nullptr;
  }
  return node;
}

void RuntimeCleanupState::requeue(PendingNode* node) noexcept {
  if (node == nullptr) return;
  node->next = nullptr;
  std::lock_guard<std::mutex> lock(queue_mutex);
  if (pending_tail == nullptr) {
    pending_head = node;
  } else {
    pending_tail->next = node;
  }
  pending_tail = node;
}

bool RuntimeCleanupState::empty() const noexcept {
  std::lock_guard<std::mutex> lock(queue_mutex);
  return pending_head == nullptr;
}

CleanupSupervisor& CleanupSupervisor::instance() {
  static CleanupSupervisor supervisor;
  return supervisor;
}

CleanupSupervisor::CleanupSupervisor() : worker_([this] { run(); }) {}

CleanupSupervisor::~CleanupSupervisor() noexcept {
  {
    std::lock_guard<std::mutex> lock(mutex_);
    stopping_ = true;
  }
  condition_.notify_all();
  if (worker_.joinable()) worker_.join();
}

void CleanupSupervisor::register_pending(
    const std::shared_ptr<RuntimeCleanupState>& state) noexcept {
  if (!state) return;
  {
    std::lock_guard<std::mutex> lock(mutex_);
    if (!state->registered) {
      state->supervisor_keepalive = state;
      state->supervisor_next = head_;
      state->registered = true;
      head_ = state.get();
    }
  }
  condition_.notify_one();
}

void CleanupSupervisor::wake() noexcept { condition_.notify_one(); }

bool CleanupSupervisor::unregister_if_empty(
    RuntimeCleanupState* state) noexcept {
  std::lock_guard<std::mutex> supervisor_lock(mutex_);
  std::lock_guard<std::mutex> queue_lock(state->queue_mutex);
  if (state->pending_head != nullptr) return false;

  RuntimeCleanupState** current = &head_;
  while (*current != nullptr && *current != state) {
    current = &(*current)->supervisor_next;
  }
  if (*current == state) {
    *current = state->supervisor_next;
    state->supervisor_next = nullptr;
    state->registered = false;
    state->supervisor_keepalive.reset();
  }
  return true;
}

void CleanupSupervisor::run() noexcept {
  auto backoff = std::chrono::milliseconds(10);
  for (;;) {
    std::shared_ptr<RuntimeCleanupState> state;
    {
      std::unique_lock<std::mutex> lock(mutex_);
      condition_.wait(lock, [this] { return stopping_ || head_ != nullptr; });
      if (stopping_) return;
      state = head_->supervisor_keepalive;
    }

    PendingNode* node = state->pop();
    if (node == nullptr) {
      (void)unregister_if_empty(state.get());
      continue;
    }

    const ui_status status = state->api->handle_release(node->token);
    if (status == kUiOk) {
      delete node;
      backoff = std::chrono::milliseconds(10);
    } else {
      state->requeue(node);
      std::unique_lock<std::mutex> lock(mutex_);
      condition_.wait_for(lock, backoff, [this] { return stopping_; });
      if (stopping_) return;
      backoff = std::min(backoff * 2, std::chrono::milliseconds(500));
    }
    (void)unregister_if_empty(state.get());
  }
}

std::shared_ptr<const AbiApi> linked_api() {
  static const auto api = std::make_shared<const AbiApi>(AbiApi{
      &api_runtime_create,
      &api_runtime_is_ui_thread,
      &api_runtime_set_ui_loop_bridge,
      &api_runtime_shutdown,
      &api_runtime_request_shutdown,
      &api_shutdown_request_query,
      &api_handle_release,
      &api_error_status,
      &api_error_code,
      &api_property_create_bool,
      &api_property_create_i64,
      &api_property_create_f64,
      &api_property_create_decimal,
      &api_property_create_utf8,
      &api_property_get_bool,
      &api_property_get_i64,
      &api_property_get_f64,
      &api_property_get_decimal,
      &api_property_get_utf8,
      &api_property_set_bool,
      &api_property_set_i64,
      &api_property_set_f64,
      &api_property_set_decimal,
      &api_property_set_utf8,
      &api_property_subscribe,
      &api_subscription_cancel,
      &api_binding_create,
      &api_binding_cancel,
      &api_binding_error_snapshot,
      &api_binding_subscribe_error,
  });
  return api;
}

UiError make_error(std::uint32_t status, ui_handle error_token,
                   std::shared_ptr<RuntimeCleanupState> state,
                   PendingNode* error_node) noexcept {
  std::optional<ErrorSnapshot> snapshot;
  if (error_token != 0 && error_node != nullptr) {
    snapshot = ErrorSnapshot(error_token, std::move(state), error_node);
  } else {
    delete error_node;
  }
  return UiError(status, std::move(snapshot));
}

Result<void> release_owned(std::uint64_t& token,
                           std::shared_ptr<RuntimeCleanupState>& state,
                           PendingNode*& node) noexcept {
  if (token == 0) return Result<void>::success();
  const ui_status status = state->api->handle_release(token);
  if (status != kUiOk) return Result<void>::failure(UiError(status));
  token = 0;
  delete node;
  node = nullptr;
  return Result<void>::success();
}

void release_in_destructor(std::uint64_t& token,
                           std::shared_ptr<RuntimeCleanupState>& state,
                           PendingNode*& node) noexcept {
  if (token == 0) {
    delete node;
    node = nullptr;
    return;
  }
  if (state->api->handle_release(token) == kUiOk) {
    token = 0;
    delete node;
    node = nullptr;
    return;
  }
  node->token = token;
  token = 0;
  (void)state->enqueue(node);
  node = nullptr;
}

Result<Runtime> RuntimeTestAccess::create(
    const std::shared_ptr<const AbiApi>& api, HostRegistration host) {
  if (!api || host.wake == nullptr) {
    return Result<Runtime>::failure(UiError(kUiInvalidArgument));
  }

  try {
    (void)CleanupSupervisor::instance();
    auto state = std::make_shared<RuntimeCleanupState>(api);
    auto runtime_node = std::unique_ptr<PendingNode>(new PendingNode());
    auto error_node = std::unique_ptr<PendingNode>(new PendingNode());
    ui_handle runtime = 0;
    ui_handle error = 0;
    ui_status status = api->runtime_create(&runtime, &error);
    if (status != kUiOk || runtime == 0) {
      return Result<Runtime>::failure(
          make_error(status == kUiOk ? kUiInternalError : status, error,
                     std::move(state), error_node.release()));
    }

    error = 0;
    status = api->runtime_set_ui_loop_bridge(
        runtime, host.wake, host.userdata, host.destroy, &error);
    if (status != kUiOk) {
      UiError failure = make_error(status, error, state, error_node.release());
      if (api->handle_release(runtime) == kUiOk) {
      } else {
        runtime_node->token = runtime;
        (void)state->enqueue(runtime_node.release());
      }
      return Result<Runtime>::failure(std::move(failure));
    }
    error_node.reset();
    return Result<Runtime>::success(
        Runtime(runtime, std::move(state), runtime_node.release()));
  } catch (const std::bad_alloc&) {
    return Result<Runtime>::failure(UiError(kUiOutOfMemory));
  } catch (...) {
    return Result<Runtime>::failure(UiError(kUiInternalError));
  }
}

}  // namespace freemvvm::detail

namespace freemvvm {

ErrorSnapshot::ErrorSnapshot(
    std::uint64_t token, std::shared_ptr<detail::RuntimeCleanupState> state,
    detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

ErrorSnapshot::ErrorSnapshot(ErrorSnapshot&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

ErrorSnapshot& ErrorSnapshot::operator=(ErrorSnapshot&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

ErrorSnapshot::~ErrorSnapshot() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

UiError::UiError(std::uint32_t raw_status,
                 std::optional<ErrorSnapshot> snapshot) noexcept
    : raw_status_(raw_status), snapshot_(std::move(snapshot)) {}

const ErrorSnapshot* UiError::snapshot() const noexcept {
  return snapshot_ ? &*snapshot_ : nullptr;
}

ShutdownResult::ShutdownResult(ShutdownDisposition disposition) noexcept
    : disposition_(disposition) {}

ShutdownResult::ShutdownResult(UiError error) noexcept
    : error_(std::move(error)) {}

ShutdownResult ShutdownResult::completed() noexcept {
  return ShutdownResult(ShutdownDisposition::kCompleted);
}

ShutdownResult ShutdownResult::accepted_pending() noexcept {
  return ShutdownResult(ShutdownDisposition::kAcceptedPending);
}

ShutdownResult ShutdownResult::failure(UiError error) noexcept {
  return ShutdownResult(std::move(error));
}

const UiError* ShutdownResult::error() const noexcept {
  return error_ ? &*error_ : nullptr;
}

ShutdownRequest::ShutdownRequest(
    std::uint64_t token, std::shared_ptr<detail::RuntimeCleanupState> state,
    detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

ShutdownRequest::ShutdownRequest(ShutdownRequest&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

ShutdownRequest& ShutdownRequest::operator=(ShutdownRequest&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

ShutdownRequest::~ShutdownRequest() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

HandlerRegistration::HandlerRegistration(
    std::weak_ptr<detail::RuntimeCleanupState> state,
    std::weak_ptr<detail::CallbackHandlerRecord> record) noexcept
    : state_(std::move(state)), record_(std::move(record)) {}

HandlerRegistration::HandlerRegistration(HandlerRegistration&& other) noexcept
    : state_(std::move(other.state_)), record_(std::move(other.record_)) {}

HandlerRegistration& HandlerRegistration::operator=(
    HandlerRegistration&& other) noexcept {
  if (this != &other) {
    (void)close();
    state_ = std::move(other.state_);
    record_ = std::move(other.record_);
  }
  return *this;
}

HandlerRegistration::~HandlerRegistration() noexcept { (void)close(); }

Result<void> HandlerRegistration::close() noexcept {
  auto state = state_.lock();
  auto record = record_.lock();
  if (state && record) {
    std::shared_ptr<detail::CallbackHandlerRecord> removed;
    {
      std::lock_guard<std::mutex> lock(state->callback_handler_mutex);
      if (state->callback_handler == record) {
        removed = std::move(state->callback_handler);
      }
    }
    removed.reset();
  }
  state_.reset();
  record_.reset();
  return Result<void>::success();
}

Result<ShutdownOutcome> ShutdownRequest::query() const {
  if (token_ == 0 || !state_) {
    return Result<ShutdownOutcome>::failure(UiError(2));
  }
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) {
    return Result<ShutdownOutcome>::failure(UiError(8));
  }
  std::uint32_t state = 0;
  std::uint32_t reason = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->shutdown_request_query(
      token_, &state, &reason, &error);
  if (status != 0) {
    return Result<ShutdownOutcome>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  if (state > static_cast<std::uint32_t>(ShutdownState::kHostUnavailable)) {
    return Result<ShutdownOutcome>::failure(UiError(9));
  }
  return Result<ShutdownOutcome>::success(
      ShutdownOutcome{static_cast<ShutdownState>(state), reason});
}

Result<void> ShutdownRequest::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

Runtime::Runtime(std::uint64_t token,
                 std::shared_ptr<detail::RuntimeCleanupState> state,
                 detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

Runtime::Runtime(Runtime&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

Runtime& Runtime::operator=(Runtime&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

Runtime::~Runtime() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<Runtime> Runtime::create(HostRegistration host) {
  try {
    return detail::RuntimeTestAccess::create(detail::linked_api(), host);
  } catch (const std::bad_alloc&) {
    return Result<Runtime>::failure(UiError(8));
  } catch (...) {
    return Result<Runtime>::failure(UiError(9));
  }
}

Result<bool> Runtime::is_ui_thread() const {
  if (token_ == 0 || !state_) return Result<bool>::failure(UiError(2));
  ui_bool result = 0;
  const ui_status status = state_->api->runtime_is_ui_thread(token_, &result);
  if (status != 0) return Result<bool>::failure(UiError(status));
  return Result<bool>::success(result != 0);
}

ShutdownResult Runtime::shutdown() const {
  if (token_ == 0 || !state_) return ShutdownResult::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return ShutdownResult::failure(UiError(8));
  ui_handle error = 0;
  const ui_status status = state_->api->runtime_shutdown(token_, &error);
  if (status == 0) {
    error_node.reset();
    return ShutdownResult::completed();
  }
  if (status == 14) {
    error_node.reset();
    return ShutdownResult::accepted_pending();
  }
  return ShutdownResult::failure(detail::make_error(
      status, error, state_, error_node.release()));
}

Result<ShutdownRequest> Runtime::request_shutdown() const {
  if (token_ == 0 || !state_) {
    return Result<ShutdownRequest>::failure(UiError(2));
  }
  std::unique_ptr<detail::PendingNode> request_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!request_node || !error_node) {
    return Result<ShutdownRequest>::failure(UiError(8));
  }
  ui_handle request = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->runtime_request_shutdown(
      token_, detail::kShutdownAsync, &request, &error);
  if ((status != 0 && status != 14) || request == 0) {
    return Result<ShutdownRequest>::failure(detail::make_error(
        status == 0 || status == 14 ? 9 : status, error, state_,
        error_node.release()));
  }
  error_node.reset();
  request_node->token = request;
  return Result<ShutdownRequest>::success(ShutdownRequest(
      request, state_, request_node.release()));
}

Result<HandlerRegistration> Runtime::set_callback_handler(
    CallbackHandler handler, void* userdata, CallbackHandlerDestroy destroy) {
  if (token_ == 0 || !state_) {
    return Result<HandlerRegistration>::failure(UiError(2));
  }
  if (handler == nullptr && (userdata != nullptr || destroy != nullptr)) {
    return Result<HandlerRegistration>::failure(UiError(1));
  }
  if (handler == nullptr) {
    std::shared_ptr<detail::CallbackHandlerRecord> removed;
    {
      std::lock_guard<std::mutex> lock(state_->callback_handler_mutex);
      removed = std::move(state_->callback_handler);
    }
    removed.reset();
    return Result<HandlerRegistration>::success(
        HandlerRegistration(state_, {}));
  }

  std::shared_ptr<detail::CallbackHandlerRecord> record;
  try {
    record = std::make_shared<detail::CallbackHandlerRecord>();
  } catch (const std::bad_alloc&) {
    return Result<HandlerRegistration>::failure(UiError(8));
  } catch (...) {
    return Result<HandlerRegistration>::failure(UiError(9));
  }
  record->handler = handler;
  record->userdata = userdata;
  record->destroy = destroy;

  std::shared_ptr<detail::CallbackHandlerRecord> removed;
  {
    std::lock_guard<std::mutex> lock(state_->callback_handler_mutex);
    removed = std::exchange(state_->callback_handler, record);
  }
  removed.reset();
  return Result<HandlerRegistration>::success(
      HandlerRegistration(state_, record));
}

Result<void> Runtime::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

}  // namespace freemvvm

namespace freemvvm {

template <typename T>
Property<T>::Property(std::uint64_t token,
                     std::shared_ptr<detail::RuntimeCleanupState> state,
                     detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

template <typename T>
Property<T>::Property(Property&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

template <typename T>
Property<T>& Property<T>::operator=(Property&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

template <typename T>
Property<T>::~Property() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

template <typename T>
Result<Property<T>> Property<T>::create(Runtime& runtime, T initial_value) {
  if (runtime.token_ == 0 || !runtime.state_) {
    return Result<Property<T>>::failure(UiError(2));
  }
  std::unique_ptr<detail::PendingNode> property_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!property_node || !error_node) {
    return Result<Property<T>>::failure(UiError(8));
  }

  ui_handle property = 0;
  ui_handle error = 0;
  ui_status status;
  if constexpr (std::is_same<T, bool>::value) {
    status = runtime.state_->api->property_create_bool(
        runtime.token_, initial_value ? 1 : 0, &property, &error);
  } else if constexpr (std::is_same<T, std::int64_t>::value) {
    status = runtime.state_->api->property_create_i64(
        runtime.token_, initial_value, &property, &error);
  } else if constexpr (std::is_same<T, double>::value) {
    status = runtime.state_->api->property_create_f64(
        runtime.token_, initial_value, &property, &error);
  } else if constexpr (std::is_same<T, Decimal64>::value) {
    status = runtime.state_->api->property_create_decimal(
        runtime.token_, initial_value.bits_, &property, &error);
  } else {
    const auto* bytes = initial_value.empty()
                            ? nullptr
                            : reinterpret_cast<const std::uint8_t*>(
                                  initial_value.data());
    status = runtime.state_->api->property_create_utf8(
        runtime.token_, bytes,
        static_cast<std::uint64_t>(initial_value.size()), &property, &error);
  }
  if (status != 0 || property == 0) {
    return Result<Property<T>>::failure(detail::make_error(
        status == 0 ? 9 : status, error, runtime.state_, error_node.release()));
  }
  error_node.reset();
  property_node->token = property;
  return Result<Property<T>>::success(Property<T>(
      property, runtime.state_, property_node.release()));
}

template <typename T>
Result<T> Property<T>::get() const {
  if (token_ == 0 || !state_) return Result<T>::failure(UiError(2));
  if constexpr (std::is_same<T, std::string>::value) {
    try {
      for (;;) {
        std::uint64_t required_length = 0;
        ui_status status = state_->api->property_get_utf8(
            token_, nullptr, 0, &required_length);
        if (status != 0) return Result<T>::failure(UiError(status));

        std::string value;
        if (required_length > value.max_size()) {
          return Result<T>::failure(UiError(8));
        }
        value.resize(static_cast<std::size_t>(required_length));
        if (required_length == 0) return Result<T>::success(std::move(value));

        std::uint64_t actual_length = 0;
        auto* buffer = reinterpret_cast<std::uint8_t*>(value.data());
        status = state_->api->property_get_utf8(
            token_, buffer, required_length, &actual_length);
        if (status == 7) continue;
        if (status != 0) return Result<T>::failure(UiError(status));
        if (actual_length > required_length) {
          return Result<T>::failure(UiError(9));
        }
        value.resize(static_cast<std::size_t>(actual_length));
        return Result<T>::success(std::move(value));
      }
    } catch (const std::bad_alloc&) {
      return Result<T>::failure(UiError(8));
    } catch (const std::length_error&) {
      return Result<T>::failure(UiError(8));
    }
  } else if constexpr (std::is_same<T, Decimal64>::value) {
    std::uint64_t bits = 0;
    const ui_status status = state_->api->property_get_decimal(token_, &bits);
    if (status != 0) return Result<T>::failure(UiError(status));
    return Result<T>::success(Decimal64(bits));
  } else {
    T value{};
    ui_status status;
    if constexpr (std::is_same<T, bool>::value) {
      ui_bool raw_value = 0;
      status = state_->api->property_get_bool(token_, &raw_value);
      value = raw_value != 0;
    } else if constexpr (std::is_same<T, std::int64_t>::value) {
      status = state_->api->property_get_i64(token_, &value);
    } else {
      status = state_->api->property_get_f64(token_, &value);
    }
    if (status != 0) return Result<T>::failure(UiError(status));
    return Result<T>::success(value);
  }
}

template <typename T>
Result<void> Property<T>::set(T value) {
  if (token_ == 0 || !state_) return Result<void>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(8));
  ui_handle error = 0;
  ui_status status;
  if constexpr (std::is_same<T, bool>::value) {
    status = state_->api->property_set_bool(token_, value ? 1 : 0, &error);
  } else if constexpr (std::is_same<T, std::int64_t>::value) {
    status = state_->api->property_set_i64(token_, value, &error);
  } else if constexpr (std::is_same<T, double>::value) {
    status = state_->api->property_set_f64(token_, value, &error);
  } else if constexpr (std::is_same<T, Decimal64>::value) {
    status = state_->api->property_set_decimal(token_, value.bits_, &error);
  } else {
    const auto* bytes = value.empty()
                            ? nullptr
                            : reinterpret_cast<const std::uint8_t*>(
                                  value.data());
    status = state_->api->property_set_utf8(
        token_, bytes, static_cast<std::uint64_t>(value.size()), &error);
  }
  if (status != 0) {
    return Result<void>::failure(
        detail::make_error(status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

template <typename T>
Result<Subscription> Property<T>::subscribe(std::function<void()> callback) {
  if (token_ == 0 || !state_) {
    return Result<Subscription>::failure(UiError(2));
  }
  if (!callback) return Result<Subscription>::failure(UiError(1));

  std::unique_ptr<detail::PendingNode> subscription_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  auto callback_state = std::unique_ptr<detail::SubscriptionCallbackState>(
      new (std::nothrow)
          detail::SubscriptionCallbackState(
              std::move(callback), state_,
              CallbackKind::kPropertyNotification));
  if (!subscription_node || !error_node || !callback_state) {
    return Result<Subscription>::failure(UiError(8));
  }

  ui_handle subscription = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->property_subscribe(
      token_, &detail::subscription_callback, callback_state.get(),
      &detail::subscription_destroy, &subscription, &error);
  if (status != 0) {
    return Result<Subscription>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (subscription == 0) {
    return Result<Subscription>::failure(
        detail::make_error(9, error, state_, error_node.release()));
  }

  callback_state.release();
  error_node.reset();
  return Result<Subscription>::success(Subscription(
      subscription, state_, subscription_node.release()));
}

template <typename T>
Result<std::optional<Binding<T>>> Property<T>::bind_to(
    Property<T>& target, BindingMode mode) {
  if (token_ == 0 || !state_ || target.token_ == 0 || !target.state_) {
    return Result<std::optional<Binding<T>>>::failure(UiError(2));
  }

  std::unique_ptr<detail::PendingNode> binding_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!binding_node || !error_node) {
    return Result<std::optional<Binding<T>>>::failure(UiError(8));
  }

  ui_handle binding = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->binding_create(
      token_, target.token_, static_cast<std::uint32_t>(mode), &binding,
      &error);
  if (status != 0) {
    if (binding != 0) {
      binding_node->token = binding;
      (void)state_->enqueue(binding_node.release());
    }
    return Result<std::optional<Binding<T>>>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (error != 0) {
    if (binding != 0) {
      binding_node->token = binding;
      (void)state_->enqueue(binding_node.release());
    } else {
      binding_node.reset();
    }
    return Result<std::optional<Binding<T>>>::failure(detail::make_error(
        9, error, state_, error_node.release()));
  }

  error_node.reset();
  if (mode == BindingMode::kOnce) {
    if (binding != 0) {
      binding_node->token = binding;
      (void)state_->enqueue(binding_node.release());
      return Result<std::optional<Binding<T>>>::failure(UiError(9));
    }
    binding_node.reset();
    return Result<std::optional<Binding<T>>>::success(std::nullopt);
  }
  if (binding == 0) {
    binding_node.reset();
    return Result<std::optional<Binding<T>>>::failure(UiError(9));
  }

  Binding<T> owned_binding(binding, state_, binding_node.release());
  std::optional<Binding<T>> result(std::move(owned_binding));
  return Result<std::optional<Binding<T>>>::success(std::move(result));
}

template <typename T>
Result<void> Property<T>::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

template class Property<bool>;
template class Property<std::int64_t>;
template class Property<double>;
template class Property<Decimal64>;
template class Property<std::string>;

template <typename T>
Binding<T>::Binding(std::uint64_t token,
                    std::shared_ptr<detail::RuntimeCleanupState> state,
                    detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

template <typename T>
Binding<T>::Binding(Binding&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

template <typename T>
Binding<T>& Binding<T>::operator=(Binding&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

template <typename T>
Binding<T>::~Binding() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

template <typename T>
Result<void> Binding<T>::cancel() noexcept {
  if (token_ == 0 || !state_) return Result<void>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(8));
  ui_handle error = 0;
  const ui_status status = state_->api->binding_cancel(token_, &error);
  if (status != 0) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

template <typename T>
Result<void> Binding<T>::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

template <typename T>
Result<std::optional<BindingError<T>>> Binding<T>::error_snapshot() const {
  if (token_ == 0 || !state_) {
    return Result<std::optional<BindingError<T>>>::failure(UiError(2));
  }
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> property_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node || !property_node) {
    return Result<std::optional<BindingError<T>>>::failure(UiError(8));
  }

  ui_handle error = 0;
  ui_handle rejected_property = 0;
  std::uint32_t raw_direction = 0;
  const ui_status status = state_->api->binding_error_snapshot(
      token_, &error, &rejected_property, &raw_direction);
  if (status != 0) {
    return Result<std::optional<BindingError<T>>>::failure(UiError(status));
  }
  if (error == 0 && rejected_property == 0 && raw_direction == 0) {
    return Result<std::optional<BindingError<T>>>::success(std::nullopt);
  }
  if (error == 0 || rejected_property == 0 ||
      (raw_direction != 1 && raw_direction != 2)) {
    if (error != 0) {
      error_node->token = error;
      (void)state_->enqueue(error_node.release());
    }
    if (rejected_property != 0) {
      property_node->token = rejected_property;
      (void)state_->enqueue(property_node.release());
    }
    return Result<std::optional<BindingError<T>>>::failure(UiError(9));
  }

  std::uint32_t error_status = 0;
  const ui_status read_status = state_->api->error_status(error, &error_status);
  if (read_status != 0) error_status = read_status;
  UiError error_value = detail::make_error(
      error_status, error, state_, error_node.release());
  BindingError<T> snapshot(
      std::move(error_value), rejected_property,
      raw_direction == 1 ? BindingDirection::kSourceToTarget
                         : BindingDirection::kTargetToSource,
      state_, property_node.release());
  return Result<std::optional<BindingError<T>>>::success(
      std::optional<BindingError<T>>(std::move(snapshot)));
}

template <typename T>
Result<Subscription> Binding<T>::subscribe_error(
    std::function<void()> callback) {
  if (token_ == 0 || !state_) return Result<Subscription>::failure(UiError(2));
  if (!callback) return Result<Subscription>::failure(UiError(1));

  std::unique_ptr<detail::PendingNode> subscription_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  auto callback_state = std::unique_ptr<detail::SubscriptionCallbackState>(
      new (std::nothrow)
          detail::SubscriptionCallbackState(
              std::move(callback), state_,
              CallbackKind::kBindingErrorNotification));
  if (!subscription_node || !error_node || !callback_state) {
    return Result<Subscription>::failure(UiError(8));
  }

  ui_handle subscription = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->binding_subscribe_error(
      token_, &detail::subscription_callback, callback_state.get(),
      &detail::subscription_destroy, &subscription, &error);
  if (status != 0) {
    return Result<Subscription>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (subscription == 0) {
    return Result<Subscription>::failure(detail::make_error(
        9, error, state_, error_node.release()));
  }

  callback_state.release();
  error_node.reset();
  return Result<Subscription>::success(Subscription(
      subscription, state_, subscription_node.release()));
}

template class Binding<bool>;
template class Binding<std::int64_t>;
template class Binding<double>;
template class Binding<Decimal64>;
template class Binding<std::string>;

Subscription::Subscription(
    std::uint64_t token, std::shared_ptr<detail::RuntimeCleanupState> state,
    detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

Subscription::Subscription(Subscription&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

Subscription& Subscription::operator=(Subscription&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

Subscription::~Subscription() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<void> Subscription::cancel() noexcept {
  if (token_ == 0 || !state_) return Result<void>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(8));
  ui_handle error = 0;
  const ui_status status = state_->api->subscription_cancel(token_, &error);
  if (status != 0) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

Result<void> Subscription::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

}  // namespace freemvvm
