#include "freemvvm/property.hpp"
#include "freemvvm/runtime.hpp"

#include "runtime_internal.hpp"

#include <algorithm>
#include <chrono>
#include <new>
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

ui_status api_property_get_bool(ui_handle property, ui_bool* out_value) {
  return ui_abi_1_property_get_bool(property, out_value);
}

ui_status api_property_get_i64(ui_handle property, std::int64_t* out_value) {
  return ui_abi_1_property_get_i64(property, out_value);
}

ui_status api_property_set_bool(ui_handle property, ui_bool value,
                                ui_handle* out_error) {
  return ui_abi_1_property_set_bool(property, value, out_error);
}

ui_status api_property_set_i64(ui_handle property, std::int64_t value,
                               ui_handle* out_error) {
  return ui_abi_1_property_set_i64(property, value, out_error);
}

}  // namespace

RuntimeCleanupState::RuntimeCleanupState(std::shared_ptr<const AbiApi> api_value)
    : api(std::move(api_value)) {}

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
      &api_property_get_bool,
      &api_property_get_i64,
      &api_property_set_bool,
      &api_property_set_i64,
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
  } else {
    status = runtime.state_->api->property_create_i64(
        runtime.token_, initial_value, &property, &error);
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
  T value{};
  ui_status status;
  if constexpr (std::is_same<T, bool>::value) {
    ui_bool raw_value = 0;
    status = state_->api->property_get_bool(token_, &raw_value);
    value = raw_value != 0;
  } else {
    status = state_->api->property_get_i64(token_, &value);
  }
  if (status != 0) return Result<T>::failure(UiError(status));
  return Result<T>::success(value);
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
  } else {
    status = state_->api->property_set_i64(token_, value, &error);
  }
  if (status != 0) {
    return Result<void>::failure(
        detail::make_error(status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

template <typename T>
Result<void> Property<T>::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

template class Property<bool>;
template class Property<std::int64_t>;

}  // namespace freemvvm
