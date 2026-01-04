/**
 * 凭证卡片组件类型定义
 */

import type { CredentialDisplay } from "@proxycast/plugin-components";

export interface CredentialCardProps {
  credential: CredentialDisplay;
  onToggle: () => void;
  onDelete: () => void;
  onReset: () => void;
  onCheckHealth: () => void;
  onRefreshToken: () => void;
  onEdit: () => void;
  deleting: boolean;
  checkingHealth: boolean;
  refreshingToken: boolean;
}

/**
 * 认证类型标签配置
 */
export const AUTH_TYPE_LABELS: Record<string, string> = {
  oauth: "WorkOS OAuth",
  api_key: "API Key",
};

/**
 * 认证类型颜色配置
 */
export const AUTH_TYPE_COLORS: Record<string, string> = {
  oauth: "blue",
  api_key: "green",
};

/**
 * 端点类型标签配置
 */
export const ENDPOINT_TYPE_LABELS: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  comm: "Chat Completions",
};
