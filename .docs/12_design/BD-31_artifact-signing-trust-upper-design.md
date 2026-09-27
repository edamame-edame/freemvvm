# MVVM UIライブラリ Artifact署名trust policyに関する上位判断案

- 状態: BD-31承認済み（2026-09-27）
- 対応: BD-29のmanifest署名を検証するtrust root、鍵更新・失効、offline importの方針
- 根拠: BD-21、BD-29、BD-30、要求仕様第9・12節。DD-00は未承認

## 既存の承認済み契約

- BD-21はfirst-party source／binary SDKへApache-2.0を適用し、改変sourceと改変binaryの再配布を許可する。公式artifactの署名・provenanceは後続へ残していた。
- BD-29はimmutable native artifact identity、署名付きmanifest、digestを要求し、利用前検証に失敗したartifactを拒否する。
- BD-30は公式artifact catalogをsource of truthとし、version/digest pinningとoffline済みcache/artifact bundleの利用を認める。
- 署名方式、publisher keyのtrusted root、key rotation/revocation、offline trust metadataの扱いは未決である。

## BD-31 承認済み判断

1. **artifact catalogから取得した公開keyだけを、その場で自動信頼してはならない。** 公式publisherのtrust rootは、検証済みwrapper/tool配布物または運用者が明示的にprovisionしたtrust bundleから得る。manifest署名keyは、この既存trust rootまで連なる場合だけ有効とする。
2. **manifestの署名key identityを固定して検証する。** key identityが未知、失効済み、trust chain不成立、署名不正の場合はfail closedとし、未署名・検証不能の公式artifactをwarning-onlyで利用可能にしない。
3. **通常rotationは旧trust rootで認証されたkeyset更新として配布し、移行期間中は旧keyと新keyの両方を明示する。** 旧keyを失効させた後は、新規取得・importでそのkeyによる署名を受け付けない。侵害時の緊急失効も同じ失効経路で通知できる構造にする。
4. **offline importは、artifact manifestに加え、運用者が信頼済みとして渡したpublic trust bundleと、そのbundleに含まれる失効情報を使えるようにする。** 取得先へ接続できないことを理由に、未知の鍵や古いartifactを自動信頼しない。trust metadataの有効期間とstale時の扱いは詳細設計で数値化する。
5. **この上位判断では暗号algorithmやkey storageを固定しない。** 署名/digest format、root-key offline storage、threshold signing、timestamp/transparency log、失効metadataのschemaと期限は詳細設計で決める。これらは安定版artifact公開とfetch/import実装前に具体化する。

## 理由とトレードオフ

- binaryと署名keyを同じ未検証catalogから取得して同時に信頼すると、攻撃者が両方を置き換えても検知できない。信頼済みの別経路を起点にする必要がある。
- rotation用の認証済みkeysetと失効処理があれば、通常のkey更新と侵害時の緊急停止を区別して実装できる。
- offline bundleを許すと隔離されたbuild/組み込み環境でも署名検証を維持できる。一方で、運用者は失効情報を更新したbundleを別経路で配布する必要がある。
- fail-closed policyはcatalogやtrust metadataの障害時に取得を止めるが、未検証binaryを誤って採用する経路を作らない。

## この判断では決めないこと

- Ed25519、ECDSA等のsignature algorithm、hash function、canonical serialization。
- Root keyの保管、threshold、HSM/secure element、CI identity、release approval workflow。
- rotation期間、metadata有効期限、revocation propagation SLA、既に導入済みartifactへの対応。
- transparency log、reproducible build attestation、SBOMへの署名方式。
- OS/package manager別のcertificate/key store integration、管理者policy API。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- publisher trust anchorはartifact catalogと独立に信頼できる経路からprovisionする。
- unknown/revoked/unverifiable keyはfail closedとし、warning-onlyで回避しない。
- routine rotationは認証済みkeysetで移行し、緊急失効も可能にする。
- offline importは明示的なtrusted bundleを使い、鍵やstale metadataを自動信頼しない。
- 暗号形式、key storage、期限と失効運用は詳細設計までに具体化する。

暗号形式、root key保管・threshold、metadata期限・失効反映時限、OS別keystore連携は後続詳細設計で定める。DD-00は未承認のため、DD-01以降へ進まない。
