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
  int notification_count = 0;
  auto subscription_result = count.subscribe([&notification_count] {
    ++notification_count;
  });
  if (!subscription_result.has_value()) return 8;
  auto subscription = std::move(*subscription_result.value());
  if (notification_count != 0) return 9;
  if (!count.set(std::numeric_limits<std::int64_t>::max()).has_value()) {
    return 10;
  }
  if (notification_count != 1) return 11;
  if (!subscription.cancel().has_value()) return 12;
  if (!count.set(0).has_value() || notification_count != 1) return 13;
  if (!subscription.close().has_value()) return 14;

  auto bool_target_result = freemvvm::Property<bool>::create(runtime, false);
  if (!bool_target_result.has_value()) return 24;
  auto bool_target = std::move(*bool_target_result.value());
  auto once_result = enabled.bind_to(bool_target, freemvvm::BindingMode::kOnce);
  auto bool_target_value = bool_target.get();
  if (!once_result.has_value() || once_result.value()->has_value() ||
      !bool_target_value.has_value() || *bool_target_value.value()) return 25;

  auto count_target_result =
      freemvvm::Property<std::int64_t>::create(runtime, -1);
  if (!count_target_result.has_value()) return 26;
  auto count_target = std::move(*count_target_result.value());
  auto one_way_result = count.bind_to(count_target,
                                      freemvvm::BindingMode::kOneWay);
  if (!one_way_result.has_value() || !one_way_result.value()->has_value()) {
    return 27;
  }
  auto one_way = std::move(**one_way_result.value());
  auto count_target_value = count_target.get();
  if (!count_target_value.has_value() ||
      *count_target_value.value() != 0) {
    return 28;
  }
  if (!count.set(73).has_value()) return 29;
  count_target_value = count_target.get();
  if (!count_target_value.has_value() || *count_target_value.value() != 73) {
    return 30;
  }
  if (!count_target.set(99).has_value() || !count.set(74).has_value()) return 31;
  count_target_value = count_target.get();
  if (!count_target_value.has_value() || *count_target_value.value() != 74) {
    return 32;
  }
  if (!one_way.cancel().has_value() || !one_way.close().has_value() ||
      !count.set(75).has_value()) return 33;
  count_target_value = count_target.get();
  if (!count_target_value.has_value() || *count_target_value.value() != 74) {
    return 34;
  }

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
  auto decimal_initial = freemvvm::Decimal64::try_from_parts(true, 2, 123);
  if (!decimal_initial.has_value() ||
      !decimal_initial.value()->is_negative() ||
      decimal_initial.value()->scale() != 2 ||
      decimal_initial.value()->coefficient() != 123) return 17;
  auto decimal_result = freemvvm::Property<freemvvm::Decimal64>::create(
      runtime, *decimal_initial.value());
  if (!decimal_result.has_value()) return 18;
  auto decimal = std::move(*decimal_result.value());
  auto decimal_value = decimal.get();
  if (!decimal_value.has_value() ||
      !decimal_value.value()->is_negative() ||
      decimal_value.value()->scale() != 2 ||
      decimal_value.value()->coefficient() != 123) return 19;
  auto decimal_replacement =
      freemvvm::Decimal64::try_from_parts(false, 31, 0);
  if (!decimal_replacement.has_value() ||
      !decimal.set(*decimal_replacement.value()).has_value()) return 20;
  decimal_value = decimal.get();
  if (!decimal_value.has_value() || decimal_value.value()->is_negative() ||
      decimal_value.value()->scale() != 31 ||
      decimal_value.value()->coefficient() != 0) return 21;
  auto ratio_target_result = freemvvm::Property<double>::create(runtime, 0.0);
  if (!ratio_target_result.has_value()) return 35;
  auto ratio_target = std::move(*ratio_target_result.value());
  auto two_way_result = ratio.bind_to(ratio_target,
                                      freemvvm::BindingMode::kTwoWay);
  if (!two_way_result.has_value() || !two_way_result.value()->has_value()) {
    return 36;
  }
  auto two_way = std::move(**two_way_result.value());
  if (!ratio.set(1.5).has_value()) return 37;
  auto ratio_target_value = ratio_target.get();
  if (!ratio_target_value.has_value() || *ratio_target_value.value() != 1.5) {
    return 38;
  }
  if (!ratio_target.set(-2.5).has_value()) return 39;
  ratio_value = ratio.get();
  if (!ratio_value.has_value() || *ratio_value.value() != -2.5) return 40;
  if (!two_way.cancel().has_value() || !two_way.close().has_value()) return 41;

  auto decimal_target_result =
      freemvvm::Property<freemvvm::Decimal64>::create(
          runtime, *decimal_replacement.value());
  if (!decimal_target_result.has_value()) return 42;
  auto decimal_target = std::move(*decimal_target_result.value());
  auto release_only_result = decimal.bind_to(
      decimal_target, freemvvm::BindingMode::kTwoWay);
  if (!release_only_result.has_value() ||
      !release_only_result.value()->has_value()) return 43;
  auto release_only = std::move(**release_only_result.value());
  if (!release_only.cancel().has_value() ||
      !release_only.close().has_value()) return 44;
  auto decimal_next = freemvvm::Decimal64::try_from_parts(true, 3, 1234);
  if (!decimal_next.has_value() || !decimal.set(*decimal_next.value()).has_value()) {
    return 45;
  }
  auto decimal_target_value = decimal_target.get();
  if (!decimal_target_value.has_value() ||
      decimal_target_value.value()->is_negative() ||
      decimal_target_value.value()->scale() != 31 ||
      decimal_target_value.value()->coefficient() != 0) return 46;
  const auto shutdown = runtime.shutdown();
  // The test host does not pump a UI loop, so an accepted pending shutdown is
  // also a valid result for this package smoke consumer.
  if (shutdown.has_error()) return 47;
  if (!enabled.close().has_value() || !bool_target.close().has_value() ||
      !count.close().has_value() || !count_target.close().has_value() ||
      !text.close().has_value() || !ratio.close().has_value() ||
      !ratio_target.close().has_value() || !decimal.close().has_value() ||
      !decimal_target.close().has_value()) return 22;

  return runtime.close().has_value() ? 0 : 23;
}
