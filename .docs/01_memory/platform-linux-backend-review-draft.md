# Category: freemvvm Linux Platform backend review-draft status and scope

- Platform internal-layers DD 1.3 and UI test design 0.5 were approved as whole documents on 2026-10-26; they supersede 1.2/0.4 as the current design baseline without changing their historical approval records.
- The approved unit split is `UNIT-CORE-075` (Core frame submission/completion seam), `UNIT-PLATFORM-001` (DRM device/output and GBM/EGL surface lifecycle), and `UNIT-PLATFORM-002` (Clip/FillRect renderer/present/resource loss). They remain planned until implementation is separately authorized and completed.
- Keep the Core render completion boundary exact on surface generation, frame identity, and scene revision; backend completion is posted to the Core dispatcher and does not call Core/user callbacks from a backend thread.
- The DRM/GBM candidate versions must be compatible: `gbm` 0.18.0's optional `drm-support` integration uses `drm` 0.14.0. Native packages, sysroot, and runtime loading must still be verified by Linux build; do not infer them from crate names.
- GlyphRun snapshot currently lacks an agreed render-ready font asset/color contract. Do not make GlyphRun an approved renderer test oracle until that contract is reviewed. A first renderer proposal may test Clip/FillRect and must not report unsupported DrawOps as successfully presented.
- Review proposal: `.docs/20_todo/platform-linux-backend-next-step-review.md`; parent plan: `.docs/20_todo/platform-first-backend.md`.
