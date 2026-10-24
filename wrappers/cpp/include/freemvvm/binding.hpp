#ifndef FREEMVVM_CPP_BINDING_HPP_
#define FREEMVVM_CPP_BINDING_HPP_

#include "freemvvm/property.hpp"

#include <cstdint>
#include <functional>
#include <memory>
#include <optional>
#include <type_traits>
#include <utility>

namespace freemvvm {

enum class BindingMode : std::uint32_t {
  kOnce = 1,
  kOneWay = 2,
  kTwoWay = 3,
};

enum class BindingDirection : std::uint32_t {
  kSourceToTarget = 1,
  kTargetToSource = 2,
};

template <typename T>
class Binding;

template <typename T>
class BindingError final {
 public:
  BindingError(const BindingError&) = delete;
  BindingError& operator=(const BindingError&) = delete;
  BindingError(BindingError&& other) noexcept = default;
  BindingError& operator=(BindingError&& other) noexcept = default;

  const UiError& error() const noexcept { return error_; }
  const Property<T>& rejected_property() const noexcept {
    return rejected_property_;
  }
  BindingDirection direction() const noexcept { return direction_; }

 private:
  friend class Binding<T>;
  BindingError(UiError error, std::uint64_t rejected_property,
               BindingDirection direction,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* property_cleanup_node) noexcept
      : error_(std::move(error)),
        rejected_property_(rejected_property, std::move(state),
                           property_cleanup_node),
        direction_(direction) {}

  UiError error_;
  Property<T> rejected_property_;
  BindingDirection direction_;
};

template <typename T>
class Binding final {
 public:
  Binding(const Binding&) = delete;
  Binding& operator=(const Binding&) = delete;
  Binding(Binding&& other) noexcept;
  Binding& operator=(Binding&& other) noexcept;
  ~Binding() noexcept;

  Result<void> cancel() noexcept;
  Result<void> close() noexcept;
  Result<std::optional<BindingError<T>>> error_snapshot() const;
  Result<Subscription> subscribe_error(std::function<void()> callback);

 private:
  friend class WebView;
  friend class Property<T>;
  Binding(std::uint64_t token,
          std::shared_ptr<detail::RuntimeCleanupState> state,
          detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Binding<std::int64_t>>::value,
              "Binding owns one native reference");
static_assert(std::is_nothrow_move_constructible<Binding<std::int64_t>>::value,
              "Binding moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_BINDING_HPP_
