#ifndef FREEMVVM_CPP_DECIMAL64_HPP_
#define FREEMVVM_CPP_DECIMAL64_HPP_

#include "freemvvm/runtime.hpp"

#include <cstdint>
#include <type_traits>

namespace freemvvm {

class Decimal64 final {
 public:
  static Result<Decimal64> try_from_parts(bool negative, std::uint8_t scale,
                                          std::uint64_t coefficient) noexcept {
    constexpr std::uint64_t kMaxCoefficient = (std::uint64_t{1} << 58) - 1;
    if (scale > 31 || coefficient > kMaxCoefficient) {
      return Result<Decimal64>::failure(UiError(1));
    }

    const std::uint64_t bits =
        (negative ? (std::uint64_t{1} << 63) : 0) |
        (static_cast<std::uint64_t>(scale) << 58) | coefficient;
    return Result<Decimal64>::success(Decimal64(bits));
  }

  bool is_negative() const noexcept { return (bits_ >> 63) != 0; }
  std::uint8_t scale() const noexcept {
    return static_cast<std::uint8_t>((bits_ >> 58) & 0x1f);
  }
  std::uint64_t coefficient() const noexcept {
    return bits_ & ((std::uint64_t{1} << 58) - 1);
  }

 private:
  template <typename T>
  friend class Property;

  explicit Decimal64(std::uint64_t canonical_bits) noexcept
      : bits_(canonical_bits) {}

  std::uint64_t bits_;
};

static_assert(sizeof(Decimal64) == sizeof(std::uint64_t),
              "Decimal64 must remain a compact canonical scalar");
static_assert(std::is_nothrow_copy_constructible<Decimal64>::value,
              "Decimal64 value copies must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_DECIMAL64_HPP_
