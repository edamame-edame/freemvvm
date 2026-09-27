# MVVM UIライブラリ ABI release version schemeに関する上位判断案

- 状態: BD-28承認済み（2026-09-27）
- 対応: Native C ABI runtimeの公開versionとA3 symbol majorの対応
- 根拠: A3、BD-22、要求仕様第9・12節。DD-00は未承認

## 既存の承認済み契約

- A3はversion付きC ABI symbolを使い、後方互換な追加をminor、破壊的変更を新majorとして扱う。旧headerで新runtimeへlinkする互換性検証も要求する。
- BD-22は最新安定ABI majorを通常保守し、直前majorを限定保守する。support状態はABI majorとtarget profileの組で公表する。
- 各language wrapperは同じnative C ABI runtimeを利用する。Python wheel、NuGet、CMake package、Cargo wrapperとnative runtime artifactの配布境界はBD-17～BD-20で上位決定済み。
- packageごとのversion表現、registry、fetch/cache/provenanceは後続事項として計画に残っている。

## BD-28 承認済み判断

1. **安定版native C ABI runtimeの公開versionは`MAJOR.MINOR.PATCH`の三要素で表す。** 各native artifactとrelease metadataはこのversionを同じ意味で使う。
2. **`MAJOR`はA3のC ABI symbol suffixのmajorと一致させる。** 異なるmajorのruntimeは異なるABI世代として扱う。破壊的変更を伴わないABI追加はMINOR、ABI互換性を変えないbug/security fixはPATCHを上げる。
3. **minor間の互換性は同一major内で後方互換とする。** 新しいminorで追加されたsymbol/featureの存在を古いminor向けwrapperが前提にしてはならない。wrapperが要求するminimum runtime minorはpackage metadataで宣言する。このruntime discovery/fallbackの実装方法は後続詳細設計に送る。
4. **wrapper packageはnative runtimeとは独立したpackage versionを持ってよいが、対応するABI majorとminimum runtime versionを明示する。** C# NuGet、Python wheel、CMake package、Cargo crateの各版番号と公開手順は、それぞれのdistribution設計で定める。
5. **この判断の適用対象は安定版native runtimeの互換version規則に限定する。** 初回stable release番号、pre-release identifier、build metadata、release cadenceはこの判断では固定せず、公開release運用を決める時点で別途定める。

## 理由とトレードオフ

- C ABI symbol、compatibility test、support matrix、target-specific artifactのmajorを一致させると、利用者がbinaryの互換世代を判定しやすい。
- MINOR/PATCHの意味をA3と揃え、wrapperとnative runtimeのpackage versionを分離することで、wrapperだけの修正がABI majorを不必要に増やさない。
- package metadataへminimum runtime requirementを持たせることで、新しいsymbolを必要とするwrapperが古いruntimeを誤って使うのを避けられる。具体的なloader拒否とdiagnosticは詳細設計で決める。
- 初回release番号やpre-release運用を切り離し、互換性規則だけを今の上位設計で扱える。

## この判断では決めないこと

- 初回stable versionの番号、beta/rc等のpre-release naming、build metadata、release cadence。
- 各wrapperのpackage version schemeとABI runtime versionへのdependency constraint表現。
- runtime loaderのversion discovery、minor mismatch時のcapability detection、runtime search order。
- target別artifactの署名、provenance、registry、公開workflow。
- 0.x開発版の互換性約束。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- stable native runtimeはMAJOR.MINOR.PATCHとし、MAJORをA3のsymbol majorへ一致させる。
- minorは後方互換なABI追加、patchはABI互換fixに限定する。
- wrapper packageは独立versionを許すが、対応ABI majorとminimum runtimeを宣言する。
- 初回stable番号・pre-release policy・具体package constraintsは別途残す。

初回stable番号・pre-release policy・具体package constraintsは後続とする。DD-00は未承認のため、DD-01以降へ進まない。
