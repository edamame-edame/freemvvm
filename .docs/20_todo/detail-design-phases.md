# 詳細設計フェーズ計画（`.docs/13_detail_design/`）

- 状態: 運用中。詳細設計の個別判断は各フェーズで確認する
- 作成日: 2026-09-26
- 対象: Rust Core、versioned C ABI、C++ / C# / Rust / Python 3 wrapper、初期縦断シナリオ
- この計画の目的: 小さな設計範囲ごとにレビューし、承認された判断だけを詳細設計として積み上げる

## 進め方

1. 各フェーズの着手時に、ADR、要件、該当する `.docs/12_design/` の本文と判断表を再確認する。文書の見出しや状態欄だけで全文の承認を推定しない。
2. 一度に一つの小さな設計範囲を扱う。`.docs/13_detail_design/` にレビュー案を作り、前提、責務、状態遷移、所有権、失敗時、スレッド、依存関係、検証観点のうち該当するものを記す。
3. 新しい設計判断には詳細設計固有のID（例: `DD-HND-01`）を付ける。上位文書の承認済みIDを出典として併記し、未承認の判断と混ぜない。承認済み判断を変更する必要があれば、詳細設計だけで上書きせず上位文書のレビューへ戻す。
4. 各レビュー案について利用者の確認を受け、判断ごとの承認・修正・保留を**同じ文書**へ反映する。承認前の案を確定仕様や実装指示として扱わない。
5. フェーズ完了時に要件と判断IDへの対応、隣接文書との整合、未決事項、単体で確認できる振る舞いを点検する。実装計画とテスト計画は、承認済みの詳細設計から別途作る。

## 前提と現在の承認境界

- `.docs/04_approval/adr-0001-rust-core.md` は `Accepted`。Rust Core、versioned C ABI、4言語の薄いwrapper、組み込みLinux対応を前提とする。
- `.docs/11_requirements/mvvm-ui-library-requirements.md` と `.docs/12_design/design-overview.md` は `Accepted`。設計範囲と初期縦断シナリオの参照元として扱う。要件内の未決事項は別途判断する。
- `.docs/12_design/runtime-contract.md` のR1～R4は承認済み。後続仕様として残された事項は別途判断する。
- `.docs/12_design/abi-spec.md` はA1～A32、`.docs/12_design/mvvm-semantics.md` はP1～P4・B1～B4・C1～C4・D1～D4・V1～V8、`.docs/12_design/language-bindings.md` はL1～L4を承認済みと記す。各文書の未定義事項は承認済みとはみなさない。
- `abi-spec.md` のC宣言は設計レビュー用であり、配布用ヘッダや実装済みAPIではない。

## フェーズ一覧

チェックは「レビュー案作成、利用者確認、同一文書への結果反映、整合確認」が終わった時点で付ける。番号は推奨順であり、前提が未承認のフェーズは進めない。

### 基盤と初期縦断シナリオ

- [ ] **DD-00 出典と判断の対応表（レビュー案作成済み・未承認）** — `.docs/13_detail_design/00-decision-map.md`。上位文書の状態、承認済みID、未決事項、初期縦断シナリオへの対応を列挙する。R1～R4を承認済みとして記録し、後続フェーズの入口条件を定める。
- [ ] **DD-01 Handle登録と検証** — `01-handle-registry.md`。トークン発行、種類・Runtime照合、retain/release、無効・二重releaseの扱い、単体確認観点。前提: R1の承認。
- [ ] **DD-02 所有権と終了** — `02-runtime-lifetime.md`。公開参照と内部参照、Subscription・Bindingの参照方向、shutdown、worker release後の破棄順。前提: DD-01、R2の承認。
- [ ] **DD-03 UI threadと投稿キュー** — `03-dispatcher.md`。UI thread判定、post受付・実行順・停止時の処理、イベントループへの接点、競合時の観測。前提: DD-02、R2の承認。
- [ ] **DD-04 Callbackと再入** — `04-callback-lifecycle.md`。closureの引受け・解除・destroy、自己解除、通知の非再帰、wrapper例外の封じ込め。前提: DD-02～03、R3・R4の承認。
- [ ] **DD-05 値の受渡しと失敗契約** — `05-boundary-values-errors.md`。UTF-8、配列・buffer、error handle、出力初期化、失敗時の原子性。前提: DD-01～04と該当するABI判断。
- [ ] **DD-06 ABIシンボルと互換性** — `06-abi-surface.md`。承認済みA1～A32と詳細設計の対応、関数群の責務、version/feature query、互換性検証方法。レビュー用C宣言と配布物の区別を明記する。前提: DD-01～05。
- [ ] **DD-07 Propertyの保存と通知** — `07-property-notification.md`。値型、等価判定、setter、通知キュー、購読と解除、shutdown時の振る舞い。前提: DD-01～06、P1～P4と対応するA判断。
- [ ] **DD-08 Bindingの伝播とグラフ** — `08-binding-graph.md`。初期同期、OneTime/OneWay/TwoWay、更新順、循環制約、解除、Bindingエラー。前提: DD-07、B1～B4・D1～D4と対応するA判断。
- [ ] **DD-09 ValidationとView入力** — `09-validation-input.md`。Property Validator、View入力拒否、TextBoxの確定値と表示値、理由の保持・解放。ControlやIMEの未定義部分は境界として残す。前提: DD-07～08、V1～V8と対応するA判断。
- [ ] **DD-10 非同期Command** — `10-async-command.md`。実行可否、状態遷移、完了報告、協調的キャンセル、終了・失敗時の後始末。前提: DD-03～04・DD-07、C1～C4と対応するA判断。

### UIとplatform（対応する基本設計の判断後）

以下は基本設計の該当範囲がまだ揃っていないため、各フェーズの前に `.docs/12_design/` で小範囲の判断をレビューし承認状況を記録する。詳細設計側だけで未定義事項を確定しない。

- [ ] **DD-11 UI treeと要素寿命** — `11-ui-tree-lifetime.md`。親子・所有、取り外し、Binding/Subscriptionの後始末。前提: DD-02、UI tree基本設計。
- [ ] **DD-12 Layout** — `12-layout.md`。Measure/Arrange、サイズ制約、変更時の再計算、StackPanelの最小動作。前提: DD-11、Layout基本設計。
- [ ] **DD-13 InputとFocus** — `13-input-focus.md`。keyboard入力、focus移動、イベント配送、UI threadとの接点。前提: DD-11～12、Input/Focus基本設計。
- [ ] **DD-14 TextBoxとIME** — `14-textbox-ime.md`。編集候補、確定、composition、Validationとの接続、破棄時の整合。前提: DD-09・DD-13、IME基本設計。
- [ ] **DD-15 初期Control** — `15-controls-v1.md`。Window、StackPanel、TextBox、Text、Buttonの最小契約とCommand/Property接続。前提: DD-08～14、Control基本設計。
- [ ] **DD-16 描画とPlatform adapter** — `16-platform-rendering.md`。描画責務、イベントループ統合、対象OS/組み込みLinux backendの境界。前提: DD-03・DD-12～15、Platform/Rendering基本設計。
- [ ] **DD-17 Accessibility** — `17-accessibility.md`。初期Controlのrole/name/state/focus、keyboard操作、platform APIへの写像。前提: DD-13～16、Accessibility基本設計。

### 言語境界と検証

- [ ] **DD-18 C++ wrapper** — `18-wrapper-cpp.md`。RAII、callback保持、明示解除、エラー変換。前提: DD-01～10、L1～L4、必要なControl詳細設計。
- [ ] **DD-19 C# wrapper** — `19-wrapper-csharp.md`。P/Invoke、delegate保持、Dispose、RID別native asset。前提はDD-18と同じ。
- [ ] **DD-20 Rust safe API** — `20-wrapper-rust.md`。型付きhandle、UI thread制約、dispatcher、dropと明示終了。前提はDD-18と同じ。
- [ ] **DD-21 Python 3 wrapper** — `21-wrapper-python.md`。ctypes/cffiの選択、callable保持、明示解除、例外変換。前提はDD-18と同じ。
- [ ] **DD-22 Buildと配布** — `22-build-distribution.md`。target別native artifact、wrapperの同梱、組み込みLinuxのtoolchain/sysroot、互換性確認。前提: DD-06・DD-16・DD-18～21、Build/Distribution基本設計。
- [ ] **DD-23 縦断検証設計** — `23-verification.md`。4言語で同じWindow/StackPanel/TextBox/Text/Buttonシナリオを確認する条件、単位ごとの失敗系、ABI互換性、version testとrelease testへの対応。前提: DD-01～22、`.docs/30_release_test/`・`.docs/31_version_test/`。

## 各フェーズの完了条件

- 対象範囲と対象外、上位判断ID、追加判断ID、レビュー結果が追跡できる。
- 依存先との所有権・スレッド・エラー・終了時の契約に矛盾がない。
- 単体で確認できる正常系と主要な失敗系が記され、未決事項に次の担当フェーズが付いている。
- 判断が未承認ならチェックを付けず、次フェーズの確定仕様として引用しない。

## 最初の作業

DD-00の対応表を作り、RuntimeのR1～R4を承認済み判断として整理する。その後、DD-01の小さなレビュー案から着手する。
