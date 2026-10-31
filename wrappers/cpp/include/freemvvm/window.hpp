#ifndef FREEMVVM_CPP_WINDOW_HPP_
#define FREEMVVM_CPP_WINDOW_HPP_

#include "freemvvm/runtime.hpp"

#include <type_traits>

namespace freemvvm {

struct RgbColor final {
  float red;
  float green;
  float blue;
};

class Window final {
 public:
  Window(const Window&) = delete;
  Window& operator=(const Window&) = delete;
  Window(Window&& other) noexcept;
  Window& operator=(Window&& other) noexcept;
  ~Window() noexcept;

  static Result<Window> create(Runtime& runtime);

  Result<void> close() noexcept;
  Result<RgbColor> background_rgb() const;
  Result<void> set_background_rgb(RgbColor color);
  Result<void> reset_background_rgb();

 private:
  friend class Runtime;
  Window(std::uint64_t token,
         std::shared_ptr<detail::RuntimeCleanupState> state,
         detail::PendingNode* cleanup_node) noexcept;

  std::uint64_t token_ = 0;
  std::shared_ptr<detail::RuntimeCleanupState> state_;
  detail::PendingNode* cleanup_node_ = nullptr;
};

static_assert(!std::is_copy_constructible<Window>::value,
              "Window owns one native reference");
static_assert(std::is_nothrow_move_constructible<Window>::value,
              "Window moves must not throw");

}  // namespace freemvvm

#endif  // FREEMVVM_CPP_WINDOW_HPP_
