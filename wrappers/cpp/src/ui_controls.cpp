#include "freemvvm/ui_controls.hpp"

#include "runtime_internal.hpp"

#include <freemvvm/abi.h>

#include <exception>
#include <memory>
#include <new>
#include <string>
#include <utility>

namespace freemvvm {
namespace {

struct CreatedHandle final {
  ui_handle token = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state;
  detail::PendingNode* cleanup_node = nullptr;
};

struct OwnedOutput final {
  ui_handle token = 0;
  detail::PendingNode* cleanup_node = nullptr;
};

std::uint32_t normalize_status(ui_status status) noexcept {
  return status == UI_OK ? static_cast<std::uint32_t>(UI_INTERNAL_ERROR)
                         : status;
}

template <typename Owner>
Result<Owner> failure(std::uint32_t status, ui_handle error,
                      std::shared_ptr<detail::RuntimeCleanupState> state,
                      std::unique_ptr<detail::PendingNode> error_node) {
  return Result<Owner>::failure(detail::make_error(
      status, error, std::move(state), error_node.release()));
}

void release_unadopted(ui_handle& token,
                       const std::shared_ptr<detail::RuntimeCleanupState>& state,
                       std::unique_ptr<detail::PendingNode>& cleanup_node) {
  if (token == 0) return;
  auto state_copy = state;
  auto* node = cleanup_node.release();
  detail::release_in_destructor(token, state_copy, node);
}

template <typename Factory>
Result<CreatedHandle> create_control(
    ui_handle runtime_token,
    const std::shared_ptr<detail::RuntimeCleanupState>& state,
    Factory factory) {
  if (runtime_token == 0 || state == nullptr) {
    return Result<CreatedHandle>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<CreatedHandle>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle token = 0;
  ui_handle error = 0;
  const ui_status status = factory(runtime_token, &token, &error);
  if (status != UI_OK || token == 0 || error != 0) {
    release_unadopted(token, state, cleanup_node);
    if (status != UI_OK) {
      return failure<CreatedHandle>(status, error, state,
                                    std::move(error_node));
    }
    return failure<CreatedHandle>(UI_INTERNAL_ERROR, error, state,
                                  std::move(error_node));
  }
  cleanup_node->token = token;
  return Result<CreatedHandle>::success(
      CreatedHandle{token, state, cleanup_node.release()});
}

template <typename Factory>
Result<CreatedHandle> create_text_control(
    ui_handle runtime_token,
    const std::shared_ptr<detail::RuntimeCleanupState>& state,
    const std::string& initial_value, Factory factory) {
  if (runtime_token == 0 || state == nullptr) {
    return Result<CreatedHandle>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<CreatedHandle>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  const auto* bytes = initial_value.empty()
                          ? nullptr
                          : reinterpret_cast<const std::uint8_t*>(
                                initial_value.data());
  ui_handle token = 0;
  ui_handle error = 0;
  const ui_status status = factory(
      runtime_token, bytes, static_cast<std::uint64_t>(initial_value.size()),
      &token, &error);
  if (status != UI_OK || token == 0 || error != 0) {
    release_unadopted(token, state, cleanup_node);
    return failure<CreatedHandle>(normalize_status(status),
                                  error, state, std::move(error_node));
  }
  cleanup_node->token = token;
  return Result<CreatedHandle>::success(
      CreatedHandle{token, state, cleanup_node.release()});
}

Result<OwnedOutput> get_owned_handle(
    ui_handle owner,
    const std::shared_ptr<detail::RuntimeCleanupState>& state,
    ui_status (*getter)(ui_handle, ui_handle*)) {
  if (owner == 0 || state == nullptr) {
    return Result<OwnedOutput>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node) {
    return Result<OwnedOutput>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle token = 0;
  const ui_status status = getter(owner, &token);
  if (status != UI_OK || token == 0) {
    if (token != 0) {
      cleanup_node->token = token;
      (void)state->enqueue(cleanup_node.release());
    }
    return Result<OwnedOutput>::failure(
        UiError(normalize_status(status)));
  }
  cleanup_node->token = token;
  return Result<OwnedOutput>::success(
      OwnedOutput{token, cleanup_node.release()});
}

bool valid_decision(CloseDecision decision) noexcept {
  return decision == CloseDecision::kAccept ||
         decision == CloseDecision::kDecline;
}

}  // namespace

namespace detail {

struct CloseRequestScope final {
  ui_handle borrowed_token = 0;
  std::shared_ptr<RuntimeCleanupState> state;
};

struct WindowCloseCallbackRecord final {
  Window::CloseRequestCallback callback;
  std::shared_ptr<RuntimeCleanupState> state;

  void invoke(ui_handle borrowed_request) noexcept {
    try {
      auto scope = std::make_shared<CloseRequestScope>();
      scope->borrowed_token = borrowed_request;
      scope->state = state;
      WindowCloseRequest request(scope);
      callback(request);
    } catch (...) {
      state->dispatch_callback_failure(CallbackFailure{
          CallbackKind::kWindowCloseRequest, std::current_exception()});
    }
  }
};

}  // namespace detail

Widget::Widget(std::uint64_t token,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

Widget::Widget(Widget&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

Widget& Widget::operator=(Widget&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

Widget::~Widget() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<Property<bool>> Widget::is_enabled_property() const {
  if (token_ == 0 || state_ == nullptr) {
    return Result<Property<bool>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!node) return Result<Property<bool>>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle property = 0;
  const ui_status status =
      state_->api->widget_is_enabled_property(token_, &property);
  if (status != UI_OK || property == 0) {
    if (property != 0) {
      node->token = property;
      (void)state_->enqueue(node.release());
    }
    return Result<Property<bool>>::failure(
        UiError(normalize_status(status)));
  }
  node->token = property;
  return Result<Property<bool>>::success(
      Property<bool>(property, state_, node.release()));
}

Result<Property<std::string>> Widget::accessible_name_property() const {
  if (token_ == 0 || state_ == nullptr) {
    return Result<Property<std::string>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!node) {
    return Result<Property<std::string>>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle property = 0;
  const ui_status status =
      state_->api->widget_accessible_name_property(token_, &property);
  if (status != UI_OK || property == 0) {
    if (property != 0) {
      node->token = property;
      (void)state_->enqueue(node.release());
    }
    return Result<Property<std::string>>::failure(
        UiError(normalize_status(status)));
  }
  node->token = property;
  return Result<Property<std::string>>::success(
      Property<std::string>(property, state_, node.release()));
}

Result<void> Widget::request_focus() {
  if (token_ == 0 || state_ == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state_->api->widget_request_focus(token_, &error);
  if (status != UI_OK) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  if (error != 0) {
    return Result<void>::failure(detail::make_error(
        UI_INTERNAL_ERROR, error, state_, error_node.release()));
  }
  return Result<void>::success();
}

Result<void> Widget::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

Container::Container(std::uint64_t token,
                     std::shared_ptr<detail::RuntimeCleanupState> state,
                     detail::PendingNode* cleanup_node) noexcept
    : Widget(token, std::move(state), cleanup_node) {}

Result<void> Container::insert(Widget& child, std::uint64_t index) {
  if (token() == 0 || child.token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->container_insert(
      token(), child.token(), index, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        node.release()));
  }
  return Result<void>::success();
}

Result<void> Container::remove(Widget& child) {
  if (token() == 0 || child.token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status =
      state()->api->container_remove(token(), child.token(), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        node.release()));
  }
  return Result<void>::success();
}

Result<void> Container::move(std::uint64_t from_index,
                             std::uint64_t to_index) {
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->container_move(
      token(), from_index, to_index, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        node.release()));
  }
  return Result<void>::success();
}

Result<std::uint64_t> Container::child_count() const {
  if (token() == 0 || state() == nullptr) {
    return Result<std::uint64_t>::failure(UiError(UI_INVALID_HANDLE));
  }
  std::uint64_t count = 0;
  const ui_status status = state()->api->container_child_count(token(), &count);
  if (status != UI_OK) {
    return Result<std::uint64_t>::failure(UiError(status));
  }
  return Result<std::uint64_t>::success(count);
}

Result<Widget> Container::child_at(std::uint64_t index) const {
  if (token() == 0 || state() == nullptr) {
    return Result<Widget>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<Widget>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle child = 0;
  ui_handle error = 0;
  const ui_status status = state()->api->container_child_at(
      token(), index, &child, &error);
  if (status != UI_OK || child == 0 || error != 0) {
    release_unadopted(child, state(), cleanup_node);
    return failure<Widget>(normalize_status(status),
                           error, state(), std::move(error_node));
  }
  cleanup_node->token = child;
  return Result<Widget>::success(
      Widget(child, state(), cleanup_node.release()));
}

Window::Window(std::uint64_t token,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* cleanup_node) noexcept
    : Container(token, std::move(state), cleanup_node) {}

Result<Window> Window::create(Runtime& runtime) {
  auto created = create_control(
      runtime.token_, runtime.state_,
      [&runtime](ui_handle token, ui_handle* out, ui_handle* error) {
        return runtime.state_->api->window_create(token, out, error);
      });
  if (!created.has_value()) {
    return Result<Window>::failure(std::move(*created.error()));
  }
  auto handle = std::move(*created.value());
  return Result<Window>::success(
      Window(handle.token, std::move(handle.state), handle.cleanup_node));
}

Result<std::optional<Widget>> Window::content() const {
  if (token() == 0 || state() == nullptr) {
    return Result<std::optional<Widget>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<std::optional<Widget>>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle child = 0;
  ui_handle error = 0;
  const ui_status status =
      state()->api->window_content(token(), &child, &error);
  if (status != UI_OK || error != 0) {
    release_unadopted(child, state(), cleanup_node);
    return failure<std::optional<Widget>>(
        normalize_status(status), error, state(),
        std::move(error_node));
  }
  if (child == 0) {
    return Result<std::optional<Widget>>::success(std::nullopt);
  }
  cleanup_node->token = child;
  std::optional<Widget> result;
  result.emplace(Widget(child, state(), cleanup_node.release()));
  return Result<std::optional<Widget>>::success(std::move(result));
}

Result<void> Window::set_content(Widget& child) {
  if (token() == 0 || child.token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status =
      state()->api->window_set_content(token(), child.token(), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<void> Window::clear_content() {
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status =
      state()->api->window_clear_content(token(), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<Property<std::string>> Window::title_property() const {
  if (token() == 0 || state() == nullptr) {
    return Result<Property<std::string>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto output = get_owned_handle(
      token(), state(), state()->api->window_title_property);
  if (!output.has_value()) {
    return Result<Property<std::string>>::failure(
        std::move(*output.error()));
  }
  auto value = std::move(*output.value());
  return Result<Property<std::string>>::success(Property<std::string>(
      value.token, state(), value.cleanup_node));
}

Result<void> Window::open() {
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->window_open(token(), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<void> Window::close_peer() {
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->window_close(token(), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

namespace {

void window_close_request_destroy(void* userdata) noexcept {
  delete static_cast<detail::WindowCloseCallbackRecord*>(userdata);
}

void window_close_request_trampoline(void* userdata,
                                     ui_handle borrowed_request) noexcept {
  auto* record = static_cast<detail::WindowCloseCallbackRecord*>(userdata);
  record->invoke(borrowed_request);
}

}  // namespace

Result<Subscription> Window::subscribe_close_request(
    CloseRequestCallback callback) {
  if (token() == 0 || state() == nullptr) {
    return Result<Subscription>::failure(UiError(UI_INVALID_HANDLE));
  }
  if (!callback) {
    return Result<Subscription>::failure(UiError(UI_INVALID_ARGUMENT));
  }
  auto callback_record = std::unique_ptr<detail::WindowCloseCallbackRecord>(
      new (std::nothrow) detail::WindowCloseCallbackRecord{
          std::move(callback), state()});
  auto subscription_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!callback_record || !subscription_node || !error_node) {
    return Result<Subscription>::failure(UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle subscription = 0;
  ui_handle error = 0;
  const ui_status status = state()->api->window_close_request_subscribe(
      token(), &window_close_request_trampoline, callback_record.get(),
      &window_close_request_destroy, &subscription, &error);
  if (status != UI_OK || subscription == 0 || error != 0) {
    if (subscription != 0) {
      subscription_node->token = subscription;
      (void)state()->enqueue(subscription_node.release());
    }
    if (status == UI_OK) {
      (void)callback_record.release();
    }
    return failure<Subscription>(normalize_status(status),
                                 error, state(), std::move(error_node));
  }
  (void)callback_record.release();
  subscription_node->token = subscription;
  return Result<Subscription>::success(Subscription(
      subscription, state(), subscription_node.release()));
}

StackPanel::StackPanel(std::uint64_t token,
                       std::shared_ptr<detail::RuntimeCleanupState> state,
                       detail::PendingNode* cleanup_node) noexcept
    : Container(token, std::move(state), cleanup_node) {}

Result<StackPanel> StackPanel::create(Runtime& runtime) {
  auto created = create_control(
      runtime.token_, runtime.state_,
      [&runtime](ui_handle token, ui_handle* out, ui_handle* error) {
        return runtime.state_->api->stack_panel_create(token, out, error);
      });
  if (!created.has_value()) {
    return Result<StackPanel>::failure(std::move(*created.error()));
  }
  auto handle = std::move(*created.value());
  return Result<StackPanel>::success(
      StackPanel(handle.token, std::move(handle.state), handle.cleanup_node));
}

Result<Orientation> StackPanel::orientation() const {
  if (token() == 0 || state() == nullptr) {
    return Result<Orientation>::failure(UiError(UI_INVALID_HANDLE));
  }
  std::uint32_t raw = 0;
  const ui_status status =
      state()->api->stack_panel_orientation(token(), &raw);
  if (status != UI_OK) {
    return Result<Orientation>::failure(UiError(status));
  }
  switch (raw) {
    case static_cast<std::uint32_t>(Orientation::kVertical):
      return Result<Orientation>::success(Orientation::kVertical);
    case static_cast<std::uint32_t>(Orientation::kHorizontal):
      return Result<Orientation>::success(Orientation::kHorizontal);
    default:
      return Result<Orientation>::failure(UiError(UI_INTERNAL_ERROR));
  }
}

Result<void> StackPanel::set_orientation(Orientation value) {
  const auto raw = static_cast<std::uint32_t>(value);
  if (raw != static_cast<std::uint32_t>(Orientation::kVertical) &&
      raw != static_cast<std::uint32_t>(Orientation::kHorizontal)) {
    return Result<void>::failure(UiError(UI_INVALID_ARGUMENT));
  }
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->stack_panel_set_orientation(
      token(), raw, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<void> StackPanel::set_child_alignment(Widget& child,
                                             Alignment alignment) {
  const auto raw = static_cast<std::uint32_t>(alignment);
  if (raw < static_cast<std::uint32_t>(Alignment::kStart) ||
      raw > static_cast<std::uint32_t>(Alignment::kStretch)) {
    return Result<void>::failure(UiError(UI_INVALID_ARGUMENT));
  }
  if (token() == 0 || child.token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->stack_panel_set_child_align(
      token(), child.token(), raw, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Text::Text(std::uint64_t token,
           std::shared_ptr<detail::RuntimeCleanupState> state,
           detail::PendingNode* cleanup_node) noexcept
    : Widget(token, std::move(state), cleanup_node) {}

Result<Text> Text::create(Runtime& runtime, const std::string& initial_text) {
  auto created = create_text_control(
      runtime.token_, runtime.state_, initial_text,
      [&runtime](ui_handle token, const std::uint8_t* bytes,
                 std::uint64_t length, ui_handle* out, ui_handle* error) {
        return runtime.state_->api->text_create(token, bytes, length, out,
                                                error);
      });
  if (!created.has_value()) {
    return Result<Text>::failure(std::move(*created.error()));
  }
  auto handle = std::move(*created.value());
  return Result<Text>::success(
      Text(handle.token, std::move(handle.state), handle.cleanup_node));
}

Result<Property<std::string>> Text::content_property() const {
  if (token() == 0 || state() == nullptr) {
    return Result<Property<std::string>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto output = get_owned_handle(
      token(), state(), state()->api->text_content_property);
  if (!output.has_value()) {
    return Result<Property<std::string>>::failure(
        std::move(*output.error()));
  }
  auto value = std::move(*output.value());
  return Result<Property<std::string>>::success(Property<std::string>(
      value.token, state(), value.cleanup_node));
}

TextBox::TextBox(std::uint64_t token,
                 std::shared_ptr<detail::RuntimeCleanupState> state,
                 detail::PendingNode* cleanup_node) noexcept
    : Widget(token, std::move(state), cleanup_node) {}

Result<TextBox> TextBox::create(Runtime& runtime,
                                const std::string& initial_value) {
  auto created = create_text_control(
      runtime.token_, runtime.state_, initial_value,
      [&runtime](ui_handle token, const std::uint8_t* bytes,
                 std::uint64_t length, ui_handle* out, ui_handle* error) {
        return runtime.state_->api->text_box_create(token, bytes, length, out,
                                                    error);
      });
  if (!created.has_value()) {
    return Result<TextBox>::failure(std::move(*created.error()));
  }
  auto handle = std::move(*created.value());
  return Result<TextBox>::success(
      TextBox(handle.token, std::move(handle.state), handle.cleanup_node));
}

Result<Property<std::string>> TextBox::value_property() const {
  if (token() == 0 || state() == nullptr) {
    return Result<Property<std::string>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto output = get_owned_handle(
      token(), state(), state()->api->text_box_value_property);
  if (!output.has_value()) {
    return Result<Property<std::string>>::failure(
        std::move(*output.error()));
  }
  auto value = std::move(*output.value());
  return Result<Property<std::string>>::success(Property<std::string>(
      value.token, state(), value.cleanup_node));
}

Button::Button(std::uint64_t token,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* cleanup_node) noexcept
    : Widget(token, std::move(state), cleanup_node) {}

Result<Button> Button::create(Runtime& runtime,
                              const std::string& initial_label) {
  auto created = create_text_control(
      runtime.token_, runtime.state_, initial_label,
      [&runtime](ui_handle token, const std::uint8_t* bytes,
                 std::uint64_t length, ui_handle* out, ui_handle* error) {
        return runtime.state_->api->button_create(token, bytes, length, out,
                                                  error);
      });
  if (!created.has_value()) {
    return Result<Button>::failure(std::move(*created.error()));
  }
  auto handle = std::move(*created.value());
  return Result<Button>::success(
      Button(handle.token, std::move(handle.state), handle.cleanup_node));
}

Result<Property<std::string>> Button::label_property() const {
  if (token() == 0 || state() == nullptr) {
    return Result<Property<std::string>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto output = get_owned_handle(
      token(), state(), state()->api->button_label_property);
  if (!output.has_value()) {
    return Result<Property<std::string>>::failure(
        std::move(*output.error()));
  }
  auto value = std::move(*output.value());
  return Result<Property<std::string>>::success(Property<std::string>(
      value.token, state(), value.cleanup_node));
}

Result<void> Button::set_command(Command& command) {
  if (token() == 0 || command.token_ == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status =
      state()->api->button_set_command(token(), command.token_, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<void> Button::clear_command() {
  if (token() == 0 || state() == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state()->api->button_set_command(token(), 0, &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state(),
        error_node.release()));
  }
  return Result<void>::success();
}

Result<std::optional<Command>> Button::command() const {
  if (token() == 0 || state() == nullptr) {
    return Result<std::optional<Command>>::failure(UiError(UI_INVALID_HANDLE));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<std::optional<Command>>::failure(
        UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle command_token = 0;
  ui_handle error = 0;
  const ui_status status = state()->api->button_command(
      token(), &command_token, &error);
  if (status != UI_OK || error != 0) {
    release_unadopted(command_token, state(), cleanup_node);
    return failure<std::optional<Command>>(
        normalize_status(status), error, state(),
        std::move(error_node));
  }
  if (command_token == 0) {
    return Result<std::optional<Command>>::success(std::nullopt);
  }
  cleanup_node->token = command_token;
  std::optional<Command> result;
  result.emplace(Command(command_token, state(), cleanup_node.release()));
  return Result<std::optional<Command>>::success(std::move(result));
}

DeferredWindowCloseRequest::DeferredWindowCloseRequest(
    std::uint64_t token,
    std::shared_ptr<detail::RuntimeCleanupState> state,
    detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

DeferredWindowCloseRequest::DeferredWindowCloseRequest(
    DeferredWindowCloseRequest&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

DeferredWindowCloseRequest& DeferredWindowCloseRequest::operator=(
    DeferredWindowCloseRequest&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

DeferredWindowCloseRequest::~DeferredWindowCloseRequest() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<void> DeferredWindowCloseRequest::resolve(CloseDecision decision) {
  if (token_ == 0 || state_ == nullptr) {
    return Result<void>::failure(UiError(UI_INVALID_HANDLE));
  }
  if (!valid_decision(decision)) {
    return Result<void>::failure(UiError(UI_INVALID_ARGUMENT));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = state_->api->window_close_request_resolve(
      token_, static_cast<std::uint32_t>(decision), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, state_,
        error_node.release()));
  }
  return Result<void>::success();
}

Result<void> DeferredWindowCloseRequest::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

WindowCloseRequest::WindowCloseRequest(
    const std::shared_ptr<detail::CloseRequestScope>& scope) noexcept
    : scope_(scope) {}

Result<void> WindowCloseRequest::resolve(CloseDecision decision) {
  auto scope = scope_.lock();
  if (!scope) return Result<void>::failure(UiError(UI_CLOSED));
  if (!valid_decision(decision)) {
    return Result<void>::failure(UiError(UI_INVALID_ARGUMENT));
  }
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(UI_OUT_OF_MEMORY));
  ui_handle error = 0;
  const ui_status status = scope->state->api->window_close_request_resolve(
      scope->borrowed_token, static_cast<std::uint32_t>(decision), &error);
  if (status != UI_OK || error != 0) {
    return Result<void>::failure(detail::make_error(
        normalize_status(status), error, scope->state,
        error_node.release()));
  }
  return Result<void>::success();
}

Result<DeferredWindowCloseRequest> WindowCloseRequest::defer() {
  auto scope = scope_.lock();
  if (!scope) {
    return Result<DeferredWindowCloseRequest>::failure(UiError(UI_CLOSED));
  }
  auto cleanup_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  auto error_node = std::unique_ptr<detail::PendingNode>(
      new (std::nothrow) detail::PendingNode());
  if (!cleanup_node || !error_node) {
    return Result<DeferredWindowCloseRequest>::failure(
        UiError(UI_OUT_OF_MEMORY));
  }
  ui_handle deferred = 0;
  ui_handle error = 0;
  const ui_status status = scope->state->api->window_close_request_defer(
      scope->borrowed_token, &deferred, &error);
  if (status != UI_OK || deferred == 0 || error != 0) {
    release_unadopted(deferred, scope->state, cleanup_node);
    return failure<DeferredWindowCloseRequest>(
        normalize_status(status), error, scope->state,
        std::move(error_node));
  }
  cleanup_node->token = deferred;
  return Result<DeferredWindowCloseRequest>::success(
      DeferredWindowCloseRequest(deferred, scope->state,
                                 cleanup_node.release()));
}
}  // namespace freemvvm
