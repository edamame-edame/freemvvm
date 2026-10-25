#include "freemvvm/command.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <exception>
#include <memory>
#include <new>

namespace freemvvm {
namespace {

struct CommandCallbackState final {
  Command::Start start;
  std::shared_ptr<detail::RuntimeCleanupState> runtime;
  std::uint64_t runtime_token = 0;
  detail::PendingNode* runtime_cleanup_node = nullptr;

  ~CommandCallbackState() noexcept {
    detail::release_in_destructor(runtime_token, runtime,
                                  runtime_cleanup_node);
  }
};

enum class PostedTerminal : std::uint8_t {
  kSuccess,
  kFailure,
  kCancelled,
};

struct TerminalPostRecord final {
  PostedTerminal kind = PostedTerminal::kSuccess;
  std::uint64_t command_token = 0;
  std::uint64_t cancel_token = 0;
  std::uint64_t runtime_token = 0;
  std::uint64_t operation_id = 0;
  std::uint32_t domain = 0;
  std::uint32_t code = 0;
  std::string message;
  std::shared_ptr<detail::RuntimeCleanupState> runtime;
  detail::PendingNode* command_cleanup_node = nullptr;
  detail::PendingNode* cancel_cleanup_node = nullptr;
  detail::PendingNode* runtime_cleanup_node = nullptr;

  ~TerminalPostRecord() noexcept {
    detail::release_in_destructor(command_token, runtime,
                                  command_cleanup_node);
    detail::release_in_destructor(cancel_token, runtime,
                                  cancel_cleanup_node);
    detail::release_in_destructor(runtime_token, runtime,
                                  runtime_cleanup_node);
  }

  void run() noexcept {
    if (runtime == nullptr || command_token == 0) return;
    switch (kind) {
      case PostedTerminal::kSuccess:
        (void)runtime->api->command_complete_success(command_token,
                                                    operation_id);
        break;
      case PostedTerminal::kFailure: {
        const auto* bytes = message.empty()
                                ? nullptr
                                : reinterpret_cast<const std::uint8_t*>(
                                      message.data());
        (void)runtime->api->command_complete_failure_utf8(
            command_token, operation_id, domain, code, bytes,
            static_cast<std::uint64_t>(message.size()));
        break;
      }
      case PostedTerminal::kCancelled:
        (void)runtime->api->command_complete_cancelled(command_token,
                                                      operation_id);
        break;
    }
  }

  void restore(std::uint64_t& command, std::uint64_t& cancel,
               std::uint64_t& runtime_token_value,
               std::shared_ptr<detail::RuntimeCleanupState>& state,
               detail::PendingNode*& command_node,
               detail::PendingNode*& cancel_node,
               detail::PendingNode*& runtime_node) noexcept {
    command = std::exchange(command_token, 0);
    cancel = std::exchange(cancel_token, 0);
    runtime_token_value = std::exchange(runtime_token, 0);
    state = std::move(runtime);
    command_node = std::exchange(command_cleanup_node, nullptr);
    cancel_node = std::exchange(cancel_cleanup_node, nullptr);
    runtime_node = std::exchange(runtime_cleanup_node, nullptr);
  }
};

void command_callback_destroy(void* userdata) noexcept {
  delete static_cast<CommandCallbackState*>(userdata);
}

void terminal_post_callback(void* userdata) noexcept {
  static_cast<TerminalPostRecord*>(userdata)->run();
}

void terminal_post_destroy(void* userdata) noexcept {
  delete static_cast<TerminalPostRecord*>(userdata);
}

}  // namespace

std::uint32_t Command::start_trampoline(void* userdata,
                                        std::uint64_t borrowed_command,
                                        std::uint64_t operation_id,
                                        std::uint64_t borrowed_cancel_token)
    noexcept {
  auto* callback = static_cast<CommandCallbackState*>(userdata);
  auto runtime_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto command_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto cancel_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!runtime_node || !command_node || !cancel_node) {
    return UI_OUT_OF_MEMORY;
  }

  ui_handle runtime_token = 0;
  ui_status status = callback->runtime->api->handle_retain(
      callback->runtime_token, &runtime_token);
  if (status != UI_OK || runtime_token == 0) {
    if (runtime_token != 0) {
      auto* cleanup = runtime_node.release();
      detail::release_in_destructor(runtime_token, callback->runtime,
                                    cleanup);
    }
    return status == UI_OK ? UI_INTERNAL_ERROR : status;
  }
  runtime_node->token = runtime_token;

  ui_handle command_token = 0;
  status = callback->runtime->api->handle_retain(borrowed_command,
                                                &command_token);
  if (status != UI_OK || command_token == 0) {
    if (command_token != 0) {
      auto* cleanup = command_node.release();
      detail::release_in_destructor(command_token, callback->runtime,
                                    cleanup);
    }
    auto* cleanup = runtime_node.release();
    detail::release_in_destructor(runtime_token, callback->runtime,
                                  cleanup);
    return status == UI_OK ? UI_INTERNAL_ERROR : status;
  }
  command_node->token = command_token;

  ui_handle cancel_token = 0;
  status = callback->runtime->api->handle_retain(borrowed_cancel_token,
                                                &cancel_token);
  if (status != UI_OK || cancel_token == 0) {
    if (cancel_token != 0) {
      auto* cleanup = cancel_node.release();
      detail::release_in_destructor(cancel_token, callback->runtime,
                                    cleanup);
    }
    auto* command_cleanup = command_node.release();
    detail::release_in_destructor(command_token, callback->runtime,
                                  command_cleanup);
    auto* runtime_cleanup = runtime_node.release();
    detail::release_in_destructor(runtime_token, callback->runtime,
                                  runtime_cleanup);
    return status == UI_OK ? UI_INTERNAL_ERROR : status;
  }
  cancel_node->token = cancel_token;

  CommandWorker worker(runtime_token, command_token, cancel_token,
                       operation_id, callback->runtime,
                       runtime_node.release(), command_node.release(),
                       cancel_node.release());
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  try {
    auto result = callback->start(std::move(worker));
    if (result.has_value()) return UI_OK;
    const auto* error = result.error();
    return error != nullptr && error->raw_status() != UI_OK
               ? error->raw_status()
               : UI_INTERNAL_ERROR;
  } catch (...) {
    callback->runtime->dispatch_callback_failure(CallbackFailure{
        CallbackKind::kCommandStart, std::current_exception()});
    return UI_INTERNAL_ERROR;
  }
#else
  auto result = callback->start(std::move(worker));
  if (result.has_value()) return UI_OK;
  const auto* error = result.error();
  return error != nullptr && error->raw_status() != UI_OK
             ? error->raw_status()
             : UI_INTERNAL_ERROR;
#endif
}

CommandWorker::CommandWorker(
    std::uint64_t runtime_token, std::uint64_t command_token,
    std::uint64_t cancel_token, std::uint64_t operation_id,
    std::shared_ptr<detail::RuntimeCleanupState> state,
    detail::PendingNode* runtime_cleanup_node,
    detail::PendingNode* command_cleanup_node,
    detail::PendingNode* cancel_cleanup_node) noexcept
    : command_token_(command_token),
      cancel_token_(cancel_token),
      operation_id_(operation_id),
      runtime_token_(runtime_token),
      state_(std::move(state)),
      command_cleanup_node_(command_cleanup_node),
      cancel_cleanup_node_(cancel_cleanup_node),
      runtime_cleanup_node_(runtime_cleanup_node) {}

CommandWorker::CommandWorker(CommandWorker&& other) noexcept
    : command_token_(std::exchange(other.command_token_, 0)),
      cancel_token_(std::exchange(other.cancel_token_, 0)),
      operation_id_(std::exchange(other.operation_id_, 0)),
      runtime_token_(std::exchange(other.runtime_token_, 0)),
      state_(std::move(other.state_)),
      command_cleanup_node_(
          std::exchange(other.command_cleanup_node_, nullptr)),
      cancel_cleanup_node_(std::exchange(other.cancel_cleanup_node_, nullptr)),
      runtime_cleanup_node_(std::exchange(other.runtime_cleanup_node_, nullptr)) {
}

CommandWorker& CommandWorker::operator=(CommandWorker&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(command_token_, state_,
                                  command_cleanup_node_);
    detail::release_in_destructor(cancel_token_, state_, cancel_cleanup_node_);
    detail::release_in_destructor(runtime_token_, state_, runtime_cleanup_node_);
    command_token_ = std::exchange(other.command_token_, 0);
    cancel_token_ = std::exchange(other.cancel_token_, 0);
    operation_id_ = std::exchange(other.operation_id_, 0);
    runtime_token_ = std::exchange(other.runtime_token_, 0);
    state_ = std::move(other.state_);
    command_cleanup_node_ =
        std::exchange(other.command_cleanup_node_, nullptr);
    cancel_cleanup_node_ = std::exchange(other.cancel_cleanup_node_, nullptr);
    runtime_cleanup_node_ =
        std::exchange(other.runtime_cleanup_node_, nullptr);
  }
  return *this;
}

CommandWorker::~CommandWorker() noexcept {
  detail::release_in_destructor(command_token_, state_,
                                command_cleanup_node_);
  detail::release_in_destructor(cancel_token_, state_, cancel_cleanup_node_);
  detail::release_in_destructor(runtime_token_, state_, runtime_cleanup_node_);
}

Result<bool> CommandWorker::is_cancellation_requested() const {
  if (cancel_token_ == 0 || state_ == nullptr) {
    return Result<bool>::failure(UiError(UI_INVALID_HANDLE));
  }
  ui_bool requested = 0;
  const ui_status status =
      state_->api->command_cancel_token_requested(cancel_token_, &requested);
  if (status != UI_OK) return Result<bool>::failure(UiError(status));
  if (requested > 1) return Result<bool>::failure(UiError(UI_INTERNAL_ERROR));
  return Result<bool>::success(requested != 0);
}

Result<void> CommandWorker::report_success() {
  return post(TerminalKind::kSuccess, 0, 0, {});
}

Result<void> CommandWorker::report_failure(std::uint32_t domain,
                                          std::uint32_t code,
                                          const std::string& message) {
  try {
    return post(TerminalKind::kFailure, domain, code, std::string(message));
  } catch (const std::bad_alloc&) {
    return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  } catch (const std::length_error&) {
    return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  }
}

Result<void> CommandWorker::report_cancelled() {
  return post(TerminalKind::kCancelled, 0, 0, {});
}

Result<void> CommandWorker::post(TerminalKind kind, std::uint32_t domain,
                                 std::uint32_t code, std::string message) {
  if (command_token_ == 0 || cancel_token_ == 0 || runtime_token_ == 0 ||
      state_ == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto record = std::unique_ptr<TerminalPostRecord>(
      new (std::nothrow) TerminalPostRecord());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!record || !error_node) {
    return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  }

  record->kind = kind == TerminalKind::kSuccess
                     ? PostedTerminal::kSuccess
                     : kind == TerminalKind::kFailure
                           ? PostedTerminal::kFailure
                           : PostedTerminal::kCancelled;
  record->operation_id = operation_id_;
  record->domain = domain;
  record->code = code;
  record->message = std::move(message);
  record->command_token = std::exchange(command_token_, 0);
  record->cancel_token = std::exchange(cancel_token_, 0);
  record->runtime_token = std::exchange(runtime_token_, 0);
  record->runtime = std::move(state_);
  record->command_cleanup_node =
      std::exchange(command_cleanup_node_, nullptr);
  record->cancel_cleanup_node =
      std::exchange(cancel_cleanup_node_, nullptr);
  record->runtime_cleanup_node =
      std::exchange(runtime_cleanup_node_, nullptr);

  ui_handle error = 0;
  const ui_status status = record->runtime->api->runtime_post(
      record->runtime_token,
      &terminal_post_callback, record.get(), &terminal_post_destroy, &error);
  if (status == UI_OK && error == 0) {
    (void)record.release();
    return Result<void>::success();
  }

  if (status == UI_OK) {
    auto state = record->runtime;
    (void)record.release();
    return Result<void>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, std::move(state), error_node.release()));
  }

  record->restore(command_token_, cancel_token_, runtime_token_, state_,
                  command_cleanup_node_, cancel_cleanup_node_,
                  runtime_cleanup_node_);
  return Result<void>::failure(detail::make_error(
      status, error, state_, error_node.release()));
}

Command::Command(std::uint64_t token,
                 std::shared_ptr<detail::RuntimeCleanupState> state,
                 detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

Command::Command(Command&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

Command& Command::operator=(Command&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

Command::~Command() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<Command> Command::create_async(Runtime& runtime,
                                      Property<bool>* eligibility,
                                      Start start) {
  if (runtime.token_ == 0 || runtime.state_ == nullptr) {
    return Result<Command>::failure(UiError(UI_INVALID_HANDLE));
  }
  if (!start) return Result<Command>::failure(UiError(UI_INVALID_ARGUMENT));
  if (eligibility != nullptr) {
    if (eligibility->token_ == 0 || eligibility->state_ == nullptr) {
      return Result<Command>::failure(UiError(UI_INVALID_HANDLE));
    }
    if (eligibility->state_.get() != runtime.state_.get()) {
      return Result<Command>::failure(UiError(UI_WRONG_RUNTIME));
    }
  }

  auto command_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto runtime_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto callback_state = std::unique_ptr<CommandCallbackState>(
      new (std::nothrow) CommandCallbackState{std::move(start),
                                              runtime.state_});
  if (!command_node || !error_node || !runtime_node || !callback_state) {
    return Result<Command>::failure(UiError(UI_OUT_OF_MEMORY));
  }

  ui_handle retained_runtime = 0;
  const ui_status retain_status = runtime.state_->api->handle_retain(
      runtime.token_, &retained_runtime);
  if (retain_status != UI_OK || retained_runtime == 0) {
    if (retained_runtime != 0) {
      auto* cleanup = runtime_node.release();
      detail::release_in_destructor(retained_runtime, runtime.state_,
                                    cleanup);
    }
    return Result<Command>::failure(
        UiError(retain_status == UI_OK ? UI_INTERNAL_ERROR : retain_status));
  }
  callback_state->runtime_token = retained_runtime;
  callback_state->runtime_cleanup_node = runtime_node.release();

  ui_handle command = 0;
  ui_handle error = 0;
  const ui_status status = runtime.state_->api->command_create_async(
      runtime.token_, eligibility == nullptr ? 0 : eligibility->token_,
      &Command::start_trampoline, callback_state.get(),
      &command_callback_destroy, &command, &error);
  if (status != UI_OK) {
    if (command != 0) {
      command_node->token = command;
      (void)runtime.state_->enqueue(command_node.release());
    }
    return Result<Command>::failure(detail::make_error(
        status, error, runtime.state_, error_node.release()));
  }
  if (command == 0 || error != 0) {
    if (command != 0) {
      command_node->token = command;
      (void)runtime.state_->enqueue(command_node.release());
    }
    callback_state.release();
    return Result<Command>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, runtime.state_, error_node.release()));
  }
  (void)callback_state.release();
  command_node->token = command;
  return Result<Command>::success(
      Command(command, runtime.state_, command_node.release()));
}

Result<std::uint64_t> Command::execute() {
  if (token_ == 0 || state_ == nullptr) {
    return Result<std::uint64_t>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) {
    return Result<std::uint64_t>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  std::uint64_t operation_id = 0;
  ui_handle error = 0;
  const ui_status status = state_->api->command_execute(
      token_, &operation_id, &error);
  if (status != UI_OK) {
    return Result<std::uint64_t>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (operation_id == 0 || error != 0) {
    return Result<std::uint64_t>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<std::uint64_t>::success(operation_id);
}

Result<void> Command::cancel(std::uint64_t operation_id) {
  if (token_ == 0 || state_ == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status =
      state_->api->command_cancel(token_, operation_id, &error);
  if (status != UI_OK) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (error != 0) {
    return Result<void>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

Result<Property<std::int64_t>> Command::state_property() const {
  if (token_ == 0 || state_ == nullptr) {
    return Result<Property<std::int64_t>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto property_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!property_node) {
    return Result<Property<std::int64_t>>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle property = 0;
  const ui_status status =
      state_->api->command_state_property(token_, &property);
  if (status != UI_OK || property == 0) {
    if (property != 0) {
      property_node->token = property;
      (void)state_->enqueue(property_node.release());
    }
    return Result<Property<std::int64_t>>::failure(
        UiError(status == UI_OK ? UI_INTERNAL_ERROR : status));
  }
  property_node->token = property;
  return Result<Property<std::int64_t>>::success(
      Property<std::int64_t>(property, state_, property_node.release()));
}

Result<Property<bool>> Command::can_execute_property() const {
  if (token_ == 0 || state_ == nullptr) {
    return Result<Property<bool>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto property_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!property_node) {
    return Result<Property<bool>>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle property = 0;
  const ui_status status =
      state_->api->command_can_execute_property(token_, &property);
  if (status != UI_OK || property == 0) {
    if (property != 0) {
      property_node->token = property;
      (void)state_->enqueue(property_node.release());
    }
    return Result<Property<bool>>::failure(
        UiError(status == UI_OK ? UI_INTERNAL_ERROR : status));
  }
  property_node->token = property;
  return Result<Property<bool>>::success(
      Property<bool>(property, state_, property_node.release()));
}

Result<std::optional<ErrorSnapshot>> Command::failure_snapshot() const {
  if (token_ == 0 || state_ == nullptr) {
    return Result<std::optional<ErrorSnapshot>>::failure(
        UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node) {
    return Result<std::optional<ErrorSnapshot>>::failure(
        UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle snapshot = 0;
  const ui_status status =
      state_->api->command_failure_snapshot(token_, &snapshot);
  if (status != UI_OK) {
    if (snapshot != 0) {
      auto state = state_;
      auto* cleanup = cleanup_node.release();
      detail::release_in_destructor(snapshot, state, cleanup);
    }
    return Result<std::optional<ErrorSnapshot>>::failure(UiError(status));
  }
  if (snapshot == 0) {
    cleanup_node.reset();
    return Result<std::optional<ErrorSnapshot>>::success(std::nullopt);
  }
  ErrorSnapshot owned_snapshot(snapshot, state_, cleanup_node.release());
  std::optional<ErrorSnapshot> result(std::move(owned_snapshot));
  return Result<std::optional<ErrorSnapshot>>::success(std::move(result));
}

Result<void> Command::close() noexcept {
  if (token_ == 0 || state_ == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state_->api->command_close(token_, &error);
  if (status != UI_OK) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (error != 0) {
    return Result<void>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

Result<void> Command::release() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

}  // namespace freemvvm
