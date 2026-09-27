# MVVM UIライブラリ Native artifact provenanceに関する上位判断案

- 状態: BD-29承認済み（2026-09-27）
- 対応: target別native artifactを取得・取り込むときの識別情報と検証境界
- 根拠: BD-16～BD-22、BD-28、Rust Cargo artifact import feature、要求仕様第9・12節。DD-00は未承認

## 既存の承認済み契約

- BD-16はbuild verifiedとruntime verifiedを区別し、Windows/Linux x64/組み込みLinux ARM targetの検証順を定める。
- BD-17～BD-20はPython、C#、C++、Rustからnative C ABI runtimeを利用する配布境界を定めた。Rustには別環境から指定target artifactを取得するfeatureがある。
- BD-20ではartifact metadataにtarget triple/profile、C ABI major、提供feature、content digestを持たせ、不一致を拒否する。
- BD-21はApache-2.0とthird-party license維持を決定したが、公式artifactの署名・provenanceは後続に残した。
- BD-28は安定版native runtime versionをMAJOR.MINOR.PATCHとし、MAJORをC ABI symbol majorに一致させる。

## BD-29 承認済み判断

1. **公式native artifactは、artifactごとの不変なmanifestと署名で出自を確認できるようにする。** manifestは配布後に同じrelease versionの内容を書き換えない。更新物は新しいartifact identityとして公開する。
2. **manifestには少なくとも以下を記録する。** native runtime version、ABI major、OS/architecture/target triple、target profileまたは互換条件、提供feature、artifact filename、byte size、cryptographic digest、release identity、署名key identityを含める。target固有の詳細属性はprofile schemaで追加できる。
3. **取得／import側は、利用前に署名とdigest、target/profile、要求ABI major、必要featureを照合する。** どれかに不一致または検証不能があればartifactを不適合として拒否する。取得しただけで自動実行せず、link/loadの可否は明示された要求とmetadataに基づいて判断する。
4. **versioned identityとlatest等の可変aliasを分ける。** cache keyはimmutable artifact identityとdigestを含める。可変aliasを使う場合も、解決後に固定されたidentityとdigestを記録し、異なるbinaryで既存cache entryを黙って置換しない。
5. **この判断ではalgorithm、trust root、registry、transport、鍵管理を決めない。** 署名形式・digest algorithm・trusted publisher登録・key rotation/revocation・offline policy・manifest schemaのversioningは詳細設計で決める。ただし、安定版を公開・取得するまでに具体化する。

## 理由とトレードオフ

- digestは壊損・取り違え検出を、署名は信頼したrelease identityとの対応確認を担うため、別々に検証する。
- target tripleだけでは組み込みLinuxのlibc、sysroot、CPU機能差を表せないことがある。profile/互換条件をmanifestへ持たせ、詳細schemaはtarget matrix確定後に定義できる。
- immutable identityとcache keyを結び付けると、別環境で取得したartifactの再現性と監査性が上がる。一方、publisher keyの管理、失効、offline環境での信頼情報配布が必要になる。
- 検証に失敗したartifactを拒否するため、検証なしで利用を続けるfallbackは設けない。key/transport policy自体は別途設計する。

## この判断では決めないこと

- 署名・digestの具体algorithm、署名形式、trusted keyのprovisioningとrotation/revocation。
- registry/download URL、HTTP protocol、proxy、offline bundle形式、authorization。
- target profileの完全な属性一覧、manifestのschema version、binary naming convention。
- CI provenance attestationとbuild environment/SBOMの完全なschema。
- 各wrapper package managerとのcache共有、runtime loaderのsearch pathとfallback。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- 公式native artifactはimmutable manifest、署名、digestで識別・検証する。
- 利用前にtarget/profile、ABI major、必要feature、署名、digestを検証し、不適合または検証不能なら拒否する。
- cacheをimmutable artifact identity＋digestへ結び付け、可変alias解決後もidentityを記録する。
- 署名方式・鍵管理・registry等は詳細設計へ送るが、安定版の公開・取得前に定める。

署名方式・鍵管理・registry、target profileの完全属性、manifest schema version、各package managerとのcache共有は後続詳細設計で決める。DD-00は未承認のため、DD-01以降へ進まない。
