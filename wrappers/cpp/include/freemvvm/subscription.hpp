#ifndef FREEMVVM_CPP_SUBSCRIPTION_HPP_
#define FREEMVVM_CPP_SUBSCRIPTION_HPP_

#include "freemvvm/runtime.hpp"

namespace freemvvm {

template <typename T>
class Property;

class Subscription final {
 public:
  Subscription(const Subscription&) = delete;
  Subscription& operator=(const Subscription&) = delete;
  Subscription(Subscription&& other) noexcept;
  Subscription& operator=(Subscription&& other) noexcept;
  ~Subscription() noexcept;

  Result<void> cancel() noexcept;
  Result<void> close() noexcept;

 private:
  template <typename T>
  friend class Property;
  friend struct detail::RuntimeTestAccess;
  Subscription(std::uint64_t token,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Subscription>::value,
              "Subscription owns one native reference");
static_assert(std::is_nothrow_move_constructible<Subscription>::value,
              "Subscription moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_SUBSCRIPTION_HPP_
