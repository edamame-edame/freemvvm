#ifndef FREEMVVM_CPP_WEBVIEW_HPP_
#define FREEMVVM_CPP_WEBVIEW_HPP_

#include "freemvvm/binding.hpp"
#include "freemvvm/property.hpp"
#include "freemvvm/runtime.hpp"

#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <type_traits>
#include <vector>

namespace freemvvm {

enum class WebViewFormat : std::uint32_t {
  kHtml = 1,
  kMarkdown = 2,
};

class WebView final {
 public:
  WebView(const WebView&) = delete;
  WebView& operator=(const WebView&) = delete;
  WebView(WebView&& other) noexcept;
  WebView& operator=(WebView&& other) noexcept;
  ~WebView() noexcept;

  static Result<WebView> create(Runtime& runtime);

  Result<void> set_source(WebViewFormat format, const std::string& source_utf8);
  Result<void> set_stylesheet(const std::string& css_utf8);
  Result<void> set_viewport(double width_dip, double height_dip);
  Result<Binding<std::string>> bind_source_property(
      WebViewFormat format, Property<std::string>& utf8_property);

  template <typename T>
  Result<void> expose_property(const std::string& alias, Property<T>& property,
                               bool allow_write);

  Result<void> expose_command(const std::string& alias,
                              std::function<void()> callback);
  Result<void> eval_script(const std::string& script_utf8);
  Result<void> click_at(double x_dip, double y_dip);
  Result<std::string> text_content() const;
  Result<std::vector<std::uint8_t>> render_rgba(std::uint32_t width_px,
                                                std::uint32_t height_px) const;
  Result<void> close() noexcept;

 private:
  WebView(std::uint64_t token,
          std::shared_ptr<detail::RuntimeCleanupState> state,
          detail::PendingNode* cleanup_node) noexcept;

  Result<void> expose_property_handle(const std::string& alias,
                                      std::uint64_t property_token,
                                      bool allow_write);

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

template <typename T>
Result<void> WebView::expose_property(const std::string& alias,
                                      Property<T>& property, bool allow_write) {
  static_assert(std::is_same<T, bool>::value ||
                    std::is_same<T, std::int64_t>::value ||
                    std::is_same<T, double>::value ||
                    std::is_same<T, std::string>::value,
                "WebView script bridge supports bool, int64_t, double, and UTF-8 string properties");
  return expose_property_handle(alias, property.token_, allow_write);
}

static_assert(!std::is_copy_constructible<WebView>::value,
              "WebView owns one native reference");
static_assert(std::is_nothrow_move_constructible<WebView>::value,
              "WebView moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_WEBVIEW_HPP_
