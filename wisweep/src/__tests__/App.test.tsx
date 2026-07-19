import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import App from "../App";

// Mock the stores
vi.mock("../stores", () => ({
  useAppStore: vi.fn(() => ({
    activeTab: "scan",
    loadFavoritePaths: vi.fn(),
    loadRules: vi.fn(),
    selectedFiles: new Set(),
    isCleaning: false,
    cleanupResult: null,
    setCleanupResult: vi.fn(),
  })),
}));

// Mock components
vi.mock("../components/common", () => ({
  Sidebar: () => <div data-testid="sidebar">Sidebar</div>,
}));

vi.mock("../components/scan", () => ({
  ScanPage: () => <div data-testid="scan-page">ScanPage</div>,
  ResultPage: () => <div data-testid="result-page">ResultPage</div>,
  CleanupDialog: () => <div data-testid="cleanup-dialog">CleanupDialog</div>,
  CleanupProgress: () => (
    <div data-testid="cleanup-progress">CleanupProgress</div>
  ),
}));

vi.mock("../components/empty-folders", () => ({
  EmptyFoldersPage: () => (
    <div data-testid="empty-folders-page">EmptyFoldersPage</div>
  ),
}));

vi.mock("../components/history", () => ({
  HistoryPage: () => <div data-testid="history-page">HistoryPage</div>,
}));

vi.mock("../components/settings", () => ({
  SettingsPage: () => <div data-testid="settings-page">SettingsPage</div>,
}));

describe("App - 禁用右键菜单功能", () => {
  it("当用户在页面上点击右键时，不应该显示浏览器默认的右键菜单", () => {
    render(<App />);

    const appElement = screen.getByTestId("sidebar").closest(".app");
    expect(appElement).toBeInTheDocument();

    // 创建一个右键事件
    const contextMenuEvent = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
    });

    // 模拟事件的 preventDefault 方法
    const preventDefaultSpy = vi.spyOn(contextMenuEvent, "preventDefault");

    // 触发右键事件
    fireEvent(appElement!, contextMenuEvent);

    // 验证 preventDefault 被调用，这意味着右键菜单被禁用
    expect(preventDefaultSpy).toHaveBeenCalled();
  });

  it("当用户在页面上按下右键时，应该阻止默认行为", () => {
    render(<App />);

    const appElement = screen.getByTestId("sidebar").closest(".app");
    expect(appElement).toBeInTheDocument();

    // 创建一个鼠标按下事件（右键）
    const mouseDownEvent = new MouseEvent("mousedown", {
      bubbles: true,
      cancelable: true,
      button: 2, // 右键
    });

    // 模拟事件的 preventDefault 方法
    const preventDefaultSpy = vi.spyOn(mouseDownEvent, "preventDefault");

    // 触发鼠标按下事件
    fireEvent(appElement!, mouseDownEvent);

    // 验证 preventDefault 被调用
    expect(preventDefaultSpy).toHaveBeenCalled();
  });
});
