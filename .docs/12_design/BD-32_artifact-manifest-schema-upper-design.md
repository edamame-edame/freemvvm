# MVVM UIライブラリ Artifact manifest schema evolutionに関する上位判断案

- 状態: BD-32承認済み（2026-09-27）
- 対応: 署名付きmanifestのschema version、署名対象、未知fieldの互換規則
- 根拠: BD-29、BD-31、要求仕様第9・12節。DD-00は未承認

## 既存の承認済み契約

- BD-29はimmutable artifact identity、署名付きmanifest、digest、target/profile、ABI、feature metadataを要求し、検証不能なartifactを拒否する。
- BD-31はpublisher trust rootとkey lifecycleを定め、未知・失効・検証不能keyを拒否する。
- native runtimeのMAJOR.MINOR.PATCH（BD-28）とmanifest schema versionは異なる目的を持つ。runtime patch releaseでもmanifest reader/writerが更新される場合がある。
- manifestのschema version、署名対象field範囲、unknown fieldの扱いは未決である。

## BD-32 承認済み判断

1. **Manifest schemaはnative runtime versionから独立したschema versionを持つ。** schema evolutionはmanifest parser/producer間の互換性を表し、runtime artifactのMAJOR.MINOR.PATCHを置き換えない。
2. **署名はmanifestの全意味内容を対象にする。** artifact digest、target/profile、ABI、feature、release identity、schema versionを署名で束縛し、署名field自体を除く全manifest contentに対する署名とする。具体的なcanonical encodingやserializationは詳細設計で定義する。
3. **同じschema major内のminor更新は後方互換とする。** readerは同一majorの新minorを受け入れてよいが、未知のcritical/required fieldや理解できない必須featureがあれば拒否する。未知optional fieldは署名検証後に無視してよいが、target選択・ABI適合・信頼判定へ影響するfieldをoptional扱いにしてはならない。
4. **未知schema major、重複・曖昧なfield表現、署名範囲を一意に解釈できないmanifestはfail closedとする。** parserの寛容なbest-effort読み替えでartifactを受け入れない。
5. **schema major変更は旧readerに解釈不能な意味変更が必要な場合に限る。** 新しいoptional metadataや後方互換な記述追加は同じschema major内に加えられる。過去schemaのreader coverageと移行期間は詳細設計で決める。

## 理由とトレードオフ

- runtime versionとschema versionを分けると、ABI互換性とmanifest format互換性を独立に進化させられる。
- manifestの安全性に関わるfieldが署名から漏れないことを契約にし、artifact本体とmetadataのすり替えを防ぐ。
- critical/requiredとoptionalの明示により、新しいreader向けmanifestを古いreaderが危険な形で部分解釈しない。
- strict parsingは未知形式を停止させるため、新schema導入時にreader更新が必要になる場合がある。manifestのminor互換範囲を守り、この頻度を抑える。

## この判断では決めないこと

- schema versionの具体番号・文字列形式、JSON/CBOR等のencoding、canonical serialization規則。
- 具体field名、型、required/optional/critical flagのwire表現。
- signature algorithm、署名fieldのbyte encoding、digest algorithm。
- manifest producer/readerが同時に複数schema majorを出す期間。
- parser implementation、resource size/depth limits、test vectors。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- manifest schema versionはruntime versionと独立させる。
- 署名はschema versionと全security-relevant manifest contentを束縛する。
- same-major minor追加は互換とし、未知critical/required field・featureは拒否、未知optional fieldは署名検証後に限り無視できる。
- unknown major、曖昧なfield、署名対象の解釈不一致はfail closedとする。
- exact encoding・schema structure・signature bytesは詳細設計で定める。

schema versionのwire表現、encoding、canonical serialization、field名とresource limitsは後続詳細設計で決める。DD-00は未承認のため、DD-01以降へ進まない。
