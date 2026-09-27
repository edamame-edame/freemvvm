# MVVM UIライブラリ UI treeの所有・寿命に関する上位判断案

- 状態: BD-04承認済み（2026-09-26）
- 対応: 要求仕様第12節「UI tree・Layout・Input・IME・Control」のうち、UI tree要素の親子所属と所有寿命
- 根拠: Acceptedの設計骨格、承認済みRuntime契約R1～R4、ABI判断A29～A32。DD-00は未承認

## BD-04 承認された判断

**Core管理のUI treeで親子所属が成立している間、親は子をCore内部の強参照で保持する。子を親から外すとその所属参照を解放するが、呼び出し側が持つ公開handleや別の明示所有者があれば子objectは存続する。** 親子所属そのものを言語wrapperのGCや参照カウントに依存させない。

| 項目 | 提案する上位契約 |
|---|---|
| 所属時の所有 | 親子リンクはCore内部の所有参照を作る。親が生存して所属中の子を保持する。公開handleの`retain/release`とは別の参照である。 |
| 付け外し | 子は同じtreeの親を最大一つ持つ。親子リンクの追加は既存の親がない場合に限り、削除時はそのリンクの参照を解放する。親子の付け替えを一操作で行うか、失敗時の原子性は後続設計で定める。 |
| 親・祖先の破棄 | subtreeをUI treeとPlatformから切り離し、所属参照を解放する。公開handle等が残る子Core objectはDetached状態で存続できる。親が破棄されたのに子が古いnative peerやOS resourceを参照する状態は許さない。 |
| Core objectとPlatform資源 | Core object/Propertyの寿命と、native control・renderer node等のPlatform資源の寿命を分ける。Platform資源は所属・接続に応じて破棄し、Detached objectを再所属させる場合は必要な資源を再生成または再接続する。具体的なpeerモデルは描画方式の判断に依存する。 |
| Detached中のhandle | Detachedでも有効な公開handleはCore objectを指し、release可能とする。所属していないobjectが描画・入力対象にならないことを上位契約とする。Propertyの読み書き・再所属等をどこまで許すかは詳細設計でAPI別に定める。 |
| 寿命 | 子を親から外しても、公開handle等の所有参照が残っていればCore objectは存続する。子の別所有参照は親破棄で無効化しない。参照が最後に解放されたUI objectの最終破棄はR2に従いUI threadで行う。 |
| 木の不変条件 | 各tree内の親子関係は循環させない。複数の親に同時所属する操作は拒否する。Logical TreeとVisual Treeをどう分け、各リンクが所有関係になるかはこの判断に含めない。 |
| Control内部の参照 | TextBoxのPropertyや入力closureの寿命はA29～A32を維持する。特にTextBox破棄時に入力closureを解除し、別途保持されたPropertyがTextBoxを保持しない。 |

この案では、親子関係によるCore内部参照、アプリがAPIから得た公開handle、OS/backendのUI資源を別の寿命として扱う。例えばWindowがStackPanelを子として保持し、StackPanelがTextBoxを保持する。Windowの破棄でTextBoxのnative peerは破棄される。アプリがTextBoxのhandleを保持していれば、Core上のTextBoxはDetachedとして参照でき、再所属時には必要なpeerを作り直す。したがって、handleは破棄済みnative peerを指さない。

## 判断範囲と後続項目

- 本判断はtree所属と寿命の上位ルールであり、Logical TreeとVisual Treeの役割、Control templateが作る要素、ツリー変更API、layout・描画・hit testへの反映を確定しない。
- Detached状態のAPIごとの可否、Property値の保持範囲、再所属時の状態復元、native resource再生成の失敗処理は詳細設計に送る。
- 親子リンクの追加・削除・付け替えの具体API、失敗時の原子性、Window close時の通知順、detach時のfocus/IME cleanupは各後続判断に送る。
- A29～A32のTextBox生成・値接続・外部確定値の扱い・破棄規則を変更しない。TextBox固有のProperty寿命を一般のtree ownershipへ広げない。
- Treeの所有参照はRuntime契約の「循環する内部参照を作らない」に従う。Logical/Visual関係を混在させたときの全体参照グラフと、template由来要素の所有者は詳細なtree設計で明示する。

## レビューで確認する点

1. 親子所属をCore内部の強参照で表し、言語wrapperのGCへ委ねないこと。
2. Core objectとnative peerの寿命を分け、親破棄時に古いpeerを残さないこと。
3. 公開handleを持つDetached objectを有効なCore objectとして扱い、再所属時に必要なPlatform資源を再生成・再接続すること。
4. 複数の親と循環を認めないこと。
5. A29～A32のTextBox固有契約を維持し、Logical/Visual Treeやdetach cleanupなどは別判断に残すこと。

## 承認記録

2026-09-26に利用者承認。親子所属はCore内部強参照で保持し、treeとPlatform peerを切り離した際に公開handleが残る子はDetached Core objectとして存続できる。複数親・循環は禁止。Logical/Visual Tree、detached API、Layout・Inputへの連携は別判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
