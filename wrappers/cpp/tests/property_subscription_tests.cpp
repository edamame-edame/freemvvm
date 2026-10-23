#include "freemvvm/property.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <exception>
#include <map>
#include <memory>
#include <mutex>
#include <optional>
#include <stdexcept>
#include <string>
#include <thread>
#include <utility>
#include <vector>

#ifdef _WIN32
#include <io.h>
#else
#include <unistd.h>
#endif

namespace {

struct SubscriptionEntry final {
  ui_callback callback;
  void* userdata;
  ui_destroy_callback destroy;
  bool cancelled = false;
  bool destroyed = false;
  int in_flight = 0;
};

struct FakeAbi final {
  std::mutex mutex;
  std::condition_variable condition;
  ui_handle next_subscription = 102;
  ui_status subscribe_status = UI_OK;
  ui_status cancel_status = UI_OK;
  int cancel_failures = 0;
  std::map<ui_handle, int> release_failures;
  std::map<ui_handle, std::int64_t> properties;
  std::map<ui_handle, SubscriptionEntry> subscriptions;
  std::map<ui_handle, int> release_counts;
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
  std::vector<std::pair<ui_destroy_callback, void*>> destroy;
  {
    std::lock_guard<std::mutex> lock(active->mutex);
    auto failure = active->release_failures.find(token);
    if (failure != active->release_failures.end() && failure->second > 0) {
      --failure->second;
      return UI_INTERNAL_ERROR;
    }
    ++active->release_counts[token];
    if (token == 101) {
      for (auto& [subscription, entry] : active->subscriptions) {
        (void)subscription;
        entry.cancelled = true;
        if (entry.in_flight == 0 && !entry.destroyed) {
          entry.destroyed = true;
          destroy.emplace_back(entry.destroy, entry.userdata);
        }
      }
      active->properties.erase(token);
    }
  }
  for (auto item : destroy) item.first(item.second);
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

ui_status FakeCreateI64(ui_handle, std::int64_t initial,
                        ui_handle* out_property, ui_handle*) {
  std::lock_guard<std::mutex> lock(active->mutex);
  *out_property = 101;
  active->properties[*out_property] = initial;
  return UI_OK;
}

ui_status FakeCreateBool(ui_handle, ui_bool, ui_handle* out_property,
                         ui_handle*) {
  *out_property = 101;
  return UI_OK;
}

ui_status FakeCreateF64(ui_handle, double, ui_handle* out_property,
                        ui_handle*) {
  *out_property = 101;
  return UI_OK;
}

ui_status FakeCreateDecimal(ui_handle, std::uint64_t, ui_handle* out_property,
                            ui_handle*) {
  *out_property = 101;
  return UI_OK;
}

ui_status FakeCreateUtf8(ui_handle, const std::uint8_t*, std::uint64_t,
                         ui_handle* out_property, ui_handle*) {
  *out_property = 101;
  return UI_OK;
}

ui_status FakeSubscribe(ui_handle, ui_callback callback, void* userdata,
                        ui_destroy_callback destroy,
                        ui_handle* out_subscription, ui_handle* out_error) {
  std::lock_guard<std::mutex> lock(active->mutex);
  if (active->subscribe_status != UI_OK) {
    *out_error = 900;
    return active->subscribe_status;
  }
  const ui_handle token = active->next_subscription++;
  active->subscriptions.emplace(
      token, SubscriptionEntry{callback, userdata, destroy});
  *out_subscription = token;
  return UI_OK;
}

ui_status FakeCancel(ui_handle token, ui_handle* out_error) {
  std::pair<ui_destroy_callback, void*> destroy{nullptr, nullptr};
  {
    std::lock_guard<std::mutex> lock(active->mutex);
    if (active->cancel_failures > 0) {
      --active->cancel_failures;
      *out_error = 900;
      return UI_INTERNAL_ERROR;
    }
    if (active->cancel_status != UI_OK) {
      *out_error = 900;
      return active->cancel_status;
    }
    auto found = active->subscriptions.find(token);
    if (found == active->subscriptions.end()) return UI_INVALID_HANDLE;
    auto& entry = found->second;
    entry.cancelled = true;
    if (entry.in_flight == 0 && !entry.destroyed) {
      entry.destroyed = true;
      destroy = {entry.destroy, entry.userdata};
    }
  }
  if (destroy.first != nullptr) destroy.first(destroy.second);
  return UI_OK;
}

bool Trigger(FakeAbi& fake, ui_handle token) {
  ui_callback callback = nullptr;
  void* userdata = nullptr;
  {
    std::lock_guard<std::mutex> lock(fake.mutex);
    auto found = fake.subscriptions.find(token);
    if (found == fake.subscriptions.end() || found->second.cancelled) {
      return false;
    }
    ++found->second.in_flight;
    callback = found->second.callback;
    userdata = found->second.userdata;
  }
  callback(userdata);

  std::pair<ui_destroy_callback, void*> destroy{nullptr, nullptr};
  {
    std::lock_guard<std::mutex> lock(fake.mutex);
    auto& entry = fake.subscriptions.at(token);
    --entry.in_flight;
    if (entry.cancelled && entry.in_flight == 0 && !entry.destroyed) {
      entry.destroyed = true;
      destroy = {entry.destroy, entry.userdata};
    }
    fake.condition.notify_all();
  }
  if (destroy.first != nullptr) destroy.first(destroy.second);
  return true;
}

void Wake(void*, std::uint32_t) {}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  static const freemvvm::detail::AbiApi api{
      &FakeRuntimeCreate,
      &FakeRuntimeIsUiThread,
      &FakeRuntimeSetBridge,
      &FakeRuntimeShutdown,
      &FakeRuntimeRequestShutdown,
      &FakeShutdownQuery,
      &FakeRelease,
      &FakeErrorStatus,
      &FakeErrorCode,
      &FakeCreateBool,
      &FakeCreateI64,
      &FakeCreateF64,
      &FakeCreateDecimal,
      &FakeCreateUtf8,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      nullptr,
      &FakeSubscribe,
      &FakeCancel,
  };
  return std::shared_ptr<const freemvvm::detail::AbiApi>(
      std::shared_ptr<const freemvvm::detail::AbiApi>{}, &api);
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
bool TestPropertyTypeSubscription(freemvvm::Runtime& runtime, FakeAbi& fake,
                                  T initial_value) {
  auto property_result =
      freemvvm::Property<T>::create(runtime, std::move(initial_value));
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());
  int calls = 0;
  auto subscription_result = property.subscribe([&calls] { ++calls; });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());
  ui_handle token = 0;
  {
    std::lock_guard<std::mutex> lock(fake.mutex);
    token = fake.next_subscription - 1;
  }
  const bool invoked_once = Trigger(fake, token) && calls == 1;
  const bool stopped = subscription.cancel().has_value() && !Trigger(fake, token);
  return Check(invoked_once && stopped && subscription.close().has_value() &&
                   property.close().has_value(),
               "each Property type should support subscribe and cancel");
}

bool TestAllPropertyTypesCanSubscribe() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto decimal = freemvvm::Decimal64::try_from_parts(false, 1, 42);
  const bool all_types = decimal.has_value() &&
                         TestPropertyTypeSubscription(runtime, fake, true) &&
                         TestPropertyTypeSubscription(runtime, fake,
                                                      std::int64_t{42}) &&
                         TestPropertyTypeSubscription(runtime, fake, 1.25) &&
                         TestPropertyTypeSubscription(runtime, fake,
                                                      *decimal.value()) &&
                         TestPropertyTypeSubscription(runtime, fake,
                                                      std::string("value"));
  return Check(all_types && runtime.close().has_value(),
               "bool, i64, f64, decimal, and UTF-8 Properties should subscribe");
}

class StderrCapture final {
 public:
  StderrCapture() : file_(std::tmpfile()) {
    if (file_ == nullptr) return;
#ifdef _WIN32
    saved_ = _dup(_fileno(stderr));
    if (saved_ >= 0) _dup2(_fileno(file_), _fileno(stderr));
#else
    saved_ = dup(fileno(stderr));
    if (saved_ >= 0) dup2(fileno(file_), fileno(stderr));
#endif
  }

  ~StderrCapture() {
    if (file_ == nullptr) return;
    std::fflush(stderr);
    if (saved_ >= 0) {
#ifdef _WIN32
      _dup2(saved_, _fileno(stderr));
      _close(saved_);
#else
      dup2(saved_, fileno(stderr));
      close(saved_);
#endif
    }
    std::fclose(file_);
  }

  std::string contents() {
    if (file_ == nullptr || saved_ < 0) return {};
    std::fflush(stderr);
    std::rewind(file_);
    std::string value;
    char buffer[256];
    while (const auto count = std::fread(buffer, 1, sizeof(buffer), file_)) {
      value.append(buffer, count);
    }
    return value;
  }

 private:
  std::FILE* file_ = nullptr;
  int saved_ = -1;
};

bool TestCancelReleaseAndMove() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!Check(runtime_result.has_value(), "Runtime creation should succeed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 5);
  if (!Check(property_result.has_value(), "Property creation should succeed")) {
    return false;
  }
  auto property = std::move(*property_result.value());
  int calls = 0;
  auto subscription_result = property.subscribe([&calls] { ++calls; });
  if (!Check(subscription_result.has_value() && calls == 0,
             "subscribe should not invoke an initial notification")) {
    return false;
  }
  auto subscription = std::move(*subscription_result.value());
  freemvvm::Subscription moved(std::move(subscription));
  if (!Check(!subscription.cancel().has_value(),
             "moved-from subscription should be invalid")) {
    return false;
  }
  if (!Check(Trigger(fake, 102) && calls == 1,
             "active subscription should invoke its callback")) {
    return false;
  }

  fake.cancel_failures = 1;
  auto failed_cancel = moved.cancel();
  if (!Check(!failed_cancel.has_value() &&
                 failed_cancel.error()->raw_status() == UI_INTERNAL_ERROR &&
                 failed_cancel.error()->snapshot() != nullptr,
             "failed cancel should preserve the native error")) {
    return false;
  }
  if (!Check(moved.cancel().has_value() && !Trigger(fake, 102),
             "successful cancel should stop future callbacks")) {
    return false;
  }
  fake.release_failures[102] = 1;
  auto failed_release = moved.close();
  if (!Check(!failed_release.has_value() &&
                 failed_release.error()->raw_status() == UI_INTERNAL_ERROR,
             "failed subscription release should report the native status")) {
    return false;
  }
  return Check(moved.close().has_value() &&
                   fake.release_counts[102] == 1 &&
                   property.close().has_value() && runtime.close().has_value(),
               "cancel and handle release should be explicit operations");
}

bool TestHandleReleaseDoesNotCancel() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());
  int calls = 0;
  auto subscription_result = property.subscribe([&calls] { ++calls; });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());
  if (!subscription.close().has_value()) return false;
  const bool still_active = Trigger(fake, 102);
  const bool destroyed_by_property = property.close().has_value();
  return Check(still_active && calls == 1 && destroyed_by_property &&
                   runtime.close().has_value(),
               "release alone should not cancel the active registration");
}

bool TestCallbackCanCancelItself() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());
  std::optional<freemvvm::Subscription> subscription;
  int calls = 0;
  bool cancel_succeeded = false;
  auto subscription_result = property.subscribe([&] {
    ++calls;
    auto result = subscription->cancel();
    cancel_succeeded = result.has_value();
  });
  if (!subscription_result.has_value()) return false;
  subscription.emplace(std::move(*subscription_result.value()));
  const bool invoked = Trigger(fake, 102);
  const bool invoked_again = Trigger(fake, 102);
  return Check(invoked && !invoked_again && calls == 1 && cancel_succeeded &&
                   subscription->close().has_value() && property.close().has_value() &&
                   runtime.close().has_value(),
               "a callback should be able to cancel its own registration");
}

bool TestCancelDefersClosureDestructionUntilInFlightCallbackReturns() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());

  struct BlockingCallback final {
    std::mutex mutex;
    std::condition_variable condition;
    bool entered = false;
    bool release = false;
    std::atomic<int>* destroyed;
    ~BlockingCallback() { destroyed->fetch_add(1); }
  };
  std::atomic<int> destroyed{0};
  auto state = std::make_shared<BlockingCallback>();
  state->destroyed = &destroyed;
  std::weak_ptr<BlockingCallback> weak_state = state;
  auto subscription_result = property.subscribe([state] {
    std::unique_lock<std::mutex> lock(state->mutex);
    state->entered = true;
    state->condition.notify_all();
    state->condition.wait(lock, [&state] { return state->release; });
  });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());
  state.reset();

  std::thread callback([&fake] { (void)Trigger(fake, 102); });
  {
    auto active_state = weak_state.lock();
    if (!active_state) {
      callback.join();
      return false;
    }
    std::unique_lock<std::mutex> lock(active_state->mutex);
    active_state->condition.wait(lock, [&active_state] {
      return active_state->entered;
    });
  }
  const bool cancelled = subscription.cancel().has_value();
  const bool retained_during_call = !weak_state.expired();
  if (auto active_state = weak_state.lock()) {
    {
      std::lock_guard<std::mutex> lock(active_state->mutex);
      active_state->release = true;
    }
    active_state->condition.notify_all();
  }
  callback.join();
  const bool released_after_call = weak_state.expired() && destroyed.load() == 1;
  return Check(cancelled && retained_during_call && released_after_call &&
                   !Trigger(fake, 102) && subscription.close().has_value() &&
                   property.close().has_value() && runtime.close().has_value(),
               "cancel should defer callback state destruction until in-flight returns");
}

bool TestFailedRegistrationKeepsOwnershipLocal() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());
  auto captured_state = std::make_shared<int>(1);
  std::weak_ptr<int> weak_state = captured_state;
  fake.subscribe_status = UI_OUT_OF_MEMORY;
  auto failed = property.subscribe([captured_state] {});
  captured_state.reset();
  if (!Check(!failed.has_value() &&
                 failed.error()->raw_status() == UI_OUT_OF_MEMORY,
             "registration failure should return the native status")) {
    return false;
  }
  const bool local_state_destroyed = weak_state.expired();
  return Check(local_state_destroyed && property.close().has_value() &&
                   runtime.close().has_value(),
               "failed registration should keep callback ownership local");
}

bool TestRuntimeHandlerAndStderrFallback() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());

  int calls = 0;
  freemvvm::CallbackHandler handler =
      [](const freemvvm::CallbackFailure& failure, void* userdata) {
        auto* count = static_cast<int*>(userdata);
        if (failure.kind == freemvvm::CallbackKind::kPropertyNotification &&
            failure.exception) {
          ++*count;
        }
      };
  auto registration_result = runtime.set_callback_handler(handler, &calls, nullptr);
  if (!Check(registration_result.has_value(), "handler registration should succeed")) {
    return false;
  }
  auto registration = std::move(*registration_result.value());
  auto subscription_result = property.subscribe([] {
    throw std::runtime_error("notification boom");
  });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());
  if (!Check(Trigger(fake, 102) && calls == 1,
             "callback exception should reach the Runtime handler")) {
    return false;
  }
  if (!registration.close().has_value()) return false;

  StderrCapture capture;
  if (!Trigger(fake, 102)) return false;
  const std::string diagnostic = capture.contents();
  const bool logged = diagnostic.find("notification boom") != std::string::npos;
  auto nonstandard_subscription_result = property.subscribe([] {
    throw 17;
  });
  if (!nonstandard_subscription_result.has_value()) return false;
  auto nonstandard_subscription =
      std::move(*nonstandard_subscription_result.value());
  StderrCapture nonstandard_capture;
  if (!Trigger(fake, 103)) return false;
  const std::string nonstandard_diagnostic = nonstandard_capture.contents();
  const bool nonstandard_logged =
      nonstandard_diagnostic.find("non-standard exception") != std::string::npos;
  if (!nonstandard_subscription.cancel().has_value() ||
      !nonstandard_subscription.close().has_value()) {
    return false;
  }
  auto throwing_handler_result = runtime.set_callback_handler(
      [](const freemvvm::CallbackFailure&, void*) {
        throw std::runtime_error("handler boom");
      },
      nullptr, nullptr);
  if (!throwing_handler_result.has_value()) return false;
  auto throwing_handler = std::move(*throwing_handler_result.value());
  StderrCapture handler_capture;
  if (!Trigger(fake, 102)) return false;
  const std::string handler_diagnostic = handler_capture.contents();
  const bool handler_logged =
      handler_diagnostic.find("callback failure handler threw") !=
      std::string::npos;
  return Check(logged && nonstandard_logged && handler_logged &&
                   throwing_handler.close().has_value() &&
                   subscription.cancel().has_value() &&
                   subscription.close().has_value() && property.close().has_value() &&
                   runtime.close().has_value(),
               "callback and handler exceptions should be reported to stderr");
}

bool TestHandlerRegistrationDestructorClearsHandler() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());
  auto subscription_result = property.subscribe([] {
    throw std::runtime_error("destructor callback");
  });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());
  {
    auto registration_result = runtime.set_callback_handler(
        [](const freemvvm::CallbackFailure&, void*) {}, nullptr, nullptr);
    if (!registration_result.has_value()) return false;
    auto registration = std::move(*registration_result.value());
  }
  StderrCapture capture;
  if (!Trigger(fake, 102)) return false;
  const bool logged =
      capture.contents().find("destructor callback") != std::string::npos;
  return Check(logged && subscription.cancel().has_value() &&
                   subscription.close().has_value() && property.close().has_value() &&
                   runtime.close().has_value(),
               "destroying handler registration should restore stderr fallback");
}

bool TestHandlerReplacementWaitsForInFlightCallback() {
  FakeAbi fake;
  auto runtime_result = CreateRuntime(fake);
  if (!runtime_result.has_value()) return false;
  auto runtime = std::move(*runtime_result.value());
  auto property_result = freemvvm::Property<std::int64_t>::create(runtime, 0);
  if (!property_result.has_value()) return false;
  auto property = std::move(*property_result.value());

  struct BlockingHandler final {
    std::mutex mutex;
    std::condition_variable condition;
    bool entered = false;
    bool release = false;
    std::atomic<int> destroyed{0};
    std::atomic<int> new_calls{0};
  } old_handler;
  auto handler = [](const freemvvm::CallbackFailure&, void* userdata) {
    auto* state = static_cast<BlockingHandler*>(userdata);
    std::unique_lock<std::mutex> lock(state->mutex);
    state->entered = true;
    state->condition.notify_all();
    state->condition.wait(lock, [state] { return state->release; });
  };
  auto destroy = [](void* userdata) {
    static_cast<BlockingHandler*>(userdata)->destroyed.fetch_add(1);
  };
  auto old_registration_result =
      runtime.set_callback_handler(handler, &old_handler, destroy);
  if (!old_registration_result.has_value()) return false;
  auto old_registration = std::move(*old_registration_result.value());
  auto subscription_result = property.subscribe([] {
    throw std::runtime_error("blocked callback");
  });
  if (!subscription_result.has_value()) return false;
  auto subscription = std::move(*subscription_result.value());

  std::thread callback([&fake] { (void)Trigger(fake, 102); });
  {
    std::unique_lock<std::mutex> lock(old_handler.mutex);
    old_handler.condition.wait(lock, [&old_handler] { return old_handler.entered; });
  }
  auto new_registration_result = runtime.set_callback_handler(
      [](const freemvvm::CallbackFailure&, void* userdata) {
        static_cast<BlockingHandler*>(userdata)->new_calls.fetch_add(1);
      }, &old_handler, nullptr);
  if (!Check(new_registration_result.has_value() &&
                 old_handler.destroyed.load() == 0,
             "replacing a handler should keep an in-flight record alive")) {
    {
      std::lock_guard<std::mutex> lock(old_handler.mutex);
      old_handler.release = true;
    }
    old_handler.condition.notify_all();
    callback.join();
    return false;
  }
  auto new_registration = std::move(*new_registration_result.value());
  if (!old_registration.close().has_value()) return false;
  {
    std::lock_guard<std::mutex> lock(old_handler.mutex);
    old_handler.release = true;
  }
  old_handler.condition.notify_all();
  callback.join();

  const bool old_destroyed_once = old_handler.destroyed.load() == 1;
  const bool still_reports_to_new_handler = Trigger(fake, 102);
  const bool new_handler_called = still_reports_to_new_handler &&
                                  old_handler.new_calls.load() == 1;
  const bool cleanup = subscription.cancel().has_value() &&
                       subscription.close().has_value() &&
                       new_registration.close().has_value() &&
                       property.close().has_value() && runtime.close().has_value();
  return Check(old_destroyed_once && new_handler_called && cleanup,
               "stale registration close should not clear the new handler");
}

}  // namespace

int main() {
  if (!TestAllPropertyTypesCanSubscribe()) return 1;
  if (!TestCancelReleaseAndMove()) return 1;
  if (!TestHandleReleaseDoesNotCancel()) return 1;
  if (!TestCallbackCanCancelItself()) return 1;
  if (!TestCancelDefersClosureDestructionUntilInFlightCallbackReturns()) return 1;
  if (!TestFailedRegistrationKeepsOwnershipLocal()) return 1;
  if (!TestRuntimeHandlerAndStderrFallback()) return 1;
  if (!TestHandlerRegistrationDestructorClearsHandler()) return 1;
  if (!TestHandlerReplacementWaitsForInFlightCallback()) return 1;
  std::puts("C++ Property subscription wrapper unit tests: OK");
  return 0;
}
