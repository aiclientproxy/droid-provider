/**
 * Droid Provider 插件主入口
 *
 * 使用主应用的组件库实现完整的 Droid 凭证管理功能
 * 支持 WorkOS OAuth 和 API Key 两种认证方式
 */

import React, { useState, useCallback } from "react";
import {
  // UI 组件
  Button,
  Modal,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  // Droid 专用组件
  DroidFormStandalone,
  EditCredentialModal,
  // 图标
  Plus,
  RefreshCw,
  Zap,
  Loader2,
  AlertCircle,
  Key,
  KeyRound,
  Bot,
  // 工具
  toast,
  // Provider Pool API
  providerPoolApi,
  type CredentialDisplay,
  type UpdateCredentialRequest,
  // 类型
  type PluginSDK,
} from "@proxycast/plugin-components";
import { CredentialCard } from "./components/CredentialCard";

/**
 * 认证方式类型
 */
type AuthMethod = "oauth" | "api_key";

/**
 * 认证方式配置
 */
const AUTH_METHODS: {
  id: AuthMethod;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
}[] = [
  { id: "oauth", label: "WorkOS OAuth", icon: Key, description: "使用 WorkOS OAuth 授权" },
  { id: "api_key", label: "API Key", icon: KeyRound, description: "使用 Factory.ai API Key" },
];

/**
 * 插件组件 Props
 */
interface DroidProviderAppProps {
  sdk: PluginSDK;
  pluginId: string;
}

/**
 * Droid Provider 插件主组件
 */
export function DroidProviderApp(_props: DroidProviderAppProps) {
  const [credentials, setCredentials] = useState<CredentialDisplay[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [refreshingAll, setRefreshingAll] = useState(false);
  const [validatingAll, setValidatingAll] = useState(false);
  const [selectedAuthMethod, setSelectedAuthMethod] = useState<AuthMethod>("oauth");

  // 编辑状态
  const [editingCredential, setEditingCredential] = useState<CredentialDisplay | null>(null);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  // 操作状态 - 按凭证 UUID 跟踪
  const [deletingIds, setDeletingIds] = useState<Set<string>>(new Set());
  const [checkingHealthIds, setCheckingHealthIds] = useState<Set<string>>(new Set());
  const [refreshingTokenIds, setRefreshingTokenIds] = useState<Set<string>>(new Set());

  // 加载凭证列表
  const loadCredentials = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      // 注意：主应用目前不支持 "droid" provider type
      // 这里暂时返回空列表，等待后端支持
      try {
        const list = await providerPoolApi.getCredentials("droid" as any);
        setCredentials(list);
      } catch {
        // 如果 API 不支持，显示空列表而不是错误
        setCredentials([]);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "加载凭证失败");
    } finally {
      setLoading(false);
    }
  }, []);

  // 初始化加载
  React.useEffect(() => {
    loadCredentials();
  }, [loadCredentials]);

  // 添加凭证成功回调
  const handleAddSuccess = useCallback(() => {
    setIsAddModalOpen(false);
    loadCredentials();
    toast.success("凭证添加成功");
  }, [loadCredentials]);

  // 删除凭证
  const handleDelete = useCallback(
    async (uuid: string) => {
      if (!confirm("确定要删除这个凭证吗？")) return;
      try {
        setDeletingIds((prev) => new Set(prev).add(uuid));
        await providerPoolApi.deleteCredential(uuid, "droid");
        toast.success("凭证已删除");
        loadCredentials();
      } catch (e) {
        toast.error(e instanceof Error ? e.message : "删除失败");
      } finally {
        setDeletingIds((prev) => {
          const next = new Set(prev);
          next.delete(uuid);
          return next;
        });
      }
    },
    [loadCredentials]
  );

  // 切换启用/禁用
  const handleToggle = useCallback(
    async (uuid: string, isDisabled: boolean) => {
      try {
        await providerPoolApi.toggleCredential(uuid, !isDisabled);
        toast.success(isDisabled ? "凭证已启用" : "凭证已禁用");
        loadCredentials();
      } catch (e) {
        toast.error(e instanceof Error ? e.message : "操作失败");
      }
    },
    [loadCredentials]
  );

  // 重置凭证
  const handleReset = useCallback(
    async (uuid: string) => {
      try {
        await providerPoolApi.resetCredential(uuid);
        toast.success("凭证已重置");
        loadCredentials();
      } catch (e) {
        toast.error(e instanceof Error ? e.message : "重置失败");
      }
    },
    [loadCredentials]
  );

  // 检测健康
  const handleCheckHealth = useCallback(
    async (uuid: string) => {
      try {
        setCheckingHealthIds((prev) => new Set(prev).add(uuid));
        const result = await providerPoolApi.checkCredentialHealth(uuid);
        if (result.success) {
          toast.success("凭证验证通过");
        } else {
          toast.error(result.message || "凭证验证失败");
        }
        loadCredentials();
      } catch (e) {
        toast.error(e instanceof Error ? e.message : "验证失败");
      } finally {
        setCheckingHealthIds((prev) => {
          const next = new Set(prev);
          next.delete(uuid);
          return next;
        });
      }
    },
    [loadCredentials]
  );

  // 刷新 Token
  const handleRefreshToken = useCallback(
    async (uuid: string) => {
      try {
        setRefreshingTokenIds((prev) => new Set(prev).add(uuid));
        await providerPoolApi.refreshCredentialToken(uuid);
        toast.success("Token 刷新成功");
        loadCredentials();
      } catch (e) {
        toast.error(e instanceof Error ? e.message : "刷新失败");
      } finally {
        setRefreshingTokenIds((prev) => {
          const next = new Set(prev);
          next.delete(uuid);
          return next;
        });
      }
    },
    [loadCredentials]
  );

  // 编辑凭证
  const handleEdit = useCallback((credential: CredentialDisplay) => {
    setEditingCredential(credential);
    setIsEditModalOpen(true);
  }, []);

  // 编辑凭证提交
  const handleEditSubmit = useCallback(
    async (uuid: string, request: UpdateCredentialRequest) => {
      try {
        await providerPoolApi.updateCredential(uuid, request);
        toast.success("凭证已更新");
        loadCredentials();
      } catch (e) {
        throw new Error(
          `编辑失败: ${e instanceof Error ? e.message : String(e)}`
        );
      }
    },
    [loadCredentials]
  );

  // 关闭编辑模态框
  const handleEditClose = useCallback(() => {
    setIsEditModalOpen(false);
    setEditingCredential(null);
  }, []);

  // 刷新所有
  const handleRefreshAll = useCallback(async () => {
    setRefreshingAll(true);
    try {
      let successCount = 0;
      let failCount = 0;
      for (const cred of credentials) {
        // 只刷新 OAuth 类型的凭证
        const authType = cred.credential_data?.auth_type;
        if (authType === "api_key") continue;

        try {
          await providerPoolApi.refreshCredentialToken(cred.uuid);
          successCount++;
        } catch {
          failCount++;
        }
      }
      if (failCount > 0) {
        toast.info(`刷新完成: ${successCount} 成功, ${failCount} 失败`);
      } else if (successCount > 0) {
        toast.success("所有 OAuth 凭证已刷新");
      } else {
        toast.info("没有需要刷新的 OAuth 凭证");
      }
      loadCredentials();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "刷新失败");
    } finally {
      setRefreshingAll(false);
    }
  }, [credentials, loadCredentials]);

  // 验证所有
  const handleValidateAll = useCallback(async () => {
    setValidatingAll(true);
    try {
      let validCount = 0;
      let invalidCount = 0;
      for (const cred of credentials) {
        const result = await providerPoolApi.checkCredentialHealth(cred.uuid);
        if (result.success) validCount++;
        else invalidCount++;
      }
      toast.info(`验证完成: ${validCount} 个有效, ${invalidCount} 个无效`);
      loadCredentials();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "验证失败");
    } finally {
      setValidatingAll(false);
    }
  }, [credentials, loadCredentials]);

  // 加载中
  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载凭证...</span>
      </div>
    );
  }

  // 错误
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center p-8">
        <AlertCircle className="h-8 w-8 text-red-500 mb-2" />
        <p className="text-red-600 mb-4">{error}</p>
        <Button variant="outline" onClick={loadCredentials}>
          <RefreshCw className="h-4 w-4 mr-2" />
          重试
        </Button>
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      {/* 工具栏 */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Droid 凭证管理</h2>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleValidateAll}
            disabled={validatingAll || credentials.length === 0}
          >
            {validatingAll ? (
              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <Zap className="h-4 w-4 mr-1" />
            )}
            检测全部
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefreshAll}
            disabled={refreshingAll || credentials.length === 0}
          >
            {refreshingAll ? (
              <Loader2 className="h-4 w-4 mr-1 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4 mr-1" />
            )}
            刷新全部
          </Button>
          <Button size="sm" onClick={() => setIsAddModalOpen(true)}>
            <Plus className="h-4 w-4 mr-1" />
            添加凭证
          </Button>
        </div>
      </div>

      {/* 凭证列表 */}
      {credentials.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-12 border-2 border-dashed rounded-xl text-muted-foreground">
          <Bot className="h-12 w-12 mb-4 opacity-50" />
          <p className="text-lg font-medium mb-1">暂无凭证</p>
          <p className="text-sm mb-4">点击"添加凭证"开始配置 Factory.ai Droid</p>
          <Button onClick={() => setIsAddModalOpen(true)}>
            <Plus className="h-4 w-4 mr-1" />
            添加第一个凭证
          </Button>
        </div>
      ) : (
        <div className="space-y-4">
          {credentials.map((credential) => (
            <CredentialCard
              key={credential.uuid}
              credential={credential}
              onToggle={() => handleToggle(credential.uuid, credential.is_disabled)}
              onDelete={() => handleDelete(credential.uuid)}
              onReset={() => handleReset(credential.uuid)}
              onCheckHealth={() => handleCheckHealth(credential.uuid)}
              onRefreshToken={() => handleRefreshToken(credential.uuid)}
              onEdit={() => handleEdit(credential)}
              deleting={deletingIds.has(credential.uuid)}
              checkingHealth={checkingHealthIds.has(credential.uuid)}
              refreshingToken={refreshingTokenIds.has(credential.uuid)}
            />
          ))}
        </div>
      )}

      {/* 添加凭证模态框 */}
      <Modal isOpen={isAddModalOpen} onClose={() => setIsAddModalOpen(false)} maxWidth="max-w-xl">
        <div className="p-6">
          <h3 className="text-lg font-semibold mb-4">添加 Droid 凭证</h3>

          {/* 认证方式选择 */}
          <Tabs value={selectedAuthMethod} onValueChange={(v) => setSelectedAuthMethod(v as AuthMethod)}>
            <TabsList className="grid grid-cols-2 mb-4">
              {AUTH_METHODS.map((method) => (
                <TabsTrigger key={method.id} value={method.id} className="flex items-center gap-2">
                  <method.icon className="h-4 w-4" />
                  {method.label}
                </TabsTrigger>
              ))}
            </TabsList>

            {/* OAuth 表单 */}
            <TabsContent value="oauth">
              <DroidFormStandalone
                authType="oauth"
                onSuccess={handleAddSuccess}
                onCancel={() => setIsAddModalOpen(false)}
              />
            </TabsContent>

            {/* API Key 表单 */}
            <TabsContent value="api_key">
              <DroidFormStandalone
                authType="api_key"
                onSuccess={handleAddSuccess}
                onCancel={() => setIsAddModalOpen(false)}
              />
            </TabsContent>
          </Tabs>
        </div>
      </Modal>

      {/* 编辑凭证模态框 */}
      <EditCredentialModal
        credential={editingCredential}
        isOpen={isEditModalOpen}
        onClose={handleEditClose}
        onEdit={handleEditSubmit}
      />
    </div>
  );
}

export default DroidProviderApp;
