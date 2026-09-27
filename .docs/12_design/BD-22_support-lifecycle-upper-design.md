# MVVM UIライブラリ サポート期間に関する上位判断案

- 状態: BD-22承認済み（2026-09-27）
- 対応: ABI major、OS／architecture／hardware profile別の保守期間、非推奨とEOL
- 根拠: 要求仕様第9・12節、ABI A3、BD-16、BD-21

## 既存の承認済み契約

- A3はABIにmajorを含め、互換追加をminor、破壊変更を新majorとして扱う。BD-22はこの互換性規則を変更しない。
- BD-16はWindows x64、Linux x64、組み込みLinux ARMv7/AArch64の順で検証を進め、cross-compile結果と実機検証を区別する。ビルド可能であることだけでは、対象機器でruntime verifiedとはみなさない。
- BD-21はApache-2.0を承認済み。サポート終了は公式保守の終了を意味し、公開済みsource／binaryのlicense上の再配布権を取り消さない。

## BD-22 承認された判断

1. **サポート対象はABI majorとtarget profileの組で明示する。** 各releaseのsupport matrixにOS、architecture、hardware profileまたはtoolchain profile、ABI major、検証状態を記す。単にcross-compileできるtargetを「実機サポート済み」と表示しない。
2. **最新安定ABI majorを通常保守する。** 通常保守には互換性を保つbug fixとsecurity fixを含める。機能追加・互換変更はA3のminor、破壊変更は新majorの規則に従う。
3. **直前のABI majorは、新majorの安定版公開後24か月、security fixと重大障害修正に限って保守する。** 新機能、非重大bug fix、target追加は旧majorへ原則backportしない。24か月は移行期間の推奨値であり、業界標準だという主張ではない。
4. **target profileの追加・非推奨・EOLはABI majorとは別に管理する。** profileを非推奨にする場合は、公式release noteで最低12か月前に通知する。重大なsecurity上の事情がある場合は短縮理由と影響を同じ告知に記録する。
5. **EOL後は公式の修正・問い合わせ対応を約束しない。** EOL対象の最後のsource、binary、support matrixとEOL告知を参照可能に保つ。既存利用者が自己保守やforkを行うこと、Apache-2.0条件に従い再配布することは妨げない。

このmajor/minor/patchの役割は、A3およびSemantic Versioning 2.0.0の互換性規則と整合する。Semantic Versioningはmajorを互換性のない公開API変更、minorを後方互換な機能追加、patchを後方互換なbug fixに割り当てる。

## 理由

- 明示されたABI major期間は、C ABIを共有する4 language wrapperと別環境から取得するnative artifactの互換確認に使える。
- 旧majorへsecurity/重大修正だけをbackportすることで、利用者に移行時間を与えながら、複数majorへの機能開発の拡散を抑える。
- target profile単位のmatrixは、Windows/Linux x64の実機検証と、組み込みLinuxのboard・sysroot別検証を混同しない。profileごとのEOLをABI EOLと切り離せる。
- 公開済みsourceとartifactを残し、公式保守のみを終了することで、OSSライセンスの継続利用とプロジェクト側の保守範囲を明確に分けられる。

## この判断では決めないこと

- 重大度分類、security reportの受付先、初動・修正公開のresponse SLA。
- 24か月経過後にsecurity修正を例外提供する条件。
- 5年等のLTS channelや、特定のABI majorを長期保守版に指定すること。
- 初期release時の正確なOS version、compiler、sysroot、board、hardware profile一覧。
- runtime-verified各profileの試験範囲と合格基準。
- EOL告知の配布channel、release metadata形式、maintainer体制。

## 承認記録

利用者は2026-09-27にBD-22を承認した。上記のABI major別保守期間、target profile別support matrix、原則12か月の非推奨通知、公式保守終了としてのEOLを上位判断として扱う。security response SLA、LTS、exact target matrixは後続判断に残す。

## 参考資料

- [Semantic Versioning 2.0.0](https://semver.org/) — major/minor/patchの互換性規則。

BD-22は上位判断として承認済み。security SLA、LTS、exact target matrixは未決のまま後続判断に残す。DD-00は未承認のため、DD-01以降へ進まない。
