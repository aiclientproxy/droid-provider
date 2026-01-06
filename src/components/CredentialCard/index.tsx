/**
 * 凭证卡片组件
 *
 * 显示单个 Droid 凭证的信息和操作按钮
 */

import {
  Button,
  Badge,
  Card,
  CardHeader,
  CardContent,
  CardFooter,
  Switch,
  Loader2,
  Trash2,
  RefreshCw,
  Zap,
  Edit,
  RotateCcw,
  Key,
  KeyRound,
  CheckCircle,
  XCircle,
  Clock,
  Mail,
  Building,
  User,
} from "@proxycast/plugin-components";
import type { CredentialCardProps } from "./types";
import { AUTH_TYPE_LABELS, AUTH_TYPE_COLORS, ENDPOINT_TYPE_LABELS } from "./types";

/**
 * 获取认证类型图标
 */
function getAuthTypeIcon(authType: string) {
  switch (authType) {
    case "oauth":
      return Key;
    case "api_key":
      return KeyRound;
    default:
      return Key;
  }
}

/**
 * 格式化日期
 */
function formatDate(dateStr: unknown): string {
  if (!dateStr || typeof dateStr !== "string") return "-";
  try {
    const date = new Date(dateStr);
    return date.toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return dateStr;
  }
}

/**
 * 检查是否支持 Token 刷新
 */
function supportsTokenRefresh(authType: string): boolean {
  return authType === "oauth";
}

/**
 * 凭证卡片组件
 */
export function CredentialCard({
  credential,
  onToggle,
  onDelete,
  onReset,
  onCheckHealth,
  onRefreshToken,
  onEdit,
  deleting,
  checkingHealth,
  refreshingToken,
}: CredentialCardProps) {
  const data = credential.credential_data || {};
  const authType = String(data.auth_type || "oauth");
  const endpointType = String(data.endpoint_type || "anthropic");
  const isHealthy = !credential.is_disabled && credential.health_status !== "unhealthy";
  const AuthIcon = getAuthTypeIcon(authType);

  // API Key 数量统计
  const apiKeys = Array.isArray(data.api_keys) ? data.api_keys : [];
  const activeKeyCount = apiKeys.filter(
    (k: unknown) => typeof k === "object" && k !== null && (k as { status?: string }).status === "active"
  ).length;
  const totalKeyCount = apiKeys.length;

  // 提取显示字段
  const ownerEmail = typeof data.owner_email === "string" ? data.owner_email : "";
  const organizationId = typeof data.organization_id === "string" ? data.organization_id : "";
  const userId = typeof data.user_id === "string" ? data.user_id : "";
  const expiresAt = typeof data.expires_at === "string" ? data.expires_at : null;
  const lastRefresh = typeof data.last_refresh === "string" ? data.last_refresh : null;

  return (
    <Card className={`${credential.is_disabled ? "opacity-60" : ""}`}>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <div className="flex items-center gap-3">
          {/* 状态指示器 */}
          <div className="flex items-center gap-2">
            {isHealthy ? (
              <CheckCircle className="h-5 w-5 text-green-500" />
            ) : (
              <XCircle className="h-5 w-5 text-red-500" />
            )}
          </div>

          {/* 凭证名称 */}
          <div>
            <h3 className="font-medium">
              {credential.name || "未命名凭证"}
            </h3>
            <p className="text-sm text-muted-foreground">
              {credential.uuid.slice(0, 8)}...
            </p>
          </div>

          {/* 认证类型标签 */}
          <Badge variant="outline" className={`bg-${AUTH_TYPE_COLORS[authType]}-50`}>
            <AuthIcon className="h-3 w-3 mr-1" />
            {AUTH_TYPE_LABELS[authType] || authType}
          </Badge>

          {/* 端点类型标签 */}
          <Badge variant="secondary">
            {ENDPOINT_TYPE_LABELS[endpointType] || endpointType}
          </Badge>
        </div>

        {/* 启用/禁用开关 */}
        <Switch
          checked={!credential.is_disabled}
          onCheckedChange={onToggle}
        />
      </CardHeader>

      <CardContent className="space-y-3">
        {/* 凭证信息 */}
        <div className="grid grid-cols-2 gap-4 text-sm">
          {/* 邮箱 (OAuth) */}
          {ownerEmail.length > 0 && (
            <div className="flex items-center gap-2">
              <Mail className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">邮箱:</span>
              <span>{ownerEmail}</span>
            </div>
          )}

          {/* 组织 ID (OAuth) */}
          {organizationId.length > 0 && (
            <div className="flex items-center gap-2">
              <Building className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">组织:</span>
              <span className="truncate">{organizationId}</span>
            </div>
          )}

          {/* 用户 ID (OAuth) */}
          {userId ? (
            <div className="flex items-center gap-2">
              <User className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">用户:</span>
              <span className="truncate">{userId}</span>
            </div>
          ) : null}

          {/* API Key 数量 (API Key) */}
          {authType === "api_key" && (
            <div className="flex items-center gap-2">
              <KeyRound className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">API Keys:</span>
              <span>
                {activeKeyCount}/{totalKeyCount} 可用
              </span>
            </div>
          )}

          {/* 过期时间 (OAuth) */}
          {expiresAt && (
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">过期:</span>
              <span>{formatDate(expiresAt)}</span>
            </div>
          )}

          {/* 最后刷新 */}
          {lastRefresh && (
            <div className="flex items-center gap-2">
              <RefreshCw className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">刷新:</span>
              <span>{formatDate(lastRefresh)}</span>
            </div>
          )}
        </div>

        {/* 使用统计 */}
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          <span>使用: {credential.usage_count || 0} 次</span>
          <span>错误: {credential.error_count || 0} 次</span>
          {credential.last_error && (
            <span className="text-red-500 truncate">
              最后错误: {credential.last_error}
            </span>
          )}
        </div>
      </CardContent>

      <CardFooter className="flex justify-end gap-2">
        {/* 编辑 */}
        <Button variant="ghost" size="sm" onClick={onEdit}>
          <Edit className="h-4 w-4" />
        </Button>

        {/* 重置 */}
        <Button variant="ghost" size="sm" onClick={onReset}>
          <RotateCcw className="h-4 w-4" />
        </Button>

        {/* 检测健康 */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onCheckHealth}
          disabled={checkingHealth}
        >
          {checkingHealth ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Zap className="h-4 w-4" />
          )}
        </Button>

        {/* 刷新 Token (仅 OAuth) */}
        {supportsTokenRefresh(authType) && (
          <Button
            variant="ghost"
            size="sm"
            onClick={onRefreshToken}
            disabled={refreshingToken}
          >
            {refreshingToken ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
          </Button>
        )}

        {/* 删除 */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onDelete}
          disabled={deleting}
          className="text-red-500 hover:text-red-600"
        >
          {deleting ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Trash2 className="h-4 w-4" />
          )}
        </Button>
      </CardFooter>
    </Card>
  );
}

export default CredentialCard;
