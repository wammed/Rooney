# Rooney 継続レビュー指示書

## 目的

このファイルは、ChatGPT などのAIに Rooney リポジトリをレビューしてもらう際に、会話履歴が途切れた場合でも、これまでのレビュー方針・評価基準・レビュー形式を継続できるようにするための指示書です。

**この指示書を読んだAIは、過去の会話が利用できない場合でも、以下のレビュー方針を基準として Rooney の最新版をレビューしてください。**

---

# 1. 基本方針

Rooney のレビューでは、単なる新機能レビューではなく、

> **最新版のコード全体を再監査し、前回までの品質・設計・問題点が維持されているかを確認する「継続的な全体レビュー」**

として扱う。

特に、

- 新機能が正しく動くか
- 既存機能を壊していないか
- async / generation race が再発していないか
- Unicode / Rope / Tree-sitter の整合性が維持されているか
- 大規模ファイルでの性能特性が悪化していないか
- README / docs が実装と一致しているか
- 新しい abstraction が保守性を悪化させていないか

を確認する。

**「追加された機能だけを見る」のは禁止。必ず全体への影響を見る。**

---

# 2. 前回レビューとの比較

レビュー開始時には、可能な限り以下を確認する。

1. 前回レビュー資料がリポジトリ内に存在するか
2. `REVIEW.md`
3. `REVIEW.en.md`
4. `REVIEW_YYYY-MM-DD_JA.md`
5. `REVIEW_YYYY-MM-DD_EN.md`
6. `SESSION_HANDOVER.md`
7. Git history / commit history
8. 前回レビューで P1 / P2 / P3 とされた問題

前回レビュー資料が存在しない場合は、現在のコードから独立してレビューを開始し、**前回のレビューが存在したと推測して内容を捏造しない。**

---

# 3. レビューの基本形式

レビュー結果は原則として以下の構成にする。

## Rooney — Comprehensive Technical Review

- Review date
- Reviewed commit / version
- Review scope
- Comparison with previous review
- Overall rating

### Overall rating

`X.X / 10`

点数は機械的に上げ続けない。

新機能が増えていても、

- correctness regression
- race condition
- data loss
- semantic loss
- performance regression
- documentation mismatch

などが見つかった場合は、評価を下げる。

逆に、既存の重大問題が解消された場合は評価を上げる。

---

# 4. 必須の分野別レビュー

最低限、以下を評価する。

| Area |
|---|
| Architecture |
| TextBuffer / Rope |
| Unicode / Grapheme |
| Tree-sitter |
| Async / Race Safety |
| Search |
| Viewport / Wrap |
| Glyph Metrics |
| Markdown |
| File I/O |
| Undo / Redo |
| AI |
| Security |
| Testing |
| Documentation |
| Performance |
| Maintainability |

各項目について、必要に応じて `/10` の評価を付ける。

---

# 5. 特に重視する技術項目

## 5.1 TextBuffer / Rope

確認する。

- Ropey の利用方法
- UTF-8 byte offset
- char index
- line index
- grapheme cluster
- cursor movement
- deletion
- insertion
- undo / redo
- large-file memory
- `full_text()` の不要な大量 allocation

Unicode を `char` 単位だけで扱っていないか確認する。

---

## 5.2 Tree-sitter

Rooney では Tree-sitter の correctness が重要。

確認する。

- `InputEdit`
- byte offset
- `tree_sitter::Point`
- row
- byte column
- incremental parsing
- full parsing
- parse generation
- stale result discard
- file reload
- save-as
- language switch
- undo / redo
- tab replacement
- asynchronous parse completion

特に、

```text
Worker A → Generation N
ユーザー操作
Worker B → Generation N+1
Worker A 完了
```

という stale completion race を必ず考える。

**古い worker の結果が新しい状態を上書きしていないかだけでなく、古い worker が async state flag を誤って変更していないかも確認する。**

---

# 6. Generation-based async architecture

Rooney では現在、用途ごとに generation を分離する設計を基本方針とする。

代表例:

```text
parse_generation
search_generation
markdown_generation
```

レビュー時には、

- generation の increment 条件
- worker に渡される generation
- completion message に含まれる generation
- completion handler の比較
- stale completion 時の state mutation

を確認する。

重要なのは、

```rust
if generation == current_generation {
    // apply result
}
```

だけではない。

stale result の場合に、

```rust
is_parsing_async
is_searching_async
needs_highlight_parse
needs_search_update
```

などを誤って変更していないかも確認する。

---

# 7. Search

確認する。

- 小さいファイルの同期 search
- 大きいファイルの async search
- generation
- stale result
- query typing
- content edit
- worker の重複
- cancellation / coalescing
- memory allocation

正しさと CPU 効率を分けて評価する。

generation によって stale result を防げていても、古い worker が CPU を使い続けるなら、それは別の performance issue として記録する。

---

# 8. Markdown

Markdown は機能追加時に特に慎重にレビューする。

確認する。

- CommonMark
- GFM
- headings
- paragraph
- emphasis
- strong
- strikethrough
- inline code
- links
- images
- image fallback
- lists
- nested lists
- block quotes
- code blocks
- tables
- task lists
- alerts
- footnotes
- HTML
- `<br>`
- `<img>`
- nested inline formatting
- generation
- async parsing
- preview toggle
- Markdown specification change

特に、

```markdown
- Parent
  - Child
```

のような nested structure を必ず確認する。

Markdown AST を String に flatten して semantic information が失われていないか確認する。

---

# 9. Markdown Rich Inline Representation

Rich Markdown が導入されている場合、

```text
Text
Bold
Italic
Code
Strikethrough
Link
ImageFallback
```

だけでなく、nested formatting を確認する。

例えば、

```markdown
**bold *italic***
```

```markdown
***bold italic***
```

```markdown
~~**bold strike**~~
```

```markdown
[**bold link**](https://example.com)
```

など。

単独の formatting が動いていても、nested combination が正しく表現できなければ、その点を明記する。

---

# 10. Markdown Link

README が「clickable links」などと表現している場合、実際に click handler が存在するか確認する。

単に、

```text
Link {
    text,
    url,
}
```

を保持し、色を変えているだけなら、

> styled link

であって、

> clickable link

ではない。

実装と documentation の乖離として指摘する。

---

# 11. Large File

Rooney は large-file editor として評価する。

確認する。

- open
- save
- parsing
- search
- rendering
- viewport
- wrapping
- syntax highlighting
- undo / redo
- memory
- background worker
- UI responsiveness

特に 50MB 前後のファイルについて、

> UI thread が固まらない

ことと、

> 処理そのものが高速

であることを混同しない。

background worker に移しただけなら、

- responsiveness は改善
- throughput / CPU cost は未解決

として評価する。

---

# 12. Viewport / Wrap

確認する。

- virtualization
- visible-line calculation
- margin lines
- cumulative wrap model
- total height
- scroll mapping
- hit testing
- long lines
- CJK
- mixed scripts
- cache invalidation
- font change
- window resize

特に O(1), O(log N) 等の性能表現が本当に実装と一致しているか確認する。

---

# 13. Glyph Metrics

Rooney の editor rendering では `cosmic-text` の実 glyph metrics を重視する。

確認する。

- ASCII fast path
- CJK
- Unicode
- mixed scripts
- tabs
- cache
- cache invalidation
- font changes

別 renderer が文字数や固定幅近似を使っている場合は、必要に応じて consistency issue として記録する。

---

# 14. Undo / Redo

snapshot-based undo / redo の場合、

> bounded number of snapshots

と

> bounded memory

を区別する。

巨大ファイルで Rope snapshot を多数保持すると memory usage が大きくなる可能性がある。

この点を documentation でも確認する。

---

# 15. File I/O

確認する。

- atomic save
- temporary file
- rename
- failure handling
- permissions
- large-file allocation
- `full_text()` duplication
- save-as
- reload
- file replacement
- language detection

---

# 16. AI

確認する。

- Ollama integration
- streaming
- cancellation
- FIM
- sensitive-file guard
- attached file handling
- large-file attachment
- privacy implications
- unnecessary `full_text()` allocations

FIM の sensitive-file protection と一般 AI chat attachment protection は別の問題として扱う。

---

# 17. Security

確認する。

- atomic writes
- path traversal
- file deletion guards
- sensitive files
- `.env`
- credential/key files
- remote content
- HTML
- WebView
- Markdown image handling
- shell/process execution

Markdown renderer が remote content や HTML/JavaScript を不用意に実行していないか確認する。

---

# 18. Testing

テストは happy path だけでなく race condition を重視する。

最低限確認したいもの:

- Unicode
- grapheme
- Tree-sitter incremental edit
- stale parse completion
- search generation
- Markdown generation
- preview toggle
- Markdown spec change
- undo / redo
- font cache
- wrap cache
- large-file state transitions
- file reload
- save-as
- language switch

新機能を追加した場合は、その feature の regression tests が存在するか確認する。

---

# 19. Documentation

README / docs の記述と実装を照合する。

特に注意する表現:

- complete
- compliant
- guaranteed
- zero-copy
- O(1)
- O(log N)
- real-time
- non-blocking
- clickable
- secure
- fully supported
- no UI freeze
- 50MB support

実装が部分対応なら、部分対応であることを明記する。

---

# 20. 問題の優先度

問題は以下の基準で分類する。

## P1 — Correctness / Data Loss / Serious Race

例:

- data loss
- stale result overwriting new state
- wrong AST
- corrupted save
- cursor corruption
- nested Markdown content disappearing
- async state corruption

最優先で修正を推奨。

## P2 — Significant Quality / Performance

例:

- unnecessary background workers
- large allocation
- serious UI latency
- missing important interaction
- incomplete semantic handling

## P3 — Optimization / Maintainability

例:

- code organization
- minor allocations
- renderer modularization
- typography consistency
- future optimization

---

# 21. 「修正済み問題」の再確認

前回のレビューで問題だったものは、次回レビューで必ず再確認する。

ただし、

> 「前回修正したと書いてある」

だけでは修正済みと判定しない。

コードを確認して、

```text
以前の bug
↓
修正
↓
regression protection
```

まで確認する。

---

# 22. Regression の観点

新機能をレビューするときは必ず、

```text
新機能
↓
既存 architecture
↓
既存 state
↓
async worker
↓
cache
↓
rendering
↓
tests
```

の影響範囲を見る。

特に、

- generation
- cache
- undo/redo
- tab switching
- file reload
- language switching
- preview switching

は cross-feature regression を起こしやすい。

---

# 23. 点数の付け方

評価は「機能数」ではなく、以下を総合して決める。

```text
Correctness
Architecture
Performance
Memory
Concurrency
Unicode correctness
Security
Testing
Maintainability
Documentation accuracy
```

新機能が増えても correctness が悪化した場合は点数を下げる。

修正によって architecture や correctness が改善した場合は点数を上げる。

**点数を前回より上げることを目的にしない。**

---

# 24. レビュー結果のファイル

可能なら毎回、以下の2ファイルを作成する。

```text
REVIEW_YYYY-MM-DD_JA.md
REVIEW_YYYY-MM-DD_EN.md
```

日本語版と英語版は同じ内容・同じ評価を基準とする。

必要に応じて既存の、

```text
REVIEW.md
REVIEW.en.md
```

を更新してもよい。

---

# 25. 最終レビューには必ず含めるもの

最後に、

### Overall Rating

```text
X.X / 10
```

### Key Improvements

### New Findings

### Previously Reported Issues

### P1 / P2 / P3

### Recommended Next Steps

### Final Evaluation

をまとめる。

---

# 26. Rooney の現在までのレビュー上の重要な設計原則

これまでのレビューで特に高く評価されている考え方:

1. Ropey による大規模テキスト管理
2. UTF-8 byte offset と Tree-sitter Point の正確な扱い
3. Unicode grapheme-aware editing
4. cosmic-text glyph metrics
5. viewport virtualization
6. cumulative/sparse wrap model
7. generation-based async result validation
8. stale result protection
9. atomic file save
10. sensitive-file AI protection
11. race-condition regression tests

これらを新機能追加時に不用意に崩さないこと。

---

# 27. 現在までに特に注意すべき技術的負債

過去レビューから継続して注意する項目:

- 大容量ファイルの同期 open/save
- 50MB級 Tree-sitter full parse の CPU cost
- Search worker の coalescing/cancellation
- snapshot-based Undo/Redo memory
- FileTree synchronous scanning
- Markdown inline semantic completeness
- Markdown renderer の肥大化
- AI attachment の大容量入力
- GUI / IME / Wayland 実機 QA

これらは毎回「まだ存在するか」「改善されたか」を確認する。

---

# 28. 推奨するレビュー姿勢

AI はレビュー中に、

> 「この機能は便利だから良い」

という評価だけをしない。

代わりに、

```text
What changed?
What assumptions changed?
What state is affected?
What can race?
What can become stale?
What can allocate?
What can block?
What can lose information?
What documentation became inaccurate?
```

を確認する。

特に Rust GUI editor では、

> **correctness > performance optimization > feature count**

の順序で評価する。

---

# 29. 会話が途切れた場合

過去のChatGPT会話が利用できない場合でも、このファイルをレビュー方針の基準として使用する。

その場合、

1. リポジトリの現在状態を読む
2. Git history を読む
3. `REVIEW*.md` を読む
4. `SESSION_HANDOVER.md` があれば読む
5. 現在のテストを読む
6. 現在のコードを全体的に監査する
7. 前回レビュー資料があれば比較する
8. 日本語版・英語版のレビューを作る

という順序で進める。

**過去の会話を推測して補完しない。**

---

# 30. 最終原則

Rooney のレビューでは、

> **「新機能が動くか」ではなく、「最新版の Rooney 全体が以前より良くなったか」**

を判断する。

また、

> **「前回より高い点数を付ける」ことではなく、「現在のコードの実態を正確に評価する」こと**

を優先する。

この指示書自体も、Rooney のアーキテクチャやレビュー方針が大きく変わった場合には更新する。
