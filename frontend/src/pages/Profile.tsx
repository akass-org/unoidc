import { useState, useEffect } from "react";
import { Camera, Mail, Fingerprint, KeyRound } from "lucide-react";
import { useSessionStore } from "#src/stores/session";
import { meApi } from "#src/api/me";
import { passkeyApi } from "#src/api/passkey";
import { useApi } from "#src/hooks";
import { Card, CardHeader, Input, Button, Avatar, useToast, Modal } from "#src/components/ui";
import { getErrorMessage } from "#src/api/client";
import { isWebAuthnSupported, startRegistration } from "#src/utils/webauthn";

export function ProfilePage() {
  const { user, setUser } = useSessionStore();
  const { addToast } = useToast();

  // Profile form state
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");

  // Email change modal state
  const [isEmailModalOpen, setIsEmailModalOpen] = useState(false);
  const [newEmail, setNewEmail] = useState("");

  // Password form state
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  // Update profile API
  const { loading: updatingProfile, execute: updateProfile } = useApi(meApi.updateProfile, {
    successMessage: "个人资料已更新",
    onSuccess: (data) => {
      setUser(
        data as {
          id: string;
          username: string;
          email: string;
          display_name: string;
          picture?: string;
          is_admin: boolean;
        },
      );
    },
  });

  // Change password API
  const { loading: changingPassword, execute: changePassword } = useApi(meApi.changePassword, {
    successMessage: "密码已修改",
    onSuccess: () => {
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    },
  });

  // Request email change API
  const { loading: requestingEmailChange, execute: requestEmailChange } = useApi(
    meApi.requestEmailChange,
    {
      successMessage: "验证链接已发送",
      onSuccess: () => {
        setNewEmail("");
        setIsEmailModalOpen(false);
      },
    },
  );

  // Passkey state
  const [passkeys, setPasskeys] = useState<
    ReturnType<typeof passkeyApi.list> extends Promise<infer T> ? T : never
  >([]);
  const [addingPasskey, setAddingPasskey] = useState(false);
  const [deletingPasskeyId, setDeletingPasskeyId] = useState<string | null>(null);
  const [isDeletePasskeyModalOpen, setIsDeletePasskeyModalOpen] = useState(false);

  // Load user data
  useEffect(() => {
    if (user) {
      setDisplayName(user.display_name || "");
      setEmail(user.email || "");
    }
  }, [user]);

  // Load passkeys
  useEffect(() => {
    let mounted = true;
    passkeyApi
      .list()
      .then((data) => {
        if (mounted) setPasskeys(data);
      })
      .catch(() => {
        // ignore load errors
      });
    return () => {
      mounted = false;
    };
  }, []);

  const handleAddPasskey = async () => {
    if (!isWebAuthnSupported()) {
      addToast({ type: "error", title: "您的浏览器不支持 Passkey" });
      return;
    }
    setAddingPasskey(true);
    try {
      const options = await passkeyApi.registerStart();
      const credential = await startRegistration(options);
      await passkeyApi.registerFinish(credential);
      const updated = await passkeyApi.list();
      setPasskeys(updated);
      addToast({ type: "success", title: "passkey 已添加" });
    } catch (err: unknown) {
      if (err instanceof Error) {
        if (err.name === "NotAllowedError" || err.message.includes("cancelled")) {
          // 用户取消，静默处理
        } else if (
          err.message.includes("InvalidStateError") ||
          err.message.includes("already registered")
        ) {
          addToast({ type: "error", title: "该凭据已绑定到您的账户" });
        } else {
          addToast({ type: "error", title: "添加失败", message: getErrorMessage(err) });
        }
      }
    } finally {
      setAddingPasskey(false);
    }
  };

  const openDeleteModal = (id: string) => {
    setDeletingPasskeyId(id);
    setIsDeletePasskeyModalOpen(true);
  };

  const handleDeletePasskey = async () => {
    if (!deletingPasskeyId) return;
    try {
      await passkeyApi.delete(deletingPasskeyId);
      setPasskeys((prev) => prev.filter((p) => p.id !== deletingPasskeyId));
      addToast({ type: "success", title: "passkey 已删除" });
    } catch (err) {
      addToast({ type: "error", title: "删除失败", message: getErrorMessage(err) });
    } finally {
      setDeletingPasskeyId(null);
      setIsDeletePasskeyModalOpen(false);
    }
  };

  const handleProfileSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await updateProfile({ display_name: displayName });
  };

  const handleEmailChangeSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await requestEmailChange({ new_email: newEmail });
  };

  const handlePasswordSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (newPassword !== confirmPassword) {
      addToast({
        type: "error",
        title: "两次输入的密码不一致",
      });
      return;
    }

    if (newPassword.length < 8) {
      addToast({
        type: "error",
        title: "新密码至少需要8个字符",
      });
      return;
    }

    await changePassword({ current_password: currentPassword, new_password: newPassword });
  };

  const handleAvatarUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 前端预检查：限制 1MB，避免大图上传超时
    if (file.size > 1024 * 1024) {
      addToast({
        type: "error",
        title: "图片过大",
        message: `文件大小 ${(file.size / 1024).toFixed(0)}KB，请选择小于 1MB 的图片`,
      });
      // 重置 input，允许重复选择同一文件
      e.target.value = "";
      return;
    }

    try {
      const result = await meApi.uploadAvatar(file);
      setUser(
        result as {
          id: string;
          username: string;
          email: string;
          display_name: string;
          picture?: string;
          is_admin: boolean;
        },
      );
      addToast({
        type: "success",
        title: "头像已更新",
      });
    } catch (err) {
      addToast({
        type: "error",
        title: "头像上传失败",
        message: getErrorMessage(err),
      });
    }
  };

  return (
    <div className="space-y-5 page-content">
      <div>
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">个人资料</h1>
        <p className="text-sm text-gray-500 mt-0.5">管理您的账户信息和安全设置</p>
      </div>

      {/* Avatar Section */}
      <Card>
        <div className="flex flex-col sm:flex-row items-center gap-5">
          <div className="relative">
            <Avatar
              name={user?.display_name || user?.username || "?"}
              src={user?.picture}
              size="xl"
            />
            <label className="absolute -bottom-1 -right-1 w-7 h-7 bg-white hover:bg-gray-200 rounded-full flex items-center justify-center cursor-pointer transition-colors shadow-lg">
              <Camera className="w-3.5 h-3.5 text-black" />
              <input
                type="file"
                accept="image/*"
                onChange={handleAvatarUpload}
                className="hidden"
              />
            </label>
          </div>
          <div className="text-center sm:text-left">
            <h2 className="text-base font-bold text-gray-900 dark:text-white">
              {user?.display_name || user?.username}
            </h2>
            <p className="text-sm text-gray-500">@{user?.username}</p>
            <p className="text-xs text-gray-500 dark:text-gray-600 mt-1">
              {user?.is_admin ? "管理员" : "普通用户"}
            </p>
          </div>
        </div>
      </Card>

      {/* Profile Form */}
      <Card>
        <CardHeader
          title={<span className="font-bold">基本信息</span>}
          subtitle="更新您的显示名称和邮箱地址"
        />
        <form onSubmit={handleProfileSubmit} className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Input label="用户名" value={user?.username || ""} disabled helper="用户名不可修改" />
            <Input
              label="显示名称"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="您的显示名称"
            />
          </div>
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <Input
                label="邮箱地址"
                type="email"
                value={email}
                disabled
                helper="邮箱修改需要通过验证流程"
              />
            </div>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => setIsEmailModalOpen(true)}
            >
              <Mail className="w-4 h-4 mr-1.5" />
              修改邮箱
            </Button>
          </div>
          <div className="flex justify-end pt-1">
            <Button type="submit" loading={updatingProfile} size="sm">
              保存更改
            </Button>
          </div>
        </form>
      </Card>

      {/* Password Form */}
      <Card>
        <CardHeader
          title={<span className="font-bold">修改密码</span>}
          subtitle="定期更改密码可以提高账户安全性"
        />
        <form onSubmit={handlePasswordSubmit} className="space-y-4 max-w-md">
          <Input
            label="当前密码"
            type="password"
            value={currentPassword}
            onChange={(e) => setCurrentPassword(e.target.value)}
            placeholder="输入当前密码"
            required
          />
          <Input
            label="新密码"
            type="password"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            placeholder="至少8位字符"
            required
          />
          <Input
            label="确认新密码"
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            placeholder="再次输入新密码"
            required
          />
          <div className="flex justify-end pt-1">
            <Button type="submit" loading={changingPassword} variant="secondary" size="sm">
              更新密码
            </Button>
          </div>
        </form>
      </Card>

      {/* Passkey Management */}
      <Card>
        <CardHeader
          title={<span className="font-bold">Passkey 管理</span>}
          subtitle="使用指纹、面容识别或安全密钥登录"
          action={
            <Button
              size="sm"
              variant="secondary"
              loading={addingPasskey}
              onClick={handleAddPasskey}
            >
              <Fingerprint className="w-4 h-4 mr-1.5" />
              添加 passkey
            </Button>
          }
        />
        <div className="space-y-3">
          {passkeys.length === 0 ? (
            <div className="text-sm text-gray-500 py-4 text-center">
              尚未注册 passkey
              <p className="text-xs text-gray-400 mt-1">添加后可以使用指纹或安全密钥快速登录</p>
            </div>
          ) : (
            passkeys.map((cred) => (
              <div
                key={cred.id}
                className="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-white/[0.04] border border-gray-100 dark:border-white/[0.06]"
              >
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-full bg-black dark:bg-white flex items-center justify-center">
                    <KeyRound className="w-4 h-4 text-white dark:text-black" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-white">
                      {cred.display_name || "Passkey"}
                    </p>
                    <p className="text-xs text-gray-500">
                      注册于 {new Date(cred.created_at).toLocaleDateString("zh-CN")}
                      {cred.last_used_at &&
                        ` · 最后使用 ${new Date(cred.last_used_at).toLocaleDateString("zh-CN")}`}
                    </p>
                  </div>
                </div>
                <Button variant="ghost" size="sm" onClick={() => openDeleteModal(cred.id)}>
                  删除
                </Button>
              </div>
            ))
          )}
        </div>
      </Card>

      {/* Email Change Modal */}
      <Modal
        isOpen={isEmailModalOpen}
        onClose={() => setIsEmailModalOpen(false)}
        title="修改邮箱地址"
        description="验证链接将发送到新邮箱，请在 24 小时内点击链接完成验证"
        footer={
          <>
            <Button variant="ghost" onClick={() => setIsEmailModalOpen(false)}>
              取消
            </Button>
            <Button
              onClick={handleEmailChangeSubmit}
              loading={requestingEmailChange}
              disabled={!newEmail || newEmail === email}
            >
              发送验证链接
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <Input label="当前邮箱" value={email} disabled />
          <Input
            label="新邮箱地址"
            type="email"
            value={newEmail}
            onChange={(e) => setNewEmail(e.target.value)}
            placeholder="new-email@example.com"
            required
          />
        </div>
      </Modal>

      {/* Delete Passkey Modal */}
      <Modal
        isOpen={isDeletePasskeyModalOpen}
        onClose={() => setIsDeletePasskeyModalOpen(false)}
        title="删除 passkey"
        description="删除后将无法使用该凭据登录，确认要继续吗？"
        footer={
          <>
            <Button variant="ghost" onClick={() => setIsDeletePasskeyModalOpen(false)}>
              取消
            </Button>
            <Button variant="danger" onClick={handleDeletePasskey}>
              删除
            </Button>
          </>
        }
      >
        <div />
      </Modal>
    </div>
  );
}
