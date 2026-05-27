/**
 * React 错误边界 — 捕获子组件渲染异常，展示降级 UI
 *
 * 当子组件抛出错误时，显示含错误信息的 Result 面板和"重试"按钮，
 * 用户点击重试后重置状态恢复正常渲染。
 */
import { Component } from 'react';
import { Result, Button } from 'antd';
import type { ReactNode, ErrorInfo } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export default class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, info);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <Result
          status="error"
          title="An error occurred"
          subTitle={this.state.error?.message}
          extra={
            <Button type="primary" onClick={this.handleReset}>
              Retry
            </Button>
          }
        />
      );
    }
    return this.props.children;
  }
}
