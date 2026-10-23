#include "freemvvm/property.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <memory>
#include <mutex>
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

std::uint64_t Bits(double value) {
  std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits));
  return bits;
}

double FromBits(std::uint64_t bits) {
  double value;
  std::memcpy(&value, &bits, sizeof(value));
  return value;
}

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

ui_status FakeCreateF64(ui_handle, double initial, ui_handle* out_property,
                        ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->create_status != UI_OK) {
    *out_error = active->error_token;
    return active->create_status;
  }
  *out_property = active->next_property++;
  active->values[*out_property] = Bits(initial);
  return UI_OK;
}

ui_status FakeGetF64(ui_handle property, double* out_value) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->get_status != UI_OK) return active->get_status;
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  *out_value = FromBits(found->second);
  return UI_OK;
}

ui_status FakeSetF64(ui_handle property, double value, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->set_status != UI_OK) {
    *out_error = active->error_token;
    return active->set_status;
  }
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  found->second = Bits(value);
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
      &FakeCreateF64,
      nullptr,
      nullptr,
      nullptr,
      &FakeGetF64,
      nullptr,
      nullptr,
      nullptr,
      &FakeSetF64,
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

bool TestF64BitsRoundTripAndMove() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  constexpr std::uint64_t kValues[] = {
      0x405ee00000000000ULL,  // 123.5
      0x7ff0000000000000ULL,  // positive infinity
      0xfff0000000000000ULL,  // negative infinity
      0x7ff8000000000042ULL,  // NaN payload
      0x0000000000000000ULL,  // positive zero
      0x8000000000000000ULL,  // negative zero
  };
  auto property_result =
      freemvvm::Property<double>::create(runtime, FromBits(kValues[0]));
  if (!Check(property_result.has_value(), "F64 Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  freemvvm::Property<double> moved(std::move(property));
  if (!Check(!property.get().has_value(), "moved-from Property should be invalid")) {
    return false;
  }
  for (std::uint64_t expected_bits : kValues) {
    if (!Check(moved.set(FromBits(expected_bits)).has_value(),
               "F64 setter should accept the exact binary64 value")) {
      return false;
    }
    auto value = moved.get();
    if (!Check(value.has_value() && Bits(*value.value()) == expected_bits,
               "F64 get/set should preserve the binary64 bit pattern")) {
      return false;
    }
  }
  return Check(moved.close().has_value(), "Property close should succeed");
}

bool TestF64ErrorsAndReleaseRetry() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());

  fake.create_status = UI_INTERNAL_ERROR;
  auto failed_create = freemvvm::Property<double>::create(runtime, 12.5);
  if (!Check(!failed_create.has_value() &&
                 failed_create.error()->raw_status() == UI_INTERNAL_ERROR &&
                 failed_create.error()->snapshot() != nullptr &&
                 failed_create.error()->snapshot()->valid(),
             "failed create should preserve status and native error snapshot")) {
    return false;
  }
  fake.create_status = UI_OK;
  auto property_result = freemvvm::Property<double>::create(runtime, 12.5);
  if (!Check(property_result.has_value(), "F64 Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());

  fake.set_status = UI_WRONG_THREAD;
  auto failed_set = property.set(99.0);
  if (!Check(!failed_set.has_value() &&
                 failed_set.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_set.error()->snapshot() != nullptr,
             "failed set should preserve status and error snapshot")) {
    return false;
  }
  fake.set_status = UI_OK;
  auto unchanged = property.get();
  if (!Check(unchanged.has_value() && Bits(*unchanged.value()) == Bits(12.5),
             "failed set must leave the current value unchanged")) {
    return false;
  }
  fake.get_status = UI_WRONG_THREAD;
  auto failed_get = property.get();
  if (!Check(!failed_get.has_value() &&
                 failed_get.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_get.error()->snapshot() == nullptr,
             "getter should preserve raw status without error snapshot")) {
    return false;
  }
  fake.get_status = UI_OK;

  constexpr ui_handle kPropertyToken = 101;
  fake.release_failures[kPropertyToken] = 1;
  if (!Check(!property.close().has_value(), "first close should fail")) {
    return false;
  }
  return Check(property.close().has_value() &&
                   fake.release_counts[kPropertyToken] == 2,
               "failed close should retain token for retry");
}

bool TestDestructorReleaseFailureReachesSupervisor() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  constexpr ui_handle kPropertyToken = 101;
  fake.release_failures[kPropertyToken] = 1;
  {
    auto property_result = freemvvm::Property<double>::create(runtime, 3.0);
    if (!Check(property_result.has_value(), "F64 Property should be created")) {
      return false;
    }
    auto property = std::move(*property_result.value());
  }
  std::unique_lock<std::mutex> lock(fake.mutex);
  const bool retried = fake.condition.wait_for(
      lock, std::chrono::seconds(3), [&fake, kPropertyToken] {
        return fake.release_counts[kPropertyToken] >= 2;
      });
  return Check(retried,
               "supervisor should retry failed Property destructor release");
}

}  // namespace

int main() {
  if (!TestF64BitsRoundTripAndMove()) return 1;
  if (!TestF64ErrorsAndReleaseRetry()) return 1;
  if (!TestDestructorReleaseFailureReachesSupervisor()) return 1;
  std::puts("C++ F64 Property wrapper unit tests: OK");
  return 0;
}
