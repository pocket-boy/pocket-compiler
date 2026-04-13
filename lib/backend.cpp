#include "backend.hpp"

extern "C" {
void *init_backend(void *, ExternEnviron);

void drop_backend(void *);

void backend_render(void *, uint8_t);
}

void BackendHandleDeleter::operator()(void *handle) const {
  ::drop_backend(handle);
}

BackendHandle::BackendHandle(CompilerHandle handle, ExternEnviron environ) {
  this->handle = std::unique_ptr<void, BackendHandleDeleter>(
      init_backend(handle.handle.get(), environ));
}

BackendHandle &BackendHandle::operator=(BackendHandle &&other) {
  if (this != &other) {
    this->handle = std::move(other.handle);
  }
  return *this;
}

void BackendHandle::render(uint8_t input) {
  ::backend_render(this->handle.get(), input);
}
