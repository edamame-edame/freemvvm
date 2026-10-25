#include "runtime_internal.hpp"

#include <freemvvm/abi.h>
#include <freemvvm/ui_controls.hpp>

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

namespace {

struct FakeAbi final {
  ui_handle next_handle = 100;
  ui_handle content = 0;
  ui_status create_status = UI_OK;
  ui_status property_status = UI_OK;
  ui_status release_status = UI_OK;
  bool partial_create = false;
  std::vector<ui_handle> children;
  std::vector<std::string> initial_texts;
  std::uint32_t orientation = 1;
  std::uint32_t last_decision = 0;
  std::uint32_t alignment = 0;
  int peer_closes = 0;
  ui_window_close_callback close_callback = nullptr;
  void* close_userdata = nullptr;
  ui_destroy_callback close_destroy = nullptr;
  int focus_requests = 0;
  int destroys = 0;
};

FakeAbi* active = nullptr;

ui_status RuntimeCreate(ui_handle* out, ui_handle*) {
  *out = 10;
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
ui_status Release(ui_handle) {
  if (active->release_status != UI_OK) return active->release_status;
  ++active->destroys;
  return UI_OK;
}
ui_handle NewHandle() { return ++active->next_handle; }
ui_status Create(ui_handle, ui_handle* out, ui_handle*) {
  if (active->create_status != UI_OK && !active->partial_create) {
    return active->create_status;
  }
  *out = NewHandle();
  return active->create_status;
}
ui_status CreateText(ui_handle, const std::uint8_t* bytes, std::uint64_t size,
                     ui_handle* out, ui_handle*) {
  active->initial_texts.emplace_back(reinterpret_cast<const char*>(bytes),
                                      static_cast<std::size_t>(size));
  *out = NewHandle();
  return UI_OK;
}
ui_status Property(ui_handle, ui_handle* out) {
  if (active->property_status != UI_OK) return active->property_status;
  *out = NewHandle();
  return UI_OK;
}
ui_status Focus(ui_handle, ui_handle*) {
  ++active->focus_requests;
  return UI_OK;
}
ui_status WindowContent(ui_handle, ui_handle* out, ui_handle*) {
  *out = active->content == 0 ? 0 : NewHandle();
  return UI_OK;
}
ui_status SetContent(ui_handle, ui_handle child, ui_handle*) {
  active->content = child;
  return UI_OK;
}
ui_status ClearContent(ui_handle, ui_handle*) {
  active->content = 0;
  return UI_OK;
}
ui_status PeerWindow(ui_handle, ui_handle*) { return UI_OK; }
ui_status ClosePeer(ui_handle, ui_handle*) {
  ++active->peer_closes;
  return UI_OK;
}
ui_status SubscribeClose(ui_handle, ui_window_close_callback callback,
                         void* userdata, ui_destroy_callback destroy,
                         ui_handle* out, ui_handle*) {
  active->close_callback = callback;
  active->close_userdata = userdata;
  active->close_destroy = destroy;
  *out = NewHandle();
  return UI_OK;
}
ui_status Cancel(ui_handle, ui_handle*) {
  if (active->close_destroy != nullptr) {
    active->close_destroy(active->close_userdata);
    active->close_destroy = nullptr;
  }
  return UI_OK;
}
ui_status Defer(ui_handle, ui_handle* out, ui_handle*) {
  *out = NewHandle();
  return UI_OK;
}
ui_status Resolve(ui_handle, std::uint32_t decision, ui_handle*) {
  active->last_decision = decision;
  return UI_OK;
}
ui_status Insert(ui_handle, ui_handle child, std::uint64_t index,
                 ui_handle*) {
  if (index > active->children.size()) return UI_INVALID_ARGUMENT;
  active->children.insert(active->children.begin() + index, child);
  return UI_OK;
}
ui_status Remove(ui_handle, ui_handle child, ui_handle*) {
  const auto found = std::find(active->children.begin(), active->children.end(),
                               child);
  if (found == active->children.end()) return UI_INVALID_ARGUMENT;
  active->children.erase(found);
  return UI_OK;
}
ui_status Move(ui_handle, std::uint64_t from, std::uint64_t to, ui_handle*) {
  if (from >= active->children.size() || to >= active->children.size()) {
    return UI_INVALID_ARGUMENT;
  }
  auto child = active->children[from];
  active->children.erase(active->children.begin() + from);
  active->children.insert(active->children.begin() + to, child);
  return UI_OK;
}
ui_status ChildCount(ui_handle, std::uint64_t* out) {
  *out = active->children.size();
  return UI_OK;
}
ui_status ChildAt(ui_handle, std::uint64_t index, ui_handle* out, ui_handle*) {
  if (index >= active->children.size()) return UI_INVALID_ARGUMENT;
  *out = NewHandle();
  return UI_OK;
}
ui_status Orientation(ui_handle, std::uint32_t* out) {
  *out = active->orientation;
  return UI_OK;
}
ui_status SetOrientation(ui_handle, std::uint32_t value, ui_handle*) {
  active->orientation = value;
  return UI_OK;
}
ui_status SetAlignment(ui_handle, ui_handle, std::uint32_t value, ui_handle*) {
  active->alignment = value;
  return UI_OK;
}

std::shared_ptr<const freemvvm::detail::AbiApi> MakeApi() {
  auto api = std::make_shared<freemvvm::detail::AbiApi>(
      *freemvvm::detail::linked_api());
  api->runtime_create = &RuntimeCreate;
  api->runtime_is_ui_thread = &RuntimeIsUiThread;
  api->runtime_set_ui_loop_bridge = &SetBridge;
  api->runtime_shutdown = &Shutdown;
  api->handle_release = &Release;
  api->window_create = &Create;
  api->stack_panel_create = &Create;
  api->text_create = &CreateText;
  api->text_box_create = &CreateText;
  api->button_create = &CreateText;
  api->widget_is_enabled_property = &Property;
  api->widget_accessible_name_property = &Property;
  api->widget_request_focus = &Focus;
  api->window_content = &WindowContent;
  api->window_set_content = &SetContent;
  api->window_clear_content = &ClearContent;
  api->window_open = &PeerWindow;
  api->window_close = &ClosePeer;
  api->window_close_request_subscribe = &SubscribeClose;
  api->subscription_cancel = &Cancel;
  api->window_close_request_defer = &Defer;
  api->window_close_request_resolve = &Resolve;
  api->container_insert = &Insert;
  api->container_remove = &Remove;
  api->container_move = &Move;
  api->container_child_count = &ChildCount;
  api->container_child_at = &ChildAt;
  api->stack_panel_orientation = &Orientation;
  api->stack_panel_set_orientation = &SetOrientation;
  api->stack_panel_set_child_align = &SetAlignment;
  return api;
}

void Wake(void*, std::uint32_t) {}

bool Check(bool condition, const char* message) {
  if (condition) return true;
  std::fprintf(stderr, "FAILED: %s\n", message);
  return false;
}

bool TestControlsAndCloseRequest() {
  FakeAbi fake;
  active = &fake;
  auto runtime_result = freemvvm::detail::RuntimeTestAccess::create(
      MakeApi(), freemvvm::HostRegistration{&Wake, nullptr, nullptr});
  if (!Check(runtime_result.has_value(), "create fake Runtime")) return false;
  auto runtime = std::move(*runtime_result.value());

  fake.create_status = UI_UNSUPPORTED;
  fake.partial_create = true;
  auto failed_create = freemvvm::Window::create(runtime);
  if (!Check(!failed_create.has_value() && fake.destroys == 1,
             "release partial output after failed factory")) {
    return false;
  }
  fake.create_status = UI_OK;
  fake.partial_create = false;

  auto window_result = freemvvm::Window::create(runtime);
  auto panel_result = freemvvm::StackPanel::create(runtime);
  const std::string initial("hi\0there", 8);
  auto text_result = freemvvm::Text::create(runtime, initial);
  auto textbox_result = freemvvm::TextBox::create(runtime, "input");
  auto button_result = freemvvm::Button::create(runtime, "submit");
  if (!Check(window_result.has_value() && panel_result.has_value() &&
                 text_result.has_value() && textbox_result.has_value() &&
                 button_result.has_value(),
             "create initial UI controls")) {
    return false;
  }
  auto window = std::move(*window_result.value());
  auto panel = std::move(*panel_result.value());
  auto text = std::move(*text_result.value());
  auto textbox = std::move(*textbox_result.value());
  auto button = std::move(*button_result.value());
  fake.property_status = UI_WRONG_THREAD;
  auto failed_property = text.is_enabled_property();
  fake.property_status = UI_OK;
  if (!Check(!failed_property.has_value(),
             "preserve status from failed owned Property getter")) {
    return false;
  }
  if (!Check(fake.initial_texts.size() == 3 &&
                 fake.initial_texts[0] == initial &&
                 fake.initial_texts[1] == "input" &&
                 fake.initial_texts[2] == "submit",
             "pass UTF-8 bytes with length")) {
    return false;
  }
  if (!Check(window.set_content(panel).has_value() &&
                 panel.insert(text, 0).has_value() &&
                 panel.insert(textbox, 1).has_value() &&
                 panel.insert(button, 2).has_value(),
             "set window content and insert children")) {
    return false;
  }
  auto count = panel.child_count();
  auto child = panel.child_at(1);
  if (!Check(count.has_value() && *count.value() == 3 && child.has_value(),
             "read tree count and generic child")) {
    return false;
  }
  if (!Check(panel.set_orientation(freemvvm::Orientation::kHorizontal)
                     .has_value() &&
                 panel.orientation().has_value() &&
                 panel.set_child_alignment(text, freemvvm::Alignment::kCenter)
                     .has_value() &&
                 fake.orientation == 2 && fake.alignment == 2,
             "set and read layout properties")) {
    return false;
  }
  if (!Check(text.request_focus().has_value() && fake.focus_requests == 1,
             "request widget focus")) {
    return false;
  }
  auto content = window.content();
  if (!Check(content.has_value() && content.value()->has_value(),
             "read independently owned Window content")) {
    return false;
  }
  if (!Check(window.close_peer().has_value() && fake.peer_closes == 1,
             "close the peer without releasing the Window handle")) {
    return false;
  }

  std::optional<freemvvm::DeferredWindowCloseRequest> deferred;
  auto subscription_result = window.subscribe_close_request(
      [&deferred](freemvvm::WindowCloseRequest& request) {
        auto deferred_result = request.defer();
        if (deferred_result.has_value()) {
          deferred.emplace(std::move(*deferred_result.value()));
          (void)request.resolve(freemvvm::CloseDecision::kDecline);
        }
      });
  if (!Check(subscription_result.has_value(), "subscribe close request")) {
    return false;
  }
  auto subscription = std::move(*subscription_result.value());
  fake.close_callback(fake.close_userdata, 700);
  if (!Check(fake.last_decision == 2 && deferred.has_value(),
             "resolve borrowed and defer during callback")) {
    return false;
  }
  if (!Check(deferred->resolve(freemvvm::CloseDecision::kAccept).has_value() &&
                 fake.last_decision == 1,
             "resolve independently owned deferred request")) {
    return false;
  }
  if (!Check(deferred->close().has_value() && subscription.cancel().has_value() &&
                 subscription.close().has_value(),
             "release deferred request and callback subscription")) {
    return false;
  }
  if (fake.close_destroy != nullptr) return false;
  fake.release_status = UI_UNSUPPORTED;
  auto failed_release = child.value()->close();
  fake.release_status = UI_OK;
  if (!Check(!failed_release.has_value() && child.value()->close().has_value(),
             "retain owner after release failure and allow retry")) {
    return false;
  }
  if (!(*content.value())->close().has_value()) {
    return false;
  }
  return window.close().has_value() && panel.close().has_value() &&
         text.close().has_value() && textbox.close().has_value() &&
         button.close().has_value() && runtime.close().has_value();
}

}  // namespace

int main() { return TestControlsAndCloseRequest() ? 0 : 1; }
