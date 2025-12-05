# Control flow support (prototype)

現状のVoltTSパーサーは簡易的な1行ブロック構文で `if` / `while` / `for` を扱います。

- 条件式は `true` または `false` のみ対応しています。
- 本家の `if { .. } else { .. }` ブロックに似ていますが、1行で完結させる必要があります。
- `for` は `for i in 0..N { <stmt> }` 形式の整数レンジループのみをサポートします。
- ブロック内は `;` 区切りで複数ステートメントを並べられます。
- 生成されるCコードは素朴な `if/else`、`while`、`for` に展開されます。

```vts
fn main() {
  if true { log.info("if branch") } else { log.warn("else branch") }
  while false { log.error("will not run") }
  for i in 0..2 { log.info("looping") }
}
```

## 制約

- 条件式の評価はブールリテラルのみです。将来的に比較演算子や変数参照を追加する予定です。
- `match`/`range`/`object` などはまだ未対応です。
- ネストは可能ですが可読性のため短いブロックに留めてください。

## テスト

`cargo test -- --nocapture control_flow_constructs_emit_expected_output`
で実際のC生成と実行を含む統合テストを走らせられます。
