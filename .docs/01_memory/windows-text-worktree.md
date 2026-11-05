# Windows Platform: text-service implementation handoff

Scope: Windows DirectWrite TextService worktree and branch handoff.

- The plan worktree can be detached at an earlier planning commit. Verify the pushed implementation branch before editing source; use its latest pushed SHA as the new feature branch base.
- DirectWrite renderer helpers for TTF load and system-face registration do not imply that a TextService caller exists. Check the Core text snapshot/profile producer and UI Scene path separately.
- Keep OS font selection/profile smoke distinct from fake-boundary CI. A Windows CI pass does not establish visible Window text quality or the OS-selected Japanese font.
