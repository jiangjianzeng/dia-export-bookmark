import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { open, save } from '@tauri-apps/plugin-dialog';
import {
  Download,
  ChevronRight,
  ChevronDown,
  Folder,
  Link,
} from 'lucide-react';
import './App.css';

interface BrowserInfo {
  name: string;
  bookmark_path: string;
}

interface BookmarkNode {
  id: string;
  name: string;
  url?: string;
  date_added?: string;
  children: BookmarkNode[];
}

function App() {
  const [browserInfo, setBrowserInfo] = useState<BrowserInfo | null>(null);
  const [bookmarks, setBookmarks] = useState<BookmarkNode[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragOver, setDragOver] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);

  useEffect(() => {
    const appWindow = getCurrentWindow();
    let unlisten: (() => void) | null = null;

    const setup = async () => {
      unlisten = await appWindow.onDragDropEvent((event) => {
        if (event.payload.type === 'over') {
          setDragOver(true);
        } else if (event.payload.type === 'drop') {
          setDragOver(false);
          if (event.payload.paths.length > 0) {
            handleDrop(event.payload.paths[0]);
          }
        } else if (event.payload.type === 'leave') {
          setDragOver(false);
        }
      });
    };

    setup();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleDrop = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    setExportSuccess(false);
    setBrowserInfo(null);
    setBookmarks([]);
    try {
      const info: BrowserInfo = await invoke('detect_browser', { path });
      setBrowserInfo(info);
      const parsed: BookmarkNode[] = await invoke('parse_bookmarks', {
        path: info.bookmark_path,
      });
      setBookmarks(parsed);
    } catch (err: any) {
      setError(typeof err === 'string' ? err : err.toString());
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSelectApp = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const path = await open({
        directory: false,
        multiple: false,
        title: '选择 Dia 浏览器',
      });
      if (path) {
        handleDrop(path as string);
      }
    } catch (err: any) {
      setError(typeof err === 'string' ? err : err.toString());
    }
  };

  const handleExport = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!browserInfo || bookmarks.length === 0) return;
    setError(null);
    setExportSuccess(false);
    try {
      const path = await save({
        filters: [{ name: 'HTML', extensions: ['html'] }],
        defaultPath: `Dia 书签.html`,
      });
      if (!path) return;
      await invoke('export_bookmarks', {
        bookmarks,
        browserName: browserInfo.name,
        outputPath: path,
      });
      setExportSuccess(true);
    } catch (err: any) {
      setError(typeof err === 'string' ? err : err.toString());
    }
  };

  const handleZoneClick = () => {
    if (!browserInfo && !loading) {
      handleSelectApp({ stopPropagation: () => {} } as React.MouseEvent);
    }
  };

  return (
    <div className="app">
      <h1 className="app-title">Dia 书签导出</h1>

      <p className="app-desc">
        将 Dia 浏览器书签导出为标准 HTML 格式，支持导入到 Chrome、Safari、Firefox 等浏览器。
      </p>

      <div
        className={`drop-zone ${dragOver ? 'drag-over' : ''} ${loading ? 'loading' : ''}`}
        onClick={handleZoneClick}
      >
        {loading ? (
          <div className="drop-inner">
            <div className="app-icon-box">
              <Download size={28} />
            </div>
            <div className="drop-text">
              <div className="drop-title">正在识别应用...</div>
            </div>
            <div className="spinner" />
          </div>
        ) : browserInfo ? (
          <div className="drop-inner">
            <div className="app-icon-box detected">
              <Download size={28} />
            </div>
            <div className="drop-text">
              <div className="drop-title">{browserInfo.name}</div>
              <div className="drop-subtitle" title={browserInfo.bookmark_path}>
                {browserInfo.bookmark_path}
              </div>
            </div>
            <button className="action-btn" onClick={handleExport}>
              <Download size={14} />
              导出书签
            </button>
          </div>
        ) : (
          <div className="drop-inner">
            <div className="app-icon-box">
              <Download size={28} />
            </div>
            <div className="drop-text">
              <div className="drop-title">
                {dragOver ? '松开以导入' : '拖拽 Dia 浏览器图标到此处'}
              </div>
              <div className="drop-subtitle">或点击「选择应用」选取 Dia.app</div>
            </div>
            <button className="action-btn secondary" onClick={handleSelectApp}>
              选择应用
            </button>
          </div>
        )}
      </div>

      <div className="status-bar">
        <span className="status-label">日志:</span>
        {error ? (
          <span className="status-text error">{error}</span>
        ) : exportSuccess ? (
          <span className="status-text success">书签导出成功！</span>
        ) : browserInfo ? (
          <span className="status-text success">已识别浏览器，准备导出...</span>
        ) : (
          <span className="status-text">等待操作...</span>
        )}
      </div>

      {bookmarks.length > 0 && (
        <div className="bookmarks-section">
          <div className="bookmarks-header">
            <h3>书签预览（共 {countBookmarks(bookmarks)} 项）</h3>
          </div>
          <div className="bookmarks-tree">
            {bookmarks.map((node) => (
              <BookmarkTreeNode key={node.id} node={node} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function countBookmarks(nodes: BookmarkNode[]): number {
  return nodes.reduce((acc, node) => {
    return acc + 1 + countBookmarks(node.children);
  }, 0);
}

function BookmarkTreeNode({ node }: { node: BookmarkNode }) {
  const [expanded, setExpanded] = useState(true);
  const hasChildren = node.children.length > 0;

  if (node.url) {
    return (
      <div className="tree-node tree-link">
        <Link size={14} className="node-icon link-icon" />
        <a href={node.url} target="_blank" rel="noreferrer" title={node.url}>
          {node.name}
        </a>
      </div>
    );
  }

  return (
    <div className="tree-node tree-folder">
      <div className="folder-header" onClick={() => setExpanded(!expanded)}>
        {hasChildren ? (
          expanded ? (
            <ChevronDown size={14} className="toggle-icon" />
          ) : (
            <ChevronRight size={14} className="toggle-icon" />
          )
        ) : (
          <span className="toggle-placeholder" />
        )}
        <Folder size={14} className="node-icon folder-icon" />
        <span className="folder-name">{node.name}</span>
      </div>
      {expanded && hasChildren && (
        <div className="folder-children">
          {node.children.map((child) => (
            <BookmarkTreeNode key={child.id} node={child} />
          ))}
        </div>
      )}
    </div>
  );
}

export default App;
