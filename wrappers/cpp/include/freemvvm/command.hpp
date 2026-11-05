#ifndef FREEMVVM_CPP_COMMAND_HPP_
#define FREEMVVM_CPP_COMMAND_HPP_

#include "freemvvm/property.hpp"
#include "freemvvm/runtime.hpp"

#include <cstdint>
#include <functional>
#include <optional>
#include <string>
#include <type_traits>
#include <utility>

namespace freemvvm {

enum class CommandState : std::int64_t {
  kIdle = 0,
  kRunning = 1,
  kCancelling = 2,
  kSucceeded = 3,
  kFailed = 4,
  kCancelled = 5,
};

class Command;

class CommandWorker final {
 public:
  CommandWorker(const CommandWorker&) = delete;
  CommandWorker& operator=(const CommandWorker&) = delete;
  CommandWorker(CommandWorker&& other) noexcept;
  CommandWorker& operator=(CommandWorker&& other) noexcept;
  ~CommandWorker() noexcept;

  std::uint64_t operation_id() const noexcept { return operation_id_; }
  Result<bool> is_cancellation_requested() const;
  Result<void> report_success();
  Result<void> report_failure(std::uint32_t domain, std::uint32_t code,
                              const std::string& message);
  Result<void> report_cancelled();

 private:
  friend class Command;
  enum class TerminalKind { kSuccess, kFailure, kCancelled };

  CommandWorker(std::uint64_t runtime_token, std::uint64_t command_token,
                std::uint64_t cancel_token, std::uint64_t operation_id,
                std::shared_ptr<detail::RuntimeCleanupState> state,
                detail::PendingNode* runtime_cleanup_node,
                detail::PendingNode* command_cleanup_node,
                detail::PendingNode* cancel_cleanup_node) noexcept;
  Result<void> post(TerminalKind kind, std::uint32_t domain,
                    std::uint32_t code, std::string message);

  std::uint64_t command_token_ = 0;
  std::uint64_t cancel_token_ = 0;
  std::uint64_t operation_id_ = 0;
  std::uint64_t runtime_token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* command_cleanup_node_ = nullptr;
  detail::PendingNode* cancel_cleanup_node_ = nullptr;
  detail::PendingNode* runtime_cleanup_node_ = nullptr;
};

class Command final {
 public:
  using Start = std::function<Result<void>(CommandWorker)>;

  Command(const Command&) = delete;
  Command& operator=(const Command&) = delete;
  Command(Command&& other) noexcept;
  Command& operator=(Command&& other) noexcept;
  ~Command() noexcept;

  static Result<Command> create_async(Runtime& runtime,
                                      Property<bool>* eligibility,
                                      Start start);
  Result<std::uint64_t> execute();
  Result<void> cancel(std::uint64_t operation_id);
  Result<Property<std::int64_t>> state_property() const;
  Result<Property<bool>> can_execute_property() const;
  Result<std::optional<ErrorSnapshot>> failure_snapshot() const;
  Result<void> close() noexcept;
  Result<void> release() noexcept;

 private:
  friend class CommandWorker;
  friend class Button;
  friend struct detail::RuntimeTestAccess;
  static std::uint32_t start_trampoline(void* userdata,
                                        std::uint64_t borrowed_command,
                                        std::uint64_t operation_id,
                                        std::uint64_t borrowed_cancel_token)
      noexcept;
  Command(std::uint64_t token,
          std::shared_ptr<detail::RuntimeCleanupState> state,
          detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Command>::value,
              "Command owns one native reference");
static_assert(std::is_nothrow_move_constructible<Command>::value,
              "Command moves must not throw");
static_assert(!std::is_copy_constructible<CommandWorker>::value,
              "CommandWorker owns one execution lease");
static_assert(std::is_nothrow_move_constructible<CommandWorker>::value,
              "CommandWorker moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_COMMAND_HPP_
