import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ScanPage } from "../components/scan/ScanPage";

// Mock the stores
vi.mock("../stores", () => ({
  useAppStore: vi.fn(() => ({
    scanConfig: {
      paths: [],
      recursive: true,
      include_hidden: true,
      scan_empty_folders: true,
      min_file_size: 1024,
    },
    setScanConfig: vi.fn(),
    scanProgress: null,
    isScanning: false,
    startScan: vi.fn(),
    pauseScan: vi.fn(),
    resumeScan: vi.fn(),
    cancelScan: vi.fn(),
    getDiskSpace: vi.fn(),
    favoritePaths: [],
  })),
}));

// Mock the dialog plugin
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

// Mock lucide-react icons
vi.mock("lucide-react", () => ({
  FolderOpen: () => <span data-testid="folder-open-icon" />,
  Play: () => <span data-testid="play-icon" />,
  Pause: () => <span data-testid="pause-icon" />,
  Square: () => <span data-testid="square-icon" />,
}));

// Mock formatSize
vi.mock("../utils/format", () => ({
  formatSize: (size: number) => `${size} bytes`,
}));

describe("ScanPage - 响应式布局", () => {
  it("当页面渲染时，应该显示开始扫描的标题", () => {
    render(<ScanPage />);
    
    // 标题和按钮都包含"开始扫描"文本
    const elements = screen.getAllByText("开始扫描");
    expect(elements.length).toBeGreaterThanOrEqual(2);
    
    // 验证标题存在
    expect(screen.getByRole("heading", { name: "开始扫描" })).toBeInTheDocument();
    expect(screen.getByText("选择要扫描的文件夹，智能识别可清理的文件")).toBeInTheDocument();
  });

  it("当页面渲染时，应该包含路径输入区域", () => {
    render(<ScanPage />);
    
    expect(screen.getByPlaceholderText("输入或粘贴路径（多个路径用分号分隔）")).toBeInTheDocument();
    expect(screen.getByText("浏览")).toBeInTheDocument();
  });

  it("当页面渲染时，应该显示扫描配置区域", () => {
    render(<ScanPage />);
    
    expect(screen.getByText("扫描配置")).toBeInTheDocument();
    expect(screen.getByText("递归扫描子目录")).toBeInTheDocument();
    expect(screen.getByText("包含隐藏文件")).toBeInTheDocument();
    expect(screen.getByText("扫描空文件夹")).toBeInTheDocument();
    expect(screen.getByText("最小文件大小:")).toBeInTheDocument();
  });

  it("当页面渲染时，应该显示开始扫描按钮", () => {
    render(<ScanPage />);
    
    const button = screen.getByRole("button", { name: "开始扫描" });
    expect(button).toBeInTheDocument();
    expect(button).toHaveClass("btn-primary");
    expect(button).toHaveClass("btn-large");
  });

  it("当页面渲染时，应该包含响应式布局的CSS类", () => {
    render(<ScanPage />);
    
    // 验证主要容器类名存在
    const scanPage = screen.getByText("扫描配置").closest(".scan-page");
    expect(scanPage).toBeInTheDocument();
    
    // 验证路径输入组类名存在
    const pathInputGroup = screen.getByPlaceholderText("输入或粘贴路径（多个路径用分号分隔）").closest(".path-input-group");
    expect(pathInputGroup).toBeInTheDocument();
    
    // 验证配置网格类名存在
    const configGrid = screen.getByText("递归扫描子目录").closest(".config-grid");
    expect(configGrid).toBeInTheDocument();
  });
});