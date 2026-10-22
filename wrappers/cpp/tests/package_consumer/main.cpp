#include <freemvvm/runtime.hpp>

#include <utility>

namespace {

void Wake(void*, std::uint32_t) {}

}  // namespace

int main() {
  auto result = freemvvm::Runtime::create({&Wake, nullptr, nullptr});
  if (!result.has_value()) return 1;

  freemvvm::Runtime runtime = std::move(*result.value());
  auto is_ui_thread = runtime.is_ui_thread();
  if (!is_ui_thread.has_value() || !*is_ui_thread.value()) return 2;

  return runtime.close().has_value() ? 0 : 3;
}
