/**
 * @proxycast/plugin-components 类型声明
 * 为插件提供类型定义，实际组件在运行时由 ProxyCast 主应用提供
 */

declare module "@proxycast/plugin-components" {
  import { ComponentType, ReactNode, FC, ButtonHTMLAttributes } from "react";

  // 基础 UI 组件
  export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link";
    size?: "default" | "sm" | "lg" | "icon";
    asChild?: boolean;
  }
  export const Button: FC<ButtonProps>;
  export const Card: FC<{ className?: string; children?: ReactNode }>;
  export const CardHeader: FC<{ className?: string; children?: ReactNode }>;
  export const CardTitle: FC<{ className?: string; children?: ReactNode }>;
  export const CardDescription: FC<{ className?: string; children?: ReactNode }>;
  export const CardContent: FC<{ className?: string; children?: ReactNode }>;
  export const CardFooter: FC<{ className?: string; children?: ReactNode }>;
  export const Tabs: FC<{ defaultValue?: string; value?: string; onValueChange?: (value: string) => void; className?: string; children?: ReactNode }>;
  export const TabsContent: FC<{ value: string; className?: string; children?: ReactNode }>;
  export const TabsList: FC<{ className?: string; children?: ReactNode }>;
  export const TabsTrigger: FC<{ value: string; className?: string; children?: ReactNode }>;
  export const Badge: FC<{ variant?: "default" | "secondary" | "destructive" | "outline"; className?: string; children?: ReactNode }>;
  export const Input: FC<React.InputHTMLAttributes<HTMLInputElement> & { className?: string }>;
  export const Textarea: FC<React.TextareaHTMLAttributes<HTMLTextAreaElement> & { className?: string }>;
  export const Switch: FC<{ checked?: boolean; onCheckedChange?: (checked: boolean) => void; disabled?: boolean; className?: string }>;
  export const Select: FC<{ value?: string; onValueChange?: (value: string) => void; children?: ReactNode }>;
  export const SelectContent: FC<{ children?: ReactNode }>;
  export const SelectItem: FC<{ value: string; children?: ReactNode }>;
  export const SelectTrigger: FC<{ className?: string; children?: ReactNode }>;
  export const SelectValue: FC<{ placeholder?: string }>;

  // Modal
  export interface ModalProps { isOpen: boolean; onClose: () => void; maxWidth?: string; children?: ReactNode; }
  export const Modal: FC<ModalProps>;

  // 表单组件
  export interface UpdateCredentialRequest { display_name?: string; is_disabled?: boolean; priority?: number; tags?: string[]; }
  export interface EditCredentialModalProps { credential: CredentialDisplay | null; isOpen: boolean; onClose: () => void; onEdit: (uuid: string, request: UpdateCredentialRequest) => Promise<void>; }
  export const EditCredentialModal: FC<EditCredentialModalProps>;

  // 工具函数
  export function cn(...inputs: (string | undefined | null | boolean)[]): string;
  export const toast: { success: (message: string) => void; error: (message: string) => void; info: (message: string) => void; warning: (message: string) => void; };

  // 图标
  export interface IconProps { className?: string; size?: number | string; }
  export const Plus: FC<IconProps>;
  export const Minus: FC<IconProps>;
  export const Check: FC<IconProps>;
  export const X: FC<IconProps>;
  export const Edit: FC<IconProps>;
  export const Trash2: FC<IconProps>;
  export const Copy: FC<IconProps>;
  export const Download: FC<IconProps>;
  export const Upload: FC<IconProps>;
  export const RefreshCw: FC<IconProps>;
  export const RotateCcw: FC<IconProps>;
  export const Loader2: FC<IconProps>;
  export const AlertCircle: FC<IconProps>;
  export const AlertTriangle: FC<IconProps>;
  export const CheckCircle: FC<IconProps>;
  export const XCircle: FC<IconProps>;
  export const Key: FC<IconProps>;
  export const KeyRound: FC<IconProps>;
  export const Lock: FC<IconProps>;
  export const Clock: FC<IconProps>;
  export const Zap: FC<IconProps>;
  export const Globe: FC<IconProps>;
  export const Mail: FC<IconProps>;
  export const User: FC<IconProps>;
  export const Users: FC<IconProps>;
  export const Building: FC<IconProps>;
  export const Cloud: FC<IconProps>;
  export const Server: FC<IconProps>;


  // 类型定义
  export interface PluginSDK { getCredentials: () => Promise<CredentialInfo[]>; refreshCredential: (uuid: string) => Promise<void>; deleteCredential: (uuid: string) => Promise<void>; showToast: (message: string, type?: "success" | "error" | "info") => void; }
  export interface CredentialInfo { uuid: string; provider_type: string; display_name: string; is_disabled: boolean; health_status: string; }

  // Provider Pool API
  export type PoolProviderType = "kiro" | "gemini" | "qwen" | "antigravity" | "openai" | "claude" | "droid";
  export type CredentialSource = "local" | "remote" | "manual" | "imported" | "private";

  export interface CredentialDisplay {
    uuid: string;
    provider_type: PoolProviderType;
    display_name: string;
    name?: string;
    credential_type?: string;
    is_disabled: boolean;
    is_healthy?: boolean;
    health_status: string;
    last_health_check?: string | null;
    last_used?: string | null;
    total_requests?: number;
    failed_requests?: number;
    usage_count?: number;
    error_count?: number;
    priority?: number;
    tags?: string[];
    source?: CredentialSource;
    last_error?: string;
    auth_type?: string;
    credential_data?: Record<string, unknown>;
  }

  export interface HealthCheckResult { success: boolean; message?: string; }

  export const providerPoolApi: {
    getCredentials: (providerType?: PoolProviderType) => Promise<CredentialDisplay[]>;
    deleteCredential: (uuid: string, providerType?: string) => Promise<void>;
    toggleCredential: (uuid: string, enabled: boolean) => Promise<void>;
    resetCredential: (uuid: string) => Promise<void>;
    checkCredentialHealth: (uuid: string) => Promise<HealthCheckResult>;
    refreshCredentialToken: (uuid: string) => Promise<void>;
    updateCredential: (uuid: string, request: UpdateCredentialRequest) => Promise<void>;
  };
}
