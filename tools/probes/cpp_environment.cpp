#include <atomic>
#include <cstdint>
#include <iostream>
#include <limits>

static_assert(__cplusplus >= 201703L, "C++17 is required");
static_assert(std::numeric_limits<float>::is_iec559,
              "IEEE floating-point representation is required");
static_assert(std::numeric_limits<double>::is_iec559,
              "IEEE floating-point representation is required");

int main() {
  std::atomic<std::uint64_t> value{1};
  std::uint64_t expected = 1;
  if (!value.compare_exchange_strong(expected, 2) || value.load() != 2) {
    return 1;
  }
  std::cout << "bootstrap C++17: uint64 atomic available; lock-free="
            << value.is_lock_free() << '\n';
  return 0;
}
