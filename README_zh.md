<p align="center">
  <a href="https://github.com/covoyage/kairoa">
    <img width="200px" src="https://github.com/covoyage/kairoa/raw/main/src-tauri/icons/icon.png">
  </a>
</p>

<h1 align="center">
  Kairoa ➟ 开发者工具箱
</h1>

<p align="center">
    为开发者打造的现代跨平台桌面版开发工具。
</p>

<div align="left">

**简体中文 | [English](./README.md)**

</div>

<p align="center">
  <img src="screenshots/kairoa_zh.png" alt="主页" style="max-width: 100%; height: auto;" />
</p>

## 功能特性

### 🔐 Hash 计算器
- 计算文本和文件的哈希值
- 支持多种算法：MD5、SHA-1、SHA-256、SHA-384、SHA-512
- 支持拖拽文件
- 复制哈希值并显示视觉反馈

### ⏰ 时间转换器
- 时间戳和日期互转
- 支持时区选择（IANA 时区列表）
- 可搜索的时区下拉菜单
- 多种格式显示结果（YYYY-MM-DD HH:mm:ss 和 ISO 格式）
- 快速获取当前时间

### 🔑 UUID 生成器
- 生成单个或多个 UUID
- 可选择是否包含连字符
- 复制单个 UUID 或一键复制全部
- 可配置生成数量

### 📝 JSON 格式化器
- 格式化和压缩 JSON
- 格式化后的 JSON 语法高亮显示
- 实时验证
- 复制格式化后的 JSON 到剪贴板

### ⚙️ 配置转换器
- 支持多种配置文件格式之间的转换：JSON、YAML、TOML、INI、XML、Properties、ENV、TOON
- 实时转换和格式验证
- 自动保存转换历史
- 支持双向转换，满足项目迁移和配置统一需求

### 🔧 编解码工具
- **Base64**: 文本和图片的编码/解码
- **URL**: URL 字符串的编码/解码
- **图片/Base64**: 图片与 Base64 格式互转
- 图片预览和下载支持
- 并排输入/输出布局

### 🌐 REST API 客户端
- 支持多种 HTTP 方法（GET、POST、PUT、DELETE、PATCH、HEAD、OPTIONS）
- 自定义请求头
- 多种请求体类型：JSON、Text、XML、Form Data、URL Encoded
- 多标签页支持，可同时管理多个请求
- 响应显示：状态码、响应头、响应体、响应时间
- **cURL 导入/导出**：从 cURL 命令导入请求配置，或将请求导出为 cURL 命令
  - 自动检测 body 参数（`-d`、`--data`、`--data-raw`、`--data-binary`、`--data-urlencode`、`-F`、`--form`），未明确指定方法时自动设置为 POST（遵循 cURL 最佳实践）
  - 格式化的 cURL 命令输出，便于阅读
  - 正确的 shell 字符串转义，确保命令可正确执行

### 🔌 WebSocket 测试工具
- 支持 WebSocket (ws://) 和加密 WebSocket (wss://) 连接
- 实时发送和接收消息
- 消息历史记录和查看
- 连接状态监控
- 支持自定义消息格式
- 方便开发和调试实时应用、监控系统

### 🐳 Docker 命令生成器
- 可视化生成常用的 Docker 命令
- 容器管理：运行、停止、删除、查看日志等
- 镜像操作：构建、拉取、推送、删除等
- 网络和卷管理
- 支持自定义参数和选项
- 无需记忆复杂命令参数，通过界面操作即可生成准确命令

### 📦 Git 命令生成器
- 快速生成常用的 Git 命令
- 提交管理：add、commit、push、pull 等
- 分支操作：创建、切换、合并、删除等
- 远程仓库管理
- 历史查看和回退
- 特别适合 Git 初学者和需要快速生成复杂命令的场景

### 📊 文本统计
- 字符数统计（含/不含空格）
- 单词数统计（支持英文和中文）
- 行数、段落数统计
- 字符类型分析：中文字符、英文字符、数字、标点符号

### 📅 Cron 表达式解析器
- 解析和验证 cron 表达式
- 生成人类可读的描述
- 显示下次执行时间

### 📄 PDF 签名校验
- 本地检查 PDF 数字签名
- 校验 CMS (PKCS#7) 分离式签名
- 显示签名者、ByteRange 与证书信息

### 🔒 TLS 版本检测
- 检测远程服务器支持的 TLS/SSL 版本
- 测试 TLS 1.0、1.1、1.2 和 1.3 支持情况
- 显示证书信息
- 识别首选 TLS 版本

### 📡 端口扫描
- 扫描远程主机的 TCP 端口并显示开放情况
- 可配置扫描范围、超时时间与最大并发
- 内置常用端口范围快捷按钮
- 展示开放端口及延迟信息

### 🔍 路由追踪
- 追踪数据包到达目标主机所经过的路由路径
- 显示每一跳的主机名、IP 地址和往返时间（RTT）
- 可配置最大跳数和超时设置
- 跨平台支持（Windows tracert，Linux/macOS traceroute）
- 适用于网络诊断、路由分析和连接故障排查

### 🌍 IP/域名查询工具
- 查询 IP 地址的地理位置、ISP 信息
- 域名解析和反向查询
- 网络详情展示
- 支持批量查询
- 方便进行网络诊断、安全分析和地理位置定位

### 🔍 DNS 查询
- 查询域名的 DNS 记录（A、AAAA、CNAME、MX、TXT、NS、SOA、SRV）
- 支持 DNS over HTTPS (DoH)，使用 Cloudflare DNS
- 可同时选择多种记录类型进行查询
- 以表格形式展示 TTL 和记录数据
- 常用记录类型快速选择按钮
- 适用于 DNS 故障排查、域名配置和网络分析

### 📋 HTTP 状态码查询
- 查询 HTTP 状态码，包含详细描述和 RFC 参考
- 支持按状态码数字或名称/描述搜索
- 默认展示常用状态码，方便快速查阅
- 支持所有标准 HTTP 状态码（1xx-5xx）
- 分类显示：信息性、成功、重定向、客户端错误、服务器错误
- 一键复制状态码
- 适用于 API 开发、调试和学习 HTTP 协议

### 📄 MIME 类型查询
- 通过文件扩展名或 MIME 类型字符串查询 MIME 类型
- 包含现代格式的完整 MIME 类型数据库
- 支持按扩展名（如 pdf、jpg）或 MIME 类型（如 application/pdf）搜索
- 显示关联的文件扩展名和描述
- 适用于 Web 开发、文件处理和内容类型配置

### 🔍 User-Agent 解析器
- 解析和分析 User-Agent 字符串，提取浏览器、操作系统和设备信息
- 显示浏览器名称、版本、渲染引擎
- 显示操作系统和设备类型
- 提供常见 User-Agent 示例供快速测试
- 一键使用当前浏览器的 User-Agent
- 适用于 Web 开发、分析和兼容性测试

### 🔑 环境变量管理器
- 管理环境变量，支持导入导出
- 添加、编辑和删除环境变量
- 从 .env 格式文本导入
- 导出为 .env 格式
- 一键复制所有变量
- 本地存储持久化
- 适用于开发环境配置和测试

### 🎨 颜色格式转换
- 多种颜色格式转换：HEX、RGB、HSL、HWB、LCH、CMYK、颜色名称
- 颜色选择器实时预览
- 支持任意格式间的双向转换
- 所有格式可编辑并实时转换

### 🔒 密码强度检测器
- 实时密码强度评估
- 多维度安全检查：长度、大小写、数字、特殊字符、字符多样性
- 常见密码检测
- 连续字符和重复字符检测
- 破解时间估算
- 针对弱密码的改进建议

### 🔐 密码保险库
- 使用 AES 加密的安全本地密码管理器
- 主密码保护所有存储的密码
- 添加、编辑和删除密码条目
- 按类别组织密码（通用、工作、个人、财务、社交、开发）
- 按标题、用户名或 URL 搜索和筛选密码
- 显示/隐藏密码可见性切换
- 导入和导出加密的密码文件
- 一键复制密码
- 所有数据加密并本地存储，永不共享或上传
- 重置保险库选项可清除所有数据

### 📜 证书查看器
- 查看和检查证书文件（PEM、CRT、CER、KEY、CERT、DER、P12、PFX）
- 显示证书详情：主题、颁发者、有效期、序列号、算法、密钥长度
- SHA-256 指纹计算
- 证书有效期状态（有效/已过期/尚未生效）
- 扩展信息显示
- 文件拖拽支持
- 完全离线处理

### 🔢 进制转换器
- 支持多种进制转换：二进制（2）、八进制（8）、十进制（10）、十六进制（16）
- 自动识别和移除前缀（0b、0o、0x）
- 双向转换，带输入验证

### 🔐 RSA 密钥生成器
- 生成 RSA 密钥对（公钥/私钥）
- 支持多种密钥长度：1024、2048、3072、4096 位
- 支持 PEM 或 DER 格式导出
- 一键复制密钥到剪贴板

## 界面特性

- 🌓 **浅色/深色主题**: 支持浅色和深色模式切换（6 种主题可选）
- 🌍 **国际化**: 支持英文和中文
- 🎨 **现代化 UI**: 使用 Tailwind CSS 构建
- 📱 **响应式设计**: 简洁直观的界面

## 技术栈

- **前端**: SvelteKit 5、TypeScript、Tailwind CSS
- **桌面端**: Tauri 2
- **图标**: Lucide Svelte
- **加密**: crypto-js

## 开发环境要求

- Node.js 18+ 和 npm
- Rust（最新稳定版）
- Tauri 的系统依赖：
  - **macOS**: Xcode Command Line Tools
  - **Linux**: 系统依赖（参见 [Tauri 文档](https://tauri.app/v1/guides/getting-started/prerequisites)）
  - **Windows**: Microsoft Visual Studio C++ Build Tools

## 安装

本项目使用 GitHub CI 构建所有平台的二进制文件。请前往 [Releases](https://github.com/covoyage/kairoa/releases) 页面下载对应平台的二进制文件。

### macOS

由于 macOS 二进制文件未使用 Apple Developer 证书进行代码签名，您可能需要先移除隔离属性才能运行应用：

```bash
xattr -r -c /Applications/kairoa.app
```

此命令会移除 macOS 为下载的应用添加的扩展属性，允许您在没有 Gatekeeper 警告的情况下运行应用。

### Linux

应用程序会自动处理 Linux 系统上常见的显示问题。如果您仍然遇到白屏，可能是由于访问图形设备的权限不足导致的。

**自动处理：**

应用程序在启动时会自动设置 `WEBKIT_DISABLE_DMABUF_RENDERER=1` 来解决 NVIDIA GPU 与 WebKit2GTK 的兼容性问题。如果硬件加速失败，会自动回退到软件渲染。

**手动解决方法（如果自动处理不起作用）：**

如果您仍然遇到问题，可以手动将您的用户添加到 `video` 和 `render` 组：

```bash
sudo usermod -a -G video,render $USER
```

将用户添加到这些组后，您需要注销并重新登录（或重启系统）才能使更改生效。

**替代方案：**

您也可以显式启用软件渲染来运行应用程序：

```bash
LIBGL_ALWAYS_SOFTWARE=1 ./kairoa
```

**注意：** 软件渲染可能会影响性能，但应该能解决显示问题。

## 开发

1. 克隆仓库：
```bash
git clone https://github.com/covoyage/kairoa.git
cd kairoa
```

2. 安装依赖：
```bash
npm install
```

3. 运行：
```bash
npm run tauri dev
```

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

### 代码规范

- 遵循 TypeScript 最佳实践
- 使用 Svelte 5 runes（`$state`、`$derived`、`$effect`）
- 遵循现有的代码风格和格式
- 为复杂逻辑添加注释

## 许可证

AGPL-3.0 License - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- 使用 [Tauri](https://tauri.app/) 构建
- UI 使用 [SvelteKit](https://kit.svelte.dev/) 构建
- 图标来自 [Lucide](https://lucide.dev/)
- 样式使用 [Tailwind CSS](https://tailwindcss.com/)

## 支持

如果您遇到任何问题或有疑问，请在 GitHub 上提交 issue。

---

使用 ❤️ 和 Tauri + SvelteKit 制作


