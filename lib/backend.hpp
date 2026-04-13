#include "compiler.hpp"
#include <cstdint>

/** ... */
struct ExternEnviron final {
  /** ... */
  void (*action_one)(ExternEnviron *environ, uint32_t value);
  /** ... */
  void (*action_two)(ExternEnviron *environ, uint32_t value);
  /** ... */
  void (*draw_tile)(ExternEnviron *environ, uint32_t tile, uint32_t x,
                    uint32_t y);
};

/** Functor for deleting `BackendHandle` handles. */
struct BackendHandleDeleter {
  void operator()(void *handle) const;
};

/** ... */
class BackendHandle final {
private:
  std::unique_ptr<void, BackendHandleDeleter> handle;

public:
  BackendHandle(CompilerHandle, ExternEnviron);

  BackendHandle(BackendHandle const &) = delete;

  BackendHandle(BackendHandle &&other) : handle(std::move(other.handle)) {}

  BackendHandle &operator=(BackendHandle const &) = delete;

  BackendHandle &operator=(BackendHandle &&other);

  void render(uint8_t);
};
