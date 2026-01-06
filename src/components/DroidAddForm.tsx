/**
 * Droid 凭证添加表单
 *
 * 支持 WorkOS OAuth 和 API Key 两种认证方式
 */

import { useState, useCallback } from "react";
import {
  Button,
  Input,
  Loader2,
  Download,
  Key,
  KeyRound,
} from "@proxycast/plugin-components";

interface DroidAddFormProps {
  authType: "oauth" | "api_key";
  onSuccess: () => void;
  onCancel?: () => void;
}

export function DroidAddForm({ authType, onSuccess, onCancel }: DroidAddFormProps) {
  const [name, setName] = useState("");
  const [credsFilePath, setCredsFilePath] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSelectFile = useCallback(async () => {
    try {
      // @ts-expect-error - Tauri plugin available at runtime
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (selected) {
        setCredsFilePath(selected as string);
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  }, []);

  const handleSubmit = useCallback(async () => {
    if (authType === "oauth" && !credsFilePath) {
      setError("请选择凭证文件");
      return;
    }
    if (authType === "api_key" && !apiKey.trim()) {
      setError("请输入 API Key");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // @ts-expect-error - Tauri API available at runtime
      const { invoke } = await import("@tauri-apps/api/core");
      
      if (authType === "oauth") {
        await invoke("add_droid_credential", {
          filePath: credsFilePath,
          displayName: name.trim() || undefined,
        });
      } else {
        await invoke("add_droid_api_key_credential", {
          apiKey: apiKey.trim(),
          displayName: name.trim() || undefined,
        });
      }
      onSuccess();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [authType, credsFilePath, apiKey, name, onSuccess]);

  return (
    <div className="space-y-4">
      <div>
        <label className="mb-1 block text-sm font-medium">名称 (可选)</label>
        <Input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="给这个凭证起个名字..."
          disabled={loading}
        />
      </div>

      {authType === "oauth" ? (
        <>
          <div className="rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-950/30">
            <p className="text-sm text-blue-700 dark:text-blue-300">
              导入 Droid CLI 的 WorkOS OAuth 凭证文件。
            </p>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">
              凭证文件 <span className="text-red-500">*</span>
            </label>
            <div className="flex gap-2">
              <Input
                type="text"
                value={credsFilePath}
                onChange={(e) => setCredsFilePath(e.target.value)}
                placeholder="选择凭证文件..."
                className="flex-1"
              />
              <Button type="button" variant="outline" onClick={handleSelectFile}>
                <Download className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </>
      ) : (
        <>
          <div className="rounded-lg border border-amber-200 bg-amber-50 p-4 dark:border-amber-800 dark:bg-amber-950/30">
            <p className="text-sm text-amber-700 dark:text-amber-300">
              使用 Factory.ai API Key 进行认证。
            </p>
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium">
              API Key <span className="text-red-500">*</span>
            </label>
            <Input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="输入 API Key..."
              disabled={loading}
            />
          </div>
        </>
      )}

      {error && (
        <div className="rounded-lg border border-red-300 bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      <div className="flex justify-end gap-2 pt-2">
        {onCancel && (
          <Button type="button" variant="outline" onClick={onCancel} disabled={loading}>
            取消
          </Button>
        )}
        <Button
          type="button"
          onClick={handleSubmit}
          disabled={loading || (authType === "oauth" ? !credsFilePath : !apiKey.trim())}
        >
          {loading ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              添加中...
            </>
          ) : (
            <>
              {authType === "oauth" ? (
                <Key className="h-4 w-4 mr-2" />
              ) : (
                <KeyRound className="h-4 w-4 mr-2" />
              )}
              添加凭证
            </>
          )}
        </Button>
      </div>
    </div>
  );
}

export default DroidAddForm;
