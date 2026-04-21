# Dia 书签导出

一个简洁的 macOS 桌面应用，用于将 [Dia 浏览器](https://www.diabrowser.com/) 的书签导出为标准 HTML 格式，方便导入到 Chrome、Safari、Firefox 等浏览器。

![App Screenshot](/screenshots/app.png)

## 功能

- **拖拽识别**：直接将 `Dia.app` 拖入应用窗口即可自动识别
- **手动选择**：点击「选择应用」从 Applications 文件夹中选取 Dia 浏览器
- **标准导出**：生成 Netscape Bookmark File Format（HTML）标准格式
- **本地处理**：所有数据均在本地处理，不上传任何信息

## 下载

前往 [Releases](https://github.com/jiangjianzeng/dia-export-bookmark/releases) 页面下载最新版本的 `.dmg` 安装包。

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/)

### 本地运行

```bash
# 安装依赖
npm install

# 启动开发模式（带 Tauri 桌面窗口）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 项目结构

```
.
├── src/                    # React 前端代码
├── src-tauri/             # Tauri (Rust) 后端代码
│   ├── src/
│   │   └── lib.rs         # 核心逻辑：应用检测、书签解析、HTML 导出
│   └── capabilities/      # 权限配置
├── .github/workflows/     # GitHub Actions CI/CD
└── package.json
```

## 技术栈

- **前端**：React + TypeScript + Vite
- **桌面框架**：Tauri v2
- **UI 图标**：Lucide React

## License

[MIT](LICENSE)
