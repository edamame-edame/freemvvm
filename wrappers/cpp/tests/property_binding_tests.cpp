#include "runtime_internal.hpp"

#include <freemvvm/abi.h>
#include <freemvvm/property.hpp>

#include <cstdint>
#include <cstdio>
#include <functional>
#include <map>
#include <memory>
#include <string>
#include <utility>
#include <vector>

namespace {

constexpr std::uint32_t kBindOnce = 1;
constexpr std::uint32_t kBindOneWay = 2;
constexpr std::uint32_t kBindTwoWay = 3;
constexpr std::uint32_t kDirectionSourceToTarget = 1;
constexpr std::uint32_t kDirectionTargetToSource = 2;

struct CallbackEntry final {
  ui_callback callback = nullptr;
  void* userdata = nullptr;
  ui_destroy_callback destroy = nullptr;
  bool cancelled = false;
};

struct FakeAbi final {
  ui_handle next_token = 1000;
  ui_status binding_create_status = UI_OK;
  ui_status binding_cancel_status = UI_OK;
  ui_status binding_subscribe_status = UI_OK;
  ui_status release_status = UI_OK;
  ui_status subscription_cancel_status = UI_OK;
  std::map<ui_handle, int> release_failures;
  int binding_cancel_calls = 0;
  bool has_binding_error = false;
  std::uint32_t error_direction = kDirectionSourceToTarget;
  std::uint32_t last_mode = 0;
  ui_handle last_binding = 0;
  int callback_destroys = 0;
  std::map<ui_handle, std::int64_t> i64_values;
  std::map<ui_handle, int> releases;
  std::map<ui_handle, CallbackEntry> subscriptions;
};

FakeAbi* active = nullptr;

ui_status FakeRuntimeCreate(ui_handle* out_runtime, ui_handle*) {
  *out_runtime = 10;
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
  *out_request = 20;
  return UI_OK;
}

ui_status FakeShutdownQuery(ui_handle, std::uint32_t* out_state,
                            std::uint32_t* out_reason, ui_handle*) {
  *out_state = UI_SHUTDOWN_REQUEST_COMPLETED;
  *out_reason = 0;
  return UI_OK;
}

ui_status FakeRelease(ui_handle token) {
  auto failure = active->release_failures.find(token);
  if (failure != active->release_failures.end() && failure->second > 0) {
    --failure->second;
    return UI_INTERNAL_ERROR;
  }
  if (active->release_status != UI_OK) return active->release_status;
  ++active->releases[token];
  return UI_OK;
}

ui_status FakeErrorStatus(ui_handle, std::uint32_t* out_status) {
  *out_status = UI_VALIDATION_FAILED;
  return UI_OK;
}

ui_status FakeErrorCode(ui_handle, std::uint32_t* out_domain,
                        std::uint32_t* out_code) {
  *out_domain = 2;
  *out_code = 17;
  return UI_OK;
}

ui_status FakeCreateBool(ui_handle, ui_bool, ui_handle* out_property,
                         ui_handle*) {
  *out_property = active->next_token++;
  return UI_OK;
}

ui_status FakeCreateI64(ui_handle, std::int64_t initial,
                        ui_handle* out_property, ui_handle*) {
  *out_property = active->next_token++;
  active->i64_values[*out_property] = initial;
  return UI_OK;
}

ui_status FakeCreateF64(ui_handle, double, ui_handle* out_property,
                        ui_handle*) {
  *out_property = active->next_token++;
  return UI_OK;
}

ui_status FakeCreateDecimal(ui_handle, std::uint64_t, ui_handle* out_property,
                            ui_handle*) {
  *out_property = active->next_token++;
  return UI_OK;
}

ui_status FakeCreateUtf8(ui_handle, const std::uint8_t*, std::uint64_t,
                         ui_handle* out_property, ui_handle*) {
  *out_property = active->next_token++;
  return UI_OK;
}

ui_status FakeGetI64(ui_handle property, std::int64_t* out_value) {
  *out_value = active->i64_values[property];
  return UI_OK;
}

ui_status FakeBindingCreate(ui_handle, ui_handle, std::uint32_t mode,
                            ui_handle* out_binding, ui_handle* out_error) {
  active->last_mode = mode;
  if (active->binding_create_status != UI_OK) {
    *out_error = 900;
    return active->binding_create_status;
  }
  *out_error = 0;
  if (mode == kBindOnce) {
    *out_binding = 0;
  } else {
    *out_binding = active->next_token++;
    active->last_binding = *out_binding;
  }
  return UI_OK;
}

ui_status FakeBindingCancel(ui_handle, ui_handle* out_error) {
  ++active->binding_cancel_calls;
  if (active->binding_cancel_status != UI_OK) *out_error = 901;
  return active->binding_cancel_status;
}

ui_status FakeBindingErrorSnapshot(ui_handle, ui_handle* out_error,
                                   ui_handle* out_rejected,
                                   std::uint32_t* out_direction) {
  if (!active->has_binding_error) {
    *out_error = 0;
    *out_rejected = 0;
    *out_direction = 0;
    return UI_OK;
  }
  *out_error = 902;
  *out_rejected = 903;
  *out_direction = active->error_direction;
  active->i64_values[*out_rejected] = 41;
  return UI_OK;
}

ui_status FakeBindingSubscribe(ui_handle, ui_callback callback, void* userdata,
                               ui_destroy_callback destroy,
                               ui_handle* out_subscription, ui_handle*) {
  if (active->binding_subscribe_status != UI_OK) return active->binding_subscribe_status;
  const ui_handle token = active->next_token++;
  active->subscriptions[token] = {callback, userdata, destroy, false};
  *out_subscription = token;
  return UI_OK;
}

ui_status FakeSubscriptionCancel(ui_handle token, ui_handle*) {
  if (active->subscription_cancel_status != UI_OK) {
    return active->subscription_cancel_status;
  }
  auto& entry = active->subscriptions[token];
  if (!entry.cancelled) {
    entry.cancelled = true;
    entry.destroy(entry.userdata);
    ++active->callback_destroys;
  }
  return UI_OK;
}

void FakeTrigger(ui_handle token) {
  const auto entry = active->subscriptions.at(token);
  if (!entry.cancelled) entry.callback(entry.userdata);
}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  static const freemvvm::detail::AbiApi api = [] {
    freemvvm::detail::AbiApi value{};
    value.runtime_create = &FakeRuntimeCreate;
    value.runtime_is_ui_thread = &FakeRuntimeIsUiThread;
    value.runtime_set_ui_loop_bridge = &FakeRuntimeSetBridge;
    value.runtime_shutdown = &FakeRuntimeShutdown;
    value.runtime_request_shutdown = &FakeRuntimeRequestShutdown;
    value.shutdown_request_query = &FakeShutdownQuery;
    value.handle_release = &FakeRelease;
    value.error_status = &FakeErrorStatus;
    value.error_code = &FakeErrorCode;
    value.property_create_bool = &FakeCreateBool;
    value.property_create_i64 = &FakeCreateI64;
    value.property_create_f64 = &FakeCreateF64;
    value.property_create_decimal = &FakeCreateDecimal;
    value.property_create_utf8 = &FakeCreateUtf8;
    value.property_get_i64 = &FakeGetI64;
    value.subscription_cancel = &FakeSubscriptionCancel;
    value.binding_create = &FakeBindingCreate;
    value.binding_cancel = &FakeBindingCancel;
    value.binding_error_snapshot = &FakeBindingErrorSnapshot;
    value.binding_subscribe_error = &FakeBindingSubscribe;
    return value;
  }();
  return std::shared_ptr<const freemvvm::detail::AbiApi>(
      std::shared_ptr<const freemvvm::detail::AbiApi>{}, &api);
}

void Wake(void*, std::uint32_t) {}

void CaptureCallbackFailure(const freemvvm::CallbackFailure& failure,
                            void* userdata) {
  *static_cast<freemvvm::CallbackKind*>(userdata) = failure.kind;
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

template <typename T>
bool TestModesForType(freemvvm::Runtime& runtime, T initial) {
  auto source_result = freemvvm::Property<T>::create(runtime, initial);
  auto target_result = freemvvm::Property<T>::create(runtime, initial);
  if (!source_result.has_value() || !target_result.has_value()) return false;
  auto source = std::move(*source_result.value());
  auto target = std::move(*target_result.value());

  auto once = source.bind_to(target, freemvvm::BindingMode::kOnce);
  if (!once.has_value() || once.value()->has_value() ||
      active->last_mode != kBindOnce) {
    return false;
  }
  auto one_way = source.bind_to(target, freemvvm::BindingMode::kOneWay);
  if (!one_way.has_value() || !one_way.value()->has_value() ||
      active->last_mode != kBindOneWay) {
    return false;
  }
  auto binding = std::move(**one_way.value());
  if (!binding.cancel().has_value() || !binding.close().has_value()) {
    return false;
  }
  auto two_way = source.bind_to(target, freemvvm::BindingMode::kTwoWay);
  if (!two_way.has_value() || !two_way.value()->has_value() ||
      active->last_mode != kBindTwoWay) {
    return false;
  }
  auto two_way_binding = std::move(**two_way.value());
  return two_way_binding.close().has_value() && source.close().has_value() &&
         target.close().has_value();
}

bool TestModesAllTypes() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto decimal = freemvvm::Decimal64::try_from_parts(false, 1, 42);
  const bool all_types = decimal.has_value() &&
                         TestModesForType(runtime, true) &&
                         TestModesForType(runtime, std::int64_t{42}) &&
                         TestModesForType(runtime, 1.25) &&
                         TestModesForType(runtime, *decimal.value()) &&
                         TestModesForType(runtime, std::string("value"));
  return Check(all_types && runtime.close().has_value(),
               "all five Property types should support each binding mode");
}

bool TestErrorSnapshotAndSubscription() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto source_result = freemvvm::Property<std::int64_t>::create(runtime, 3);
  auto target_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!source_result.has_value() || !target_result.has_value()) return false;
  auto source = std::move(*source_result.value());
  auto target = std::move(*target_result.value());
  auto binding_result = source.bind_to(target, freemvvm::BindingMode::kOneWay);
  if (!binding_result.has_value() || !binding_result.value()->has_value()) {
    return false;
  }
  auto binding = std::move(**binding_result.value());

  auto empty = binding.error_snapshot();
  if (!Check(empty.has_value() && !empty.value()->has_value(),
             "empty Binding error state should return an empty optional")) {
    return false;
  }
  fake.has_binding_error = true;
  fake.error_direction = kDirectionTargetToSource;
  auto snapshot_result = binding.error_snapshot();
  if (!Check(snapshot_result.has_value() &&
                 snapshot_result.value()->has_value(),
             "active Binding error should return an owned snapshot")) {
    return false;
  }
  auto snapshot = std::move(**snapshot_result.value());
  auto rejected = snapshot.rejected_property().get();
  if (!Check(snapshot.error().raw_status() == UI_VALIDATION_FAILED &&
                 snapshot.error().snapshot() != nullptr &&
                 snapshot.error().snapshot()->valid() && rejected.has_value() &&
                 *rejected.value() == 41 &&
                 snapshot.direction() ==
                     freemvvm::BindingDirection::kTargetToSource,
             "Binding error should preserve status, rejected Property, and direction")) {
    return false;
  }

  int notifications = 0;
  bool callback_snapshot = false;
  auto observed_kind = freemvvm::CallbackKind::kPropertyNotification;
  auto handler_result = runtime.set_callback_handler(
      &CaptureCallbackFailure, &observed_kind, nullptr);
  if (!handler_result.has_value()) return false;
  auto handler = std::move(*handler_result.value());
  fake.binding_subscribe_status = UI_INTERNAL_ERROR;
  const auto failed_subscription = binding.subscribe_error([] {});
  if (!Check(!failed_subscription.has_value() &&
                 fake.callback_destroys == 0,
             "failed error subscription should retain and destroy no Core state")) {
    return false;
  }
  fake.binding_subscribe_status = UI_OK;
  auto subscription_result = binding.subscribe_error([&] {
    ++notifications;
    auto current = binding.error_snapshot();
    callback_snapshot = current.has_value() && current.value()->has_value();
    throw 7;
  });
  if (!Check(subscription_result.has_value() && notifications == 0,
             "error subscription should not invoke an initial callback")) {
    return false;
  }
  auto subscription = std::move(*subscription_result.value());
  const ui_handle subscription_token = fake.next_token - 1;
  FakeTrigger(subscription_token);
  const bool invoked = notifications == 1;
  const bool cancelled = subscription.cancel().has_value();
  const bool destroyed = fake.callback_destroys == 1;
  const bool closed = subscription.close().has_value();
  const bool handler_closed = handler.close().has_value();
  fake.has_binding_error = false;
  auto cleared_result = binding.error_snapshot();
  auto stable_rejected = snapshot.rejected_property().get();
  const bool snapshot_stable = snapshot.error().raw_status() ==
                                   UI_VALIDATION_FAILED &&
                               stable_rejected.has_value() &&
                               *stable_rejected.value() == 41;
  const bool binding_closed = binding.cancel().has_value() &&
                              binding.close().has_value();
  const bool properties_closed = source.close().has_value() &&
                                 target.close().has_value();
  return Check(invoked && callback_snapshot && cancelled && destroyed && closed &&
                   handler_closed &&
                   observed_kind ==
                       freemvvm::CallbackKind::kBindingErrorNotification &&
                   binding_closed && cleared_result.has_value() &&
                   !cleared_result.value()->has_value() && snapshot_stable &&
                   properties_closed && runtime.close().has_value(),
               "error subscription cancellation and owned snapshots should clean up");
}

bool TestFailuresAndReleaseSeparation() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto source_result = freemvvm::Property<std::int64_t>::create(runtime, 1);
  auto target_result = freemvvm::Property<std::int64_t>::create(runtime, 2);
  if (!source_result.has_value() || !target_result.has_value()) return false;
  auto source = std::move(*source_result.value());
  auto target = std::move(*target_result.value());

  fake.binding_create_status = UI_WRONG_RUNTIME;
  auto failed_create = source.bind_to(target, freemvvm::BindingMode::kTwoWay);
  if (!Check(!failed_create.has_value() &&
                 failed_create.error()->raw_status() == UI_WRONG_RUNTIME &&
                 failed_create.error()->snapshot() != nullptr,
             "failed Binding creation should preserve the C ABI error")) {
    return false;
  }
  fake.binding_create_status = UI_OK;
  auto result = source.bind_to(target, freemvvm::BindingMode::kOneWay);
  if (!result.has_value() || !result.value()->has_value()) return false;
  auto binding = std::move(**result.value());
  const ui_handle token = fake.last_binding;

  fake.binding_cancel_status = UI_INTERNAL_ERROR;
  if (!Check(!binding.cancel().has_value() &&
                 fake.releases[token] == 0 &&
                 fake.binding_cancel_calls == 1,
             "failed cancel should preserve the owned Binding token")) {
    return false;
  }
  fake.binding_cancel_status = UI_OK;
  if (!Check(binding.cancel().has_value() &&
                 fake.binding_cancel_calls == 2,
             "cancel should be retryable after failure")) {
    return false;
  }
  fake.release_failures[token] = 1;
  const bool failed_close = !binding.close().has_value();
  const bool retried_close = binding.close().has_value();

  auto release_only_result = source.bind_to(target, freemvvm::BindingMode::kTwoWay);
  if (!release_only_result.has_value() ||
      !release_only_result.value()->has_value()) {
    return false;
  }
  auto release_only = std::move(**release_only_result.value());
  const int cancels_before_release = fake.binding_cancel_calls;
  const bool release_only_closed = release_only.close().has_value();
  const bool close_did_not_cancel =
      fake.binding_cancel_calls == cancels_before_release;
  const bool props_closed = source.close().has_value() && target.close().has_value();
  return Check(failed_close && retried_close && release_only_closed &&
                   close_did_not_cancel && props_closed &&
                   runtime.close().has_value(),
               "Binding close should retry release without cancelling the connection");
}

}  // namespace

int main() {
  return TestModesAllTypes() && TestErrorSnapshotAndSubscription() &&
                 TestFailuresAndReleaseSeparation()
             ? 0
             : 1;
}
