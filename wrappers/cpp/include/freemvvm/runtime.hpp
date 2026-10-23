#ifndef FREEMVVM_CPP_RUNTIME_HPP_
#define FREEMVVM_CPP_RUNTIME_HPP_

#include <cstdint>
#include <exception>
#include <memory>
#include <optional>
#include <type_traits>
#include <utility>
#include <variant>

namespace freemvvm {

class UiError;
class HandlerRegistration;
template <typename T>
class Property;

namespace detail {
struct RuntimeCleanupState;
struct CallbackHandlerRecord;
struct PendingNode;
struct RuntimeTestAccess;
UiError make_error(std::uint32_t status, std::uint64_t error_token,
                   std::shared_ptr<RuntimeCleanupState> state,
                   PendingNode* error_node) noexcept;
}

class ErrorSnapshot final {
 public:
  ErrorSnapshot(const ErrorSnapshot&) = delete;
  ErrorSnapshot& operator=(const ErrorSnapshot&) = delete;
  ErrorSnapshot(ErrorSnapshot&& other) noexcept;
  ErrorSnapshot& operator=(ErrorSnapshot&& other) noexcept;
  ~ErrorSnapshot() noexcept;

  bool valid() const noexcept { return token_ != 0; }

 private:
  friend struct detail::RuntimeTestAccess;
  friend UiError detail::make_error(
      std::uint32_t, std::uint64_t,
      std::shared_ptr<detail::RuntimeCleanupState>, detail::PendingNode*)
      noexcept;
  friend class Runtime;
  friend class ShutdownRequest;
  ErrorSnapshot(std::uint64_t token,
                std::shared_ptr<detail::RuntimeCleanupState> state,
                detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

class UiError final {
 public:
  UiError(std::uint32_t raw_status,
          std::optional<ErrorSnapshot> snapshot = std::nullopt) noexcept;

  std::uint32_t raw_status() const noexcept { return raw_status_; }
  const ErrorSnapshot* snapshot() const noexcept;

 private:
  std::uint32_t raw_status_;
  std::optional<ErrorSnapshot> snapshot_;
};

template <typename T>
class Result final {
 public:
  static Result success(T value) { return Result(std::move(value)); }
  static Result failure(UiError error) { return Result(std::move(error)); }

  bool has_value() const noexcept { return std::holds_alternative<T>(value_); }
  T* value() noexcept { return std::get_if<T>(&value_); }
  const T* value() const noexcept { return std::get_if<T>(&value_); }
  UiError* error() noexcept { return std::get_if<UiError>(&value_); }
  const UiError* error() const noexcept { return std::get_if<UiError>(&value_); }

 private:
  explicit Result(T value) : value_(std::in_place_type<T>, std::move(value)) {}
  explicit Result(UiError error)
      : value_(std::in_place_type<UiError>, std::move(error)) {}

  std::variant<T, UiError> value_;
};

template <>
class Result<void> final {
 public:
  static Result success() { return Result(); }
  static Result failure(UiError error) { return Result(std::move(error)); }

  bool has_value() const noexcept { return !error_.has_value(); }
  UiError* error() noexcept { return error_ ? &*error_ : nullptr; }
  const UiError* error() const noexcept { return error_ ? &*error_ : nullptr; }

 private:
  Result() = default;
  explicit Result(UiError error) : error_(std::move(error)) {}

  std::optional<UiError> error_;
};

struct HostRegistration final {
  using Wake = void (*)(void* userdata, std::uint32_t priority);
  using Destroy = void (*)(void* userdata);

  Wake wake = nullptr;
  void* userdata = nullptr;
  Destroy destroy = nullptr;
};

enum class CallbackKind {
  kPropertyNotification,
  kBindingErrorNotification,
};

struct CallbackFailure final {
  CallbackKind kind;
  std::exception_ptr exception;
};

using CallbackHandler = void (*)(const CallbackFailure&, void* userdata);
using CallbackHandlerDestroy = void (*)(void* userdata);

enum class ShutdownState : std::uint32_t {
  kPending = 0,
  kCompleted = 1,
  kHostUnavailable = 2,
};

struct ShutdownOutcome final {
  ShutdownState state;
  std::uint32_t reason;
};

enum class ShutdownDisposition { kCompleted, kAcceptedPending };

class ShutdownResult final {
 public:
  static ShutdownResult completed() noexcept;
  static ShutdownResult accepted_pending() noexcept;
  static ShutdownResult failure(UiError error) noexcept;

  bool has_error() const noexcept { return error_.has_value(); }
  ShutdownDisposition disposition() const noexcept { return disposition_; }
  const UiError* error() const noexcept;

 private:
  explicit ShutdownResult(ShutdownDisposition disposition) noexcept;
  explicit ShutdownResult(UiError error) noexcept;

  ShutdownDisposition disposition_ = ShutdownDisposition::kCompleted;
  std::optional<UiError> error_;
};

class ShutdownRequest final {
 public:
  ShutdownRequest(const ShutdownRequest&) = delete;
  ShutdownRequest& operator=(const ShutdownRequest&) = delete;
  ShutdownRequest(ShutdownRequest&& other) noexcept;
  ShutdownRequest& operator=(ShutdownRequest&& other) noexcept;
  ~ShutdownRequest() noexcept;

  Result<ShutdownOutcome> query() const;
  Result<void> close() noexcept;

 private:
  friend class Runtime;
  friend struct detail::RuntimeTestAccess;
  ShutdownRequest(std::uint64_t token,
                  std::shared_ptr<detail::RuntimeCleanupState> state,
                  detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

class HandlerRegistration final {
 public:
  HandlerRegistration(const HandlerRegistration&) = delete;
  HandlerRegistration& operator=(const HandlerRegistration&) = delete;
  HandlerRegistration(HandlerRegistration&& other) noexcept;
  HandlerRegistration& operator=(HandlerRegistration&& other) noexcept;
  ~HandlerRegistration() noexcept;

  Result<void> close() noexcept;

 private:
  friend class Runtime;
  friend struct detail::RuntimeTestAccess;
  HandlerRegistration(
      std::weak_ptr<detail::RuntimeCleanupState> state,
      std::weak_ptr<detail::CallbackHandlerRecord> record) noexcept;

  std::weak_ptr<detail::RuntimeCleanupState> state_;
  std::weak_ptr<detail::CallbackHandlerRecord> record_;
};

class Runtime final {
 public:
  Runtime(const Runtime&) = delete;
  Runtime& operator=(const Runtime&) = delete;
  Runtime(Runtime&& other) noexcept;
  Runtime& operator=(Runtime&& other) noexcept;
  ~Runtime() noexcept;

  static Result<Runtime> create(HostRegistration host);

  Result<bool> is_ui_thread() const;
  ShutdownResult shutdown() const;
  Result<ShutdownRequest> request_shutdown() const;
  Result<HandlerRegistration> set_callback_handler(
      CallbackHandler handler, void* userdata,
      CallbackHandlerDestroy destroy);
  Result<void> close() noexcept;

 private:
  template <typename T>
  friend class Property;
  friend struct detail::RuntimeTestAccess;
  Runtime(std::uint64_t token,
          std::shared_ptr<detail::RuntimeCleanupState> state,
          detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Runtime>::value,
              "Runtime owns one native reference");
static_assert(std::is_nothrow_move_constructible<Runtime>::value,
              "Runtime moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_RUNTIME_HPP_
