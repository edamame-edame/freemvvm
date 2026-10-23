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
#include <string>
#include <unordered_map>
#include <utility>
#include <variant>

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
  bool grow_on_copy = false;
  std::unordered_map<ui_handle, std::string> values;
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
  *out_status = UI_INVALID_ARGUMENT;
  return UI_OK;
}

ui_status FakeErrorCode(ui_handle, std::uint32_t* out_domain,
                        std::uint32_t* out_code) {
  *out_domain = 1;
  *out_code = 7;
  return UI_OK;
}

bool IsValidUtf8(const std::string& value) {
  const auto* bytes = reinterpret_cast<const unsigned char*>(value.data());
  std::size_t index = 0;
  while (index < value.size()) {
    const unsigned char lead = bytes[index++];
    if (lead <= 0x7f) continue;
    std::size_t continuation_count;
    unsigned char second_min = 0x80;
    unsigned char second_max = 0xbf;
    if (lead >= 0xc2 && lead <= 0xdf) {
      continuation_count = 1;
    } else if (lead >= 0xe0 && lead <= 0xef) {
      continuation_count = 2;
      if (lead == 0xe0) second_min = 0xa0;
      if (lead == 0xed) second_max = 0x9f;
    } else if (lead >= 0xf0 && lead <= 0xf4) {
      continuation_count = 3;
      if (lead == 0xf0) second_min = 0x90;
      if (lead == 0xf4) second_max = 0x8f;
    } else {
      return false;
    }
    if (continuation_count > value.size() - index) return false;
    const unsigned char second = bytes[index++];
    if (second < second_min || second > second_max) return false;
    for (std::size_t i = 1; i < continuation_count; ++i) {
      const unsigned char continuation = bytes[index++];
      if (continuation < 0x80 || continuation > 0xbf) return false;
    }
  }
  return true;
}

std::string CopyBytes(const std::uint8_t* bytes, std::uint64_t length) {
  if (length == 0) return {};
  return std::string(reinterpret_cast<const char*>(bytes),
                     static_cast<std::size_t>(length));
}

ui_status FakeCreateUtf8(ui_handle, const std::uint8_t* bytes,
                         std::uint64_t length, ui_handle* out_property,
                         ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  const std::string value = CopyBytes(bytes, length);
  if (active->create_status != UI_OK || !IsValidUtf8(value)) {
    *out_error = active->error_token;
    return active->create_status == UI_OK ? UI_INVALID_ARGUMENT
                                          : active->create_status;
  }
  *out_property = active->next_property++;
  active->values[*out_property] = value;
  return UI_OK;
}

ui_status FakeGetUtf8(ui_handle property, std::uint8_t* buffer,
                      std::uint64_t capacity, std::uint64_t* out_length) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->get_status != UI_OK) return active->get_status;
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  if (buffer == nullptr && capacity == 0) {
    *out_length = found->second.size();
    return UI_OK;
  }
  if (active->grow_on_copy) {
    active->grow_on_copy = false;
    found->second.append("+");
    *out_length = found->second.size();
    return UI_BUFFER_TOO_SMALL;
  }
  if (capacity < found->second.size()) {
    *out_length = found->second.size();
    return UI_BUFFER_TOO_SMALL;
  }
  if (!found->second.empty()) {
    std::memcpy(buffer, found->second.data(), found->second.size());
  }
  *out_length = found->second.size();
  return UI_OK;
}

ui_status FakeSetUtf8(ui_handle property, const std::uint8_t* bytes,
                      std::uint64_t length, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->set_status != UI_OK) {
    *out_error = active->error_token;
    return active->set_status;
  }
  auto found = active->values.find(property);
  if (found == active->values.end()) return UI_INVALID_HANDLE;
  std::string value = CopyBytes(bytes, length);
  if (!IsValidUtf8(value)) {
    *out_error = active->error_token;
    return UI_INVALID_ARGUMENT;
  }
  found->second = std::move(value);
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
      &FakeCreateUtf8,
      nullptr,
      nullptr,
      nullptr,
      &FakeGetUtf8,
      nullptr,
      nullptr,
      nullptr,
      &FakeSetUtf8,
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

bool TestBytePreservingRoundTrip() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  const char embedded_bytes[] = "\xe6\x97\xa5\xe6\x9c\xac\0tail";
  const std::string initial(embedded_bytes, sizeof(embedded_bytes) - 1);
  auto property_result = freemvvm::Property<std::string>::create(runtime, "");
  if (!Check(property_result.has_value(), "empty Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  auto empty = property.get();
  if (!Check(empty.has_value() && empty.value()->empty(),
             "empty value should round-trip")) {
    return false;
  }
  if (!Check(property.set("ASCII").has_value(), "ASCII set should succeed")) {
    return false;
  }
  if (!Check(property.set(initial).has_value(),
             "Japanese bytes and embedded NUL should set")) {
    return false;
  }
  auto value = property.get();
  if (!Check(value.has_value() && *value.value() == initial &&
                 value.value()->size() == sizeof(embedded_bytes) - 1,
             "get should preserve exact UTF-8 bytes and embedded NUL")) {
    return false;
  }
  const std::string replacement("tail\0\xe3\x81\x82", 8);
  if (!Check(property.set(replacement).has_value(),
             "embedded NUL replacement should set")) {
    return false;
  }
  value = property.get();
  if (!Check(value.has_value() && *value.value() == replacement,
             "set/get should preserve exact replacement bytes")) {
    return false;
  }
  return Check(property.close().has_value(), "Property close should succeed");
}

bool TestInvalidUtf8AndFailures() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  const std::string invalid("\xc0\xaf", 2);
  auto failed_create = freemvvm::Property<std::string>::create(runtime, invalid);
  if (!Check(!failed_create.has_value() &&
                 failed_create.error()->raw_status() == UI_INVALID_ARGUMENT &&
                 failed_create.error()->snapshot() != nullptr &&
                 failed_create.error()->snapshot()->valid(),
             "invalid UTF-8 create should preserve status and error snapshot")) {
    return false;
  }
  auto property_result = freemvvm::Property<std::string>::create(runtime, "ok");
  if (!Check(property_result.has_value(), "valid Property should be created")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  auto failed_set = property.set(invalid);
  if (!Check(!failed_set.has_value() &&
                 failed_set.error()->raw_status() == UI_INVALID_ARGUMENT &&
                 failed_set.error()->snapshot() != nullptr &&
                 failed_set.error()->snapshot()->valid(),
             "invalid UTF-8 set should preserve status and error snapshot")) {
    return false;
  }
  auto unchanged = property.get();
  if (!Check(unchanged.has_value() && *unchanged.value() == "ok",
             "failed set must leave the current value unchanged")) {
    return false;
  }
  fake.set_status = UI_WRONG_THREAD;
  auto wrong_thread = property.set("other");
  if (!Check(!wrong_thread.has_value() &&
                 wrong_thread.error()->raw_status() == UI_WRONG_THREAD &&
                 wrong_thread.error()->snapshot() != nullptr,
             "wrong-thread set should preserve status and snapshot")) {
    return false;
  }
  fake.set_status = UI_OK;
  fake.get_status = UI_WRONG_THREAD;
  auto failed_get = property.get();
  if (!Check(!failed_get.has_value() &&
                 failed_get.error()->raw_status() == UI_WRONG_THREAD &&
                 failed_get.error()->snapshot() == nullptr,
             "get should preserve raw status without an error token")) {
    return false;
  }
  return property.close().has_value();
}

bool TestBufferGrowthRetryAndCleanup() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::string>::create(runtime, "old");
  if (!Check(property_result.has_value(), "Property creation should succeed")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  fake.grow_on_copy = true;
  auto value = property.get();
  if (!Check(value.has_value() && *value.value() == "old+",
             "getter should re-query and retry when the value grows")) {
    return false;
  }
  const ui_handle token = 101;
  fake.release_failures[token] = 1;
  if (!Check(!property.close().has_value(), "first close should fail")) {
    return false;
  }
  if (!Check(property.close().has_value() && fake.release_counts[token] == 2,
             "failed explicit close should retain token for retry")) {
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
  auto runtime = std::move(*runtime_result.value());
  const ui_handle property_token = 101;
  fake.release_failures[property_token] = 1;
  {
    auto property_result =
        freemvvm::Property<std::string>::create(runtime, "destructor");
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
  if (!TestBytePreservingRoundTrip()) return 1;
  if (!TestInvalidUtf8AndFailures()) return 1;
  if (!TestBufferGrowthRetryAndCleanup()) return 1;
  if (!TestDestructorReleaseFailureReachesSupervisor()) return 1;
  std::puts("C++ UTF-8 Property wrapper unit tests: OK");
  return 0;
}
