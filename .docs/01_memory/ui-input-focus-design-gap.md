# UI設計: Input / Focus配送の承認境界

RQ-09-068のhandled/bubbling保証は初期共通基盤の要求。承認済み`ui/input-focus.md`はroute候補と内部InputRouter案を持つが、public subscription・handled state・ABIを未決としている。Property subscriptionは引数なしcallbackなのでInput event payloadには使えない。Input実装計画では先にInput/Focus・Controls・ABI・unit-test文書を同時にレビューし、仕様承認と実装開始を分ける。Pointer move/wheel/captureとkeybindingはRQ上の追加候補・除外範囲と照合し、実装都合で増やさない。
