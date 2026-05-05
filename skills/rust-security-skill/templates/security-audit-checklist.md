# 安全审计清单 — ClipVault / Tauri v2

> 发布前、重大变更后执行此清单。P0 全部 PASS 方可发布。

## 审计信息

| 项目 | 值 |
|------|---|
| 审计日期 | YYYY-MM-DD |
| 代码版本 | commit hash |
| 审计范围 | 全量 / 增量（说明变更区域） |

---

## P0 — 阻断发布（必须修复）

### P0-1: FTS5 注入防护（维度 3）
- [ ] 无 `format!()` 拼接 MATCH 子句
- [ ] 所有 FTS 查询使用 `?1` 参数绑定
- [ ] 用户输入经过 `sanitize_fts_query()` 清洗
```bash
rg -n 'format!.*MATCH\|format!.*WHERE.*clips' src-tauri/src/    # 期望：无结果
rg -n 'MATCH.*\?1\|params!\[' src-tauri/src/                      # 期望：有匹配
rg -n 'sanitize_fts' src-tauri/src/                                # 期望：有匹配
```

### P0-2: 敏感内容存储（维度 1）
- [ ] 剪贴板写入前经过敏感内容检测
- [ ] Critical 级别内容加密存储或不存储
- [ ] Critical 级别内容不进 FTS 索引
- [ ] preview 字段对敏感内容脱敏
```bash
rg -n 'SensitiveFilter\|SensitivityLevel' src-tauri/src/          # 期望：有匹配
rg -n 'should_encrypt\|encrypt_content\|should_index' src-tauri/src/
rg -n 'sanitized_preview\|mask_pii' src-tauri/src/
```

### P0-3: Shell 执行权限（维度 2）
- [ ] 无 `shell:allow-execute` 权限声明
- [ ] `shell:allow-open` 仅打开白名单 URL
```bash
rg 'allow-execute' src-tauri/capabilities/                         # 期望：无结果
rg 'shell:allow-open' src-tauri/src/                               # 审计所有 open 调用
```

### P0-4: CSP 安全（维度 5）
- [ ] CSP 不包含 `unsafe-eval`
- [ ] `script-src` 不包含 `unsafe-inline`
```bash
rg 'unsafe-eval' src-tauri/tauri.conf.json                         # 期望：无结果
rg "script-src" src-tauri/tauri.conf.json                          # 确认不含 unsafe-inline
```

### P0-5: FFI 裸 unwrap（维度 4）
- [ ] 无 `arboard::*.unwrap()` 或 `enigo::*.unwrap()` 裸调用
- [ ] FFI 调用有 `catch_unwind` 或 `safe_ffi` 包裹
```bash
rg -n 'Clipboard::new\(\).*unwrap\|Enigo::new\(\).*unwrap' src-tauri/src/  # 期望：无结果
rg -n 'catch_unwind\|safe_ffi' src-tauri/src/                               # 期望：有匹配
```

### P0-6: 依赖漏洞（维度 6）
- [ ] `cargo audit` 无已知漏洞
```bash
cd src-tauri && cargo audit 2>&1                                   # 期望：无漏洞
```

---

## P1 — 限期修复（下个版本）

### P1-1: Capabilities 权限最小化（维度 2）
- [ ] 无 `allow-*` 通配权限
- [ ] 每项权限对应实际使用
```bash
rg 'allow-\*' src-tauri/capabilities/                              # 期望：无结果
```

### P1-2: FFI 超时控制（维度 4）
- [ ] 剪贴板操作有超时（<=5秒）
- [ ] FFI 调用在 `spawn_blocking` 中执行
```bash
rg -n 'timeout\|Duration' src-tauri/src/
rg -n 'spawn_blocking' src-tauri/src/
```

### P1-3: 前端 XSS 防护（维度 5）
- [ ] 无 `innerHTML` 直接赋值
- [ ] 无 `v-html` / `dangerouslySetInnerHTML`
```bash
rg 'innerHTML\|v-html\|dangerouslySetInnerHTML\|document\.write' src/    # 期望：无结果
```

### P1-4: WebView 导航限制（维度 5）
- [ ] 外部链接在系统浏览器打开
- [ ] 无未验证的导航
```bash
rg 'navigation\|onNavigation' src-tauri/src/
```

---

## P2 — 持续改进

### P2-1: 敏感内容检测覆盖率（维度 1）
- [ ] 正则模式覆盖 AWS/GCP/Azure/Stripe 密钥
- [ ] 有 Luhn 校验信用卡号
```bash
rg -n 'RegexSet\|SensitiveFilter' src-tauri/src/
```

### P2-2: 审计日志（维度 6）
- [ ] 安全事件有日志记录
- [ ] 日志不包含敏感内容本身
```bash
rg -n 'tracing::\|warn!\|error!' src-tauri/src/
rg -n 'sensitive\|intercept' src-tauri/src/
```

### P2-3: 安全测试自动化（维度 6）
- [ ] FTS 注入有单元测试覆盖
- [ ] 敏感内容检测有单元测试
- [ ] FFI 安全封装有测试
```bash
rg -n '#\[test\]' src-tauri/src/
rg -n 'sanitize_fts\|SensitiveFilter\|safe_ffi' src-tauri/src/
```

### P2-4: unsafe 使用审计（维度 4）
- [ ] 所有 `unsafe` 块有 `// SAFETY:` 注释
```bash
rg -n 'unsafe\s*\{' src-tauri/src/     # 列出所有 unsafe 块
rg -n 'SAFETY:' src-tauri/src/          # 确认有对应注释
```

---

## 审计结论

| 级别 | 通过 / 总数 | 状态 |
|------|------------|------|
| P0 | /6 | PASS / FAIL |
| P1 | /4 | PASS / FAIL |
| P2 | /4 | PASS / FAIL |

**结论：** PASS / FAIL / CONDITIONAL（附说明）

**P0 FAIL 项必须修复后重新审计。**

---

## 审计记录

| 日期 | 范围 | P0 | P1 | P2 | 结论 | 审计人 |
|------|------|----|----|----|----|--------|
| | | /6 | /4 | /4 | | |
