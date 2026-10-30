# Scope: Linux Platform Docker provisional validation

- Keep Debian container validation separate from the physical DRM/KMS hardware smoke. A container result can only be called `provisional_pass`, never a Debian hardware, RPi, or release pass.
- The Docker runner should build a pinned Rust/Debian image, run focused Core/Linux adapter tests, Linux check/Clippy/minimum-toolchain check, and explicitly run the ignored surfaceless EGL/GLES pbuffer pixel test.
- The pbuffer oracle uses a few in-memory RGBA pixels for Core scale, background clear, Clip, and FillRect. Do not generate frame sequences or RawMovie output.
- Do not claim container verification when the Docker engine is unavailable. `tools/dev.py linux-platform-docker` writes `.local/verification/linux-platform-docker.json`; `blocked` means no image/test result, not a provisional pass.
