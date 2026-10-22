#include <freemvvm/runtime.hpp>
#include <freemvvm/property.hpp>

#include <cstdint>
#include <limits>
#include <utility>

namespace {

void Wake(void*, std::uint32_t) {}

}  // namespace

int main() {
  auto result = freemvvm::Runtime::create({&Wake, nullptr, nullptr});
  if (!result.has_value()) return 1;

  freemvvm::Runtime runtime = std::move(*result.value());
  auto is_ui_thread = runtime.is_ui_thread();
  if (!is_ui_thread.has_value() || !*is_ui_thread.value()) return 2;

  auto enabled_result = freemvvm::Property<bool>::create(runtime, true);
  if (!enabled_result.has_value()) return 3;
  auto enabled = std::move(*enabled_result.value());
  auto enabled_value = enabled.get();
  if (!enabled_value.has_value() || !*enabled_value.value()) return 4;
  if (!enabled.set(false).has_value()) return 5;

  auto count_result = freemvvm::Property<std::int64_t>::create(
      runtime, std::numeric_limits<std::int64_t>::min());
  if (!count_result.has_value()) return 6;
  auto count = std::move(*count_result.value());
  auto count_value = count.get();
  if (!count_value.has_value() ||
      *count_value.value() != std::numeric_limits<std::int64_t>::min()) {
    return 7;
  }
  if (!count.set(std::numeric_limits<std::int64_t>::max()).has_value()) return 8;
  if (!enabled.close().has_value() || !count.close().has_value()) return 9;

  return runtime.close().has_value() ? 0 : 10;
}
