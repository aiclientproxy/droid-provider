/**
 * 插件组件类型声明
 */

declare module "@proxycast/plugin-components" {
  import { ComponentType, ReactNode } from "react";

  // UI 组件
  export const Button: ComponentType<{
    children?: ReactNode;
    variant?: "default" | "outline" | "ghost" | "destructive";
    size?: "default" | "sm" | "lg";
    onClick?: () => void;
    disabled?: boolean;
    className?: string;
  }>;

  export const Modal: ComponentType<{
    isOpen: boolean;
    onClose: () => void;
    children?: ReactNode;
    maxWidth?: string;
  }>;

  export const Tabs: ComponentType<{
    value: string;
    onValueChange: (value: string) => void;
    children?: ReactNode;
  }>;

  export const TabsList: ComponentType<{
    children?: ReactNode;
    className?: string;
  }>;

  export const TabsTrigger: ComponentType<{
    value: string;
    children?: ReactNode;
    className?: string;
  }>;

  export const TabsContent: ComponentType<{
    value: string;
    children?: ReactNode;
  }>;

  export const Badge: ComponentType<{
    children?: ReactNode;
    variant?: "default" | "outline" | "secondary";
    className?: string;
  }>;

  export const Card: ComponentType<{
    children?: ReactNode;
    className?: string;
  }>;

  export const CardHeader: ComponentType<{
    children?: ReactNode;
    className?: string;
  }>;

  export const CardContent: ComponentType<{
    children?: ReactNode;
    className?: string;
  }>;

  export const CardFooter: ComponentType<{
    children?: ReactNode;
    className?: string;
  }>;

  export const Switch: ComponentType<{
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
  }>;

  // Droid 专用组件
  export const DroidFormStandalone: ComponentType<{
    authType: string;
    onSuccess: () => void;
    onCancel: () => void;
  }>;

  export const EditCredentialModal: ComponentType<{
    credential: CredentialDisplay | null;
    isOpen: boolean;
    onClose: () => void;
    onEdit: (uuid: string, request: UpdateCredentialRequest) => Promise<void>;
  }>;

  // 图标
  export const Plus: ComponentType<{ className?: string }>;
  export const RefreshCw: ComponentType<{ className?: string }>;
  export const Zap: ComponentType<{ className?: string }>;
  export const Loader2: ComponentType<{ className?: string }>;
  export const AlertCircle: ComponentType<{ className?: string }>;
  export const Key: ComponentType<{ className?: string }>;
  export const KeyRound: ComponentType<{ className?: string }>;
  export const Bot: ComponentType<{ className?: string }>;
  export const CheckCircle: ComponentType<{ className?: string }>;
  export const XCircle: ComponentType<{ className?: string }>;
  export const Clock: ComponentType<{ className?: string }>;
  export const Mail: ComponentType<{ className?: string }>;
  export const Building: ComponentType<{ className?: string }>;
  export const User: ComponentType<{ className?: string }>;
  export const Trash2: ComponentType<{ className?: string }>;
  export const Edit: ComponentType<{ className?: string }>;
  export const RotateCcw: ComponentType<{ className?: string }>;

  // Toast
  export const toast: {
    success: (message: string) => void;
    error: (message: string) => void;
    info: (message: string) => void;
  };

  // Provider Pool API
  export const providerPoolApi: {
    getCredentials: (providerId: string) => Promise<CredentialDisplay[]>;
    deleteCredential: (uuid: string, providerId: string) => Promise<void>;
    toggleCredential: (uuid: string, enabled: boolean) => Promise<void>;
    resetCredential: (uuid: string) => Promise<void>;
    checkCredentialHealth: (uuid: string) => Promise<{ success: boolean; message?: string }>;
    refreshCredentialToken: (uuid: string) => Promise<void>;
    updateCredential: (uuid: string, request: UpdateCredentialRequest) => Promise<void>;
  };

  // 类型
  export interface CredentialDisplay {
    uuid: string;
    name: string | null;
    provider_id: string;
    is_disabled: boolean;
    health_status: string;
    usage_count: number;
    error_count: number;
    last_error: string | null;
    credential_data: Record<string, unknown>;
    created_at: string;
    updated_at: string;
  }

  export interface UpdateCredentialRequest {
    name?: string;
    credential_data?: Record<string, unknown>;
  }

  export interface PluginSDK {
    pluginId: string;
  }
}
