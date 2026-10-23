#include "freemvvm/property.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <chrono>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <limits>
#include <memory>
#include <mutex>
#include <thread>
#include <unordered_map>
#include <utility>
#include <variant>

namespace {

using freemvvm::detail::AbiApi;
using PropertyValue = std::variant<ui_bool, std::int64_t>;

struct FakeAbi final {
  std::mutex mutex;
  std::condition_variable condition;
  ui_handle next_property = 101;
  ui_status create_status = UI_OK;
  ui_status get_status = UI_OK;
  ui_status set_status = UI_OK;
  ui_handle error_token = 900;
  std::unordered_map<ui_handle, PropertyValue> values;
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

ui_status FakeCreateBool(ui_handle, ui_bool initial, ui_handle* out_property,
                         ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->create_status != UI_OK) {
    *out_error = active->error_token;
    return active->create_status;
  }
  *out_property = active->next_property++;
  active->values[*out_property] = initial;
  return UI_OK;
}

ui_status FakeCreateI64(ui_handle, std::int64_t initial,
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

template <typename T>
ui_status FakeGet(ui_handle property, T* out_value) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->get_status != UI_OK) return active->get_status;
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  const auto* value = std::get_if<T>(&found->second);
  if (value == nullptr) return UI_WRONG_TYPE;
  *out_value = *value;
  return UI_OK;
}

ui_status FakeGetBool(ui_handle property, ui_bool* out_value) {
  return FakeGet(property, out_value);
}

ui_status FakeGetI64(ui_handle property, std::int64_t* out_value) {
  return FakeGet(property, out_value);
}

template <typename T>
ui_status FakeSet(ui_handle property, T value, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->set_status != UI_OK) {
    *out_error = active->error_token;
    return active->set_status;
  }
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  auto* current = std::get_if<T>(&found->second);
  if (current == nullptr) return UI_WRONG_TYPE;
  *current = value;
  return UI_OK;
}

ui_status FakeSetBool(ui_handle property, ui_bool value, ui_handle* out_error) {
  return FakeSet(property, value, out_error);
}

ui_status FakeSetI64(ui_handle property, std::int64_t value,
                     ui_handle* out_error) {
  return FakeSet(property, value, out_error);
}

void Wake(void*, std::uint32_t) {}

std::shared_ptr<const AbiApi> MakeApi() {
  static const AbiApi api{
      &FakeRuntimeCreate,      &FakeRuntimeIsUiThread,
      &FakeRuntimeSetBridge,   &FakeRuntimeShutdown,
      &FakeRuntimeRequestShutdown, &FakeShutdownQuery,
      &FakeRelease,            &FakeErrorStatus,
      &FakeErrorCode,           &FakeCreateBool,
      &FakeCreateI64,           nullptr,
      nullptr,                  &FakeGetBool,
      &FakeGetI64,              nullptr,
      nullptr,                  &FakeSetBool,
      &FakeSetI64,              nullptr,
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

bool TestBoolAndI64RoundTripAndMove() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  freemvvm::Runtime runtime = std::move(*runtime_result.value());
  auto bool_result = freemvvm::Property<bool>::create(runtime, true);
  auto i64_result = freemvvm::Property<std::int64_t>::create(
      runtime, std::numeric_limits<std::int64_t>::min());
  if (!Check(bool_result.has_value() && i64_result.has_value(),
             "BOOL and I64 properties should be created")) {
    return false;
  }

  auto boolean = std::move(*bool_result.value());
  auto integer = std::move(*i64_result.value());
  if (!Check(!bool_result.value()->get().has_value(),
             "moved-from property should be invalid")) {
    return false;
  }
  if (!Check(boolean.get().value() != nullptr && *boolean.get().value(),
             "BOOL initial value should round-trip")) {
    return false;
  }
  if (!Check(integer.get().value() != nullptr &&
                 *integer.get().value() == std::numeric_limits<std::int64_t>::min(),
             "I64 minimum should round-trip")) {
    return false;
  }
  if (!Check(boolean.set(false).has_value() &&
                 integer.set(std::numeric_limits<std::int64_t>::max()).has_value(),
             "BOOL and I64 updates should succeed")) {
    return false;
  }
  const auto updated_bool = boolean.get();
  const auto updated_i64 = integer.get();
  if (!Check(updated_bool.has_value() && !*updated_bool.value() &&
                 updated_i64.has_value() &&
                 *updated_i64.value() == std::numeric_limits<std::int64_t>::max(),
             "updated BOOL and I64 values should round-trip")) {
    return false;
  }
  return Check(boolean.close().has_value() && integer.close().has_value(),
               "explicit close should release both property tokens");
}

bool TestFailurePreservesSnapshotAndValueAndCloseRetry() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  freemvvm::Runtime runtime = std::move(*runtime_result.value());

  fake.create_status = UI_INTERNAL_ERROR;
  auto failed_create = freemvvm::Property<std::int64_t>::create(runtime, 41);
  if (!Check(!failed_create.has_value() &&
                 failed_create.error()->raw_status() == UI_INTERNAL_ERROR &&
                 failed_create.error()->snapshot() != nullptr &&
                 failed_create.error()->snapshot()->valid(),
             "failed create should preserve native status and error snapshot")) {
    return false;
  }
  fake.create_status = UI_OK;
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 42);
  if (!Check(property_result.has_value(), "Property creation should succeed")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  fake.set_status = UI_WRONG_THREAD;
  auto failed_set = property.set(99);
  if (!Check(!failed_set.has_value() && failed_set.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_set.error()->snapshot() != nullptr &&
                 failed_set.error()->snapshot()->valid(),
             "failed update should preserve thread status and error snapshot")) {
    return false;
  }
  fake.set_status = UI_OK;
  auto unchanged = property.get();
  if (!Check(unchanged.has_value() && *unchanged.value() == 42,
             "failed update must leave the value unchanged")) {
    return false;
  }
  fake.get_status = UI_WRONG_THREAD;
  auto failed_get = property.get();
  if (!Check(!failed_get.has_value() &&
                 failed_get.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_get.error()->snapshot() == nullptr,
             "get should preserve its raw status without an ABI error token")) {
    return false;
  }
  fake.get_status = UI_OK;

  const ui_handle token = 101;
  fake.release_failures[token] = 1;
  auto first_close = property.close();
  if (!Check(!first_close.has_value(), "first release should fail")) return false;
  auto retry = property.close();
  if (!Check(retry.has_value() && fake.release_counts[token] == 2,
             "failed explicit close should retain and retry the token")) {
    return false;
  }
  return true;
}

bool TestDestructorReleaseFailureReachesSupervisor() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  freemvvm::Runtime runtime = std::move(*runtime_result.value());
  const ui_handle property_token = 101;
  fake.release_failures[property_token] = 1;
  {
    auto property_result = freemvvm::Property<bool>::create(runtime, true);
    if (!Check(property_result.has_value(), "Property creation should succeed")) {
      return false;
    }
    auto property = std::move(*property_result.value());
  }
  std::unique_lock<std::mutex> lock(fake.mutex);
  const bool retried = fake.condition.wait_for(
      lock, std::chrono::seconds(3), [&fake, property_token] {
        return fake.release_counts[property_token] >= 2;
      });
  return Check(retried,
               "supervisor should retry failed Property destructor release");
}

}  // namespace

int main() {
  if (!TestBoolAndI64RoundTripAndMove()) return 1;
  if (!TestFailurePreservesSnapshotAndValueAndCloseRetry()) return 1;
  if (!TestDestructorReleaseFailureReachesSupervisor()) return 1;
  std::puts("C++ Property wrapper unit tests: OK");
  return 0;
}
