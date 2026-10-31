#include "freemvvm/window.hpp"

#include "runtime_internal.hpp"

#include <new>
#include <utility>

namespace freemvvm {

Window::Window(std::uint64_t token,
               std::shared_ptr<detail::RuntimeCleanupState> state,
               detail::PendingNode* cleanup_node) noexcept
    : token_(token), state_(std::move(state)), cleanup_node_(cleanup_node) {}

Window::Window(Window&& other) noexcept
    : token_(std::exchange(other.token_, 0)),
      state_(std::move(other.state_)),
      cleanup_node_(std::exchange(other.cleanup_node_, nullptr)) {}

Window& Window::operator=(Window&& other) noexcept {
  if (this != &other) {
    detail::release_in_destructor(token_, state_, cleanup_node_);
    token_ = std::exchange(other.token_, 0);
    state_ = std::move(other.state_);
    cleanup_node_ = std::exchange(other.cleanup_node_, nullptr);
  }
  return *this;
}

Window::~Window() noexcept {
  detail::release_in_destructor(token_, state_, cleanup_node_);
}

Result<Window> Window::create(Runtime& runtime) {
  if (runtime.token_ == 0 || !runtime.state_) {
    return Result<Window>::failure(UiError(2));
  }

  std::unique_ptr<detail::PendingNode> window_node(
      new (std::nothrow) detail::PendingNode());
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!window_node || !error_node) {
    return Result<Window>::failure(UiError(8));
  }

  ui_handle window = 0;
  ui_handle error = 0;
  const ui_status status = runtime.state_->api->window_create(
      runtime.token_, &window, &error);
  if (status != 0 || window == 0) {
    if (window != 0) {
      window_node->token = window;
      if (runtime.state_->api->handle_release(window) != 0) {
        (void)runtime.state_->enqueue(window_node.release());
      }
    }
    return Result<Window>::failure(detail::make_error(
        status == 0 ? 9 : status, error, runtime.state_, error_node.release()));
  }

  error_node.reset();
  window_node->token = window;
  return Result<Window>::success(
      Window(window, runtime.state_, window_node.release()));
}

Result<void> Window::close() noexcept {
  return detail::release_owned(token_, state_, cleanup_node_);
}

Result<RgbColor> Window::background_rgb() const {
  if (token_ == 0 || !state_) return Result<RgbColor>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<RgbColor>::failure(UiError(8));

  RgbColor color{};
  ui_handle error = 0;
  const ui_status status = state_->api->window_background_rgb(
      token_, &color.red, &color.green, &color.blue, &error);
  if (status != 0) {
    return Result<RgbColor>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<RgbColor>::success(color);
}

Result<void> Window::set_background_rgb(RgbColor color) {
  if (token_ == 0 || !state_) return Result<void>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(8));

  ui_handle error = 0;
  const ui_status status = state_->api->window_set_background_rgb(
      token_, color.red, color.green, color.blue, &error);
  if (status != 0) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

Result<void> Window::reset_background_rgb() {
  if (token_ == 0 || !state_) return Result<void>::failure(UiError(2));
  std::unique_ptr<detail::PendingNode> error_node(
      new (std::nothrow) detail::PendingNode());
  if (!error_node) return Result<void>::failure(UiError(8));

  ui_handle error = 0;
  const ui_status status =
      state_->api->window_reset_background_rgb(token_, &error);
  if (status != 0) {
    return Result<void>::failure(detail::make_error(
        status, error, state_, error_node.release()));
  }
  error_node.reset();
  return Result<void>::success();
}

}  // namespace freemvvm
