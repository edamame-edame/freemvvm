#ifndef FREEMVVM_CPP_PROPERTY_HPP_
#define FREEMVVM_CPP_PROPERTY_HPP_

#include "freemvvm/decimal64.hpp"
#include "freemvvm/runtime.hpp"
#include "freemvvm/subscription.hpp"

#include <cstdint>
#include <functional>
#include <string>
#include <type_traits>
#include <utility>

namespace freemvvm {

template <typename T>
class Binding;
template <typename T>
class BindingError;
enum class BindingMode : std::uint32_t;

template <typename T>
class Property final {
  static_assert(std::is_same<T, bool>::value ||
                    std::is_same<T, std::int64_t>::value ||
                    std::is_same<T, double>::value ||
                    std::is_same<T, Decimal64>::value ||
                    std::is_same<T, std::string>::value,
                "This Property wrapper supports bool, int64_t, double, Decimal64, and UTF-8 string");

 public:
  Property(const Property&) = delete;
  Property& operator=(const Property&) = delete;
  Property(Property&& other) noexcept;
  Property& operator=(Property&& other) noexcept;
  ~Property() noexcept;

  static Result<Property<T>> create(Runtime& runtime, T initial_value);

  Result<T> get() const;
  Result<void> set(T value);
  Result<Subscription> subscribe(std::function<void()> callback);
  Result<std::optional<Binding<T>>> bind_to(Property<T>& target,
                                            BindingMode mode);
  Result<void> close() noexcept;

 private:
  friend struct detail::RuntimeTestAccess;
  template <typename U>
  friend class Binding;
  template <typename U>
  friend class BindingError;
  Property(std::uint64_t token,
           std::shared_ptr<detail::RuntimeCleanupState> state,
           detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Property<bool>>::value,
              "Property owns one native reference");
static_assert(std::is_nothrow_move_constructible<Property<std::int64_t>>::value,
              "Property moves must not throw");
static_assert(std::is_nothrow_move_constructible<Property<double>>::value,
              "Property moves must not throw");
static_assert(std::is_nothrow_move_constructible<Property<Decimal64>>::value,
              "Property moves must not throw");
static_assert(std::is_nothrow_move_constructible<Property<std::string>>::value,
              "Property moves must not throw");

}  // namespace freemvvm

#include "freemvvm/binding.hpp"

#endif  // FREEMVVM_CPP_PROPERTY_HPP_
