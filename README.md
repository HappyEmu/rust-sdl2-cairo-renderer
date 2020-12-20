# High Performance Software Renderer with SDL2, GTK Cairo and Rust

The goal of this little project is to implement a software renderer capable of drawing 3D
meshes in a very constrained environment (STM32, ESP32).

Eventually, SDL2 and Cairo will be removed, as we will directly access the frame buffer on these constrained devices.

Steps:
- [x] Draw to SDL2 texture
- [x] Implement 3D transformation Pipeline, perspective projection
- [ ]  Triangle rasterization using edge functions
- [ ]  Shader programs
- [ ]  Port to `no-std`
- [ ]  Remove SDL2, Cairo, draw to frame buffer directly

### Current state
![Renderer State](https://drive.google.com/uc?id=1IYrtD8lfHR6xknQrJGbthWuuXUNjnPZk)
