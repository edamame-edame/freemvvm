#ifndef FREEMVVM_CPP_UI_CONTROLS_HPP_
#define FREEMVVM_CPP_UI_CONTROLS_HPP_

#include "freemvvm/command.hpp"
#include "freemvvm/subscription.hpp"

#include <cstdint>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <type_traits>

namespace freemvvm {

enum class Orientation : std::uint32_t {
  kVertical = 1,
  kHorizontal = 2,
};

enum class Alignment : std::uint32_t {
  kStart = 1,
  kCenter = 2,
  kEnd = 3,
  kStretch = 4,
};

enum class CloseDecision : std::uint32_t {
  kAccept = 1,
  kDecline = 2,
};

namespace detail {
struct CloseRequestScope;
struct WindowCloseCallbackRecord;
}

class WindowCloseRequest;

class DeferredWindowCloseRequest final {
 public:
  DeferredWindowCloseRequest(const DeferredWindowCloseRequest&) = delete;
  DeferredWindowCloseRequest& operator=(const DeferredWindowCloseRequest&) = delete;
  DeferredWindowCloseRequest(DeferredWindowCloseRequest&& other) noexcept;
  DeferredWindowCloseRequest& operator=(
      DeferredWindowCloseRequest&& other) noexcept;
  ~DeferredWindowCloseRequest() noexcept;

  Result<void> resolve(CloseDecision decision);
  Result<void> close() noexcept;

 private:
  friend class WindowCloseRequest;
  friend struct detail::RuntimeTestAccess;
  DeferredWindowCloseRequest(
      std::uint64_t token,
      std::shared_ptr<detail::RuntimeCleanupState> state,
      detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

class WindowCloseRequest final {
 public:
  WindowCloseRequest(const WindowCloseRequest&) = delete;
  WindowCloseRequest& operator=(const WindowCloseRequest&) = delete;

  Result<void> resolve(CloseDecision decision);
  Result<DeferredWindowCloseRequest> defer();

 private:
  friend class Window;
  friend struct detail::WindowCloseCallbackRecord;
  explicit WindowCloseRequest(
      const std::shared_ptr<detail::CloseRequestScope>& scope) noexcept;

  std::weak_ptr<detail::CloseRequestScope> scope_;
};

class Widget {
 public:
  Widget(const Widget&) = delete;
  Widget& operator=(const Widget&) = delete;
  Widget(Widget&& other) noexcept;
  Widget& operator=(Widget&& other) noexcept;
  virtual ~Widget() noexcept;

  Result<Property<bool>> is_enabled_property() const;
  Result<Property<std::string>> accessible_name_property() const;
  Result<void> request_focus();
  Result<void> close() noexcept;

 protected:
  Widget(std::uint64_t token,
         std::shared_ptr<detail::RuntimeCleanupState> state,
         detail::PendingNode* cleanup_node) noexcept;
  std::uint64_t token() const noexcept { return token_; }
  const std::shared_ptr<detail::RuntimeCleanupState>& state() const noexcept {
    return state_;
  }

 private:
  friend class Runtime;
  friend class Container;
  friend class Window;
  friend class StackPanel;
  friend class Button;
  friend struct detail::RuntimeTestAccess;
  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

class Container : public Widget {
 public:
  Container(const Container&) = delete;
  Container& operator=(const Container&) = delete;
  Container(Container&& other) noexcept = default;
  Container& operator=(Container&& other) noexcept = default;

  Result<void> insert(Widget& child, std::uint64_t index);
  Result<void> remove(Widget& child);
  Result<void> move(std::uint64_t from_index, std::uint64_t to_index);
  Result<std::uint64_t> child_count() const;
  Result<Widget> child_at(std::uint64_t index) const;

 protected:
  Container(std::uint64_t token,
            std::shared_ptr<detail::RuntimeCleanupState> state,
            detail::PendingNode* cleanup_node) noexcept;
};

class Window final : public Container {
 public:
  using CloseRequestCallback = std::function<void(WindowCloseRequest&)>;

  Window(const Window&) = delete;
  Window& operator=(const Window&) = delete;
  Window(Window&& other) noexcept = default;
  Window& operator=(Window&& other) noexcept = default;

  static Result<Window> create(Runtime& runtime);
  Result<std::optional<Widget>> content() const;
  Result<void> set_content(Widget& child);
  Result<void> clear_content();
  Result<Property<std::string>> title_property() const;
  Result<void> open();
  Result<void> close_peer();
  Result<Subscription> subscribe_close_request(CloseRequestCallback callback);

 private:
  friend struct detail::RuntimeTestAccess;
  Window(std::uint64_t token,
         std::shared_ptr<detail::RuntimeCleanupState> state,
         detail::PendingNode* cleanup_node) noexcept;
};

class StackPanel final : public Container {
 public:
  StackPanel(const StackPanel&) = delete;
  StackPanel& operator=(const StackPanel&) = delete;
  StackPanel(StackPanel&& other) noexcept = default;
  StackPanel& operator=(StackPanel&& other) noexcept = default;

  static Result<StackPanel> create(Runtime& runtime);
  Result<Orientation> orientation() const;
  Result<void> set_orientation(Orientation orientation);
  Result<void> set_child_alignment(Widget& child, Alignment alignment);

 private:
  friend struct detail::RuntimeTestAccess;
  StackPanel(std::uint64_t token,
             std::shared_ptr<detail::RuntimeCleanupState> state,
             detail::PendingNode* cleanup_node) noexcept;
};

class Text final : public Widget {
 public:
  Text(const Text&) = delete;
  Text& operator=(const Text&) = delete;
  Text(Text&& other) noexcept = default;
  Text& operator=(Text&& other) noexcept = default;

  static Result<Text> create(Runtime& runtime, const std::string& initial_text);
  Result<Property<std::string>> content_property() const;

 private:
  friend struct detail::RuntimeTestAccess;
  Text(std::uint64_t token, std::shared_ptr<detail::RuntimeCleanupState> state,
       detail::PendingNode* cleanup_node) noexcept;
};

class TextBox final : public Widget {
 public:
  TextBox(const TextBox&) = delete;
  TextBox& operator=(const TextBox&) = delete;
  TextBox(TextBox&& other) noexcept = default;
  TextBox& operator=(TextBox&& other) noexcept = default;

  static Result<TextBox> create(Runtime& runtime,
                                const std::string& initial_value);
  Result<Property<std::string>> value_property() const;

 private:
  friend struct detail::RuntimeTestAccess;
  TextBox(std::uint64_t token,
          std::shared_ptr<detail::RuntimeCleanupState> state,
          detail::PendingNode* cleanup_node) noexcept;
};

class Button final : public Widget {
 public:
  Button(const Button&) = delete;
  Button& operator=(const Button&) = delete;
  Button(Button&& other) noexcept = default;
  Button& operator=(Button&& other) noexcept = default;

  static Result<Button> create(Runtime& runtime,
                               const std::string& initial_label);
  Result<Property<std::string>> label_property() const;
  Result<void> set_command(Command& command);
  Result<void> clear_command();
  Result<std::optional<Command>> command() const;

 private:
  friend struct detail::RuntimeTestAccess;
  Button(std::uint64_t token,
         std::shared_ptr<detail::RuntimeCleanupState> state,
         detail::PendingNode* cleanup_node) noexcept;
};

static_assert(!std::is_copy_constructible<Widget>::value,
              "Widget owns one native reference");
static_assert(std::is_nothrow_move_constructible<Widget>::value,
              "Widget moves must not throw");
static_assert(!std::is_copy_constructible<DeferredWindowCloseRequest>::value,
              "Deferred request owns one native reference");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_UI_CONTROLS_HPP_
