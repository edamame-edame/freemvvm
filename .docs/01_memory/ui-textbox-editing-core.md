# freemvvm: Core TextBox editing state / UNIT-CORE-092

## Scope

Core UI TextBox editing state, UTF-8 byte offsets, candidate transaction, Property Validator result, and committed Value synchronization.

## Learned

- Keep the committed TextBox Value separate from the immutable editing snapshot. Store UTF-8 byte offsets for anchor/caret, and initialize both at the initial text's byte length.
- Compare equal external writes with the last committed Value, not with editing text. A Property Validator rejection can leave a draft visible while the committed Value remains unchanged; an equal external write must preserve that draft.
- Prepare the full replacement and next snapshot before changing state. For an accepted candidate, commit the Value Property and editing snapshot in the same UI-thread operation. For a Property Validator rejection, retain the editing candidate and reason without changing the committed Property. Validator failure leaves both unchanged.
- TextBox layout reads the editing snapshot. Direct and Binding Value updates converge through `mark_layout_dirty_for_display_property`, which synchronizes an unequal committed Value into the TextBox state.
- Route TextCommit only after bubbling remains unhandled, the original target is still valid, and it is still the active Focus in that Window. Apply the default action only when that target is a TextBox; no-focus, non-TextBox, disabled, detached, or handled events do not edit.
- `UNIT-CORE-092` owns editing state and its candidate transaction. Input target selection, handled behavior, and the TextCommit default-action connection stay in `UNIT-CORE-091`, which depends on `UNIT-CORE-092`.

## References

- [Implementation plan](../20_todo/textbox-input-editing-core-implementation.md)
- [TextBox detailed design](../13_detail_design/ui/text-editing.md)
- [UI test design](../32_unit_test/ui.md#editingvalidationime)
- [Implementation unit registry](../../tools/units.yaml)
- [Git branch and commit rules](../02_git/rule.md)
