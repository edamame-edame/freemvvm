#include "runtime_internal.hpp"

#include <freemvvm/abi.h>
#include <freemvvm/ui_controls.hpp>

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <memory>
#include <optional>
#include <string>
#include <type_traits>
#include <utility>

namespace {

struct FakeAbi final {
  ui_handle next = 100;
  ui_status subscribe_status = UI_OK;
  ui_input_callback callback = nullptr;
  void* userdata = nullptr;
  ui_destroy_callback destroy = nullptr;
  std::uint32_t kind = 1;
  std::uint32_t key = 65;
  std::uint32_t modifiers = 3;
  std::string text = "hello";
  double x = 12.5;
  double y = 22.25;
  bool handled = false;
  bool cancelled_flag = false;
  bool in_callback = false;
  bool destroy_pending = false;
  int destroyed = 0;
  int cancelled = 0;
  int failure_count = 0;
  freemvvm::CallbackKind failure_kind = freemvvm::CallbackKind::kPropertyNotification;
};

FakeAbi* active = nullptr;

ui_status RuntimeCreate(ui_handle* out, ui_handle*) {
  *out = 1;
  return UI_OK;
}
ui_status RuntimeIsUiThread(ui_handle, ui_bool* out) {
  *out = 1;
  return UI_OK;
}
ui_status SetBridge(ui_handle, ui_wake_callback, void*, ui_destroy_callback,
                    ui_handle*) {
  return UI_OK;
}
ui_status Shutdown(ui_handle, ui_handle*) { return UI_OK; }
ui_status Release(ui_handle) { return UI_OK; }
ui_status CreateWidget(ui_handle, ui_handle* out, ui_handle*) {
  *out = ++active->next;
  return UI_OK;
}
ui_status CreateTextWidget(ui_handle, const std::uint8_t*, std::uint64_t,
                           ui_handle* out, ui_handle*) {
  *out = ++active->next;
  return UI_OK;
}
ui_status Subscribe(ui_handle, std::uint64_t, ui_input_callback callback,
                    void* userdata, ui_destroy_callback destroy,
                    ui_handle* out, ui_handle* error) {
  if (active->subscribe_status != UI_OK) return active->subscribe_status;
  active->callback = callback;
  active->userdata = userdata;
  active->destroy = destroy;
  active->cancelled_flag = false;
  active->destroy_pending = false;
  *out = ++active->next;
  *error = 0;
  return UI_OK;
}
ui_status Cancel(ui_handle, ui_handle*) {
  ++active->cancelled;
  active->cancelled_flag = true;
  if (active->destroy != nullptr) {
    if (active->in_callback) {
      active->destroy_pending = true;
    } else {
      active->destroy(active->userdata);
      active->destroy = nullptr;
      ++active->destroyed;
    }
  }
  return UI_OK;
}
ui_status EventKind(ui_handle, std::uint32_t* out) {
  *out = active->kind;
  return UI_OK;
}
ui_status EventKey(ui_handle, std::uint32_t* key, std::uint32_t* modifiers) {
  if (active->kind != 1 && active->kind != 2) return UI_WRONG_TYPE;
  *key = active->key;
  *modifiers = active->modifiers;
  return UI_OK;
}
ui_status EventText(ui_handle, std::uint8_t* buffer, std::uint64_t capacity,
                    std::uint64_t* length) {
  if (active->kind != 3) return UI_WRONG_TYPE;
  *length = active->text.size();
  if (capacity < active->text.size()) return UI_BUFFER_TOO_SMALL;
  if (!active->text.empty()) {
    std::memcpy(buffer, active->text.data(), active->text.size());
  }
  return UI_OK;
}
ui_status EventPointer(ui_handle, double* x, double* y, std::uint32_t* button) {
  if (active->kind != 4 && active->kind != 5) return UI_WRONG_TYPE;
  *x = active->x;
  *y = active->y;
  *button = 1;
  return UI_OK;
}
ui_status SetHandled(ui_handle, ui_bool handled, ui_handle*) {
  active->handled = handled != 0;
  return UI_OK;
}
void CallbackFailure(const freemvvm::CallbackFailure& failure,
                     void* userdata) {
  auto* fake = static_cast<FakeAbi*>(userdata);
  ++fake->failure_count;
  fake->failure_kind = failure.kind;
}

void DispatchInput(ui_handle current_widget = 501, ui_handle original_target = 502,
                   ui_handle borrowed_event = 503) {
  if (active->cancelled_flag || active->callback == nullptr) return;
  active->in_callback = true;
  active->callback(active->userdata, current_widget, original_target,
                   borrowed_event);
  active->in_callback = false;
  if (active->destroy_pending && active->destroy != nullptr) {
    active->destroy(active->userdata);
    active->destroy = nullptr;
    active->destroy_pending = false;
    ++active->destroyed;
  }
}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  auto api = std::make_shared<freemvvm::detail::AbiApi>(
      *freemvvm::detail::linked_api());
  api->runtime_create = &RuntimeCreate;
  api->runtime_is_ui_thread = &RuntimeIsUiThread;
  api->runtime_set_ui_loop_bridge = &SetBridge;
  api->runtime_shutdown = &Shutdown;
  api->handle_release = &Release;
  api->window_create = &CreateWidget;
  api->stack_panel_create = &CreateWidget;
  api->text_create = &CreateTextWidget;
  api->text_box_create = &CreateTextWidget;
  api->button_create = &CreateTextWidget;
  api->widget_input_subscribe = &Subscribe;
  api->input_event_kind = &EventKind;
  api->input_event_key = &EventKey;
  api->input_event_text = &EventText;
  api->input_event_pointer = &EventPointer;
  api->input_event_set_handled = &SetHandled;
  api->subscription_cancel = &Cancel;
  return api;
}

void Wake(void*, std::uint32_t) {}

bool Check(bool condition, const char* message) {
  if (condition) return true;
  std::fprintf(stderr, "FAILED: %s\n", message);
  return false;
}

static_assert(!std::is_copy_constructible<freemvvm::InputEventView>::value);
static_assert(!std::is_move_constructible<freemvvm::InputEventView>::value);
static_assert(!std::is_copy_constructible<freemvvm::Subscription>::value);

bool TestInputEventsAndSubscription() {
  FakeAbi fake;
  active = &fake;
  auto runtime_result = freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
  if (!Check(runtime_result.has_value(), "create fake Runtime")) return false;
  auto runtime = std::move(*runtime_result.value());
  auto handler_result =
      runtime.set_callback_handler(&CallbackFailure, &fake, nullptr);
  if (!Check(handler_result.has_value(), "install callback failure handler")) {
    return false;
  }
  auto handler = std::move(*handler_result.value());
  auto widget_result = freemvvm::Button::create(runtime, "button");
  if (!Check(widget_result.has_value(), "create fake Button")) return false;
  auto widget = std::move(*widget_result.value());

  fake.subscribe_status = UI_UNSUPPORTED;
  auto failed_registration = widget.subscribe_input(
      freemvvm::InputEventMask::kAll,
      [](freemvvm::InputEventView&) {});
  fake.subscribe_status = UI_OK;
  if (!Check(!failed_registration.has_value() && fake.destroy == nullptr,
             "registration failure keeps callback root wrapper-owned")) {
    return false;
  }

  int calls = 0;
  bool payloads_ok = true;
  auto subscription_result = widget.subscribe_input(
      freemvvm::InputEventMask::kAll,
      [&calls, &payloads_ok](freemvvm::InputEventView& event) {
        ++calls;
        auto kind = event.kind();
        if (!kind.has_value()) {
          payloads_ok = false;
          return;
        }
        switch (*kind.value()) {
          case freemvvm::InputEventKind::kKeyDown:
          case freemvvm::InputEventKind::kKeyUp: {
            auto key = event.key();
            auto modifiers = event.modifiers();
            payloads_ok = payloads_ok && key.has_value() &&
                          *key.value() == 65 && modifiers.has_value() &&
                          *modifiers.value() == 3 && !event.text().has_value() &&
                          !event.pointer_position().has_value();
            break;
          }
          case freemvvm::InputEventKind::kTextCommit: {
            auto text = event.text();
            payloads_ok = payloads_ok && text.has_value() &&
                          *text.value() == "hello" && !event.key().has_value();
            break;
          }
          case freemvvm::InputEventKind::kPrimaryPointerDown:
          case freemvvm::InputEventKind::kPrimaryPointerUp: {
            auto point = event.pointer_position();
            payloads_ok = payloads_ok && point.has_value() &&
                          point.value()->x == 12.5 &&
                          point.value()->y == 22.25 && !event.key().has_value();
            break;
          }
        }
        if (!event.set_handled().has_value()) payloads_ok = false;
      });
  if (!Check(subscription_result.has_value(), "subscribe all input events")) {
    return false;
  }
  auto subscription = std::move(*subscription_result.value());
  for (fake.kind = 1; fake.kind <= 5; ++fake.kind) {
    DispatchInput();
    if (!fake.handled) return Check(false, "event marked handled");
  }
  if (!Check(calls == 5 && payloads_ok,
             "read kind-specific payloads and reject wrong-kind getters")) {
    return false;
  }
  if (!subscription.cancel().has_value() || fake.destroy != nullptr) {
    return Check(false, "cancel input subscription and destroy callback root");
  }
  if (!Check(fake.destroyed == 1, "cancel destroys callback root once")) {
    return false;
  }
  DispatchInput();
  if (!Check(calls == 5, "cancel prevents later callback entry")) return false;

  fake.handled = false;
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
  auto throwing = widget.subscribe_input(
      freemvvm::InputEventMask::kKeyDown,
      [](freemvvm::InputEventView&) { throw 42; });
  if (!Check(throwing.has_value(), "subscribe throwing callback")) return false;
  auto throwing_subscription = std::move(*throwing.value());
  fake.kind = 1;
  DispatchInput();
  if (!Check(fake.handled && fake.failure_count == 1 &&
                 fake.failure_kind == freemvvm::CallbackKind::kInputEvent,
             "callback exception is reported and event handled")) {
    return false;
  }
  if (!throwing_subscription.cancel().has_value()) return false;
#endif

  std::optional<freemvvm::Subscription> self_cancelling;
  const int destroy_count_before_self_cancel = fake.destroyed;
  auto self_cancel_result = widget.subscribe_input(
      freemvvm::InputEventMask::kKeyDown,
      [&self_cancelling](freemvvm::InputEventView&) {
        (void)self_cancelling->cancel();
      });
  if (!Check(self_cancel_result.has_value(), "subscribe self-cancelling input")) {
    return false;
  }
  self_cancelling.emplace(std::move(*self_cancel_result.value()));
  const int calls_before_self_cancel = calls;
  fake.kind = 1;
  DispatchInput();
  if (!Check(fake.cancelled >= 2 && fake.destroy == nullptr,
             "self-cancel defers callback root destroy until callback exits")) {
    return false;
  }
  if (!Check(fake.destroyed == destroy_count_before_self_cancel + 1,
             "self-cancel destroys callback root once")) {
    return false;
  }
  DispatchInput();
  if (!Check(calls == calls_before_self_cancel,
             "self-cancel prevents a subsequent callback entry")) {
    return false;
  }

  auto release_only_result = widget.subscribe_input(
      freemvvm::InputEventMask::kAll,
      [&calls](freemvvm::InputEventView&) { ++calls; });
  if (!Check(release_only_result.has_value(), "subscribe release-only case")) {
    return false;
  }
  auto release_only = std::move(*release_only_result.value());
  const int cancellations_before_release = fake.cancelled;
  const int destroy_count_before_release = fake.destroyed;
  if (!Check(release_only.close().has_value() &&
                 fake.cancelled == cancellations_before_release,
             "close releases subscription handle without cancel")) {
    return false;
  }
  fake.kind = 2;
  DispatchInput();
  if (!Check(calls == 6 && fake.destroy != nullptr,
             "close alone leaves Core callback registration active")) {
    return false;
  }
  fake.destroy(fake.userdata);
  fake.destroy = nullptr;
  ++fake.destroyed;
  if (!Check(fake.destroyed == destroy_count_before_release + 1,
             "external teardown destroys callback root once")) {
    return false;
  }
  if (!widget.close().has_value() || !handler.close().has_value() ||
      !runtime.close().has_value()) {
    return Check(false, "release fake owners");
  }
  return true;
}

}  // namespace

int main() { return TestInputEventsAndSubscription() ? 0 : 1; }
