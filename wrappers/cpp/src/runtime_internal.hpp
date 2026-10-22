#ifndef FREEMVVM_CPP_RUNTIME_INTERNAL_HPP_
#define FREEMVVM_CPP_RUNTIME_INTERNAL_HPP_

#include "freemvvm/runtime.hpp"

#include <freemvvm/abi.h>

#include <condition_variable>
#include <memory>
#include <mutex>
#include <thread>

namespace freemvvm::detail {

struct AbiApi final {
  ui_status (*runtime_create)(ui_handle*, ui_handle*);
  ui_status (*runtime_is_ui_thread)(ui_handle, ui_bool*);
  ui_status (*runtime_set_ui_loop_bridge)(ui_handle, ui_wake_callback, void*,
                                          ui_destroy_callback, ui_handle*);
  ui_status (*runtime_shutdown)(ui_handle, ui_handle*);
  ui_status (*runtime_request_shutdown)(ui_handle, std::uint32_t, ui_handle*,
                                        ui_handle*);
  ui_status (*shutdown_request_query)(ui_handle, std::uint32_t*, std::uint32_t*,
                                      ui_handle*);
  ui_status (*handle_release)(ui_handle);
  ui_status (*error_status)(ui_handle, std::uint32_t*);
  ui_status (*error_code)(ui_handle, std::uint32_t*, std::uint32_t*);
};

struct PendingNode final {
  ui_handle token = 0;
  PendingNode* next = nullptr;
};

struct RuntimeCleanupState final
    : public std::enable_shared_from_this<RuntimeCleanupState> {
  explicit RuntimeCleanupState(std::shared_ptr<const AbiApi> api_value);
  ~RuntimeCleanupState();

  bool enqueue(PendingNode* node) noexcept;
  PendingNode* pop() noexcept;
  void requeue(PendingNode* node) noexcept;
  bool empty() const noexcept;

  std::shared_ptr<const AbiApi> api;
  mutable std::mutex queue_mutex;
  PendingNode* pending_head = nullptr;
  PendingNode* pending_tail = nullptr;
  RuntimeCleanupState* supervisor_next = nullptr;
  std::shared_ptr<RuntimeCleanupState> supervisor_keepalive;
  bool registered = false;
};

class CleanupSupervisor final {
 public:
  static CleanupSupervisor& instance();
  CleanupSupervisor(const CleanupSupervisor&) = delete;
  CleanupSupervisor& operator=(const CleanupSupervisor&) = delete;
  ~CleanupSupervisor() noexcept;

  void register_pending(
      const std::shared_ptr<RuntimeCleanupState>& state) noexcept;
  void wake() noexcept;

 private:
  CleanupSupervisor();
  void run() noexcept;
  bool unregister_if_empty(RuntimeCleanupState* state) noexcept;

  std::mutex mutex_;
  std::condition_variable condition_;
  RuntimeCleanupState* head_ = nullptr;
  bool stopping_ = false;
  std::thread worker_;
};

std::shared_ptr<const AbiApi> linked_api();
UiError make_error(std::uint32_t status, ui_handle error_token,
                  std::shared_ptr<RuntimeCleanupState> state,
                  PendingNode* error_node) noexcept;
Result<void> release_owned(std::uint64_t& token,
                           std::shared_ptr<RuntimeCleanupState>& state,
                           PendingNode*& node) noexcept;
void release_in_destructor(std::uint64_t& token,
                           std::shared_ptr<RuntimeCleanupState>& state,
                           PendingNode*& node) noexcept;

struct RuntimeTestAccess final {
  static Result<Runtime> create(const std::shared_ptr<const AbiApi>& api,
                                HostRegistration host);
  static std::weak_ptr<RuntimeCleanupState> state_of(const Runtime& runtime);
};

}  // namespace freemvvm::detail

#endif  // FREEMVVM_CPP_RUNTIME_INTERNAL_HPP_
