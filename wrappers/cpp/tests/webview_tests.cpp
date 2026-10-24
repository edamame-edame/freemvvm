#include "freemvvm/webview.hpp"

#include "freemvvm/binding.hpp"
#include "freemvvm/property.hpp"
#include "freemvvm/runtime.hpp"

#include <cstdio>
#include <string>
#include <vector>

namespace {

bool Expect(bool condition, const char* message) {
  if (!condition) {
    std::fprintf(stderr, "webview_tests failure: %s\n", message);
  }
  return condition;
}

bool TestWebViewMarkdownHtmlBindingScriptAndRender() {
  freemvvm::HostRegistration host{
      [](void*, std::uint32_t) {},
      nullptr,
      [](void*) {},
  };
  auto runtime_result = freemvvm::Runtime::create(host);
  if (!Expect(runtime_result.has_value(), "Runtime::create failed")) {
    return false;
  }
  auto runtime = std::move(*runtime_result.value());

  auto webview_result = freemvvm::WebView::create(runtime);
  if (!Expect(webview_result.has_value(), "WebView::create failed")) {
    return false;
  }
  auto webview = std::move(*webview_result.value());

  auto vp_res = webview.set_viewport(240.0, 120.0);
  if (!Expect(vp_res.has_value(), "WebView::set_viewport failed")) {
    return false;
  }

  auto source_res = webview.set_source(freemvvm::WebViewFormat::kMarkdown,
                                       "# C++ Title\n\nBody text.");
  if (!Expect(source_res.has_value(), "WebView::set_source failed")) {
    return false;
  }

  auto css_res = webview.set_stylesheet("h1 { color: #0055aa; }");
  if (!Expect(css_res.has_value(), "WebView::set_stylesheet failed")) {
    return false;
  }

  auto text_res = webview.text_content();
  if (!Expect(text_res.has_value(), "WebView::text_content failed")) {
    return false;
  }
  const std::string& initial_text = *text_res.value();
  if (!Expect(initial_text.find("C++ Title") != std::string::npos,
              "initial text missing C++ Title")) {
    return false;
  }

  // Test OneWay source property binding.
  auto source_prop_res = freemvvm::Property<std::string>::create(
      runtime,
      "<h1 id=\"t\">Initial</h1><button id=\"b\" onclick=\"save('ok')\">Go</button>");
  if (!Expect(source_prop_res.has_value(),
              "Property<std::string>::create failed")) {
    return false;
  }
  auto source_prop = std::move(*source_prop_res.value());

  auto bind_res = webview.bind_source_property(
      freemvvm::WebViewFormat::kHtml, source_prop);
  if (!Expect(bind_res.has_value(),
              "WebView::bind_source_property failed")) {
    return false;
  }
  auto binding = std::move(*bind_res.value());

  // Expose a read/write property and a command to the sandboxed script VM.
  auto status_prop_res =
      freemvvm::Property<std::string>::create(runtime, "ready");
  if (!Expect(status_prop_res.has_value(), "status Property::create failed")) {
    return false;
  }
  auto status_prop = std::move(*status_prop_res.value());

  auto expose_prop_res = webview.expose_property("status", status_prop, true);
  if (!Expect(expose_prop_res.has_value(),
              "WebView::expose_property failed")) {
    return false;
  }

  bool command_ran = false;
  auto expose_cmd_res = webview.expose_command(
      "save", [&command_ran]() { command_ran = true; });
  if (!Expect(expose_cmd_res.has_value(),
              "WebView::expose_command failed")) {
    return false;
  }

  auto eval_res = webview.eval_script(
      "let s = freemvvm.getProperty('status'); "
      "document.getElementById('t').textContent = 'Status=' + s; "
      "freemvvm.setProperty('status', 'done'); "
      "freemvvm.invokeCommand('save');");
  if (!Expect(eval_res.has_value(), "WebView::eval_script failed")) {
    return false;
  }

  if (!Expect(command_ran, "expose_command callback did not run")) {
    return false;
  }

  auto status_val_res = status_prop.get();
  if (!Expect(status_val_res.has_value() &&
                  *status_val_res.value() == "done",
              "freemvvm.set('status', 'done') did not update Property")) {
    return false;
  }

  auto updated_text_res = webview.text_content();
  if (!Expect(updated_text_res.has_value() &&
                  updated_text_res.value()->find("Status=ready") !=
                      std::string::npos,
              "DOM setText did not update text_content")) {
    return false;
  }

  // Test sandbox rejection of forbidden globals.
  auto forbidden_res = webview.eval_script("fetch('https://example.invalid');");
  if (!Expect(!forbidden_res.has_value(),
              "sandboxed eval_script must reject fetch")) {
    return false;
  }

  // Test render_rgba output.
  auto rgba_res = webview.render_rgba(240, 120);
  if (!Expect(rgba_res.has_value(), "WebView::render_rgba failed")) {
    return false;
  }
  const auto& rgba = *rgba_res.value();
  if (!Expect(rgba.size() == 240u * 120u * 4u,
              "unexpected RGBA buffer byte size")) {
    return false;
  }

  (void)binding.cancel();
  (void)binding.close();
  (void)webview.close();
  return true;
}

}  // namespace

int main() {
  if (!TestWebViewMarkdownHtmlBindingScriptAndRender()) {
    return 1;
  }
  return 0;
}
