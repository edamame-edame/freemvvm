#include <freemvvm/runtime.hpp>
#include <freemvvm/property.hpp>

#include <cmath>
#include <cstdint>
#include <limits>
#include <string>
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
  const char utf8_bytes[] = "\xe6\x97\xa5\xe6\x9c\xac\0tail";
  const std::string initial(utf8_bytes, sizeof(utf8_bytes) - 1);
  auto text_result = freemvvm::Property<std::string>::create(runtime, initial);
  if (!text_result.has_value()) return 9;
  auto text = std::move(*text_result.value());
  auto text_value = text.get();
  if (!text_value.has_value() || *text_value.value() != initial) return 10;
  const std::string replacement("tail\0\xe3\x81\x82", 8);
  if (!text.set(replacement).has_value()) return 11;
  text_value = text.get();
  if (!text_value.has_value() || *text_value.value() != replacement) return 12;
  auto ratio_result = freemvvm::Property<double>::create(runtime, 42.25);
  if (!ratio_result.has_value()) return 13;
  auto ratio = std::move(*ratio_result.value());
  auto ratio_value = ratio.get();
  if (!ratio_value.has_value() || *ratio_value.value() != 42.25) return 14;
  if (!ratio.set(std::numeric_limits<double>::infinity()).has_value()) return 15;
  ratio_value = ratio.get();
  if (!ratio_value.has_value() || !std::isinf(*ratio_value.value()) ||
      std::signbit(*ratio_value.value())) return 16;
  if (!enabled.close().has_value() || !count.close().has_value() ||
      !text.close().has_value() || !ratio.close().has_value()) return 17;

  return runtime.close().has_value() ? 0 : 18;
}
