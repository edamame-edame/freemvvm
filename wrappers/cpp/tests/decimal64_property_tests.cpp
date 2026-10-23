#include "freemvvm/property.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <memory>
#include <mutex>
#include <string>
#include <type_traits>
#include <unordered_map>
#include <utility>

namespace {

using freemvvm::detail::AbiApi;

struct FakeAbi final {
  std::mutex mutex;
  std::condition_variable condition;
  ui_handle next_property = 101;
  ui_status create_status = UI_OK;
  ui_status get_status = UI_OK;
  ui_status set_status = UI_OK;
  ui_handle error_token = 900;
  std::unordered_map<ui_handle, std::uint64_t> values;
  std::unordered_map<ui_handle, int> release_counts;
  std::unordered_map<ui_handle, int> release_failures;
};

FakeAbi* active = nullptr;

ui_status FakeRuntimeCreate(ui_handle* out_runtime, ui_handle*) {
  *out_runtime = 100;
  return UI_OK;
}

ui_status FakeRuntimeIsUiThread(ui_handle, ui_bool* out_yes) {
  *out_yes = 1;
  return UI_OK;
}

ui_status FakeRuntimeSetBridge(ui_handle, ui_wake_callback, void*,
                               ui_destroy_callback, ui_handle*) {
  return UI_OK;
}

ui_status FakeRuntimeShutdown(ui_handle, ui_handle*) { return UI_OK; }

ui_status FakeRuntimeRequestShutdown(ui_handle, std::uint32_t,
                                     ui_handle* out_request, ui_handle*) {
  *out_request = 200;
  return UI_OK;
}

ui_status FakeShutdownQuery(ui_handle, std::uint32_t* out_state,
                            std::uint32_t* out_reason, ui_handle*) {
  *out_state = UI_SHUTDOWN_REQUEST_COMPLETED;
  *out_reason = 0;
  return UI_OK;
}

ui_status FakeRelease(ui_handle token) {
  std::lock_guard<std::mutex> lock(active->mutex);
  ++active->release_counts[token];
  active->condition.notify_all();
  int& failures = active->release_failures[token];
  if (failures > 0) {
    --failures;
    return UI_INTERNAL_ERROR;
  }
  active->values.erase(token);
  return UI_OK;
}

ui_status FakeErrorStatus(ui_handle, std::uint32_t* out_status) {
  *out_status = UI_INTERNAL_ERROR;
  return UI_OK;
}

ui_status FakeErrorCode(ui_handle, std::uint32_t* out_domain,
                        std::uint32_t* out_code) {
  *out_domain = 1;
  *out_code = 7;
  return UI_OK;
}

ui_status FakeCreateDecimal(ui_handle, std::uint64_t initial,
                            ui_handle* out_property, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->create_status != UI_OK) {
    *out_error = active->error_token;
    return active->create_status;
  }
  *out_property = active->next_property++;
  active->values[*out_property] = initial;
  return UI_OK;
}

ui_status FakeGetDecimal(ui_handle property, std::uint64_t* out_value) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->get_status != UI_OK) return active->get_status;
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  *out_value = found->second;
  return UI_OK;
}

ui_status FakeSetDecimal(ui_handle property, std::uint64_t value,
                         ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->set_status != UI_OK) {
    *out_error = active->error_token;
    return active->set_status;
  }
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  found->second = value;
  return UI_OK;
}

void Wake(void*, std::uint32_t) {}

std::shared_ptr<const AbiApi> MakeApi() {
  static const AbiApi api{
      &FakeRuntimeCreate,
      &FakeRuntimeIsUiThread,
      &FakeRuntimeSetBridge,
      &FakeRuntimeShutdown,
      &FakeRuntimeRequestShutdown,
      &FakeShutdownQuery,
      &FakeRelease,
      &FakeErrorStatus,
      &FakeErrorCode,
      nullptr,
      nullptr,
      nullptr,
      &FakeCreateDecimal,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      &FakeGetDecimal,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      &FakeSetDecimal,
      nullptr,
  };
  return std::shared_ptr<const AbiApi>(std::shared_ptr<const AbiApi>{}, &api);
}

bool Check(bool condition, const char* message) {
  if (condition) return true;
  std::fprintf(stderr, "FAILED: %s\n", message);
  return false;
}

freemvvm::Result<freemvvm::Runtime> CreateRuntime(FakeAbi& fake) {
  active = &fake;
  return freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
}

constexpr std::uint64_t CanonicalBits(bool negative, std::uint8_t scale,
                                      std::uint64_t coefficient) {
  return (negative ? (std::uint64_t{1} << 63) : 0) |
         (static_cast<std::uint64_t>(scale) << 58) | coefficient;
}

bool TestPartsFactoryAndBounds() {
  constexpr std::uint64_t kMaxCoefficient = (std::uint64_t{1} << 58) - 1;
  auto minimum = freemvvm::Decimal64::try_from_parts(false, 0, 0);
  auto maximum = freemvvm::Decimal64::try_from_parts(true, 31, kMaxCoefficient);
  if (!Check(minimum.has_value() && !minimum.value()->is_negative() &&
                 minimum.value()->scale() == 0 &&
                 minimum.value()->coefficient() == 0,
             "parts factory should represent positive zero")) {
    return false;
  }
  if (!Check(maximum.has_value() && maximum.value()->is_negative() &&
                 maximum.value()->scale() == 31 &&
                 maximum.value()->coefficient() == kMaxCoefficient,
             "parts factory should preserve all field boundaries")) {
    return false;
  }

  auto bad_scale = freemvvm::Decimal64::try_from_parts(false, 32, 0);
  auto bad_coefficient =
      freemvvm::Decimal64::try_from_parts(false, 0, kMaxCoefficient + 1);
  return Check(!bad_scale.has_value() &&
                   bad_scale.error()->raw_status() == UI_INVALID_ARGUMENT &&
                   bad_scale.error()->snapshot() == nullptr &&
                   !bad_coefficient.has_value() &&
                   bad_coefficient.error()->raw_status() == UI_INVALID_ARGUMENT &&
                   bad_coefficient.error()->snapshot() == nullptr,
               "out-of-range parts should fail locally without native snapshot");
}

bool TestCanonicalBitsRoundTripMoveAndCloseRetry() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  constexpr std::uint64_t kMaxCoefficient = (std::uint64_t{1} << 58) - 1;
  constexpr std::uint64_t kValues[] = {
      CanonicalBits(false, 0, 0),
      CanonicalBits(true, 31, 0),
      CanonicalBits(false, 2, 100),
      CanonicalBits(true, 0, kMaxCoefficient),
  };
  auto initial = freemvvm::Decimal64::try_from_parts(false, 2, 100);
  if (!Check(initial.has_value(), "Initial Decimal64 should be valid")) {
    return false;
  }
  auto property_result =
      freemvvm::Property<freemvvm::Decimal64>::create(runtime, *initial.value());
  if (!Check(property_result.has_value(), "DECIMAL Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  freemvvm::Property<freemvvm::Decimal64> moved(std::move(property));
  if (!Check(!property.get().has_value(), "moved-from Property should be invalid")) {
    return false;
  }

  for (std::uint64_t bits : kValues) {
    auto value = freemvvm::Decimal64::try_from_parts(
        (bits >> 63) != 0, static_cast<std::uint8_t>((bits >> 58) & 0x1f),
        bits & kMaxCoefficient);
    if (!Check(value.has_value() && moved.set(*value.value()).has_value(),
               "DECIMAL setter should accept canonical values")) {
      return false;
    }
    auto actual = moved.get();
    if (!Check(actual.has_value() &&
                   CanonicalBits(actual.value()->is_negative(),
                                 actual.value()->scale(),
                                 actual.value()->coefficient()) == bits,
               "DECIMAL get/set should preserve canonical bits")) {
      return false;
    }
  }

  constexpr ui_handle kPropertyToken = 101;
  fake.release_failures[kPropertyToken] = 1;
  if (!Check(!moved.close().has_value(), "first close should fail")) {
    return false;
  }
  return Check(moved.close().has_value() &&
                   fake.release_counts[kPropertyToken] == 2,
               "failed close should retain the token for retry");
}

bool TestNativeErrorsAndDestructorRetry() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  auto initial = freemvvm::Decimal64::try_from_parts(false, 1, 25);
  if (!Check(initial.has_value(), "Initial Decimal64 should be valid")) {
    return false;
  }

  fake.create_status = UI_INTERNAL_ERROR;
  auto failed_create = freemvvm::Property<freemvvm::Decimal64>::create(
      runtime, *initial.value());
  if (!Check(!failed_create.has_value() &&
                 failed_create.error()->raw_status() == UI_INTERNAL_ERROR &&
                 failed_create.error()->snapshot() != nullptr &&
                 failed_create.error()->snapshot()->valid(),
             "failed create should retain native error snapshot")) {
    return false;
  }
  fake.create_status = UI_OK;
  auto property_result = freemvvm::Property<freemvvm::Decimal64>::create(
      runtime, *initial.value());
  if (!Check(property_result.has_value(), "DECIMAL Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());

  auto replacement = freemvvm::Decimal64::try_from_parts(true, 7, 123456);
  if (!Check(replacement.has_value(), "Replacement Decimal64 should be valid")) {
    return false;
  }
  fake.set_status = UI_WRONG_THREAD;
  auto failed_set = property.set(*replacement.value());
  if (!Check(!failed_set.has_value() &&
                 failed_set.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_set.error()->snapshot() != nullptr,
             "failed set should preserve native status and snapshot")) {
    return false;
  }
  fake.set_status = UI_OK;
  auto unchanged = property.get();
  if (!Check(unchanged.has_value() &&
                 CanonicalBits(unchanged.value()->is_negative(),
                               unchanged.value()->scale(),
                               unchanged.value()->coefficient()) ==
                     CanonicalBits(false, 1, 25),
             "failed set should leave the current representation unchanged")) {
    return false;
  }
  fake.get_status = UI_WRONG_THREAD;
  auto failed_get = property.get();
  if (!Check(!failed_get.has_value() &&
                 failed_get.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_get.error()->snapshot() == nullptr,
             "getter should retain raw status without a native error snapshot")) {
    return false;
  }
  fake.get_status = UI_OK;

  constexpr ui_handle kPropertyToken = 101;
  constexpr ui_handle kDoomedPropertyToken = 102;
  fake.release_failures[kDoomedPropertyToken] = 1;
  {
    auto doomed_result = freemvvm::Property<freemvvm::Decimal64>::create(
        runtime, *initial.value());
    if (!Check(doomed_result.has_value(),
               "supervisor test Property should be created")) {
      return false;
    }
    auto doomed = std::move(*doomed_result.value());
  }
  std::unique_lock<std::mutex> lock(fake.mutex);
  const bool retried = fake.condition.wait_for(
      lock, std::chrono::seconds(3), [&fake, kDoomedPropertyToken] {
        return fake.release_counts[kDoomedPropertyToken] >= 2;
      });
  if (!Check(retried,
             "supervisor should retry failed Property destructor release")) {
    return false;
  }
  lock.unlock();
  return property.close().has_value();
}

}  // namespace

int main() {
  static_assert(!std::is_convertible<double, freemvvm::Decimal64>::value,
                "Decimal64 must not convert implicitly from double");
  static_assert(!std::is_convertible<std::string, freemvvm::Decimal64>::value,
                "Decimal64 must not convert implicitly from string");
  if (!TestPartsFactoryAndBounds()) return 1;
  if (!TestCanonicalBitsRoundTripMoveAndCloseRetry()) return 1;
  if (!TestNativeErrorsAndDestructorRetry()) return 1;
  std::puts("C++ DECIMAL Property wrapper unit tests: OK");
  return 0;
}
